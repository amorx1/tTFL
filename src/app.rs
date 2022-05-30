use std::io;

use crossterm::event::{self, Event, KeyCode};
use tui::{backend::Backend, Terminal};

use crate::ui::ui;

pub enum InputMode {
    Normal,
    Insert,
}

pub enum Focus {
    InputBlock,
}

pub struct App {
    pub input: String,
    pub input_mode: InputMode,
    pub messages: Vec<String>,
    pub focus: Option<Focus>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            focus: None,
        }
    }
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
    }
}
