use std::io;

use tui::layout::Rect;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    symbols::DOT,
    text::{Spans, Text},
    widgets::canvas::{Canvas, Line, Map, MapResolution},
    widgets::BorderType,
    widgets::{Block, Borders, ListState, Paragraph, Tabs, Wrap},
    Frame,
};

use futures::stream::{self, StreamExt};

use crate::app::AppAction;

use crate::geolocation::IP;
use crate::input::UserInput;
use crate::states::{AppState, StatelessList};
use crate::views::{
    commits::CommitView, installations::InstallationView, servers::ServerView, AppView, Drawable,
    InputProcessor,
};

#[derive(Copy, Clone)]
enum Tab {
    Servers,
    Installations,
    Commits,
    Map,
}

impl Tab {
    async fn name(&self, app: &AppState) -> String {
        match self {
            Self::Servers => {
                let servers = &app.servers;

                format!("servers [{}]", servers.count().await)
            }
            Self::Installations => {
                let installations = &app.installations;

                format!("installations [{}]", installations.count().await)
            }
            Self::Commits => "commits".to_owned(),
            Self::Map => "temp map".to_owned(),
        }
    }

    const fn all() -> [Self; 4] {
        [
            Self::Servers {},
            Self::Installations {},
            Self::Commits {},
            Self::Map {},
        ]
    }

    const fn tab_count() -> usize {
        Self::all().len()
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
    state: StatelessList<ListState>,
    view_servers: ServerView,
    view_installations: InstallationView,
    view_commits: CommitView,
}

impl TabView {
    pub fn new() -> Self {
        let mut state = StatelessList::new(ListState::default(), true);

        state.select_first(Tab::tab_count());

        Self {
            state,
            view_servers: ServerView::new(),
            view_installations: InstallationView::new(),
            view_commits: CommitView::new(),
        }
    }

    fn selected_tab(&self) -> Tab {
        *Tab::all()
            .get(self.state.selected().unwrap_or_default())
            .unwrap_or(&Tab::Servers)
    }

    fn select_tab(&mut self, tab: Tab) {
        self.state.select_index(tab.into());
    }
}

#[async_trait::async_trait]
impl InputProcessor for TabView {
    async fn on_input(&mut self, input: &UserInput, app: &AppState) -> Option<AppAction> {
        match input {
            UserInput::Char('q' | 'Q') => Some(AppAction::Exit),
            UserInput::Char('s' | 'S') => {
                self.select_tab(Tab::Servers);
                None
            }
            UserInput::Char('i' | 'I') => {
                self.select_tab(Tab::Installations);
                None
            }
            UserInput::Char('c' | 'C') => {
                self.select_tab(Tab::Commits);
                None
            }
            UserInput::Char('m' | 'M') => {
                self.select_tab(Tab::Map);
                None
            }
            UserInput::Tab => {
                self.state.select_next(Tab::tab_count());
                None
            }
            // cannot move this to function because of match limitation for arms
            // even if they implement same trait
            _ => match self.selected_tab() {
                Tab::Servers => self.view_servers.on_input(input, app).await,
                Tab::Installations => self.view_installations.on_input(input, app).await,
                Tab::Commits => self.view_commits.on_input(input, app).await,
                Tab::Map => None,
            },
        }
    }
}

impl AppView for TabView {}

#[async_trait::async_trait]
impl Drawable for TabView {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    ) {
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

        let titles = stream::iter(Tab::all())
            .then(|t| async move { Spans::from(t.name(app).await) })
            .collect()
            .await;

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
            Tab::Servers => self.view_servers.draw(f, chunks[1], app).await,
            Tab::Installations => self.view_installations.draw(f, chunks[1], app).await,
            Tab::Commits => self.view_commits.draw(f, chunks[1], app).await,
            Tab::Map => draw_map(f, chunks[1], app).await,
        };
    }
}

// temporarily resides here until I decide where to put it
// TODO: render selected with labels by default, all without labels
// TODO: zoom and map navigation
async fn draw_map(f: &mut Frame<'_, CrosstermBackend<io::Stdout>>, area: Rect, app: &AppState) {
    let locations = app.locations.items.read().await;
    let servers = app.servers.items.read().await;

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

            if let Some(user_location) = locations.get(&IP::Local) {
                for sv in servers.values() {
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

            for sv in servers.values() {
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

    // map (canvas specifically) panics with overflow if area is 0
    // area of size 0 could happen on wild terminal resizes
    // this check uses 2 instead of 0 because borders add 2 to each dimension
    if area.height > 2 && area.width > 2 {
        f.render_widget(map, area);
    }
}
