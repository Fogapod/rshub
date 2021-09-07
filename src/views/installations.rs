use std::io;
use std::sync::Arc;

use bytesize::ByteSize;

use crossterm::event::KeyCode;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Row, Table, TableState},
    Frame,
};

use crate::app::AppAction;
use crate::datatypes::{game_version::DownloadUrl, installation::InstallationKind};
use crate::input::UserInput;
use crate::states::help::HotKey;
use crate::states::{AppState, StatelessList};
use crate::views::{Drawable, HotKeys, InputProcessor, Named};

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

impl Named for InstallationView {
    fn name(&self) -> String {
        "Installation List".to_owned()
    }
}

impl HotKeys for InstallationView {
    fn hotkeys(&self) -> Vec<HotKey> {
        let mut hotkeys = vec![
            HotKey {
                description: "Refresh installations list",
                key: KeyCode::F(5),
                modifiers: None,
            },
            HotKey {
                description: "Run selected version (installs if needed)",
                key: KeyCode::Enter,
                modifiers: None,
            },
        ];

        hotkeys.append(&mut self.state.hotkeys());

        hotkeys
    }
}

#[async_trait::async_trait]
impl InputProcessor for InstallationView {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction> {
        match input {
            UserInput::Refresh => {
                let mut installations = app.installations.write().await;

                installations.refresh(app.clone()).await;

                if let Some(i) = self.state.selected() {
                    if i >= installations.count() {
                        self.state.unselect();
                    }
                }

                None
            }
            UserInput::Char('d' | 'D') => {
                if let Some(i) = self.state.selected() {
                    Some(AppAction::UninstallVersion(
                        app.installations.read().await.items[i].version.clone(),
                    ))
                } else {
                    None
                }
            }
            UserInput::Char('a' | 'A') => {
                if let Some(i) = self.state.selected() {
                    Some(AppAction::AbortVersionInstallation(
                        app.installations.read().await.items[i].version.clone(),
                    ))
                } else {
                    None
                }
            }
            UserInput::Enter => {
                if let Some(i) = self.state.selected() {
                    Some(AppAction::LaunchVersion(
                        app.installations.read().await.items[i].version.clone(),
                    ))
                } else {
                    None
                }
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
        let installations = &app.installations.read().await.items;

        let mut downloading = Vec::new();

        let items: Vec<Row> = installations
            .iter()
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
                            downloading.push((i.version.clone(), *progress as f64, *total as f64));
                            "downloading".to_owned()
                        }
                        InstallationKind::Installed { .. } => "installed".to_owned(),
                        InstallationKind::Unpacking => "unpacking".to_owned(),
                    },
                    match &i.kind {
                        InstallationKind::Installed { size } => size.to_string(),
                        InstallationKind::Downloading { progress, .. } => {
                            ByteSize::b(*progress).to_string()
                        }
                        _ => "0".to_owned(),
                    },
                ])
            })
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(if !downloading.is_empty() {
                    2 + downloading.len() as u16
                } else {
                    0
                }),
            ])
            .split(area);

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

        if !downloading.is_empty() {
            for download in downloading {
                let ratio = download.1 / download.2;
                let label = Span::styled(
                    format!("downloading {}: {:.2}%", download.0, ratio * 100.0),
                    Style::default().fg(Color::Black),
                );

                let gauge = Gauge::default()
                    .block(
                        Block::default()
                            .title("PROGRESS")
                            .title_alignment(Alignment::Center)
                            .borders(Borders::ALL),
                    )
                    .ratio(ratio)
                    .label(label)
                    .gauge_style(
                        Style::default()
                            .fg(Color::Green)
                            .bg(Color::Red)
                            .add_modifier(Modifier::BOLD),
                    );

                f.render_widget(gauge, chunks[1])
            }
        }
    }
}
