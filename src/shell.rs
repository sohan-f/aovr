//! Runs `cmd overlay` via su on a device, or replays a recorded list
//! ("virtual mode") on hosts that are not Android.

use std::{
    env,
    ffi::{OsStr, OsString},
    io,
    process::{Command, Output},
};
#[cfg(feature = "virtual")]
use std::{fs, path::PathBuf};

use crate::parsers::{self, Targets};

/// Root helper used before running `cmd overlay`.
#[cfg(target_os = "android")]
const DEFAULT_SU: &str = "/system/bin/su";
#[cfg(not(target_os = "android"))]
const DEFAULT_SU: &str = "su";

const SU_ENV_VAR: &str = "AOVR_SU";
#[cfg(feature = "virtual")]
const TARGETS_FILE_ENV: &str = "AOVR_TARGETS_FILE";

/// Real-device capture used as the default list on non-Android hosts.
#[cfg(feature = "virtual")]
const BUNDLED_VIRTUAL_LIST: &str = include_str!("../assets/virtual_targets.txt");

/// `cmd` prints these on binder failure — even when it exits 0, so exit
/// status alone is not a reliable failure signal.
const CMD_FAILURE_MARKERS: [&str; 2] = ["Failure calling service", "Can't find service"];

#[derive(Debug)]
pub enum Backend {
    /// Real device: `su -c "cmd overlay …"` via the given root helper.
    Su(OsString),
    /// Replayed list; toggles stay in-memory. Dev builds only.
    #[cfg(feature = "virtual")]
    Virtual(VirtualStore),
}

#[derive(Debug)]
#[cfg(feature = "virtual")]
pub struct VirtualStore {
    source: VirtualSource,
    targets: Targets,
    loaded: bool,
}

#[derive(Debug)]
#[cfg(feature = "virtual")]
enum VirtualSource {
    Bundled,
    File(PathBuf),
}

impl Default for Backend {
    fn default() -> Self {
        #[cfg(feature = "virtual")]
        return Self::virtual_bundled();
        #[cfg(not(feature = "virtual"))]
        return Self::Su(DEFAULT_SU.into());
    }
}

#[cfg(feature = "virtual")]
impl VirtualStore {
    #[cfg(all(test, feature = "virtual"))]
    pub fn new(targets: Targets) -> Self {
        Self {
            source: VirtualSource::Bundled,
            targets,
            loaded: true,
        }
    }

    pub fn bundled() -> Self {
        Self {
            source: VirtualSource::Bundled,
            targets: parsers::parse_overlays(BUNDLED_VIRTUAL_LIST.as_bytes()),
            loaded: true,
        }
    }

    /// Read lazily on the first `list()`; a failed read stays unread so a
    /// retry (`r`) re-attempts it, like a real device load.
    fn file(path: PathBuf) -> Self {
        Self {
            source: VirtualSource::File(path),
            targets: Targets::default(),
            loaded: false,
        }
    }

    fn list(&mut self) -> io::Result<Targets> {
        if !self.loaded {
            if let VirtualSource::File(path) = &self.source {
                let bytes = fs::read(path).map_err(|err| {
                    io::Error::other(format!("virtual target list '{}': {err}", path.display()))
                })?;
                self.targets = parsers::parse_overlays(&bytes);
            }
            self.loaded = true;
        }
        Ok(self.targets.clone())
    }

    fn set_overlay(&mut self, enabled: bool, overlay: &str) -> io::Result<()> {
        for store in self.targets.values_mut() {
            if store.broken.iter().any(|name| name == overlay) {
                return Err(io::Error::other(format!(
                    "{overlay} is broken; refusing to toggle"
                )));
            }
            if enabled {
                if let Some(index) = store.disabled.iter().position(|name| name == overlay) {
                    store.enabled.push(store.disabled.remove(index));
                    return Ok(());
                }
                if store.enabled.iter().any(|name| name == overlay) {
                    return Ok(());
                }
            } else {
                if let Some(index) = store.enabled.iter().position(|name| name == overlay) {
                    store.disabled.push(store.enabled.remove(index));
                    return Ok(());
                }
                if store.disabled.iter().any(|name| name == overlay) {
                    return Ok(());
                }
            }
        }
        Err(io::Error::other(format!(
            "{overlay} not found in virtual target list"
        )))
    }
}

