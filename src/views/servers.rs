use std::cmp::Ordering;
use std::io;

use tui::layout::Rect;

use tui::backend::CrosstermBackend;
use tui::terminal::Frame;

use crate::app::AppAction;
use crate::datatypes::server::Server;
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
    async fn on_input(&mut self, input: &UserInput, app: &AppState) -> Option<AppAction> {
        self.state.on_input(input, app.servers.read().await.count())
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

        let offline_servers = servers.values().filter(|s| s.offline).count();

        let mut servers_to_be_sorted = servers.values().collect::<Vec<&Server>>();
        // TODO: custom sorts by each field
        // TODO: search by pattern
        // sorting priorities:
        //  - server is online
        //  - player count
        //  - server name
        // https://stackoverflow.com/a/40369685
        servers_to_be_sorted.sort_by(|a, b| match a.offline.cmp(&b.offline) {
            Ordering::Equal => match a.data.players.cmp(&b.data.players).reverse() {
                Ordering::Equal => a.data.name.cmp(&b.data.name),
                other => other,
            },
            other => other,
        });

        let selected_server = self.state.selected().map(|s| servers_to_be_sorted[s]);

        let chunks = Layout::default()
            .constraints([
                Constraint::Min(0),
                Constraint::Length(if selected_server.is_some() { 5 } else { 0 }),
            ])
            .direction(Direction::Vertical)
            .split(area);

        let rows = servers_to_be_sorted.iter().map(|s| {
            let style = if s.offline {
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT)
            } else if s.data.players == 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };

            Row::new(vec![
                s.data.name.clone(),
                s.data.build.to_string(),
                s.data.map.clone(),
                s.data.players.to_string(),
            ])
            .style(style)
        });

        let table = Table::new(rows)
            .header(
                Row::new(vec!["NAME", "BUILD", "MAP", "PLAYERS"]).style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(
                Block::default()
                    .title(Span::styled(
                        format!("SERVERS {}:{}", servers_to_be_sorted.len(), offline_servers),
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL),
            )
            .widths(&[
                Constraint::Percentage(45),
                Constraint::Percentage(15),
                Constraint::Percentage(25),
                Constraint::Percentage(15),
            ])
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
            } else {
                Color::White
            });

            let text1 = Text::from(format!(
                r#"version: {} {}
                   map:     {} ({})
                   address: {}:{}"#,
                selected.data.fork,
                selected.data.build,
                selected.data.map,
                selected.data.gamemode,
                selected.data.ip,
                selected.data.port,
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
                selected.data.fps, selected.data.time, selected_location
            ));

            let par1 = Paragraph::new(text1)
                .block(
                    Block::default()
                        .borders(Borders::ALL - Borders::RIGHT)
                        .border_style(border_style)
                        .title(Spans::from(vec![
                            Span::styled(
                                format!(" {}", selected.data.name),
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
                            format!("{} ", selected.data.players),
                            Style::default().add_modifier(Modifier::BOLD).fg(
                                if selected.data.players > 0 {
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
