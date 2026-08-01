#[cfg(not(target_os = "android"))]
compile_error!("aovr only supports Android.");

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

    let mut app = match App::load() {
        Ok(app) => app,
        Err(err) => {
            let mut app = App::empty();
            app.status = format!("Load failed: {err}");
            app
        }
    };

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
