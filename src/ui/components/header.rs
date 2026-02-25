use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::ui::theme::*;

#[allow(dead_code)]
pub fn render_header(frame: &mut Frame<'_>, area: Rect) {
    let title = Paragraph::new(" TUIHub ")
        .style(Style::default().fg(C_TEXT))
        .block(
            Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Rounded)
                .border_style(Style::default().fg(C_PANEL)),
        );
    frame.render_widget(title, area);
}
