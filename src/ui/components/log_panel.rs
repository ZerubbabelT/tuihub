use ratatui::{layout::Rect, prelude::*, widgets::Paragraph, Frame};

use crate::app::state::{App, LogLevel};
use crate::ui::theme::*;

#[allow(dead_code)]
pub fn render_log_panel(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let now = std::time::Instant::now();
    app.logs
        .retain(|l| now.duration_since(l.created_at) < std::time::Duration::from_secs(3));

    if app.logs.is_empty() {
        return;
    }

    let log_lines: Vec<Line> = app
        .logs
        .iter()
        .map(|log| {
            let color = match log.level {
                LogLevel::Success => C_SUCCESS,
                LogLevel::Error => C_WARNING,
                LogLevel::Info => C_PRIMARY,
            };
            Line::from(Span::styled(
                log.message.clone(),
                Style::default().fg(color),
            ))
        })
        .collect();

    let log_widget = Paragraph::new(log_lines).style(Style::default().fg(C_TEXT));

    frame.render_widget(log_widget, area);
}
