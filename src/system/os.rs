use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Wsl,
    Mac,
    Windows,
    Unknown,
}

impl Platform {
    pub fn detect() -> Self {
        if cfg!(target_os = "windows") {
            return Self::Windows;
        }
        if cfg!(target_os = "macos") {
            return Self::Mac;
        }
        if is_wsl() {
            return Self::Wsl;
        }
        if cfg!(target_os = "linux") {
            return Self::Linux;
        }
        Self::Unknown
    }

    pub fn label(&self) -> &'static str {
        match self {
            Platform::Linux => "Linux",
            Platform::Wsl => "WSL",
            Platform::Mac => "macOS",
            Platform::Windows => "Windows",
            Platform::Unknown => "Unknown",
        }
    }
}

pub fn is_wsl() -> bool {
    if std::env::var("WSL_DISTRO_NAME").is_ok() || std::env::var("WSL_INTEROP").is_ok() {
        return true;
    }

    if let Ok(version) = fs::read_to_string("/proc/version") {
        return version.to_ascii_lowercase().contains("microsoft");
    }

    false
}

pub fn platform_label(platform: Platform) -> &'static str {
    match platform {
        Platform::Linux => "Linux",
        Platform::Wsl => "WSL",
        Platform::Mac => "macOS",
        Platform::Windows => "Windows",
        Platform::Unknown => "Unknown",
    }
}
