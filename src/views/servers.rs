use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{collections::HashMap, sync::Arc};

use parking_lot::{Condvar, Mutex, RwLock};
use std::io;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::TableState;

use tui::backend::CrosstermBackend;
use tui::terminal::Frame;

use crate::app::{ActionResult, AppState, AppView, Drawable, ViewType};
use crate::geolocation::{Location, IP};
use crate::input::UserInput;
use crate::types::{Server, ServerListData};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    symbols::DOT,
    text::{Span, Spans, Text},
    widgets::{
        canvas::{Canvas, Line, Map, MapResolution, Rectangle},
        BorderType,
    },
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Clear, Dataset, Gauge, LineGauge, List,
        ListItem, Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub struct StatelessList {
    pub state: TableState,
}

impl StatelessList {
    pub fn new() -> Self {
        Self {
            state: TableState::default(),
        }
    }

    pub fn next(&mut self, item_count: usize) {
        if item_count == 0 {
            self.state.select(None);
        } else {
            match self.selected() {
                None => self.state.select(Some(0)),
                Some(i) => {
                    if i < item_count - 1 {
                        self.state.select(Some(i + 1))
                    }
                }
            }
        }
    }

    pub fn previous(&mut self, item_count: usize) {
        if item_count == 0 {
            self.state.select(None);
        } else {
            match self.state.selected() {
                None => self.state.select(Some(0)),
                Some(i) => {
                    if i != 0 {
                        self.state.select(Some(i - 1))
                    }
                }
            }
        }
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }
}

pub struct ServerView {
    state: StatelessList,
    servers: Arc<RwLock<HashMap<String, Server>>>,
}

impl ServerView {
    pub fn new(servers: Arc<RwLock<HashMap<String, Server>>>) -> Self {
        Self {
            state: StatelessList::new(),
            servers,
        }
    }

    pub fn update(&mut self, data: ServerListData) {
        let mut servers = self.servers.write();
        let mut existing = servers.clone();

        for sv in data.servers {
            if let Some(sv_existing) = servers.get_mut(&sv.ip) {
                existing.remove(&sv.ip);

                if calculate_hash(&sv_existing.data) != calculate_hash(&sv) {
                    sv_existing.updated = true;
                }

                sv_existing.offline = false;
                sv_existing.data = sv;
            } else {
                servers.insert(sv.ip.clone(), Server::new(&sv));
            }
        }

        for ip in existing.keys() {
            servers.get_mut(ip).unwrap().offline = true;
        }
    }
}

impl Drawable for ServerView {
    fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &mut AppState,
    ) -> Option<Rect> {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .direction(Direction::Horizontal)
            .split(area);

        let servers = app.servers.read();

        let offline_servers = servers.values().filter(|s| s.offline).count();

        let mut servers_to_be_sorted = servers.values().collect::<Vec<&Server>>();
        // TODO: custom sorts by each field
        // TODO: search by pattern
        // https://stackoverflow.com/a/40369685
        servers_to_be_sorted.sort_by(|a, b| match a.data.players.cmp(&b.data.players).reverse() {
            Ordering::Equal => a.data.name.cmp(&b.data.name),
            other => other,
        });

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
                        format!("SERVERS {}:{}", servers.len(), offline_servers),
                        Style::default().add_modifier(Modifier::BOLD),
                    ))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Plain),
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

        drop(servers);
        //let mut servers = app.servers.write();

        f.render_stateful_widget(table, chunks[0], &mut self.state.state);

        None
    }
}

impl AppView for ServerView {
    fn view_type(&self) -> ViewType {
        ViewType::Server
    }

    fn on_input(&mut self, input: &UserInput, app: &AppState) -> ActionResult {
        match input {
            UserInput::Up => {
                self.state.previous(app.servers.read().len());
                ActionResult::Stop
            }
            UserInput::Down => {
                self.state.next(app.servers.read().len());
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
