use std::{io, time::Duration};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use crate::{
    app::{App, CHROME_ROWS, SEARCH_BAR_H, SEARCH_GAP_H},
    screens::Screen,
};

fn is_ctrl(key: &KeyEvent) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
}

/// Viewport of the Targets/Detail list (chrome + gap + search bar + 1-row
/// padding removed).
fn list_viewport_height() -> u16 {
    crossterm::terminal::size()
        .map(|(_, rows)| rows.saturating_sub(CHROME_ROWS + 2 + 2 + SEARCH_GAP_H + SEARCH_BAR_H))
        .unwrap_or(10)
}

fn about_viewport_height() -> u16 {
    crossterm::terminal::size()
        .map(|(_, rows)| rows.saturating_sub(CHROME_ROWS + 2))
        .unwrap_or(10)
}

/// At least one row, so a tiny terminal still moves.
fn half_page(height: u16) -> u16 {
    (height / 2).max(1)
}

pub fn handle(app: &mut App) -> io::Result<bool> {
    if !event::poll(Duration::from_millis(50))? {
        return Ok(false);
    }

    let event = event::read()?;
    if let Event::Mouse(mouse) = event {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if app.search_clear_hit(mouse.column, mouse.row) {
                    app.cancel_search();
                } else if app.search_bar_hit(mouse.column, mouse.row) {
                    match app.screen {
                        Screen::Targets | Screen::Detail => app.start_search(),
                        Screen::About => {}
                    }
                }
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                if app.searching {
                    if !app.search_has_matches() {
                        return Ok(false);
                    }
                    app.confirm_search();
                }
                let down = mouse.kind == MouseEventKind::ScrollDown;
                match app.screen {
                    Screen::Targets => {
                        if down {
                            app.move_target_down();
                        } else {
                            app.move_target_up();
                        }
                    }
                    Screen::Detail => {
                        if down {
                            app.move_overlay_down();
                        } else {
                            app.move_overlay_up();
                        }
                        app.clamp_detail_scroll(list_viewport_height());
                    }
                    Screen::About => {
                        if down {
                            let max = crate::ui::views::about::content_len(app.is_virtual())
                                .saturating_sub(about_viewport_height());
                            app.scroll_about_down(max);
                        } else {
                            app.scroll_about_up();
                        }
                    }
                }
            }
            _ => {}
        }
        return Ok(false);
    }

    let Event::Key(key) = event else {
        return Ok(false);
    };

    if key.kind != KeyEventKind::Press {
        return Ok(false);
    }

    if app.searching {
        match app.screen {
            Screen::Targets | Screen::Detail => {
                let detail = app.screen == Screen::Detail;
                match key.code {
                    KeyCode::Esc => app.cancel_search(),
                    KeyCode::Enter => {
                        app.confirm_search();
                        if detail {
                            app.clamp_detail_scroll(list_viewport_height());
                        }
                    }
                    KeyCode::Backspace | KeyCode::Delete => {
                        app.pop_search_char();
                        if detail {
                            app.clamp_detail_scroll(list_viewport_height());
                        }
                    }
                    KeyCode::Char('u') if is_ctrl(&key) => {
                        app.clear_search_line();
                        if detail {
                            app.clamp_detail_scroll(list_viewport_height());
                        }
                    }
                    KeyCode::Char('c') if is_ctrl(&key) => app.cancel_search(),
                    KeyCode::Char(c) if !is_ctrl(&key) => {
                        app.push_search_char(c);
                        if detail {
                            app.clamp_detail_scroll(list_viewport_height());
                        }
                    }
                    _ => {}
                }
            }
            Screen::About => app.searching = false,
        }
        return Ok(false);
    }

    match app.screen {
        Screen::Targets => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('a') => app.enter_about(),
            KeyCode::Char('r') => app.reload(),

            KeyCode::Char('/') => app.start_search(),
            KeyCode::Esc if !app.target_search.is_empty() => app.cancel_search(),

            KeyCode::Up | KeyCode::Char('k') => app.move_target_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_target_down(),
            KeyCode::Home | KeyCode::Char('g') => app.select_target_first(),
            KeyCode::End | KeyCode::Char('G') => app.select_target_last(),

            KeyCode::PageUp => app.page_target_up(list_viewport_height()),
            KeyCode::PageDown => app.page_target_down(list_viewport_height()),
            KeyCode::Char('u') if is_ctrl(&key) => {
                app.page_target_up(half_page(list_viewport_height()))
            }
            KeyCode::Char('d') if is_ctrl(&key) => {
                app.page_target_down(half_page(list_viewport_height()))
            }

            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('l') | KeyCode::Right => {
                app.enter_detail()
            }
            _ => {}
        },

        Screen::Detail => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('a') => app.enter_about(),
            KeyCode::Char('/') => app.start_search(),
            KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
                if key.code == KeyCode::Esc && !app.overlay_search.is_empty() {
                    app.cancel_search();
                } else {
                    app.overlay_search.clear();
                    app.clear_status();
                    app.screen = Screen::Targets;
                }
            }

            KeyCode::Up | KeyCode::Char('k') => {
                app.move_overlay_up();
                app.clamp_detail_scroll(list_viewport_height());
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.move_overlay_down();
                app.clamp_detail_scroll(list_viewport_height());
            }
            KeyCode::Home | KeyCode::Char('g') => {
                app.select_overlay_first();
                app.clamp_detail_scroll(list_viewport_height());
            }
            KeyCode::End | KeyCode::Char('G') => {
                app.select_overlay_last();
                app.clamp_detail_scroll(list_viewport_height());
            }

            KeyCode::PageUp => {
                app.page_overlay_up(list_viewport_height());
                app.clamp_detail_scroll(list_viewport_height());
            }
            KeyCode::PageDown => {
                app.page_overlay_down(list_viewport_height());
                app.clamp_detail_scroll(list_viewport_height());
            }
            KeyCode::Char('u') if is_ctrl(&key) => {
                app.page_overlay_up(half_page(list_viewport_height()));
                app.clamp_detail_scroll(list_viewport_height());
            }
            KeyCode::Char('d') if is_ctrl(&key) => {
                app.page_overlay_down(half_page(list_viewport_height()));
                app.clamp_detail_scroll(list_viewport_height());
            }

            KeyCode::Char(' ') => {
                app.apply_selected_overlay()?;
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                app.apply_selected_overlay()?;
                app.screen = Screen::Targets;
            }
            _ => {}
        },

        Screen::About => {
            let max_scroll = crate::ui::views::about::content_len(app.is_virtual())
                .saturating_sub(about_viewport_height());
            let page = about_viewport_height();

            match key.code {
                KeyCode::Char('q') => return Ok(true),
                // Leaf screen: every "forward/back" key dismisses it.
                KeyCode::Char('a')
                | KeyCode::Esc
                | KeyCode::Enter
                | KeyCode::Char('h')
                | KeyCode::Char('l')
                | KeyCode::Left
                | KeyCode::Right => app.back_from_about(),

                KeyCode::Up | KeyCode::Char('k') => app.scroll_about_up(),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_about_down(max_scroll),
                KeyCode::Home | KeyCode::Char('g') => app.scroll_about_to_top(),
                KeyCode::End | KeyCode::Char('G') => app.scroll_about_down(max_scroll),

                KeyCode::PageUp => app.scroll_about_page_up(page),
                KeyCode::PageDown => app.scroll_about_page_down(page, max_scroll),
                KeyCode::Char('u') if is_ctrl(&key) => {
                    app.scroll_about_page_up(half_page(page));
                }
                KeyCode::Char('d') if is_ctrl(&key) => {
                    app.scroll_about_page_down(half_page(page), max_scroll);
                }
                _ => {}
            }
        }
    }

    Ok(false)
}
