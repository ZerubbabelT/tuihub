use std::io::Stdout;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::actions::suspend_tui_for_command;
use super::state::{App, ConfirmAction, LogLevel};
use crate::registry::model::AppEntry;
use crate::system::exec::{command_for_platform, run_install_cmd};
use crate::system::os::Platform;
use crate::system::tmux::{has_tmux, launch_in_tmux, tmux_install_hint};
use crate::ui::draw::ui;

pub fn refresh_filter(app: &mut App) {
    app.filtered_indices = app
        .entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| app.matches_tab(entry))
        .filter(|(_, entry)| app.matches_search(entry))
        .map(|(index, _)| index)
        .collect();

    let new_idx = match app.list_state.selected() {
        Some(idx) if idx < app.filtered_indices.len() => Some(idx),
        _ if app.filtered_indices.is_empty() => None,
        _ => Some(0),
    };
    app.list_state.select(new_idx);
}

pub fn cycle_tab_right(app: &mut App) {
    const TABS: [&str; 3] = ["All", "Installed", "Categories"];
    app.selected_tab = (app.selected_tab + 1) % TABS.len();
    refresh_filter(app);
}

pub fn cycle_tab_left(app: &mut App) {
    const TABS: [&str; 3] = ["All", "Installed", "Categories"];
    app.selected_tab = if app.selected_tab == 0 {
        TABS.len() - 1
    } else {
        app.selected_tab - 1
    };
    refresh_filter(app);
}

pub fn category_right(app: &mut App) {
    if app.selected_tab != 2 || app.categories.is_empty() {
        return;
    }
    app.selected_category = (app.selected_category + 1) % app.categories.len();
    refresh_filter(app);
}

pub fn category_left(app: &mut App) {
    if app.selected_tab != 2 || app.categories.is_empty() {
        return;
    }
    app.selected_category = if app.selected_category == 0 {
        app.categories.len() - 1
    } else {
        app.selected_category - 1
    };
    refresh_filter(app);
}

