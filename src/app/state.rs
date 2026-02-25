use std::collections::{BTreeSet, HashSet};

use ratatui::widgets::ListState;

use crate::registry::model::AppEntry;
use crate::system::exec::is_binary_installed;
use crate::system::os::Platform;

#[derive(Clone)]
pub enum ConfirmAction {
    Uninstall(Vec<AppEntry>),
}

#[derive(Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
    pub created_at: std::time::Instant,
}

#[derive(Clone, Copy)]
pub enum LogLevel {
    Success,
    Error,
    Info,
}

pub struct App {
    pub entries: Vec<AppEntry>,
    pub installed_ids: HashSet<String>,
    pub selected_tab: usize,
    pub categories: Vec<String>,
    pub selected_category: usize,
    pub filtered_indices: Vec<usize>,
    pub list_state: ListState,
    pub selected_ids: HashSet<String>,
    pub search_mode: bool,
    pub search_input: String,
    pub status: String,
    pub platform: Platform,
    pub confirm_mode: bool,
    pub confirm_action: Option<ConfirmAction>,
    pub confirm_selected: bool,
    pub logs: Vec<LogEntry>,
}

impl App {
    pub fn new(entries: Vec<AppEntry>) -> Self {
        let mut categories: Vec<String> = entries
            .iter()
            .map(|entry| entry.category.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if categories.is_empty() {
            categories.push("uncategorized".to_string());
        }

        let mut app = Self {
            entries,
            installed_ids: HashSet::new(),
            selected_tab: 0,
            categories,
            selected_category: 0,
            filtered_indices: Vec::new(),
            list_state: ListState::default(),
            selected_ids: HashSet::new(),
            search_mode: false,
            search_input: String::new(),
            status: "Ready. Navigate with arrows/jk. Space select, I install, L launch, / search."
                .to_string(),
            platform: Platform::detect(),
            confirm_mode: false,
            confirm_action: None,
            confirm_selected: false,
            logs: Vec::new(),
        };
        app.refresh_installed_cache();
        app
    }

    pub fn log(&mut self, message: String, level: LogLevel) {
        let now = std::time::Instant::now();
        self.logs
            .retain(|l| now.duration_since(l.created_at) < std::time::Duration::from_secs(3));
        self.logs.push(LogEntry {
            message,
            level,
            created_at: now,
        });
        if self.logs.len() > 3 {
            self.logs.remove(0);
        }
    }

    pub fn refresh_installed_cache(&mut self) {
        self.installed_ids = self
            .entries
            .iter()
            .filter(|entry| is_binary_installed(&entry.binary))
            .map(|entry| entry.id.clone())
            .collect();
    }

    pub fn is_installed(&self, entry: &AppEntry) -> bool {
        self.installed_ids.contains(&entry.id)
    }

    pub fn current_entry(&self) -> Option<&AppEntry> {
        let idx = self.list_state.selected()?;
        let entry_idx = *self.filtered_indices.get(idx)?;
        self.entries.get(entry_idx)
    }

    pub fn move_down(&mut self) {
        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
            return;
        }

        let next = match self.list_state.selected() {
            Some(i) if i + 1 < self.filtered_indices.len() => i + 1,
            _ => 0,
        };
        self.list_state.select(Some(next));
    }

    pub fn move_up(&mut self) {
        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
            return;
        }

        let prev = match self.list_state.selected() {
            Some(0) | None => self.filtered_indices.len() - 1,
            Some(i) => i.saturating_sub(1),
        };
        self.list_state.select(Some(prev));
    }

    pub fn toggle_selected_current(&mut self) {
        let Some(entry_id) = self.current_entry().map(|entry| entry.id.clone()) else {
            return;
        };

        if self.selected_ids.contains(&entry_id) {
            self.selected_ids.remove(&entry_id);
        } else {
            self.selected_ids.insert(entry_id);
        }
    }

    pub fn selected_entries(&self) -> Vec<AppEntry> {
        let mut results: Vec<AppEntry> = self
            .entries
            .iter()
            .filter(|entry| self.selected_ids.contains(&entry.id))
            .cloned()
            .collect();

        if results.is_empty() {
            if let Some(entry) = self.current_entry() {
                results.push(entry.clone());
            }
        }

        results
    }

    pub fn set_status<S: Into<String>>(&mut self, message: S) {
        self.status = message.into();
    }

    pub fn clear_selection(&mut self) {
        self.selected_ids.clear();
    }

    pub fn matches_tab(&self, entry: &AppEntry) -> bool {
        match self.selected_tab {
            0 => true,
            1 => self.is_installed(entry),
            2 => self
                .categories
                .get(self.selected_category)
                .map(|cat| entry.category.eq_ignore_ascii_case(cat))
                .unwrap_or(true),
            _ => true,
        }
    }

    pub fn matches_search(&self, entry: &AppEntry) -> bool {
        if self.search_input.trim().is_empty() {
            return true;
        }
        let needle = self.search_input.to_ascii_lowercase();
        entry.name.to_ascii_lowercase().contains(&needle)
            || entry.description.to_ascii_lowercase().contains(&needle)
            || entry.category.to_ascii_lowercase().contains(&needle)
            || entry.id.to_ascii_lowercase().contains(&needle)
    }
}
