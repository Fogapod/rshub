use std::io;
use std::sync::Arc;

use tui::backend::CrosstermBackend;
use tui::layout::Rect;
use tui::terminal::Frame;

use crate::app::AppAction;
use crate::datatypes::installation::InstallationAction;
use crate::input::UserInput;
use crate::states::{AppState, StatelessList};
use crate::views::{Drawable, InputProcessor};

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

#[async_trait::async_trait]
impl InputProcessor for ServerView {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction> {
        match input {
            UserInput::Char('d' | 'D') => {
                if let Some(i) = self.state.selected() {
                    let selected = &app.servers.read().await.items[i];

                    app.installations
                        .read()
                        .await
                        .queue
                        .send(InstallationAction::Install(selected.version.clone()))
                        .expect("cannot send install");
                    Some(AppAction::Accepted)
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

        let selected_server = self.state.selected().map(|s| &servers[s]);

        let chunks = Layout::default()
            .constraints([
                Constraint::Min(0),
                Constraint::Length(if selected_server.is_some() { 5 } else { 0 }),
            ])
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
                            format!("SERVERS {} ", DOT,),
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
                            count_offline.to_string(),
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
        if let Some(selected) = selected_server {
            let chunks = Layout::default()
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .direction(Direction::Horizontal)
                .split(chunks[1]);

            let border_style = Style::default().fg(if selected.offline {
                Color::Red
            } else if selected.players == 0 {
                Color::Yellow
            } else {
                Color::White
            });

            let text1 = Text::from(format!(
                r#"version: {}
                   map:     {} ({})
                   address: {}:{}"#,
                selected.version, selected.map, selected.gamemode, selected.ip, selected.port,
            ));

            let selected_location =
                if let Some(location) = app.locations.read().await.items.get(&selected.ip) {
                    format!("{}/{}", location.country, location.city)
                } else {
                    "unknown".to_owned()
                };

            let text2 = Text::from(format!(
                r#"fps:      {}
                   time:     {}
                   location: {}"#,
                selected.fps, selected.time, selected_location
            ));

            let par1 = Paragraph::new(text1)
                .block(
                    Block::default()
                        .borders(Borders::ALL - Borders::RIGHT)
                        .border_style(border_style)
                        .title(Spans::from(vec![
                            Span::styled(
                                format!(" {}", selected.name),
                                Style::default()
                                    .add_modifier(Modifier::BOLD)
                                    .fg(Color::Blue),
                            ),
                            Span::styled(
                                format!(" {} ", DOT),
                                Style::default().add_modifier(Modifier::BOLD),
                            ),
                        ]))
                        .title_alignment(Alignment::Right),
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });

            let par2 = Paragraph::new(text2)
                .block(
                    Block::default()
                        .borders(Borders::ALL - Borders::LEFT)
                        .border_style(border_style)
                        .title(Span::styled(
                            format!("{} ", selected.players),
                            Style::default().add_modifier(Modifier::BOLD).fg(
                                if selected.players > 0 {
                                    Color::Green
                                } else {
                                    Color::Red
                                },
                            ),
                        ))
                        .title_alignment(Alignment::Left),
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });

            f.render_widget(par1, chunks[0]);
            f.render_widget(par2, chunks[1]);
        }

        f.render_stateful_widget(table, chunks[0], &mut self.state.state);
    }
}
