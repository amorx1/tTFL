use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap, Tabs},
    Frame,
};

use unicode_width::UnicodeWidthStr;

use crate::app::{App, Focus, InputMode};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {

    // split into tab row / rest
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

        // get tab names
        let titles = app.tab_titles.iter().map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::LightYellow)),
            ])
        })
        .collect();

        // create and render tabs
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Tabs"))
            .select(app.tab_index)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::DarkGray),
            );
        f.render_widget(tabs, chunks[0]);

        // check current tab
        match app.tab_index {

            // Status
            0 => {
                {
                    // render dashboard in remaining frame (no need to split unless another block is added) 
                    // let chunks = Layout::default()
                    //     .margin(0)
                    //     .constraints([Constraint::Length(100), Constraint::Min(100)].as_ref())
                    //     .split(chunks[1]);

                    draw_dashboard(f, app, chunks[1]);
                }
            },

            // Timetable
            1 => {
                {
                    // split remaining frame into input and timetable
                    let chunks = Layout::default()
                        .margin(0)
                        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                        .split(chunks[1]);
                    
                    // render input in first split
                    draw_input(f, app, chunks[0]);
                    {
                        // render timetable in remaining frame (no need to split unless another block is added)
                        // let chunks = Layout::default()
                        //     .constraints([Constraint::Percentage(100), Constraint::Percentage(100)].as_ref())
                        //     .direction(Direction::Horizontal)
                        //     .split(chunks[1]);
                
                        draw_timetable(f, app, chunks[1]);
                    }
                }
            },
            _ => unreachable!()
        }
}

fn draw_timetable<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::raw("Timetable"));

    match app.this_StopTimetable.arrivals.len() {
        0 => f.render_widget(block, area),
        _ => {
                // let items = app.this_StopTimetable.unique_lines.iter()
                //     .map(|a| String::from(a))
                //     .map(|i| ListItem::new(i))
                //     .collect::<Vec<_>>();

                // this is just a list of the unique lines
                // let lines = List::new(items)
                //     .block(
                //         Block::default()
                //         .title("Result")
                //         .title_alignment(Alignment::Center)
                //         .borders(Borders::ALL)
                //         .border_type(BorderType::Rounded),
                //     );
                // f.render_widget(lines, area)

                let constraints = match app.this_StopTimetable.unique_lines.len() {
                    0 => [Constraint::Percentage(0)].as_ref(),
                    1 => [Constraint::Percentage(100)].as_ref(),
                    2 => [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                    3 => [Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)].as_ref(),
                    4 => [Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)].as_ref(),
                    5 => [Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20)].as_ref(),
                    _ => [Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)].as_ref()
                };

                let rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(area);

                let mut row_count = 0;
                for line in &app.this_StopTimetable.unique_lines {
                    f.render_widget(Block::default()
                            .title(line.clone())
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(Color::LightYellow))
                        ,rows[row_count]
                    );
                    row_count += 1;
                }
        }
    }
}

fn draw_input<B: Backend>(f: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::White))
        .title(Span::raw("Enter station:"));

    let (input_normal_mode_message, input_normal_mode_style) = (
        vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to exit, ", Style::default().fg(Color::DarkGray)),
            Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" to insert.", Style::default().fg(Color::DarkGray)),
        ],
        Style::default().add_modifier(Modifier::RAPID_BLINK),
    );

    let mut text = Text::from(Spans::from(input_normal_mode_message));
    text.patch_style(input_normal_mode_style);

    let input = match app.input_mode {
        InputMode::Insert => Paragraph::new(app.input.as_ref())
            .style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )
            .block(block),
        InputMode::Normal => Paragraph::new(text).style(Style::default()).block(block),
    };

    f.render_widget(input, area);

    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Insert => f.set_cursor(area.x + app.input.width() as u16 + 1, area.y + 1),
    }
}

fn draw_messages<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .direction(Direction::Vertical)
        .split(area);

    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)])
            .direction(Direction::Vertical)
            .split(chunks[0]);

        let all_item = ListItem::new("All");
        let mut items = app
            .lineNames
            .iter()
            .map(String::from)
            .map(ListItem::new)
            .collect::<Vec<_>>();
        items.push(all_item);
        app.lines_tree_size = Some(items.len());

        let lines = List::new(items)
            .block(
                Block::default()
                    .title("Lines")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(match app.focus {
                Some(Focus::LinesBlock) => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::White),
            })
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(" ");

        let mut state = ListState::default();
        state.select(app.line_selected);
        f.render_stateful_widget(lines, area, &mut state)
    }
}

fn draw_dashboard<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let block = Block::default()
        .title("Dashboard")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White))
        .border_type(BorderType::Rounded)
        .style(Style::default());
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(area);

    let mut q = app.lineData.clone();

    // create rows
    let mut rows: Vec<Vec<Rect>> = Vec::new();
    for i in 0..3 {
        rows.push(
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ]
                    .as_ref(),
                )
                .split(chunks[i]),
        );
    }

    // populate grid
    for x in 0..3 {
        for y in 0..3 {
            let item = q.pop().unwrap();
            f.render_widget(
                Block::default()
                    .title(item.name)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(match &item.lineStatuses[0] {
                        Some(s) => {
                            if s.statusSeverity != 10 {
                                Color::LightRed
                            } else {
                                Color::LightGreen
                            }
                        }
                        _ => Color::LightGreen,
                    })),
                rows[x][y],
            );
            {
                let chunks = Layout::default()
                    .margin(1)
                    .direction(Direction::Vertical)
                    .constraints(match &item.lineStatuses[0] {
                        Some(s) => match &s.reason {
                            Some(r) => {
                                [Constraint::Percentage(30), Constraint::Percentage(30)]
                                    .as_ref()
                            }
                            None => {
                                [Constraint::Percentage(20), Constraint::Percentage(20)]
                                    .as_ref()
                            }
                        },
                        None => [Constraint::Percentage(20), Constraint::Percentage(20)]
                            .as_ref(),
                    })
                    .split(rows[x][y]);

                f.render_widget(
                    Paragraph::new(match &item.lineStatuses[0] {
                        Some(s) => match &s.reason {
                            Some(r) => r,
                            None => "Good Service",
                        },
                        None => "No LineStatus",
                    })
                    .style(Style::default())
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .title("Status")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded),
                    ),
                    chunks[0],
                );
            }
        }
    }
}
