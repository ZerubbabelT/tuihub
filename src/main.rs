use std::io;

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod input;
mod registry;
mod system;
mod ui;
mod utils;

use app::{refresh_filter, run, App};
use registry::load_entries;

fn main() -> Result<()> {
    let entries = load_entries("data/apps.json")?;
    let mut app = App::new(entries);
    refresh_filter(&mut app);

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

    run(&mut app, &mut terminal)
}
