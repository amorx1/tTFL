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

// pub enum MainView {
//     Dashboard,
//     Timetable
// }

pub struct App<'a> {
    pub tab_titles: Vec<&'a str>,
    pub tab_index: usize,
    pub input: String,
    pub input_mode: InputMode,
    pub messages: Vec<String>,
    pub lineNames: Vec<String>,
    pub lineData: Vec<Line>,
    pub focus: Option<Focus>,
    pub line_selected: Option<usize>,
    pub lines_tree_size: Option<usize>,
    // pub main_view: MainView,
    pub this_station_name: String,
    pub this_StopPoint: Option<StopPoint>
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            tab_titles: vec!["Line Status", "Timetable"],
            tab_index: 0,
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            lineNames: Vec::new(),
            lineData: Vec::new(),
            focus: None,
            line_selected: Some(0),
            lines_tree_size: Some(0),
            // main_view: MainView::Dashboard,
            this_station_name: String::new(),
            this_StopPoint: None,
        }
    }

    pub fn next(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.tab_titles.len();
    }

    pub fn previous(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = self.tab_titles.len() - 1;
        }
    }
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
pub struct StopPoint {
    pub zone: String,
    pub id: String,
    pub name: String
}

impl Default for StopPoint {
    fn default() -> StopPoint {
        StopPoint {
            zone: String::new(),
            id: String::new(),
            name: String::new()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Line {
    pub id: String,
    pub name: String,
    pub modeName: String,
    pub disruptions: Vec<Disruption>,
    pub lineStatuses: Vec<Option<LineStatus>>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StopPointResponse {
    pub query: String,
    pub total: i32,
    pub matches: Vec<Option<StopPoint>>,
}

// impl Default for App {
//     fn default() -> App {
//         App {
//             input: String::new(),
//             input_mode: InputMode::Normal,
//             messages: Vec::new(),
//             lineNames: Vec::new(),
//             lineData: Vec::new(),
//             focus: None,
//             line_selected: Some(0),
//             lines_tree_size: Some(0),
//             main_view: MainView::Dashboard,
//             this_station_name: String::new(),
//             this_StopPoint: None,
//         }
//     }
// }

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
                    // navigate tabs
                    KeyCode::Right => app.next(),
                    KeyCode::Left => app.previous(),

                    //insert mode
                    KeyCode::Char('i') => {
                        app.input_mode = InputMode::Insert;
                        app.focus = Some(Focus::InputBlock);
                    }
                    // KeyCode::Char('l') => {
                    //     app.focus = Some(Focus::LinesBlock);
                    // }

                    // quit app
                    KeyCode::Char('q') => {
                        return Ok(());
                    }

                    // refresh data
                    KeyCode::Char('r') => {
                        // refresh all data here manually
                        let result = reqwest::get("https://api.tfl.gov.uk/line/mode/tube/status").await.unwrap().json::<Vec<Line>>().await.unwrap();
                        let names = result.iter().map(|i| String::from(&i.name)).collect::<Vec<_>>();
                        app.lineNames = names;
                        app.lineData = result;
                    }

                    // leave focus
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
                    }
                    _ => {}
                }
                InputMode::Insert => match key.code {
                    KeyCode::Enter => {
                        app.this_station_name = app.input.drain(..).collect();
                        let res = reqwest::get(format!("https://api.tfl.gov.uk/StopPoint/Search/{}?modes=tube&includeHubs=false", app.this_station_name)).await.unwrap().json::<StopPointResponse>().await.unwrap();
                        app.this_StopPoint = match &res.matches.len() {
                            0 => {None}
                            1 => {
                                res.matches[0].clone()
                            }
                            _ => {None}
                        }
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
                }
            }
        }
    }
}