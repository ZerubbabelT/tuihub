use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::app::state::{App, ConfirmAction};
use crate::ui::components::{
    app_list::render_app_list, detail_panel::render_detail_panel, footer::render_footer,
    tabs::render_main_tabs,
};
use crate::ui::layout::centered_rect;
use crate::ui::theme::*;

pub fn ui(frame: &mut Frame<'_>, app: &mut App) {
    frame.render_widget(
        Block::default().style(Style::default().bg(C_BG)),
        frame.area(),
    );

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(if app.selected_tab == 2 { 3 } else { 0 }),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(4),
        ])
        .split(frame.area());

    render_main_tabs(frame, vertical[0], app);

    if app.selected_tab == 2 {
        use crate::ui::components::tabs::render_category_tabs;
        render_category_tabs(frame, vertical[1], app);
    }

    let search_title = if app.search_mode {
        " Search mode (/): typing... Enter apply, Esc close "
    } else {
        " Search (/ to start, Esc clear) "
    };

    let search_text = if app.search_input.is_empty() {
        "Type to filter by name, id, category, description".to_string()
    } else {
        app.search_input.clone()
    };
    let search = Paragraph::new(search_text)
        .block(
            Block::default()
                .title(search_title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if app.search_mode {
                    C_PRIMARY
                } else {
                    C_PANEL
                })),
        )
        .style(if app.search_mode {
            Style::default().fg(C_TEXT)
        } else {
            Style::default().fg(C_MUTED)
        });

    frame.render_widget(search, vertical[2]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(vertical[3]);

    render_app_list(frame, body[0], app);
    render_detail_panel(frame, body[1], app);

    render_footer(frame, vertical[4], app);

    if app.confirm_mode {
        let area = centered_rect(50, 25, frame.area());
        frame.render_widget(Clear, area);

        let msg = if let Some(ConfirmAction::Uninstall(ref targets)) = app.confirm_action {
            let names = targets
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            format!("Are you sure you want to uninstall:\n{}?", names)
        } else {
            "Confirm action?".to_string()
        };

        let block = Paragraph::new(msg)
            .style(Style::default().fg(C_TEXT))
            .wrap(Wrap { trim: true })
            .alignment(ratatui::prelude::Alignment::Center)
            .block(
                Block::default()
                    .title(" Confirm Uninstall ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(C_PANEL)),
            );
        frame.render_widget(block, area);

        let btn_area = Rect::new(
            area.x + 2,
            area.y + area.height - 3,
            area.x + area.width - 2,
            area.y + area.height - 1,
        );

        let yes_style = if app.confirm_selected {
            Style::default()
                .fg(C_BG)
                .bg(C_SUCCESS)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(C_SUCCESS).add_modifier(Modifier::BOLD)
        };
        let no_style = if !app.confirm_selected {
            Style::default()
                .fg(C_BG)
                .bg(C_WARNING)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(C_WARNING).add_modifier(Modifier::BOLD)
        };

        let btns = Paragraph::new(vec![Line::from(vec![
            Span::styled("[ Yes ] ", yes_style),
            Span::styled("[ No ] ", no_style),
        ])
        .alignment(ratatui::prelude::Alignment::Center)])
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(btns, btn_area);
    }

    if app.search_mode {
        let cursor_x = vertical[2].x + 1 + app.search_input.chars().count() as u16;
        let cursor_y = vertical[2].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }
}

#[allow(dead_code)]
pub fn show_transient_message(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    msg: &str,
) -> anyhow::Result<()> {
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
