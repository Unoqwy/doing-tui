use std::io;

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;

mod app;
mod input;
mod model;
mod prompts;
mod storage;
mod ui;

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run<B: Backend>(terminal: &mut Terminal<B>) -> anyhow::Result<()> {
    let mut app = app::init()?;

    loop {
        terminal.draw(|frame| ui::draw_frame(frame, &app))?;
        if input::handle_event(&mut app, event::read()?)? {
            break;
        }
    }
    Ok(())
}
