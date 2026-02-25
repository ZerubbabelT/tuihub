use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub repo: String,
    pub binary: String,
    pub install: InstallCommands,
    pub uninstall: InstallCommands,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallCommands {
    pub linux: String,
    pub wsl: String,
    pub mac: String,
    pub windows: String,
}
