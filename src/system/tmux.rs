use std::process::Command;

use anyhow::{Context, Result};
use chrono::Utc;

use super::os::Platform;
use crate::registry::model::AppEntry;

pub fn has_tmux() -> bool {
    which::which("tmux").is_ok()
}

pub fn in_tmux_session() -> bool {
    std::env::var("TMUX")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

pub fn tmux_install_hint(platform: Platform) -> &'static str {
    match platform {
        Platform::Linux | Platform::Wsl => "Install tmux: sudo apt install tmux",
        Platform::Mac => "Install tmux: brew install tmux",
        Platform::Windows => "Install tmux in WSL, then run TUIHub inside WSL terminal.",
        Platform::Unknown => "Install tmux for your platform.",
    }
}

pub fn sanitize_tmux_name(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "app".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn launch_in_tmux(entry: &AppEntry) -> Result<String> {
    let timestamp = Utc::now().timestamp();
    let safe_name = sanitize_tmux_name(&entry.id);

    if in_tmux_session() {
        let window_name = format!("th-{safe_name}-{timestamp}");
        let status = Command::new("tmux")
            .args(["new-window", "-n", &window_name, &entry.binary])
            .status()
            .context("failed to create tmux window")?;

        if !status.success() {
            anyhow::bail!("failed to create tmux window (status: {status})");
        }
        return Ok(format!("window:{window_name}"));
    }

    let session_name = format!("tuihub-{safe_name}-{timestamp}");
    let status = Command::new("tmux")
        .args(["new-session", "-d", "-s", &session_name, &entry.binary])
        .status()
        .context("failed to create tmux session")?;

    if !status.success() {
        anyhow::bail!("failed to create tmux session (status: {status})");
    }

    Ok(format!("session:{session_name}"))
}