impl Backend {
    #[cfg(feature = "virtual")]
    pub fn virtual_bundled() -> Self {
        Self::Virtual(VirtualStore::bundled())
    }

    #[cfg(all(test, feature = "virtual"))]
    pub fn virtual_targets(targets: Targets) -> Self {
        Self::Virtual(VirtualStore::new(targets))
    }

    pub fn from_env() -> Self {
        Self::from_env_impl(|key| env::var_os(key), cfg!(target_os = "android"))
    }

    /// Priority: `AOVR_TARGETS_FILE` > (`AOVR_SU` set or Android) > bundled
    /// virtual list.
    #[cfg(feature = "virtual")]
    fn from_env_impl(get: impl Fn(&str) -> Option<OsString>, android: bool) -> Self {
        if let Some(path) = get(TARGETS_FILE_ENV).filter(|value| !value.is_empty()) {
            return Self::Virtual(VirtualStore::file(PathBuf::from(path)));
        }

        let su = get(SU_ENV_VAR).filter(|value| !value.is_empty());
        if android || su.is_some() {
            return Self::Su(su.unwrap_or_else(|| DEFAULT_SU.into()));
        }

        Self::virtual_bundled()
    }

    #[cfg(not(feature = "virtual"))]
    fn from_env_impl(get: impl Fn(&str) -> Option<OsString>, _android: bool) -> Self {
        let su = get(SU_ENV_VAR).filter(|value| !value.is_empty());
        Self::Su(su.unwrap_or_else(|| DEFAULT_SU.into()))
    }

    pub fn is_virtual(&self) -> bool {
        #[cfg(feature = "virtual")]
        return matches!(self, Self::Virtual(_));
        #[cfg(not(feature = "virtual"))]
        return false;
    }

    pub fn list(&mut self) -> io::Result<Targets> {
        match self {
            Self::Su(bin) => {
                let output = run_su(bin, "cmd overlay list")?;
                targets_from_output(&output)
            }
            #[cfg(feature = "virtual")]
            Self::Virtual(store) => store.list(),
        }
    }

    pub fn set_overlay(&mut self, enabled: bool, overlay: &str) -> io::Result<()> {
        match self {
            Self::Su(bin) => {
                let action = if enabled { "enable" } else { "disable" };
                let output = run_su(
                    bin,
                    &format!("cmd overlay {action} {}", shell_quote(overlay)),
                )?;
                match command_failure(&output) {
                    Some(msg) => Err(io::Error::other(format!("{action} failed: {msg}"))),
                    None => Ok(()),
                }
            }
            #[cfg(feature = "virtual")]
            Self::Virtual(store) => store.set_overlay(enabled, overlay),
        }
    }
}

fn launch_error(bin: &OsStr, err: io::Error) -> io::Error {
    let bin = bin.to_string_lossy();
    if err.kind() == io::ErrorKind::NotFound {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "root helper '{bin}' not found — AOVR needs a rooted device (Magisk/KernelSU), or set ${SU_ENV_VAR}"
            ),
        )
    } else {
        io::Error::other(format!("failed to run '{bin}': {err}"))
    }
}

fn exit_desc(output: &Output) -> String {
    match output.status.code() {
        Some(code) => format!("exit code {code}"),
        None => String::from("killed by signal"),
    }
}

