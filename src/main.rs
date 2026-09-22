// Builds on any host for development; CI cross-compiles release binaries for
// `aarch64-linux-android`. See README "Building".
mod app;
mod event;
mod parsers;
mod screens;
mod shell;
mod ui;

use std::io;

use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> io::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::load();

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        app.tick();
        if event::handle(&mut app)? {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
