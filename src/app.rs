use std::io::Result;

use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;

use crate::app_context;
use crate::app_state::AppState;
use crate::components::{Dispatcher, Error, ErrorType, HeadLine, Help, MainTab, StatusBar};
use crate::components::{PoolPicker, Renderable};
use crate::keymap::Action;
use crate::screen::Screen;

#[derive(Debug)]
pub struct App {
    pub current_screen: Screen,
    pub main_tab: MainTab,
    pub pool_picker: PoolPicker,
    pub help: Help,
    pub running: bool,
    show_help: bool,
    current_error: Option<Error>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            current_screen: Screen::PoolOverview,
            main_tab: MainTab::default(),
            pool_picker: PoolPicker::default(),
            help: Help,
            running: true,
            show_help: false,
            current_error: None,
        }
    }
}

impl App {
    pub fn dispatch(&mut self, action: Action, state: &mut AppState) {
        match &action {
            Action::Quit => {
                self.running = false;
                return;
            }
            Action::Help => {
                self.show_help = !self.show_help;
                return;
            }
            _ if self.show_help => return,
            _ => (),
        }

        self.current_error = self
            .main_tab
            .dispatch(&action, state)
            .or_else(|| self.pool_picker.dispatch(&action, state));

        match action {
            Action::ToggleUnit => {
                let mut ctx = app_context::write();

                ctx.unit.toggle();
            }
            Action::CycleTheme => {
                app_context::write().cycle_theme();
            }
            Action::Refresh => {
                if let Err(e) = state.refresh() {
                    self.current_error = Some(Error {
                        error_type: ErrorType::Error,
                        message: format!("refresh failed: {e}"),
                        tips: vec![Line::from("press r to retry")],
                    });
                }
            }
            _ => (),
        }
    }

    pub fn draw(&mut self, terminal: &mut DefaultTerminal, state: &AppState) -> Result<()> {
        terminal.draw(|frame| {
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
            let [header, _, content, _, footer] = layout[..] else {
                unreachable!()
            };

            HeadLine.render(frame, header);

            let underlying = if let Some(ref mut error) = self.current_error {
                error.render(frame, content);
                Screen::Error
            } else if state.pools.len() > 1 && state.current_pool.is_none() {
                self.pool_picker.render(frame, content, state);
                Screen::PoolPicker
            } else {
                self.main_tab.render(frame, content, state);
                Screen::PoolOverview
            };

            self.current_screen = if self.show_help {
                self.help.render(frame, frame.area());
                Screen::Help
            } else {
                underlying
            };

            StatusBar {
                screen: self.current_screen.clone(),
            }
            .render(frame, footer);
        })?;

        Ok(())
    }
}
