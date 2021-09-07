use std::io;
use std::sync::Arc;

use crossterm::event::KeyCode;

use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::terminal::Frame;

use crate::app::AppAction;
use crate::datatypes::server::Server;
use crate::input::UserInput;
use crate::states::help::HotKey;
use crate::states::{AppState, StatelessList};
use crate::views::{Drawable, HotKeys, InputProcessor, Named, ViewType};

use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Wrap},
};

pub struct ServerView {
    state: StatelessList<TableState>,
}

impl ServerView {
    pub fn new() -> Self {
        Self {
            state: StatelessList::new(TableState::default(), false),
        }
    }
}

impl Named for ServerView {
    fn name(&self) -> String {
        "Server List".to_owned()
    }
}

impl HotKeys for ServerView {
    fn hotkeys(&self) -> Vec<HotKey> {
        let mut hotkeys = vec![
            #[cfg(feature = "geolocation")]
            HotKey {
                description: "Show world map",
                key: KeyCode::Char('m'),
                modifiers: None,
            },
            HotKey {
                description: "Connect to selected server (installs version if needed)",
                key: KeyCode::Enter,
                modifiers: None,
            },
        ];

        hotkeys.append(&mut self.state.hotkeys());

        hotkeys
    }
}

#[async_trait::async_trait]
impl InputProcessor for ServerView {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction> {
        match input {
            #[cfg(feature = "geolocation")]
            UserInput::Char('m' | 'M') => Some(AppAction::OpenView(ViewType::World)),
            UserInput::Enter => {
                if let Some(i) = self.state.selected() {
                    let server = &app.servers.read().await.items[i];

                    Some(AppAction::ConnectToServer {
                        version: server.version.clone(),
                        address: server.address.clone(),
                    })
                } else {
                    None
                }
            }
            _ => self.state.on_input(input, app.servers.read().await.count()),
        }
    }
}

#[async_trait::async_trait]
impl Drawable for ServerView {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    ) {
        let servers = &app.servers.read().await.items;

        let mut count_online = 0;
        let mut count_no_players = 0;
        let mut count_offline = 0;

        let mut count_players = 0;

        for s in servers {
            count_players += s.players;

            if s.offline {
                count_offline += 1;
            } else if s.players == 0 {
                count_no_players += 1;
            } else {
                count_online += 1;
            }
        }

        let chunks = Layout::default()
            .constraints([Constraint::Min(0), Constraint::Length(5)])
            .direction(Direction::Vertical)
            .split(area);

        // "BUILD".len()
        let mut longest_build_name = 5;
        let mut longest_map_name = 3;

        let rows: Vec<Row> = servers
            .iter()
            .map(|s| {
                if s.version.build.len() > longest_build_name {
                    longest_build_name = s.version.build.len();
                }
                if s.map.len() > longest_map_name {
                    longest_map_name = s.map.len();
                }

                let style = if s.offline {
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT)
                } else if s.players == 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                };

                Row::new(vec![
                    s.name.clone(),
                    s.version.build.clone(),
                    s.map.clone(),
                    s.players.to_string(),
                ])
                .style(style)
            })
            .collect();

        let pop_header = format!("POP [{}]", count_players);

        let widths = [
            Constraint::Percentage(60),
            Constraint::Length(longest_build_name as u16),
            // until https://github.com/fdehau/tui-rs/issues/525 is fixed
            Constraint::Length(longest_map_name as u16),
            Constraint::Length(pop_header.len() as u16),
        ];

        let table = Table::new(rows)
            .header(
                Row::new(vec![
                    "NAME".to_owned(),
                    "BUILD".to_owned(),
                    "MAP".to_owned(),
                    pop_header,
                ])
                .style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(
                Block::default()
                    .title(Spans::from(vec![
                        Span::styled(
                            format!(" SERVERS {} ", DOT,),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            count_online.to_string(),
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Green),
                        ),
                        Span::styled("-", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            count_no_players.to_string(),
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Yellow),
                        ),
                        Span::styled("-", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("{} ", count_offline),
                            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                        ),
                    ]))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL),
            )
            .widths(&widths)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        // draw server info
        if let Some(selected) = self.state.selected().map(|s| &servers[s]) {
            draw_server_info(f, chunks[1], app, selected).await;
        } else {
            draw_info(f, chunks[1], app);
        }

        f.render_stateful_widget(table, chunks[0], &mut self.state.state);
    }
}

async fn draw_server_info(
    f: &mut Frame<'_, CrosstermBackend<io::Stdout>>,
    area: Rect,
    app: &AppState,
    selected: &Server,
) {
    #[cfg(feature = "geolocation")]
    let selected_location =
        if let Some(location) = app.locations.read().await.items.get(&selected.address.ip) {
            format!("{}/{}", location.country, location.city)
        } else {
            "unknown".to_owned()
        };
    #[cfg(not(feature = "geolocation"))]
    let selected_location = "unknown".to_owned();

    let rows = vec![
        Row::new(vec![
            format!("version : {}", selected.version),
            format!("fps      : {}", selected.fps),
        ]),
        Row::new(vec![
            format!("map     : {} ({})", selected.map, selected.gamemode),
            format!("time     : {}", selected.time),
        ]),
        Row::new(vec![
            format!("address : {}", selected.address),
            format!("location : {}", selected_location),
        ]),
    ];

    let table = Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if selected.offline {
                    Color::Red
                } else if selected.players == 0 {
                    Color::Yellow
                } else {
                    Color::White
                }))
                .title(Spans::from(vec![
                    Span::styled(
                        selected.name.clone(),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Blue),
                    ),
                    Span::styled(
                        format!(" {} ", DOT),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{} ", selected.players),
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(if selected.players > 0 {
                                Color::Green
                            } else {
                                Color::Red
                            }),
                    ),
                ]))
                .title_alignment(Alignment::Left),
        )
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(table, area);
}

fn draw_info(f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(100)])
        .split(area);

    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                format!(
                    " {} {} {} ",
                    env!("CARGO_PKG_NAME"),
                    DOT,
                    env!("CARGO_PKG_VERSION")
                ),
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .title_alignment(Alignment::Center),
        chunks[0],
    );

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    f.render_widget(
        Paragraph::new(Text::from(
            r#"
            help :
            build mode :
            data directory :"#,
        ))
        .alignment(Alignment::Right)
        .wrap(Wrap { trim: true }),
        chunks[0],
    );
    f.render_widget(
        Paragraph::new(Text::from(format!(
            r#"
 F1
 {}
 {}"#,
            if cfg!(debug_assertions) {
                "DEBUG"
            } else {
                "RELEASE"
            },
            app.config.dirs.data_dir.display()
        )))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false }),
        chunks[1],
    );
}
