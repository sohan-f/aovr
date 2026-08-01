use std::{io, process::Command};

use crate::parsers::{self, Targets};

pub fn load_targets() -> io::Result<Targets> {
    let output = Command::new("/system/bin/su")
        .arg("-c")
        .args(["cmd", "overlay", "list"])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "cmd overlay list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(parsers::parse_overlays(&output.stdout))
}

pub fn set_overlay(enabled: bool, overlay: &str) -> io::Result<()> {
    let action = if enabled { "enable" } else { "disable" };
    let cmd = format!("cmd overlay {action} {overlay}");

    let status = Command::new("/system/bin/su").arg("-c").arg(cmd).status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "overlay command failed for {overlay}"
        )))
    }
}
