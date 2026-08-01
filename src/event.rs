use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{
    app::{App, CHROME_ROWS},
    screens::Screen,
};

fn detail_viewport_height() -> u16 {
    crossterm::terminal::size()
        .map(|(_, rows)| rows.saturating_sub(CHROME_ROWS + 2 + 2))
        .unwrap_or(10)
}

fn about_max_scroll(content_len: u16) -> u16 {
    let viewport_height = crossterm::terminal::size()
        .map(|(_, rows)| rows.saturating_sub(CHROME_ROWS + 2))
        .unwrap_or(10);

    content_len.saturating_sub(viewport_height)
}

pub fn handle(app: &mut App) -> io::Result<bool> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(false);
    }

    let Event::Key(key) = event::read()? else {
        return Ok(false);
    };

    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }

    match app.screen {
        Screen::Targets => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('a') => app.enter_about(),
            KeyCode::Up | KeyCode::Char('k') => app.move_target_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_target_down(),
            KeyCode::Enter | KeyCode::Char(' ') => app.enter_detail(),
            _ => {}
        },

        Screen::Detail => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('a') => app.enter_about(),
            KeyCode::Esc => {
                app.status.clear();
                app.screen = Screen::Targets;
            }

            KeyCode::Up | KeyCode::Char('k') => {
                app.move_overlay_up();
                app.clamp_detail_scroll(detail_viewport_height());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.move_overlay_down();
                app.clamp_detail_scroll(detail_viewport_height());
            }

            KeyCode::Char(' ') => {
                app.apply_selected_overlay()?;
            }
            KeyCode::Enter => {
                app.apply_selected_overlay()?;
                app.screen = Screen::Targets;
            }
            _ => {}
        },

        Screen::About => {
            let max_scroll = about_max_scroll(28);

            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('a') | KeyCode::Esc | KeyCode::Enter => app.back_from_about(),

                KeyCode::Up | KeyCode::Char('k') => app.scroll_about_up(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_about_down(max_scroll),
                KeyCode::PageUp => app.scroll_about_page_up(5),
                KeyCode::PageDown => app.scroll_about_page_down(5, max_scroll),
                _ => {}
            }
        }
    }

    Ok(false)
}
