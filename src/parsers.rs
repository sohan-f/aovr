use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct TargetOverlays {
    pub enabled: Vec<String>,
    pub disabled: Vec<String>,
    pub broken: Vec<String>,
}

impl TargetOverlays {
    pub fn total(&self) -> usize {
        self.enabled.len() + self.disabled.len() + self.broken.len()
    }
}

pub type Targets = HashMap<String, TargetOverlays>;

/// Bucket for overlay lines printed before any target header
/// (`cmd overlay list <pkg>` omits the header).
pub const UNGROUPED_TARGET: &str = "(ungrouped)";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Enabled,
    Disabled,
    Broken,
}

fn status_prefix(line: &str) -> Option<(State, &str)> {
    [
        ("[x]", State::Enabled),
        ("[ ]", State::Disabled),
        ("---", State::Broken),
    ]
    .into_iter()
    .find_map(|(prefix, state)| line.strip_prefix(prefix).map(|rest| (state, rest.trim())))
    .filter(|(_, name)| !name.is_empty())
}

/// Plausible Android package name — keeps tooling noise (e.g. the real-device
/// `cmd: Failure calling service …` line) from becoming a bogus target. No dot
/// required: the `android` framework package has none.
fn is_package_name(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
}

/// Parse `cmd overlay list` output into targets grouped by package. A bare
/// line starts a group only when status-prefixed lines follow (AOSP's
/// `runList()` grammar), which keeps usage-text words from becoming targets.
pub fn parse_overlays(bytes: &[u8]) -> Targets {
    let input = String::from_utf8_lossy(bytes);

    let lines: Vec<&str> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    let mut map: Targets = HashMap::new();
    let mut target: Option<String> = None;

    for (index, line) in lines.iter().enumerate() {
        if let Some((state, name)) = status_prefix(line) {
            let key = target
                .clone()
                .unwrap_or_else(|| UNGROUPED_TARGET.to_string());
            let entry = map.entry(key).or_default();
            match state {
                State::Enabled => entry.enabled.push(name.to_string()),
                State::Disabled => entry.disabled.push(name.to_string()),
                State::Broken => entry.broken.push(name.to_string()),
            }
        } else if is_package_name(line)
            && lines
                .get(index + 1)
                .is_some_and(|next| status_prefix(next).is_some())
        {
            target = Some(line.to_string());
            map.entry(line.to_string()).or_default();
        }
        // Anything else (warnings, tool errors) is dropped.
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    const OK: &str = include_str!("../tests/fixtures/cmd_overlay_list_ok.txt");
    const DEVICE_FAILURE: &str = include_str!("../tests/fixtures/device_cmd_failure.txt");
    const DEVICE_LIST: &str = include_str!("../assets/virtual_targets.txt");
    const DEVICE_USAGE: &str = include_str!("../tests/fixtures/device_cmd_overlay_usage.txt");

    #[test]
    fn groups_overlays_by_target() {
        let map = parse_overlays(OK.as_bytes());

        assert_eq!(map.len(), 3);

        let android = &map["android"];
        assert_eq!(
            android.enabled,
            ["com.android.internal.systemui.navbar.gestural"]
        );
        assert_eq!(android.disabled.len(), 2);
        assert!(android.broken.is_empty());

        let systemui = &map["com.android.systemui"];
        assert_eq!(systemui.enabled, ["com.android.systemui.navbar.gestural"]);
        assert_eq!(
            systemui.disabled,
            ["com.android.theme.icon_pack.rounded.systemui"]
        );
        assert_eq!(systemui.broken, ["com.android.systemui.broken.rro"]);

        let pc = &map["com.android.permissioncontroller"];
        assert_eq!(pc.total(), 1);
        assert_eq!(pc.disabled[0], "com.permissioncontroller.googlecarui.rro");
    }

    #[test]
    fn totals_match_fixture() {
        let map = parse_overlays(OK.as_bytes());
        let total: usize = map.values().map(TargetOverlays::total).sum();
        assert_eq!(total, 7);
    }

    #[test]
    fn device_failure_output_yields_no_targets() {
        assert!(
            parse_overlays(DEVICE_FAILURE.as_bytes()).is_empty(),
            "error text was misfiled as a target"
        );
    }

    /// Real-device capture; counts below were measured directly against it.
    #[test]
    fn real_device_list_is_grouped() {
        let map = parse_overlays(DEVICE_LIST.as_bytes());

        assert_eq!(map.len(), 31);

        let enabled: usize = map.values().map(|t| t.enabled.len()).sum();
        let disabled: usize = map.values().map(|t| t.disabled.len()).sum();
        let broken: usize = map.values().map(|t| t.broken.len()).sum();
        assert_eq!((enabled, disabled, broken), (67, 39, 20));

        let android = &map["android"];
        assert_eq!((android.enabled.len(), android.disabled.len()), (35, 20));
        assert_eq!(android.broken.len(), 2);
        // Overlay identifiers carrying `:name` suffixes target the framework.
        assert!(
            android
                .enabled
                .iter()
                .any(|n| n.as_str() == "com.android.systemui:neutral")
        );
        assert!(
            android
                .broken
                .iter()
                .any(|n| n.as_str() == "android.auto_generated_characteristics_rro")
        );

        let systemui = &map["com.android.systemui"];
        assert_eq!((systemui.enabled.len(), systemui.disabled.len()), (7, 8));

        // Target whose only overlay shares its package name.
        let qti = &map["com.qualcomm.qti.telephonyservice"];
        assert_eq!(qti.broken, ["com.qualcomm.qti.telephonyservice"]);
    }

    #[test]
    fn usage_output_is_never_misfiled_as_targets() {
        let map = parse_overlays(DEVICE_USAGE.as_bytes());
        assert!(
            map.is_empty(),
            "usage text produced bogus targets: {:?}",
            map.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn junk_lines_are_ignored() {
        let junk = "\
Error: Unknown option: --bogus
cmd: Can't find service overlay
)
";
        assert!(parse_overlays(junk.as_bytes()).is_empty());
    }

    #[test]
    fn tolerates_crlf_line_endings() {
        let crlf = "com.acme.app\r\n[x] com.acme.app.overlay\r\n";
        let map = parse_overlays(crlf.as_bytes());
        assert_eq!(map["com.acme.app"].enabled, ["com.acme.app.overlay"]);
    }

    #[test]
    fn accepts_overlay_identifiers_with_colon() {
        let out = "com.acme.app\n[x] com.acme.overlay:dark\n";
        let map = parse_overlays(out.as_bytes());
        assert_eq!(map["com.acme.app"].enabled, ["com.acme.overlay:dark"]);
    }

    #[test]
    fn overlay_without_header_goes_to_ungrouped() {
        let map = parse_overlays("[ ] com.acme.orphan\n".as_bytes());
        assert_eq!(map[UNGROUPED_TARGET].disabled, ["com.acme.orphan"]);
    }

    #[test]
    fn status_prefix_without_identifier_is_ignored() {
        let map = parse_overlays("[x]\n---\n".as_bytes());
        assert!(map.is_empty());
    }
}