/// Error message when a finished `cmd` invocation failed by exit status or
/// printed its binder-failure signature (which can happen on exit 0).
pub(crate) fn command_failure(output: &Output) -> Option<String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let marker_line = stdout
        .lines()
        .chain(stderr.lines())
        .map(str::trim)
        .find(|line| {
            CMD_FAILURE_MARKERS
                .iter()
                .any(|marker| line.contains(marker))
        });
    if let Some(line) = marker_line {
        return Some(line.to_string());
    }

    if !output.status.success() {
        let detail = stdout
            .lines()
            .chain(stderr.lines())
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("no output");
        return Some(format!("{}: {detail}", exit_desc(output)));
    }

    // Exit 0 with empty stdout but stderr noise: report it instead of
    // pretending the list is legitimately empty.
    let detail = stderr.lines().map(str::trim).find(|line| !line.is_empty());
    if stdout.trim().is_empty()
        && let Some(detail) = detail
    {
        return Some(format!("no output on stdout: {detail}"));
    }

    None
}

pub(crate) fn targets_from_output(output: &Output) -> io::Result<Targets> {
    if let Some(msg) = command_failure(output) {
        return Err(io::Error::other(msg));
    }
    Ok(parsers::parse_overlays(&output.stdout))
}

fn run_su(bin: &OsStr, script: &str) -> io::Result<Output> {
    Command::new(bin)
        .arg("-c")
        .arg(script)
        .output()
        .map_err(|err| launch_error(bin, err))
}

