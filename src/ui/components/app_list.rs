use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::state::App;
use crate::ui::theme::*;
use crate::utils::truncate_with_ellipsis;

pub fn render_app_list(frame: &mut Frame<'_>, area: Rect, app: &mut App) {
    let catalog_block = Block::default()
        .title(" Catalog ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(C_PANEL))
        .style(Style::default().bg(C_BG));
    let catalog_inner = catalog_block.inner(area);
    frame.render_widget(catalog_block, area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(4)])
        .split(catalog_inner);

    let header_line =
        Paragraph::new("Sel  Name                 Category        State       Description")
            .style(Style::default().fg(C_MUTED).add_modifier(Modifier::BOLD));
    frame.render_widget(header_line, left_chunks[0]);

    let list_width = left_chunks[1].width as usize;
    let desc_width = if list_width > 58 { list_width - 58 } else { 12 };
    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .filter_map(|idx| app.entries.get(*idx))
        .map(|entry| {
            let installed = app.is_installed(entry);
            let selected = app.selected_ids.contains(&entry.id);
            let checkbox = if selected { "[x]" } else { "[ ]" };
            let install_badge = if installed { "installed" } else { "available" };
            let display_name = truncate_with_ellipsis(&entry.name, 20);
            let display_category = truncate_with_ellipsis(&entry.category, 14);
            let display_desc = truncate_with_ellipsis(&entry.description, desc_width);

            let line = Line::from(vec![
                Span::styled(format!("{:<4}", checkbox), Style::default().fg(C_PRIMARY)),
                Span::styled(format!("{:<21}", display_name), Style::default().fg(C_TEXT)),
                Span::styled(
                    format!("{:<16}", display_category),
                    Style::default().fg(C_MUTED),
                ),
                Span::styled(
                    format!("{:<11}", install_badge),
                    Style::default().fg(if installed { C_SUCCESS } else { C_WARNING }),
                ),
                Span::styled(display_desc, Style::default().fg(C_TEXT)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let app_list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(32, 57, 84))
                .fg(C_TEXT)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true);

    frame.render_stateful_widget(app_list, left_chunks[1], &mut app.list_state);
}
