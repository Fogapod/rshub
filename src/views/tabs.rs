use std::io;

use tui::layout::Rect;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    symbols::DOT,
    text::{Span, Spans, Text},
    widgets::canvas::{Canvas, Line, Map, MapResolution},
    widgets::BorderType,
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::app::ActionResult;
use crate::geolocation::IP;
use crate::input::UserInput;
use crate::states::{AppState, StatefulList};
use crate::views::{
    commits::CommitView, installations::InstallationView, servers::ServerView, AppView, Drawable,
    InputProcessor,
};

enum Tab {
    Servers,
    Installations,
    Commits,
    Map,
}

impl Tab {
    fn name(&self, app: &AppState) -> String {
        match self {
            Self::Servers => {
                let servers = &app.servers;

                format!("servers [{}]", servers.items.read().len())
            }
            Self::Installations => {
                let installations = &app.installations;

                format!("installations [{}]", installations.items.read().len())
            }
            Self::Commits => "commits".to_owned(),
            Self::Map => "temp map".to_owned(),
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::Servers {},
            Self::Installations {},
            Self::Commits {},
            Self::Map {},
        ]
    }
}

impl From<Tab> for usize {
    fn from(value: Tab) -> usize {
        match value {
            Tab::Servers => 0,
            Tab::Installations => 1,
            Tab::Commits => 2,
            Tab::Map => 3,
        }
    }
}

pub struct TabView {
    state: StatefulList<Tab>,
    view_servers: ServerView,
    view_installations: InstallationView,
    view_commits: CommitView,
}

impl TabView {
    pub fn new() -> Self {
        let mut state = StatefulList::with_items(Tab::all());

        // select first item
        state.next(false);

        Self {
            state,
            view_servers: ServerView::new(),
            view_installations: InstallationView::new(),
            view_commits: CommitView::new(),
        }
    }

    fn selected_tab(&self) -> Tab {
        match self.state.selected() {
            Some(0) => Tab::Servers,
            Some(1) => Tab::Installations,
            Some(2) => Tab::Commits,
            Some(3) => Tab::Map,
            // not possible
            _ => Tab::Servers,
        }
    }

    fn select_tab(&mut self, tab: Tab) {
        self.state.select_index(tab.into());
    }
}

impl InputProcessor for TabView {
    fn on_input(&mut self, input: &UserInput, app: &AppState) -> ActionResult {
        match input {
            UserInput::Char('q') => ActionResult::Exit,
            UserInput::Char('s') => {
                self.select_tab(Tab::Servers);
                ActionResult::Continue
            }
            UserInput::Char('i') => {
                self.select_tab(Tab::Installations);
                ActionResult::Continue
            }
            UserInput::Char('c') => {
                self.select_tab(Tab::Commits);
                ActionResult::Continue
            }
            UserInput::Char('m') => {
                self.select_tab(Tab::Map);
                ActionResult::Continue
            }
            UserInput::Tab => {
                self.state.next(true);
                ActionResult::Continue
            }
            // cannot move this to function because of match limitation for arms
            // even if they implement same trait
            _ => match self.selected_tab() {
                Tab::Servers => self.view_servers.on_input(input, app),
                Tab::Installations => self.view_installations.on_input(input, app),
                Tab::Commits => self.view_commits.on_input(input, app),
                Tab::Map => ActionResult::Continue,
            },
        }
    }
}

impl AppView for TabView {}

impl Drawable for TabView {
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect, app: &AppState) {
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
            // normal space is getting trimmed, so have to use this weird one
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

        let selected = self.state.selected().unwrap_or(0);

        let tabs = Tabs::new(titles)
            .block(Block::default().border_type(BorderType::Plain))
            .highlight_style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(DOT)
            .select(selected);

        f.render_widget(tabs, header_chunks[0]);

        // cannot move this to function because of match limitation for arms
        // even if they implement same trait
        match self.selected_tab() {
            Tab::Servers => self.view_servers.draw(f, chunks[1], app),
            Tab::Installations => self.view_installations.draw(f, chunks[1], app),
            Tab::Commits => self.view_commits.draw(f, chunks[1], app),
            Tab::Map => draw_map(f, chunks[1], app),
        };
    }
}

// temporarily resides here until I decide where to put it
// TODO: render selected with labels by default, all without labels
// TODO: zoom and map navigation
fn draw_map(f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(100)])
        .split(area);

    let map = Canvas::default()
        .block(Block::default().borders(Borders::ALL))
        .marker(symbols::Marker::Braille)
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
        .paint(|ctx| {
            ctx.draw(&Map {
                color: Color::Blue,
                resolution: MapResolution::High,
            });
            ctx.layer();

            // acquire lock once instead of doing it 20 times ahead
            let locations = app.locations.items.read();

            if let Some(user_location) = locations.get(&IP::Local) {
                for sv in app.servers.items.read().values() {
                    if let Some(location) = locations.get(&IP::Remote(sv.data.ip.clone())) {
                        ctx.draw(&Line {
                            x1: user_location.longitude,
                            y1: user_location.latitude,
                            x2: location.longitude,
                            y2: location.latitude,
                            color: Color::Yellow,
                        });
                    }
                }

                ctx.print(
                    user_location.longitude,
                    user_location.latitude,
                    "X",
                    Color::Red,
                );
            }

            for sv in app.servers.items.read().values() {
                if let Some(location) = locations.get(&IP::Remote(sv.data.ip.clone())) {
                    let color = if sv.data.players != 0 {
                        Color::Green
                    } else {
                        Color::Red
                    };
                    ctx.print(location.longitude, location.latitude, "O", color);
                }
            }
        });

    f.render_widget(map, chunks[0]);
}
