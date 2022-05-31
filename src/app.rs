use std::io;

use crossterm::event::{self, Event, KeyCode};
use serde_derive::{Serialize, Deserialize};
use tui::{backend::Backend, Terminal};
use reqwest::{self, Client};

use crate::ui::ui;

pub enum InputMode {
    Normal,
    Insert,
}

pub enum Focus {
    InputBlock,
    LinesBlock
}

pub struct App {
    pub input: String,
    pub input_mode: InputMode,
    pub messages: Vec<String>,
    pub lineNames: Vec<String>,
    pub lineData: Vec<Line>,
    pub focus: Option<Focus>,
    pub line_selected: Option<usize>,
    pub lines_tree_size: Option<usize>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineStatus {
    pub id: i32,
    pub statusSeverity: i32,
    pub statusSeverityDescription: String,
    pub reason: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Disruption {
    category: String,
    categoryDescription: String,
    description: String,
    summary: String,
    additionalInfo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Line {
    pub id: String,
    pub name: String,
    pub modeName: String,
    pub disruptions: Vec<Disruption>,
    pub lineStatuses: Vec<Option<LineStatus>>,
    // routeSections: Vec<String>,
    // serviceTypes: Vec<ServiceType>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            lineNames: Vec::new(),
            lineData: Vec::new(),
            focus: None,
            line_selected: Some(0),
            lines_tree_size: Some(0)
        }
    }
}

#[tokio::main]
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    // load data once here before loop
    let result = reqwest::get("https://api.tfl.gov.uk/line/mode/tube/status").await.unwrap().json::<Vec<Line>>().await.unwrap();
    let names = result.iter().map(|i| String::from(&i.name)).collect::<Vec<_>>();
    app.lineNames = names;
    app.lineData = result;

    // begin loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('i') => {
                        app.input_mode = InputMode::Insert;
                        app.focus = Some(Focus::InputBlock);
                    }
                    KeyCode::Char('l') => {
                        app.focus = Some(Focus::LinesBlock);
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('r') => {
                        // refresh all data here manually
                        let result = reqwest::get("https://api.tfl.gov.uk/line/mode/tube/status").await.unwrap().json::<Vec<Line>>().await.unwrap();
                        let names = result.iter().map(|i| String::from(&i.name)).collect::<Vec<_>>();
                        app.lineNames = names;
                        app.lineData = result;
                    }
                    KeyCode::Esc => {
                        app.focus = None;
                    }
                    KeyCode::Char('j') => match app.focus {
                        Some(Focus::LinesBlock) => {
                            if app.lines_tree_size
                                > usize::checked_add(
                                    app.line_selected.unwrap(),
                                    usize::try_from(1).unwrap(),
                                )
                            {
                                app.line_selected = usize::checked_add(
                                    app.line_selected.unwrap(),
                                    usize::try_from(1).unwrap(),
                                );
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Char('k') => match app.focus {
                        Some(Focus::LinesBlock) => {
                            if app.line_selected != Some(0) {
                                app.line_selected = usize::checked_sub(
                                    app.line_selected.unwrap(),
                                    usize::try_from(1).unwrap(),
                                );
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                InputMode::Insert => match key.code {
                    KeyCode::Enter => {
                        app.messages.push(app.input.drain(..).collect());
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.focus = None;
                    }
                    _ => {}
                },
            }
        }
    }
}