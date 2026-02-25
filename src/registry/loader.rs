use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::model::AppEntry;

pub fn load_entries(path: impl AsRef<Path>) -> Result<Vec<AppEntry>> {
    let file = fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read {}", path.as_ref().display()))?;
    let entries: Vec<AppEntry> = serde_json::from_str(&file)
        .with_context(|| format!("invalid json in {}", path.as_ref().display()))?;
    Ok(entries)
}
