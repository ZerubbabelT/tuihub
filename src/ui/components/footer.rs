use ratatui::{
    layout::Rect,
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::state::{App, LogLevel};
use crate::system::os::platform_label;
use crate::ui::theme::*;

pub fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let now = std::time::Instant::now();
    app.logs
        .retain(|l| now.duration_since(l.created_at) < std::time::Duration::from_secs(3));

    let installed_total = app.installed_ids.len();
    let selected_total = app.selected_ids.len();
    let visible_total = app.filtered_indices.len();

    let mut second_line: Vec<Span> = vec![
        Span::styled("Actions ", Style::default().fg(C_MUTED)),
        Span::styled(
            "Enter Quick Launch",
            Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            "I Install",
            Style::default().fg(C_SUCCESS).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            "L Launch",
            Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            "U Uninstall",
            Style::default().fg(C_WARNING).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   |   ", Style::default().fg(C_PANEL)),
        Span::styled(
            format!(
                "visible:{} selected:{} installed:{} [{}]",
                visible_total,
                selected_total,
                installed_total,
                platform_label(app.platform)
            ),
            Style::default().fg(C_MUTED),
        ),
    ];

    for l in &app.logs {
        let color = match l.level {
            LogLevel::Success => C_SUCCESS,
            LogLevel::Error => C_WARNING,
            LogLevel::Info => C_PRIMARY,
        };
        second_line.push(Span::styled("  ", Style::default()));
        second_line.push(Span::styled(l.message.clone(), Style::default().fg(color)));
    }

    let footer_lines = vec![
        Line::from(vec![
            Span::styled("Move ", Style::default().fg(C_MUTED)),
            Span::styled(
                "↑/↓ j/k",
                Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Tabs ", Style::default().fg(C_MUTED)),
            Span::styled(
                "Tab/Shift+Tab",
                Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Category ", Style::default().fg(C_MUTED)),
            Span::styled(
                "←/→",
                Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Search ", Style::default().fg(C_MUTED)),
            Span::styled(
                "/",
                Style::default().fg(C_PRIMARY).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Select ", Style::default().fg(C_MUTED)),
            Span::styled(
                "Space",
                Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Clear ", Style::default().fg(C_MUTED)),
            Span::styled(
                "C",
                Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Quit ", Style::default().fg(C_MUTED)),
            Span::styled(
                "Q",
                Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(second_line),
    ];
    let footer = Paragraph::new(footer_lines).block(
        Block::default()
            .title(" Command Bar ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(C_PANEL)),
    );
    frame.render_widget(footer, area);
}