fn shell_quote(value: &str) -> String {
    let safe = !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-'));
    if safe {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{os::unix::process::ExitStatusExt, process::Output};

    /// Byte-exact capture from a real Android device.
    const DEVICE_FAILURE: &str = include_str!("../tests/fixtures/device_cmd_failure.txt");
    const CLEAN_LIST: &str = include_str!("../tests/fixtures/cmd_overlay_list_ok.txt");

    /// `ExitStatus::from_raw` takes a Unix wait status: `(code << 8)`.
    fn output(raw: i32, stdout: &str, stderr: &str) -> Output {
        Output {
            status: std::process::ExitStatus::from_raw(raw),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn clean_list_is_parsed() {
        let out = output(0, CLEAN_LIST, "");
        let targets = targets_from_output(&out).expect("clean output must parse");
        assert_eq!(targets.len(), 3);
    }

    #[test]
    fn device_failure_is_an_error_even_on_exit_zero() {
        let out = output(0, DEVICE_FAILURE, "");
        let err = targets_from_output(&out).expect_err("binder failure must not parse as a list");
        assert!(err.to_string().contains("Failure calling service overlay"));
        assert!(err.to_string().contains("2147483646"));
    }

    #[test]
    fn device_failure_on_stderr_is_detected() {
        let out = output(256, "", DEVICE_FAILURE);
        let err = targets_from_output(&out).expect_err("stderr failure must be reported");
        assert!(err.to_string().contains("Failure calling service overlay"));
    }

    #[test]
    fn unknown_service_is_detected() {
        let out = output(0, "", "cmd: Can't find service overlay\n");
        let err = command_failure(&out).expect("missing service must be an error");
        assert!(err.contains("Can't find service overlay"));
    }

    #[test]
    fn nonzero_exit_without_marker_reports_code_and_detail() {
        let out = output(256, "", "sh: cmd: not found\n");
        let err = command_failure(&out).expect("non-zero exit must be an error");
        assert!(err.contains("exit code 1"), "got: {err}");
        assert!(err.contains("cmd: not found"), "got: {err}");
    }

    #[test]
    fn success_with_empty_output_is_not_an_error() {
        let out = output(0, "", "");
        assert!(command_failure(&out).is_none());
        assert!(targets_from_output(&out).unwrap().is_empty());
    }

    #[test]
    fn silent_stdout_with_stderr_noise_is_reported() {
        let out = output(0, "", "cat: /missing: No such file or directory\n");
        let err = command_failure(&out).expect("stderr noise must surface");
        assert!(err.to_string().contains("No such file or directory"));
    }

    #[test]
    fn shell_quote_passes_identifiers_through() {
        assert_eq!(
            shell_quote("com.acme.overlay:dark"),
            "com.acme.overlay:dark"
        );
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(shell_quote("x'y"), r"'x'\''y'");
    }

    #[test]
    #[cfg(feature = "virtual")]
    fn bundled_virtual_list_is_the_real_device_capture() {
        let mut backend = Backend::default();
        assert!(backend.is_virtual());
        assert_eq!(backend.list().unwrap().len(), 31);
    }

    #[test]
    #[cfg(feature = "virtual")]
    fn virtual_toggle_applies_in_memory() {
        let mut backend = Backend::default();

        backend
            .set_overlay(false, "android.auto_generated_rro_vendor__")
            .unwrap();
        let targets = backend.list().unwrap();
        let android = &targets["android"];
        assert!(
            !android
                .enabled
                .iter()
                .any(|n| n == "android.auto_generated_rro_vendor__")
        );
        assert!(
            android
                .disabled
                .iter()
                .any(|n| n == "android.auto_generated_rro_vendor__")
        );

        backend
            .set_overlay(true, "android.auto_generated_rro_vendor__")
            .unwrap();
        let targets = backend.list().unwrap();
        assert!(
            targets["android"]
                .enabled
                .iter()
                .any(|n| n == "android.auto_generated_rro_vendor__")
        );
    }

    #[test]
    #[cfg(feature = "virtual")]
    fn virtual_toggle_rejects_unknown_and_broken_overlays() {
        let mut backend = Backend::default();
        let err = backend
            .set_overlay(true, "com.acme.does.not.exist")
            .unwrap_err();
        assert!(err.to_string().contains("not found"));

        let err = backend
            .set_overlay(true, "android.auto_generated_characteristics_rro")
            .unwrap_err();
        assert!(err.to_string().contains("broken"), "got: {err}");
    }

    #[test]
    #[cfg(feature = "virtual")]
    fn backend_selection_priority() {
        let targets_file =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/virtual_targets.txt");

        let mut backend = Backend::from_env_impl(
            |key| match key {
                TARGETS_FILE_ENV => Some(targets_file.clone().into_os_string()),
                SU_ENV_VAR => Some("/x/su".into()),
                _ => None,
            },
            true,
        );
        assert!(backend.is_virtual());
        assert_eq!(backend.list().unwrap().len(), 31);

        let backend = Backend::from_env_impl(|_| None, true);
        assert!(matches!(backend, Backend::Su(_)));

        let backend =
            Backend::from_env_impl(|key| (key == SU_ENV_VAR).then(|| "/x/su".into()), false);
        assert!(matches!(backend, Backend::Su(_)));

        let backend = Backend::from_env_impl(|_| None, false);
        assert!(backend.is_virtual());
    }

    #[test]
    #[cfg(not(feature = "virtual"))]
    fn backend_is_always_su() {
        let backend =
            Backend::from_env_impl(|key| (key == SU_ENV_VAR).then(|| "/x/su".into()), false);
        assert!(matches!(backend, Backend::Su(_)));

        let backend = Backend::from_env_impl(|_| None, true);
        assert!(matches!(backend, Backend::Su(_)));
        assert!(!backend.is_virtual());
    }

    #[test]
    #[cfg(feature = "virtual")]
    fn missing_targets_file_is_a_friendly_error() {
        let mut backend = Backend::from_env_impl(
            |key| (key == TARGETS_FILE_ENV).then(|| "/nonexistent/list.txt".into()),
            false,
        );
        assert!(backend.is_virtual());
        let err = backend.list().unwrap_err();
        assert!(err.to_string().contains("virtual target list"));
        assert!(err.to_string().contains("/nonexistent/list.txt"));
        // The path is retained, not consumed: `r` retries and fails again.
        assert!(backend.list().is_err());
    }
}
