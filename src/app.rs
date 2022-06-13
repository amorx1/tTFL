use std::{io, collections::{HashMap, HashSet, BTreeMap, LinkedList}};
use chrono::DateTime;
// use rust_bert::pipelines::ner::NERModel;

use crossterm::event::{self, Event, KeyCode};
use serde_derive::{Serialize, Deserialize};
use tui::{backend::Backend, Terminal, widgets::canvas::Rectangle, style::Color};
use reqwest::{self, Client};

use crate::ui::ui;

trait WithStationName {
    fn new(stop_name: String) -> Self;
}
pub enum InputMode {
    Normal,
    Insert,
}
pub enum Focus {
    InputBlock,
    LinesBlock
}
pub struct App<'a> {
    pub tab_titles: Vec<&'a str>,
    pub tab_index: usize,
    pub input: String,
    pub input_mode: InputMode,
    pub lineNames: Vec<String>,
    pub lineData: Vec<Line>,
    pub focus: Option<Focus>,
    pub line_selected: Option<usize>,
    pub lines_tree_size: Option<usize>,
    pub this_station_name: String,
    pub this_StopTimetable: StopTimetable,
    pub api_client: Option<Client>,
    pub line_cache: BTreeMap<String, Vec<String>>,
    pub stop_cache: BTreeMap<String, StopTimetable>,
}
impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        App {
            tab_titles: vec!["Line Status", "Timetable"],
            tab_index: 0,
            input: String::new(),
            input_mode: InputMode::Normal,
            lineNames: Vec::new(),
            lineData: Vec::new(),
            focus: None,
            line_selected: Some(0),
            lines_tree_size: Some(0),
            this_station_name: String::new(),
            this_StopTimetable: StopTimetable::default(),
            api_client: None,
            line_cache: BTreeMap::new(),
            stop_cache: BTreeMap::new()
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
    pub name: String,
}
impl Default for StopPoint {
    fn default() -> StopPoint {
        StopPoint {
            zone: String::new(),
            id: String::new(),
            name: String::new(),
        }
    }
}
#[derive(Clone)]
pub struct StopTimetable {
    pub stop_point: Option<StopPoint>,
    pub unique_lines: HashSet<String>,
    pub unique_platforms: HashMap<String, Vec<String>>,
    pub arrivals: Vec<Arrival>,
    pub live_maps: BTreeMap<String, LiveMap>,
    pub station_nodes: BTreeMap<String, Vec<Vec<StationNode>>>,
}
impl Default for StopTimetable {
    fn default() -> StopTimetable {
        StopTimetable { stop_point: None, unique_lines: HashSet::new(), unique_platforms: HashMap::new(), arrivals: Vec::new(), live_maps: BTreeMap::new(), station_nodes: BTreeMap::new() }
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArrivalsResponse {
    pub arrivals: Vec<Arrival>
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Arrival {
    pub stationName: String,
    pub lineId: String,
    pub platformName: String,
    // pub direction: String,
    // pub destinationName: String,
    pub timeToStation: i32,
    pub currentLocation: String,
    pub expectedArrival: String,
    pub towards: String
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RouteResponse {
    pub lineId: String,
    pub direction: String,
    pub orderedLineRoutes: Vec<Route>
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Route {
    pub name: String,
    pub naptanIds: Vec<String>
}
#[derive(Clone)]
pub struct Station {
    pub naptan_id: String,
}
impl WithStationName for Station {
    fn new(stop_name: String) -> Self {
        Station { naptan_id: stop_name }
    }
}
pub struct Link {
    pub link_name: String,
    pub is_current: bool
}
#[derive(Clone)]
pub struct LiveMap {
    pub stops_0: Vec<Station>,
    // pub links: LinkedList<Link>,
    pub stops_1: Vec<Station>,
    pub trains_currently_at: Vec<String>
}
impl Default for LiveMap {
    fn default() -> LiveMap {
        LiveMap { stops_0: Vec::new(), stops_1: Vec::new(), trains_currently_at: Vec::new() }
    }
}
#[derive(Clone)]
pub struct StationNode {
    pub naptan_id: String,
    pub rect: Rectangle,
}

#[tokio::main]
pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    // create reqwest client
    app.api_client = Some(Client::new());

    // load data once here before loop
    let result = app.api_client.as_ref().unwrap().get("https://api.tfl.gov.uk/line/mode/tube/status").send().await.unwrap().json::<Vec<Line>>().await.unwrap();
    let names = result.iter().map(|i| String::from(&i.name)).collect::<Vec<_>>();
    app.lineNames = names;
    app.lineData = result;

    app.line_cache.insert(String::from("lineNames"), app.lineNames.clone());

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

                    // quit app
                    KeyCode::Char('q') => {
                        return Ok(());
                    }

                    // refresh data
                    KeyCode::Char('r') => {
                        // refresh all data here manually
                        let result = app.api_client.as_ref().unwrap().get("https://api.tfl.gov.uk/line/mode/tube/status").send().await.unwrap().json::<Vec<Line>>().await.unwrap();
                        app.lineNames = app.line_cache["lineNames"].clone();
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
                        let _ = app.this_StopTimetable.unique_lines.drain();

                        // check if we have station data in cache
                        if app.stop_cache.contains_key(&app.this_station_name) {
                            
                            // retrieve the cache
                            app.this_StopTimetable.stop_point = app.stop_cache[&app.this_station_name].stop_point.clone();
                            app.this_StopTimetable.unique_lines = app.stop_cache[&app.this_station_name].unique_lines.clone();
                            app.this_StopTimetable.unique_platforms = app.stop_cache[&app.this_station_name].unique_platforms.clone();
                            app.this_StopTimetable.live_maps = app.stop_cache[&app.this_station_name].live_maps.clone();
                            // CANNOT CACHE WHEN CURRENT STATIONS ARE IMPLEMENTED
                            app.this_StopTimetable.station_nodes = app.stop_cache[&app.this_station_name].station_nodes.clone();

                            // only arrivals needs refreshing
                            app.this_StopTimetable.arrivals = app.api_client.as_ref().unwrap().get(format!("https://api.tfl.gov.uk/StopPoint/{}/Arrivals?mode=tube", app.this_StopTimetable.stop_point.as_ref().unwrap().id))
                                .send()
                                .await
                                .unwrap()
                                .json::<Vec<Arrival>>()
                                .await
                                .unwrap();

                            // for each line
                            let mut dispatch: Vec<String> = Vec::new();
                            for line in &app.this_StopTimetable.unique_lines {
                                let _ = app.this_StopTimetable.arrivals
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, a)| a.lineId == line.clone())
                                    .map(|(_, e)| dispatch.push(String::from(e.currentLocation.clone())))
                                    .collect::<Vec<_>>();
                            }

                            // send to NER service
                            // let parsed_game = app.api_client.as_ref().unwrap().get("").send().await.unwrap();

                            // update cache
                            app.stop_cache.entry(app.this_station_name.clone()).or_insert(app.this_StopTimetable.clone());
                        }

                        // not in cache
                        else {
                            // get stop ID -> stop_point.id
                            let stop_id_search =  app.api_client.as_ref().unwrap().get(format!("https://api.tfl.gov.uk/StopPoint/Search/{}?modes=tube&includeHubs=false", app.this_station_name))
                                .send()
                                .await
                                .unwrap()
                                .json::<StopPointResponse>()
                                .await
                                .unwrap();
                            app.this_StopTimetable.stop_point = match &stop_id_search.matches.len() {
                                0 => Some(StopPoint::default()),
                                _ => stop_id_search.matches[0].clone()
                            };

                            // use id to fetch arrivals
                            app.this_StopTimetable.arrivals =  app.api_client.as_ref().unwrap().get(format!("https://api.tfl.gov.uk/StopPoint/{}/Arrivals?mode=tube", app.this_StopTimetable.stop_point.as_ref().unwrap().id))
                                .send()
                                .await
                                .unwrap()
                                .json::<Vec<Arrival>>()
                                .await
                                .unwrap();

                            for arrival in &app.this_StopTimetable.arrivals {
                                app.this_StopTimetable.unique_lines.insert(arrival.lineId.clone());
                            }

                            // over all lines in this station
                            for u_line in &app.this_StopTimetable.unique_lines {
                                let platforms_for_this_line = app.this_StopTimetable.arrivals
                                    .iter()
                                    .enumerate()
                                    .filter(|&(_,i)| i.lineId == u_line.clone())
                                    .map(|(_,e)| e.platformName.clone())
                                    .collect::<Vec<String>>();

                                // sort platforms by line
                                let mut map: BTreeMap<String, _> = BTreeMap::new();
                                for platform in platforms_for_this_line {
                                    map.entry(platform.clone()).or_insert(platform);
                                }
                                let mut platforms: Vec<String> = Vec::new();
                                for (platform, _) in &map {
                                    platforms.push(platform.clone());
                                }
                                // { key: line(String), value: platform(String) }
                                app.this_StopTimetable.unique_platforms.insert(u_line.to_string(), platforms);


                                //
                                let res =  app.api_client.as_ref().unwrap().get(format!("https://api.tfl.gov.uk/Line/{}/Route/Sequence/all", u_line))
                                    .send()
                                    .await
                                    .unwrap()
                                    .json::<RouteResponse>()
                                    .await
                                    .unwrap();
                                
                                app.this_StopTimetable.live_maps.insert(u_line.to_string(), LiveMap { 
                                    stops_0: res.orderedLineRoutes[0].naptanIds
                                            .iter()
                                            .map(|s| Station::new(s.to_string()))
                                            .collect::<Vec<Station>>(),
                                    stops_1: res.orderedLineRoutes[1].naptanIds
                                            .iter()
                                            .map(|s| Station::new(s.to_string()))
                                            .collect::<Vec<Station>>(),
                                    trains_currently_at: Vec::new()
                                    }
                                );



                                let mut x_0 = 12.5;
                                let y = 50.0;
                                let mut x_1 = 12.5;
                                let mut rects_0: Vec<StationNode> = Vec::new();
                                let mut rects_1: Vec<StationNode> = Vec::new();
                                for stop in &app.this_StopTimetable.live_maps[u_line].stops_0 {
                                    rects_0.push(
                                        StationNode {
                                            naptan_id: stop.naptan_id.clone(),
                                            rect: Rectangle {
                                                x:x_0,
                                                y:y,
                                                width:2.0,
                                                height:10.0,
                                                color: match &stop.naptan_id == &app.this_StopTimetable.stop_point.clone().unwrap().id {
                                                    true => Color::LightGreen,
                                                    false => Color::LightYellow
                                                }
                                            },
                                        }
                                    );
                                    x_0 += 3.5;
                                }
                                for stop in &app.this_StopTimetable.live_maps[u_line].stops_1 {
                                    rects_1.push(
                                        StationNode { 
                                            naptan_id: stop.naptan_id.clone(),
                                            rect: Rectangle {
                                                x:x_1,
                                                y:y,
                                                width:2.0,
                                                height:10.0,
                                                color: match &stop.naptan_id == &app.this_StopTimetable.stop_point.clone().unwrap().id {
                                                    true => Color::LightGreen,
                                                    false => Color::LightYellow
                                                }
                                            },
                                        }
                                    );
                                    x_1 += 3.5;
                                }
                                app.this_StopTimetable.station_nodes.insert(u_line.to_string(), vec!(rects_0, rects_1));
                            }
                            app.stop_cache.insert(format!("{}", app.this_station_name), app.this_StopTimetable.clone());
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