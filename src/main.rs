use crossterm::{
    ExecutableCommand,
    event::Event,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::DefaultTerminal;
use ratatui_themes::{Theme, ThemeName};
use std::io::{Result, stderr};

use app::App;
use app_context::{AppContext, init};
use keymap::key_to_action;

use crate::app_context::UnitMode;

mod app;
mod app_context;
mod components;
mod keymap;
mod screen;
mod theme;

fn main() -> Result<()> {
    init(AppContext {
        theme: Theme::new(ThemeName::Dracula),
        unit: UnitMode::Decimal,
    });

    stderr().execute(EnterAlternateScreen)?;

    ratatui::run(render)?;

    stderr().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn render(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = App::new();

    loop {
        app.draw(terminal);

        if let Event::Key(key) = crossterm::event::read()?
            && let Some(action) = key_to_action(key, &app.current_screen)
        {
            app.dispatch(action);
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}
