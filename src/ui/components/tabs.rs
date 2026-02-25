use ratatui::{
    layout::Rect,
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, Tabs},
    Frame,
};

use crate::app::state::App;
use crate::ui::theme::*;

const TABS: [&str; 3] = ["All", "Installed", "Categories"];

pub fn render_main_tabs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let tab_titles = TABS
        .iter()
        .map(|title| Line::from(*title))
        .collect::<Vec<_>>();
    let tabs = Tabs::new(tab_titles)
        .select(app.selected_tab)
        .block(
            Block::default()
                .title(" TUIHub ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(C_PANEL)),
        )
        .style(Style::default().fg(C_MUTED))
        .highlight_style(
            Style::default()
                .fg(C_PRIMARY)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider(" | ");
    frame.render_widget(tabs, area);
}

pub fn render_category_tabs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let category_titles = app
        .categories
        .iter()
        .map(|c| Line::from(c.to_string()))
        .collect::<Vec<_>>();
    let cat_tabs = Tabs::new(category_titles)
        .select(app.selected_category)
        .block(
            Block::default()
                .title(" Category Filter ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(C_PANEL)),
        )
        .style(Style::default().fg(C_MUTED))
        .highlight_style(Style::default().fg(C_SUCCESS).add_modifier(Modifier::BOLD))
        .divider(" | ");
    frame.render_widget(cat_tabs, area);
}
