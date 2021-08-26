use std::io;

use tui::layout::Rect;

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    symbols::DOT,
    text::{Span, Spans, Text},
    widgets::{
        canvas::{Canvas, Line, Map, MapResolution, Rectangle},
        BorderType, ListState,
    },
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Clear, Dataset, Gauge, LineGauge, List,
        ListItem, Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};

use crate::app::{ActionResult, AppState, AppView, Drawable, ViewType};
use crate::input::UserInput;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_items(items: Vec<T>) -> Self {
        Self {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            self.state.select(None);
        } else {
            match self.selected() {
                None => self.state.select(Some(0)),
                Some(i) => {
                    if i < self.items.len() - 1 {
                        self.state.select(Some(i + 1))
                    }
                }
            }
        }
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
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

enum Tab {
    Servers,
    Installations,
    Commits,
}

impl Tab {
    fn name(&self, app: &AppState) -> String {
        match self {
            Self::Servers => {
                let servers = &app.servers;

                format!("servers [{}]", servers.read().len())
            }
            Self::Installations => {
                let servers = &app.servers;

                format!("installations [{}]", servers.read().len())
            }
            Self::Commits => "commits".to_owned(),
        }
    }

    fn all() -> Vec<Self> {
        vec![Self::Servers {}, Self::Installations {}, Self::Commits {}]
    }
}

pub struct TabView {
    state: StatefulList<Tab>,
}

impl TabView {
    pub fn new() -> Self {
        let mut state = StatefulList::with_items(Tab::all());

        // select first item
        state.next();

        Self { state }
    }

    fn index_to_view(&self) -> ViewType {
        match self.state.selected() {
            Some(0) => ViewType::Server,
            Some(1) => ViewType::Installations,
            Some(2) => ViewType::Commits,
            _ => ViewType::Server,
        }
    }
}

impl AppView for TabView {
    fn view_type(&self) -> ViewType {
        ViewType::Tab
    }

    fn on_input(&mut self, input: &UserInput, _: &AppState) -> ActionResult {
        match input {
            UserInput::Left => {
                self.state.previous();
                ActionResult::ReplaceView(self.index_to_view())
            }
            UserInput::Right => {
                self.state.next();
                ActionResult::ReplaceView(self.index_to_view())
            }
            _ => ActionResult::Continue,
        }
    }
}

impl Drawable for TabView {
    fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &mut AppState,
    ) -> Option<Rect> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(area);

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            // TODO: figure out how to give tabs higher priority, drawing on top of name
            .constraints([Constraint::Min(0), Constraint::Length(25)])
            .split(chunks[0]);

        let paragraph = Paragraph::new(Text::from(format!(
            // NOTE: space at the end to prevent italic text go off screen
            "{}-{}\u{00a0}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )))
        .alignment(Alignment::Right)
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        )
        .wrap(Wrap { trim: false });

        f.render_widget(paragraph, header_chunks[1]);

        let titles = self
            .state
            .items
            .iter()
            .map(|t| Spans::from(Span::styled(t.name(app), Style::default().fg(Color::Green))))
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().border_type(BorderType::Plain))
            .highlight_style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(DOT)
            .select(self.state.selected().unwrap_or(0));

        f.render_widget(tabs, header_chunks[0]);

        Some(chunks[1])
    }
}
