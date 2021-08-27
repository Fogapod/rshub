use std::cmp::Ordering;

use std::io;

use tui::layout::Rect;

use tui::backend::CrosstermBackend;
use tui::terminal::Frame;

use crate::datatypes::server::Server;
use crate::input::UserInput;
use crate::states::AppState;
use crate::views::{ActionResult, AppView, Drawable, StatelessList, ViewType};

use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
};

pub struct ServerView {
    state: StatelessList,
}

impl ServerView {
    pub fn new() -> Self {
        Self {
            state: StatelessList::new(),
        }
    }
}

impl Drawable for ServerView {
    fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    ) -> Option<Rect> {
        let servers = app.servers.servers.read();

        let offline_servers = servers.values().filter(|s| s.offline).count();

        let mut servers_to_be_sorted = servers.values().collect::<Vec<&Server>>();
        // TODO: custom sorts by each field
        // TODO: search by pattern
        // https://stackoverflow.com/a/40369685
        servers_to_be_sorted.sort_by(|a, b| match a.data.players.cmp(&b.data.players).reverse() {
            Ordering::Equal => a.data.name.cmp(&b.data.name),
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
                Style::default().fg(Color::Red)
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

        if let Some(selected) = selected_server {
            let chunks = Layout::default()
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .direction(Direction::Horizontal)
                .split(chunks[1]);

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
            let text2 = Text::from(format!(
                r#"fps:  {}
                   time: {}"#,
                selected.data.fps, selected.data.time
            ));

            let par1 = Paragraph::new(text1)
                .block(
                    Block::default()
                        .borders(Borders::ALL - Borders::RIGHT)
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

        drop(servers);

        f.render_stateful_widget(table, chunks[0], &mut self.state.state);

        None
    }
}

impl AppView for ServerView {
    fn view_type(&self) -> ViewType {
        ViewType::Servers
    }

    fn on_input(&mut self, input: &UserInput, app: &AppState) -> ActionResult {
        match input {
            UserInput::Up => {
                self.state.previous(app.servers.servers.read().len());
                ActionResult::Stop
            }
            UserInput::Down => {
                self.state.next(app.servers.servers.read().len());
                ActionResult::Stop
            }
            UserInput::Back => {
                self.state.unselect();
                ActionResult::Stop
            }
            _ => ActionResult::Continue,
        }
    }
}
