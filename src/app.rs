use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Direction, Layout};

use crate::app_context;
use crate::components::Renderable;
use crate::components::{Dispatcher, HeadLine, MainTab, StatusBar};
use crate::keymap::Action;
use crate::screen::Screen;

pub struct App {
    pub current_screen: Screen,
    pub running: bool,
    pub main_tab: MainTab,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_screen: Screen::PoolOverview,
            main_tab: MainTab::default(),
            running: true,
        }
    }

    pub fn dispatch(&mut self, action: Action) {
        self.main_tab.dispatch(&action);

        match action {
            Action::Quit => self.running = false,
            Action::ToggleUnit => {
                let mut ctx = app_context::write();

                ctx.unit.toggle();
            }
            Action::CycleTheme => {
                let mut ctx = app_context::write();

                ctx.theme.next();
            }
            _ => (),
        }
    }

    pub fn draw(&mut self, terminal: &mut DefaultTerminal) {
        let _ = terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(1), // Title bar
                    Constraint::Length(1),
                    Constraint::Fill(1), // Main Tab
                    Constraint::Length(1),
                    Constraint::Length(1), // Status bar
                ])
                .split(frame.area());

            HeadLine.render(frame, layout[0]);
            self.main_tab.render(frame, layout[2]);
            StatusBar.render(frame, layout[4]);
        });
    }
}
