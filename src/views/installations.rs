use std::io;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::AppAction;

use crate::datatypes::installation::InstallationKind;
use crate::input::UserInput;
use crate::states::{AppState, StatelessList};
use crate::views::{Drawable, InputProcessor};

pub struct InstallationView {
    state: StatelessList<ListState>,
}

impl InstallationView {
    pub fn new() -> Self {
        Self {
            state: StatelessList::new(ListState::default(), false),
        }
    }
}

#[async_trait::async_trait]
impl InputProcessor for InstallationView {
    async fn on_input(&mut self, input: &UserInput, app: &AppState) -> Option<AppAction> {
        self.state
            .on_input(input, app.installations.read().await.count())
    }
}

#[async_trait::async_trait]
impl Drawable for InstallationView {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(100)])
            .split(area);

        let installations = &app.installations.read().await.items;

        let items: Vec<ListItem> = installations
            .values()
            .map(|i| {
                ListItem::new(format!(
                    "{}-{} {}",
                    i.version.fork,
                    i.version.build,
                    match &i.kind {
                        InstallationKind::Discovered => "discovered".to_owned(),
                        InstallationKind::Downloading { progress, total } => {
                            format!("downloading {}/{}", progress, total)
                        }
                        InstallationKind::Installed(size) => format!("installed: {} bytes", size),
                        InstallationKind::Corrupted(reason) => format!("corrupted: {}", reason),
                        InstallationKind::Unpacking => "unpacking".to_owned(),
                    }
                ))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, chunks[0], &mut self.state.state);
    }
}
