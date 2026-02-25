use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use which::which;

use super::os::Platform;
use crate::registry::model::InstallCommands;

pub fn command_for_platform(commands: &InstallCommands, platform: Platform) -> Option<&str> {
    let cmd = match platform {
        Platform::Linux => &commands.linux,
        Platform::Wsl => &commands.wsl,
        Platform::Mac => &commands.mac,
        Platform::Windows => &commands.windows,
        Platform::Unknown => return None,
    };
    if cmd.trim().is_empty() {
        None
    } else {
        Some(cmd)
    }
}

pub fn shell_for_platform(platform: Platform) -> (&'static str, &'static str) {
    match platform {
        Platform::Windows => ("cmd", "/C"),
        _ => ("sh", "-lc"),
    }
}

pub fn is_binary_installed(binary: &str) -> bool {
    which(binary).is_ok()
}

pub fn run_install_cmd(cmd: &str, platform: Platform) -> Result<()> {
    let (shell, arg) = shell_for_platform(platform);
    let status = Command::new(shell)
        .arg(arg)
        .arg(cmd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to execute install command: {cmd}"))?;

    if !status.success() {
        anyhow::bail!("command failed with status {status}");
    }

    Ok(())
}
