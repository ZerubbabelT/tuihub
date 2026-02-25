use ratatui::{
    layout::Rect,
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::state::App;
use crate::system::exec::command_for_platform;
use crate::ui::theme::*;

pub fn render_detail_panel(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let details_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(C_PANEL))
        .style(Style::default().bg(C_BG));
    let details_inner = details_block.inner(area);
    frame.render_widget(details_block, area);

    let details_lines = if let Some(entry) = app.current_entry() {
        let install_cmd = command_for_platform(&entry.install, app.platform);
        let uninstall_cmd = command_for_platform(&entry.uninstall, app.platform);
        let installed = app.is_installed(entry);

        let install_display = install_cmd
            .map(|s| s.to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let uninstall_display = uninstall_cmd
            .map(|s| s.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(C_MUTED)),
                Span::styled(
                    entry.name.clone(),
                    Style::default().fg(C_TEXT).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(C_MUTED)),
                Span::styled(entry.id.clone(), Style::default().fg(C_TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Category: ", Style::default().fg(C_MUTED)),
                Span::styled(entry.category.clone(), Style::default().fg(C_TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Platform: ", Style::default().fg(C_MUTED)),
                Span::styled(app.platform.label(), Style::default().fg(C_TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Installed: ", Style::default().fg(C_MUTED)),
                Span::styled(
                    if installed { "yes" } else { "no" },
                    Style::default().fg(if installed { C_SUCCESS } else { C_WARNING }),
                ),
            ]),
            Line::from(vec![
                Span::styled("Binary: ", Style::default().fg(C_MUTED)),
                Span::styled(entry.binary.clone(), Style::default().fg(C_TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Repo: ", Style::default().fg(C_MUTED)),
                Span::styled(entry.repo.clone(), Style::default().fg(C_PRIMARY)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Install: ", Style::default().fg(C_MUTED)),
                Span::styled(install_display, Style::default().fg(C_TEXT)),
            ]),
            Line::from(vec![
                Span::styled("Uninstall: ", Style::default().fg(C_MUTED)),
                Span::styled(uninstall_display, Style::default().fg(C_TEXT)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No apps match the current tab/filter/search.",
            Style::default().fg(C_MUTED),
        ))]
    };

    let details_widget = Paragraph::new(details_lines).wrap(Wrap { trim: true });
    frame.render_widget(details_widget, details_inner);

    let tip_line = Line::from(Span::styled(
        "Tip: Press q in tmux to return",
        Style::default().fg(C_MUTED).add_modifier(Modifier::ITALIC),
    ));
    let tip_widget = Paragraph::new(tip_line)
        .style(Style::default().fg(C_MUTED))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(C_PANEL)),
        );
    let tip_area = Rect::new(
        details_inner.x,
        details_inner.y + details_inner.height.saturating_sub(1),
        details_inner.x + details_inner.width,
        details_inner.y + details_inner.height,
    );
    frame.render_widget(tip_widget, tip_area);
}
