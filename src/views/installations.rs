use std::io;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Row, Table, TableState},
    Frame,
};

use crate::app::AppAction;
use crate::datatypes::{installation::InstallationKind, server::DownloadUrl};
use crate::input::UserInput;
use crate::states::{AppState, InstallationsState, StatelessList};
use crate::views::{Drawable, InputProcessor};

pub struct InstallationView {
    state: StatelessList<TableState>,
}

impl InstallationView {
    pub fn new() -> Self {
        Self {
            state: StatelessList::new(TableState::default(), false),
        }
    }
}

#[async_trait::async_trait]
impl InputProcessor for InstallationView {
    async fn on_input(&mut self, input: &UserInput, app: &AppState) -> Option<AppAction> {
        match input {
            UserInput::Refresh => {
                InstallationsState::spawn_installation_finder(
                    &app.config,
                    app.installations.clone(),
                )
                .await;
                return Some(AppAction::Accepted);
            }
            _ => self
                .state
                .on_input(input, app.installations.read().await.count()),
        }
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

        let items: Vec<Row> = installations
            .iter()
            .rev()
            .map(|i| {
                Row::new(vec![
                    format!("{}-{}", i.version.fork, i.version.build),
                    match &i.kind {
                        InstallationKind::Discovered => {
                            format!(
                                "discovered{}",
                                match i.version.download {
                                    DownloadUrl::Untrusted(_) => " [untrusted download]",
                                    DownloadUrl::Invalid(_) => " [invalid download]",
                                    _ => "",
                                }
                            )
                        }
                        InstallationKind::Downloading { progress, total } => {
                            format!("downloading {}/{}", progress, total)
                        }
                        InstallationKind::Installed { .. } => "installed".to_owned(),
                        InstallationKind::Unpacking => "unpacking".to_owned(),
                    },
                    match &i.kind {
                        InstallationKind::Installed { size } => format!("{}", size),
                        InstallationKind::Downloading { progress, .. } => format!("{}", progress),
                        _ => "0".to_owned(),
                    },
                ])
            })
            .collect();

        let table = Table::new(items)
            .header(
                Row::new(vec![
                    "VERSION".to_owned(),
                    "STATUS".to_owned(),
                    "SIZE".to_owned(),
                ])
                .style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(Block::default().borders(Borders::ALL))
            .widths(&[
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 6),
            ])
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(table, chunks[0], &mut self.state.state);
    }
}
