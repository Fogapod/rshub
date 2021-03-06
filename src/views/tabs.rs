use std::io;
use std::sync::Arc;

use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::Spans,
    widgets::BorderType,
    widgets::{Block, ListState, Tabs},
    Frame,
};

use futures::stream::{self, StreamExt};

use crate::app::AppAction;

use crate::input::UserInput;
use crate::states::help::HotKey;
use crate::states::{AppState, StatelessList};
use crate::views::{
    commits::CommitView, servers::ServerView, versions::VersionView, AppView, Drawable, HotKeys,
    InputProcessor, Named,
};

#[derive(Copy, Clone)]
enum Tab {
    Servers,
    Versions,
    Commits,
}

impl Tab {
    async fn name(&self, app: Arc<AppState>) -> String {
        match self {
            Self::Servers => {
                format!("servers [{}]", app.servers.read().await.count())
            }
            Self::Versions => {
                format!("versions [{}]", app.versions.read().await.count())
            }
            Self::Commits => format!("commits [{}]", app.commits.read().await.items.len()),
        }
    }

    const fn all() -> [Self; 3] {
        [Self::Servers {}, Self::Versions {}, Self::Commits {}]
    }

    const fn tab_count() -> usize {
        Self::all().len()
    }
}

impl From<Tab> for usize {
    fn from(value: Tab) -> usize {
        match value {
            Tab::Servers => 0,
            Tab::Versions => 1,
            Tab::Commits => 2,
        }
    }
}

pub struct TabView {
    state: StatelessList<ListState>,
    view_servers: ServerView,
    view_versions: VersionView,
    view_commits: CommitView,
}

impl TabView {
    pub fn new() -> Self {
        let mut state = StatelessList::new(ListState::default(), true);

        state.select_first(Tab::tab_count());

        Self {
            state,
            view_servers: ServerView::new(),
            view_versions: VersionView::new(),
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

impl AppView for TabView {}

impl Named for TabView {
    fn name(&self) -> String {
        format!(
            "Tab: {}",
            match self.selected_tab() {
                Tab::Servers => self.view_servers.name(),
                Tab::Versions => self.view_versions.name(),
                Tab::Commits => self.view_commits.name(),
            }
        )
    }
}

impl HotKeys for TabView {
    fn hotkeys(&self) -> Vec<HotKey> {
        let mut hotkeys = vec![
            HotKey {
                description: "Go to next tab",
                key: KeyCode::Tab,
                modifiers: None,
            },
            HotKey {
                description: "Go Servers tab",
                key: KeyCode::Char('s'),
                modifiers: None,
            },
            HotKey {
                description: "Go versions tab",
                key: KeyCode::Char('v'),
                modifiers: None,
            },
            HotKey {
                description: "Go Commits tab",
                key: KeyCode::Char('c'),
                modifiers: None,
            },
        ];

        hotkeys.append(&mut match self.selected_tab() {
            Tab::Servers => self.view_servers.hotkeys(),
            Tab::Versions => self.view_versions.hotkeys(),
            Tab::Commits => self.view_commits.hotkeys(),
        });

        hotkeys
    }
}

#[async_trait::async_trait]
impl InputProcessor for TabView {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction> {
        match input {
            UserInput::Char('s' | 'S') => {
                self.select_tab(Tab::Servers);
                None
            }
            UserInput::Char('v' | 'V') => {
                self.select_tab(Tab::Versions);
                None
            }
            UserInput::Char('c' | 'C') => {
                self.select_tab(Tab::Commits);
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
                Tab::Versions => self.view_versions.on_input(input, app).await,
                Tab::Commits => self.view_commits.on_input(input, app).await,
            },
        }
    }
}

#[async_trait::async_trait]
impl Drawable for TabView {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: Arc<AppState>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(area);

        let header = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0)])
            .split(chunks[0]);

        let tabs = stream::iter(Tab::all())
            .then(|t| {
                let cloned_app = Arc::clone(&app);
                async move { Spans::from(t.name(cloned_app).await) }
            })
            .collect()
            .await;

        f.render_widget(
            Tabs::new(tabs)
                .block(Block::default().border_type(BorderType::Plain))
                .highlight_style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .divider(DOT)
                .select(self.state.selected().unwrap_or_default()),
            header[0],
        );

        // cannot move this to function because of match limitation for arms
        // even if they implement same trait
        match self.selected_tab() {
            Tab::Servers => self.view_servers.draw(f, chunks[1], app).await,
            Tab::Versions => self.view_versions.draw(f, chunks[1], app).await,
            Tab::Commits => self.view_commits.draw(f, chunks[1], app).await,
        };
    }
}
