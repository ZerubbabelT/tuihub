use std::{
    collections::{BTreeSet, HashSet},
    fs,
    io::{self, Stdout},
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{Context, Result};
use chrono::Utc;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use serde::Deserialize;
use which::which;

const TABS: [&str; 3] = ["All", "Installed", "Categories"];

#[derive(Debug, Clone, Deserialize)]
struct AppEntry {
    id: String,
    name: String,
    description: String,
    category: String,
    repo: String,
    binary: String,
    install: InstallCommands,
    uninstall: InstallCommands,
}

#[derive(Debug, Clone, Deserialize)]
struct InstallCommands {
    linux: String,
    wsl: String,
    mac: String,
    windows: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    Linux,
    Wsl,
    Mac,
    Windows,
}

impl Platform {
    fn detect() -> Self {
        if cfg!(target_os = "windows") {
            return Self::Windows;
        }
        if cfg!(target_os = "macos") {
            return Self::Mac;
        }
        if is_wsl() {
            return Self::Wsl;
        }
        Self::Linux
    }
}

fn is_wsl() -> bool {
    if std::env::var("WSL_DISTRO_NAME").is_ok() || std::env::var("WSL_INTEROP").is_ok() {
        return true;
    }

    if let Ok(version) = fs::read_to_string("/proc/version") {
        return version.to_ascii_lowercase().contains("microsoft");
    }

    false
}

struct App {
    entries: Vec<AppEntry>,
    selected_tab: usize,
    categories: Vec<String>,
    selected_category: usize,
    filtered_indices: Vec<usize>,
    list_state: ListState,
    selected_ids: HashSet<String>,
    search_mode: bool,
    search_input: String,
    status: String,
    platform: Platform,
}

impl App {
    fn new(entries: Vec<AppEntry>) -> Self {
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
            selected_tab: 0,
            categories,
            selected_category: 0,
            filtered_indices: Vec::new(),
            list_state: ListState::default(),
            selected_ids: HashSet::new(),
            search_mode: false,
            search_input: String::new(),
            status: "Ready. Use arrows/jk to navigate, Space to select, I install, L launch, / search.".to_string(),
            platform: Platform::detect(),
        };
        app.refresh_filter();
        app
    }

    fn refresh_filter(&mut self) {
        self.filtered_indices = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| self.matches_tab(entry))
            .filter(|(_, entry)| self.matches_search(entry))
            .map(|(index, _)| index)
            .collect();

        let new_idx = match self.list_state.selected() {
            Some(idx) if idx < self.filtered_indices.len() => Some(idx),
            _ if self.filtered_indices.is_empty() => None,
            _ => Some(0),
        };
        self.list_state.select(new_idx);
    }

    fn matches_tab(&self, entry: &AppEntry) -> bool {
        match self.selected_tab {
            0 => true,
            1 => is_installed(entry),
            2 => self
                .categories
                .get(self.selected_category)
                .map(|cat| entry.category.eq_ignore_ascii_case(cat))
                .unwrap_or(true),
            _ => true,
        }
    }

    fn matches_search(&self, entry: &AppEntry) -> bool {
        if self.search_input.trim().is_empty() {
            return true;
        }
        let needle = self.search_input.to_ascii_lowercase();
        entry.name.to_ascii_lowercase().contains(&needle)
            || entry.description.to_ascii_lowercase().contains(&needle)
            || entry.category.to_ascii_lowercase().contains(&needle)
            || entry.id.to_ascii_lowercase().contains(&needle)
    }

    fn current_entry(&self) -> Option<&AppEntry> {
        let idx = self.list_state.selected()?;
        let entry_idx = *self.filtered_indices.get(idx)?;
        self.entries.get(entry_idx)
    }

    fn move_down(&mut self) {
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

    fn move_up(&mut self) {
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

    fn toggle_selected_current(&mut self) {
        let Some(entry_id) = self.current_entry().map(|entry| entry.id.clone()) else {
            return;
        };

        if self.selected_ids.contains(&entry_id) {
            self.selected_ids.remove(&entry_id);
        } else {
            self.selected_ids.insert(entry_id);
        }
    }

    fn cycle_tab_right(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % TABS.len();
        self.refresh_filter();
    }

    fn cycle_tab_left(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            TABS.len() - 1
        } else {
            self.selected_tab - 1
        };
        self.refresh_filter();
    }

    fn category_right(&mut self) {
        if self.selected_tab != 2 || self.categories.is_empty() {
            return;
        }
        self.selected_category = (self.selected_category + 1) % self.categories.len();
        self.refresh_filter();
    }

    fn category_left(&mut self) {
        if self.selected_tab != 2 || self.categories.is_empty() {
            return;
        }
        self.selected_category = if self.selected_category == 0 {
            self.categories.len() - 1
        } else {
            self.selected_category - 1
        };
        self.refresh_filter();
    }

    fn selected_entries(&self) -> Vec<AppEntry> {
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

    fn set_status<S: Into<String>>(&mut self, message: S) {
        self.status = message.into();
    }
}

fn command_for_platform(commands: &InstallCommands, platform: Platform) -> &str {
    match platform {
        Platform::Linux => &commands.linux,
        Platform::Wsl => &commands.wsl,
        Platform::Mac => &commands.mac,
        Platform::Windows => &commands.windows,
    }
}

fn shell_for_platform(platform: Platform) -> (&'static str, &'static str) {
    match platform {
        Platform::Windows => ("cmd", "/C"),
        _ => ("sh", "-lc"),
    }
}

fn is_installed(entry: &AppEntry) -> bool {
    which(&entry.binary).is_ok()
}

fn load_entries(path: impl AsRef<Path>) -> Result<Vec<AppEntry>> {
    let file = fs::read_to_string(path.as_ref())
        .with_context(|| format!("failed to read {}", path.as_ref().display()))?;
    let entries: Vec<AppEntry> = serde_json::from_str(&file)
        .with_context(|| format!("invalid json in {}", path.as_ref().display()))?;
    Ok(entries)
}

fn run_install_cmd(cmd: &str, platform: Platform) -> Result<()> {
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

fn launch_in_tmux(entry: &AppEntry) -> Result<String> {
    which("tmux").context("tmux is not installed or not in PATH")?;

    let timestamp = Utc::now().timestamp();
    let mut session_name = format!("tuihub-{}-{timestamp}", entry.id);
    session_name = session_name.replace(' ', "-");

    let status = Command::new("tmux")
        .args([
            "new-session",
            "-d",
            "-s",
            &session_name,
            &format!("{}", entry.binary),
        ])
        .status()
        .context("failed to create tmux session")?;

    if !status.success() {
        anyhow::bail!("failed to create tmux session (status: {status})");
    }

    Ok(session_name)
}

fn ui(frame: &mut Frame<'_>, app: &mut App) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(if app.selected_tab == 2 { 3 } else { 0 }),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let tab_titles = TABS
        .iter()
        .map(|title| Line::from(*title).centered())
        .collect::<Vec<_>>();

    let tabs = Tabs::new(tab_titles)
        .select(app.selected_tab)
        .block(
            Block::default()
                .title(" TUIHub ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );

    frame.render_widget(tabs, vertical[0]);

    if app.selected_tab == 2 {
        let category_titles = app
            .categories
            .iter()
            .map(|c| Line::from(c.to_string()))
            .collect::<Vec<_>>();
        let cat_tabs = Tabs::new(category_titles)
            .select(app.selected_category)
            .block(
                Block::default()
                    .title(" Categories ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().fg(Color::Gray))
            .highlight_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
        frame.render_widget(cat_tabs, vertical[1]);
    }

    let search_title = if app.search_mode {
        " Search (/): typing... Enter to apply, Esc to cancel "
    } else {
        " Search (/ to start) "
    };

    let search = Paragraph::new(app.search_input.as_str())
        .block(
            Block::default()
                .title(search_title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(if app.search_mode {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    frame.render_widget(search, vertical[2]);

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .filter_map(|idx| app.entries.get(*idx))
        .map(|entry| {
            let installed = is_installed(entry);
            let selected = app.selected_ids.contains(&entry.id);
            let checkbox = if selected { "[x]" } else { "[ ]" };
            let install_badge = if installed {
                "installed"
            } else {
                "not-installed"
            };

            let line = format!(
                "{checkbox} {:<14} {:<14} {:<14} {}",
                entry.name, entry.category, install_badge, entry.description
            );

            let style = if installed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let app_list = List::new(items)
        .block(
            Block::default()
                .title(" TUI Catalog ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().bg(Color::Blue).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(app_list, vertical[3], &mut app.list_state);

    let details = if let Some(entry) = app.current_entry() {
        let install_cmd = command_for_platform(&entry.install, app.platform);
        let uninstall_cmd = command_for_platform(&entry.uninstall, app.platform);
        let installed = if is_installed(entry) { "yes" } else { "no" };

        format!(
            "name: {}\ncategory: {}\ninstalled: {}\nbinary: {}\nrepo: {}\ninstall: {}\nuninstall: {}",
            entry.name,
            entry.category,
            installed,
            entry.binary,
            entry.repo,
            install_cmd,
            uninstall_cmd
        )
    } else {
        "No apps match this filter/search.".to_string()
    };

    let details_widget = Paragraph::new(details)
        .block(
            Block::default()
                .title(" Details ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(details_widget, vertical[4]);

    let status = Paragraph::new(app.status.as_str())
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(status, vertical[5]);

    let help = Paragraph::new(
        "q quit | Tab/Shift+Tab tabs | ←/→ categories | ↑/↓ or j/k move | Space select | / search | I install | L launch | U uninstall",
    )
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::TOP));
    frame.render_widget(help, vertical[6]);

    if app.search_mode {
        let cursor_x = vertical[2].x + 1 + app.search_input.chars().count() as u16;
        let cursor_y = vertical[2].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

fn show_transient_message(terminal: &mut Terminal<CrosstermBackend<Stdout>>, msg: &str) -> Result<()> {
    terminal.draw(|frame| {
        let area = centered_rect(70, 20, frame.area());
        frame.render_widget(Clear, area);
        let block = Paragraph::new(msg)
            .style(Style::default().fg(Color::Yellow))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title(" Running External Command ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick),
            );
        frame.render_widget(block, area);
    })?;
    Ok(())
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn suspend_tui_for_command(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    msg: &str,
    f: impl FnOnce() -> Result<()>,
) -> Result<()> {
    show_transient_message(terminal, msg)?;
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    let run_result = f();

    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    run_result
}

fn run(mut app: App, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            if app.search_mode {
                match key.code {
                    KeyCode::Esc => {
                        app.search_mode = false;
                    }
                    KeyCode::Enter => {
                        app.search_mode = false;
                        app.set_status(format!("Search applied: '{}'", app.search_input));
                    }
                    KeyCode::Backspace => {
                        app.search_input.pop();
                        app.refresh_filter();
                    }
                    KeyCode::Char(c) => {
                        if !key.modifiers.contains(KeyModifiers::CONTROL) {
                            app.search_input.push(c);
                            app.refresh_filter();
                        }
                    }
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Tab => app.cycle_tab_right(),
                KeyCode::BackTab => app.cycle_tab_left(),
                KeyCode::Left => app.category_left(),
                KeyCode::Right => app.category_right(),
                KeyCode::Char(' ') => app.toggle_selected_current(),
                KeyCode::Char('/') => {
                    app.search_mode = true;
                    app.search_input.clear();
                }
                KeyCode::Char('i') | KeyCode::Char('I') => {
                    let targets = app.selected_entries();
                    if targets.is_empty() {
                        app.set_status("No app selected to install.");
                        continue;
                    }

                    for target in targets {
                        let install_cmd = command_for_platform(&target.install, app.platform).to_string();
                        app.set_status(format!("Installing {} using: {}", target.name, install_cmd));

                        let message = format!(
                            "About to run install command for {}.\n\nCommand:\n{}\n\nIf sudo asks for password, type normally.",
                            target.name, install_cmd
                        );

                        let result = suspend_tui_for_command(terminal, &message, || {
                            run_install_cmd(&install_cmd, app.platform)
                        });

                        match result {
                            Ok(_) => app.set_status(format!("Installed {} successfully.", target.name)),
                            Err(e) => app.set_status(format!("Install failed for {}: {}", target.name, e)),
                        }
                    }
                    app.refresh_filter();
                }
                KeyCode::Char('u') | KeyCode::Char('U') => {
                    let targets = app.selected_entries();
                    if targets.is_empty() {
                        app.set_status("No app selected to uninstall.");
                        continue;
                    }

                    for target in targets {
                        let uninstall_cmd = command_for_platform(&target.uninstall, app.platform).to_string();
                        app.set_status(format!("Uninstalling {} using: {}", target.name, uninstall_cmd));

                        let message = format!(
                            "About to run uninstall command for {}.\n\nCommand:\n{}\n\nIf sudo asks for password, type normally.",
                            target.name, uninstall_cmd
                        );

                        let result = suspend_tui_for_command(terminal, &message, || {
                            run_install_cmd(&uninstall_cmd, app.platform)
                        });

                        match result {
                            Ok(_) => app.set_status(format!("Uninstalled {} successfully.", target.name)),
                            Err(e) => app.set_status(format!("Uninstall failed for {}: {}", target.name, e)),
                        }
                    }
                    app.refresh_filter();
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    let targets = app.selected_entries();
                    if targets.is_empty() {
                        app.set_status("No app selected to launch.");
                        continue;
                    }

                    for target in targets {
                        if !is_installed(&target) {
                            app.set_status(format!("{} is not installed yet. Install first.", target.name));
                            continue;
                        }

                        match launch_in_tmux(&target) {
                            Ok(session_name) => app.set_status(format!(
                                "Launched {} in tmux session '{}'. Attach: tmux attach -t {}",
                                target.name, session_name, session_name
                            )),
                            Err(e) => app.set_status(format!("Launch failed for {}: {}", target.name, e)),
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let entries = load_entries("data/apps.json")?;
    let app = App::new(entries);

    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(io::stdout(), EnterAlternateScreen).context("failed to enter alt screen")?;

    struct TerminalGuard;
    impl Drop for TerminalGuard {
        fn drop(&mut self) {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
        }
    }
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).context("failed to init terminal")?;

    run(app, &mut terminal)
}
