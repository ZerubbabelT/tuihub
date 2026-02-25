use std::io::{self, Stdout};

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub fn suspend_tui_for_command(
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

fn show_transient_message(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    msg: &str,
) -> Result<()> {
    use crate::ui::layout::centered_rect;
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

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