pub fn run(app: &mut App, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    loop {
        terminal.draw(|frame| ui(frame, app))?;

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
                        refresh_filter(app);
                    }
                    KeyCode::Char(c) => {
                        if !key.modifiers.contains(KeyModifiers::CONTROL) {
                            app.search_input.push(c);
                            refresh_filter(app);
                        }
                    }
                    _ => {}
                }
                continue;
            }

            if app.confirm_mode {
                match key.code {
                    KeyCode::Enter => {
                        if app.confirm_selected {
                            if let Some(ConfirmAction::Uninstall(targets)) =
                                app.confirm_action.clone()
                            {
                                app.confirm_mode = false;
                                app.confirm_action = None;

                                for target in targets {
                                    let uninstall_cmd =
                                        match command_for_platform(&target.uninstall, app.platform)
                                        {
                                            Some(cmd) => cmd.to_string(),
                                            None => continue,
                                        };
                                    app.set_status(format!(
                                        "Uninstalling {} using: {}",
                                        target.name, uninstall_cmd
                                    ));

                                    let message = format!(
                                        "About to run uninstall command for {}.\n\nCommand:\n{}\n\nIf sudo asks for password, type normally.",
                                        target.name, uninstall_cmd
                                    );

                                    let result =
                                        suspend_tui_for_command(terminal, &message, || {
                                            run_install_cmd(&uninstall_cmd, app.platform)
                                        });

                                    match result {
                                        Ok(_) => {
                                            app.log(
                                                format!("Uninstalled {}", target.name),
                                                LogLevel::Success,
                                            );
                                            app.set_status(format!(
                                                "Uninstalled {} successfully.",
                                                target.name
                                            ))
                                        }
                                        Err(e) => {
                                            app.log(format!("Error: {}", e), LogLevel::Error);
                                            app.set_status(format!(
                                                "Uninstall failed for {}: {}",
                                                target.name, e
                                            ))
                                        }
                                    }
                                }
                                app.refresh_installed_cache();
                                refresh_filter(app);
                            }
                        } else {
                            app.confirm_mode = false;
                            app.confirm_action = None;
                            app.set_status("Uninstall cancelled.");
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.confirm_selected = true;
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        app.confirm_selected = false;
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.confirm_mode = false;
                        app.confirm_action = None;
                        app.set_status("Uninstall cancelled.");
                    }
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Tab => cycle_tab_right(app),
                KeyCode::BackTab => cycle_tab_left(app),
                KeyCode::Left => category_left(app),
                KeyCode::Right => category_right(app),
                KeyCode::Char(' ') => app.toggle_selected_current(),
                KeyCode::Char('/') => {
                    app.search_mode = true;
                }
                KeyCode::Esc => {
                    if !app.search_input.is_empty() {
                        app.search_input.clear();
                        refresh_filter(app);
                        app.set_status("Search cleared.");
                    }
                }
                KeyCode::Char('c') | KeyCode::Char('C') => app.clear_selection(),
                KeyCode::Enter | KeyCode::Char('\r') => {
                    let idx = match app.list_state.selected() {
                        Some(i) => i,
                        None => {
                            app.set_status("No app focused to launch.");
                            continue;
                        }
                    };
                    let entry_idx = match app.filtered_indices.get(idx) {
                        Some(&idx) => idx,
                        None => {
                            app.set_status("No app focused to launch.");
                            continue;
                        }
                    };
                    let target = match app.entries.get(entry_idx) {
                        Some(entry) => entry,
                        None => {
                            app.set_status("No app focused to launch.");
                            continue;
                        }
                    };

                    let target_name = target.name.clone();

                    if !has_tmux() {
                        app.set_status(format!(
                            "tmux is required for launch. {}",
                            tmux_install_hint(app.platform)
                        ));
                        continue;
                    }

                    if !app.is_installed(&target) {
                        app.set_status(format!(
                            "{} is not installed. Press I to install.",
                            target_name
                        ));
                        continue;
                    }

                    match launch_in_tmux(&target) {
                        Ok(target_loc) => {
                            if let Some(session_name) = target_loc.strip_prefix("session:") {
                                app.log(
                                    format!("Session '{}' opened", session_name),
                                    LogLevel::Info,
                                );
                                app.set_status(format!(
                                    "Launched {} in tmux session '{}'. Attach: tmux attach -t {}",
                                    target_name, session_name, session_name
                                ));
                            } else if let Some(window_name) = target_loc.strip_prefix("window:") {
                                app.log(format!("Window '{}' opened", window_name), LogLevel::Info);
                                app.set_status(format!(
                                    "Launched {} in tmux window '{}'.",
                                    target_name, window_name
                                ));
                            } else {
                                app.log(format!("Launched {}", target_name), LogLevel::Info);
                                app.set_status(format!("Launched {} in tmux.", target_name));
                            }
                        }
                        Err(e) => {
                            app.log(format!("Error: {}", e), LogLevel::Error);
                            app.set_status(format!("Launch failed for {}: {}", target_name, e))
                        }
                    }
                }
                KeyCode::Char('i') | KeyCode::Char('I') => {
                    let targets = app.selected_entries();
                    if targets.is_empty() {
                        app.set_status("No app selected to install.");
                        continue;
                    }

                    if app.platform == Platform::Unknown {
                        app.set_status("Unknown platform. Cannot install.");
                        continue;
                    }

                    for target in targets {
                        if app.is_installed(&target) {
                            app.set_status(format!("{} already installed", target.name));
                            app.log(format!("{} already installed", target.name), LogLevel::Info);
                            continue;
                        }

                        let install_cmd = match command_for_platform(&target.install, app.platform)
                        {
                            Some(cmd) if !cmd.trim().is_empty() => cmd.to_string(),
                            _ => {
                                app.set_status(format!(
                                    "No install command defined for {} on {}.",
                                    target.name,
                                    app.platform.label()
                                ));
                                continue;
                            }
                        };
                        app.set_status(format!(
                            "Installing {} using: {}",
                            target.name, install_cmd
                        ));

                        let message = format!(
                            "About to run install command for {}.\n\nCommand:\n{}\n\nIf sudo asks for password, type normally.",
                            target.name, install_cmd
                        );

                        let result = suspend_tui_for_command(terminal, &message, || {
                            run_install_cmd(&install_cmd, app.platform)
                        });

                        match result {
                            Ok(_) => {
                                app.log(format!("Installed {}", target.name), LogLevel::Success);
                                app.set_status(format!("Installed {} successfully.", target.name))
                            }
                            Err(e) => {
                                app.log(format!("Error: {}", e), LogLevel::Error);
                                app.set_status(format!("Install failed for {}: {}", target.name, e))
                            }
                        }
                    }
                    app.refresh_installed_cache();
                    refresh_filter(app);
                }
                KeyCode::Char('u') | KeyCode::Char('U') => {
                    let targets = app.selected_entries();
                    if targets.is_empty() {
                        app.set_status("No app selected to uninstall.");
                        continue;
                    }

                    if app.platform == Platform::Unknown {
                        app.set_status("Unknown platform. Cannot uninstall.");
                        continue;
                    }

                    let installed_targets: Vec<_> = targets
                        .iter()
                        .filter(|target| app.is_installed(target))
                        .filter(|target| {
                            if let Some(cmd) = command_for_platform(&target.uninstall, app.platform)
                            {
                                !cmd.trim().is_empty()
                            } else {
                                false
                            }
                        })
                        .cloned()
                        .collect();

                    if installed_targets.is_empty() {
                        let not_installed: Vec<_> = targets
                            .iter()
                            .filter(|t| !app.is_installed(t))
                            .map(|t| t.name.clone())
                            .collect();
                        if !not_installed.is_empty() {
                            app.set_status(format!("{} not installed.", not_installed.join(", ")));
                            app.log(
                                format!("{} not installed", not_installed.join(", ")),
                                LogLevel::Info,
                            );
                        } else {
                            app.set_status(
                                "No uninstall command defined for selected apps on this platform.",
                            );
                        }
                        continue;
                    }

                    app.confirm_mode = true;
                    app.confirm_selected = true;
                    app.confirm_action = Some(ConfirmAction::Uninstall(installed_targets));
                    app.set_status("Press Enter to confirm uninstall, Esc to cancel.");
                }
                KeyCode::Char('l') | KeyCode::Char('L') => {
                    let targets: Vec<AppEntry> = if app.selected_ids.is_empty() {
                        match app.list_state.selected() {
                            Some(idx) => app
                                .filtered_indices
                                .get(idx)
                                .and_then(|&entry_idx| app.entries.get(entry_idx))
                                .cloned()
                                .into_iter()
                                .collect(),
                            None => vec![],
                        }
                    } else {
                        app.selected_entries()
                    };

                    if targets.is_empty() {
                        app.set_status("No app selected or focused to launch.");
                        continue;
                    }

                    if !has_tmux() {
                        app.set_status(format!(
                            "tmux is required for launch. {}",
                            tmux_install_hint(app.platform)
                        ));
                        continue;
                    }

                    for target in targets {
                        let target_name = target.name.clone();
                        if !app.is_installed(&target) {
                            app.set_status(format!(
                                "{} is not installed yet. Install first.",
                                target_name
                            ));
                            app.log(format!("{} not installed", target_name), LogLevel::Info);
                            continue;
                        }

                        match launch_in_tmux(&target) {
                            Ok(target_loc) => {
                                if let Some(session_name) = target_loc.strip_prefix("session:") {
                                    app.log(
                                        format!("Session '{}' opened", session_name),
                                        LogLevel::Info,
                                    );
                                    app.set_status(format!(
                                        "Launched {} in tmux session '{}'. Attach: tmux attach -t {}",
                                        target_name, session_name, session_name
                                    ));
                                } else if let Some(window_name) = target_loc.strip_prefix("window:")
                                {
                                    app.log(
                                        format!("Window '{}' opened", window_name),
                                        LogLevel::Info,
                                    );
                                    app.set_status(format!(
                                        "Launched {} in tmux window '{}'.",
                                        target_name, window_name
                                    ));
                                } else {
                                    app.log(format!("Launched {}", target_name), LogLevel::Info);
                                    app.set_status(format!("Launched {} in tmux.", target_name));
                                }
                            }
                            Err(e) => {
                                app.log(format!("Error: {}", e), LogLevel::Error);
                                app.set_status(format!("Launch failed for {}: {}", target_name, e))
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
