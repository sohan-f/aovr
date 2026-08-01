use std::collections::HashMap;

#[derive(Debug, Default)]
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
//                          ^ target package

pub fn parse_overlays(byte: &[u8]) -> Targets {
    let input = String::from_utf8_lossy(byte).to_string();

    let mut map: Targets = HashMap::new();
    let mut target = String::new();

    for line in input.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if line.starts_with("[x] ") {
            map.entry(target.clone())
                .or_default()
                .enabled
                .push(line.trim_start_matches("[x] ").to_string());
        } else if line.starts_with("[ ] ") {
            map.entry(target.clone())
                .or_default()
                .disabled
                .push(line.trim_start_matches("[ ] ").to_string());
        } else if line.starts_with("--- ") {
            map.entry(target.clone())
                .or_default()
                .broken
                .push(line.trim_start_matches("--- ").to_string());
        } else {
            target = line.to_string();
        }
    }

    map
}
