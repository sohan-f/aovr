pub mod components;
pub mod theme;
pub mod views;

use ratatui::{prelude::*, widgets::Block};

use components::draw_header_banner;
use theme::PANEL_BG;

use crate::{app::App, screens::Screen};

pub fn draw(frame: &mut Frame<'_>, app: &mut App) {
    let area = frame.area();

    frame.render_widget(Block::default().bg(PANEL_BG), area);

    let banner_h = if area.height >= 22 { 3 } else { 1 };

    let chunks = Layout::vertical([
        Constraint::Length(banner_h),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(3),
    ])
    .split(area);

    draw_header_banner(frame, chunks[0], banner_h == 3);

    match app.screen {
        Screen::Targets => views::targets::draw(frame, app, chunks[1], chunks[2], chunks[3]),
        Screen::Detail => views::detail::draw(frame, app, chunks[1], chunks[2], chunks[3]),
        Screen::About => views::about::draw(frame, app, chunks[1], chunks[2], chunks[3]),
    }
}
