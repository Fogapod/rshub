// TODO: rename this to versions.rs

use std::io;
use std::sync::Arc;

use bytesize::ByteSize;

use crossterm::event::KeyCode;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Row, Table, TableState},
    Frame,
};

use crate::app::AppAction;
use crate::datatypes::{
    game_version::{DownloadUrl, GameVersion},
    installation::InstallationKind,
};
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
        "Version List".to_owned()
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
                description: "Install selected version",
                key: KeyCode::Char('i'),
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

enum Progress {
    Downloading {
        version: GameVersion,
        progress: u64,
        total: Option<u64>,
    },
    Unpacking {
        version: GameVersion,
    },
}

impl Progress {
    fn ratio(&self) -> Option<f64> {
        match self {
            Self::Downloading {
                progress, total, ..
            } => total.map(|v| *progress as f64 / v as f64),
            _ => None,
        }
    }

    fn label(&self) -> Span {
        match self {
            Self::Unpacking { version } => Span::styled(
                format!("unpacking {}: it is a mystery%", version),
                Style::default().fg(Color::Black),
            ),
            Self::Downloading {
                version, progress, ..
            } => {
                if let Some(ratio) = self.ratio() {
                    Span::styled(
                        format!("downloading {}: {:.2}%", version, ratio * 100.0),
                        Style::default().fg(Color::Black),
                    )
                } else {
                    Span::styled(
                        format!("downloading {}: {} / ?", version, ByteSize::b(*progress)),
                        Style::default().fg(Color::Black),
                    )
                }
            }
        }
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
            UserInput::Char('i' | 'I') => {
                if let Some(i) = self.state.selected() {
                    Some(AppAction::InstallVersion(
                        app.installations.read().await.items[i].version.clone(),
                    ))
                } else {
                    None
                }
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

        let mut in_progress = Vec::new();

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
                            in_progress.push(Progress::Downloading {
                                version: i.version.clone(),
                                progress: *progress,
                                total: *total,
                            });
                            "downloading".to_owned()
                        }
                        InstallationKind::Installed { .. } => "installed".to_owned(),
                        InstallationKind::Unpacking => {
                            in_progress.push(Progress::Unpacking {
                                version: i.version.clone(),
                            });
                            "unpacking".to_owned()
                        }
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

        let mut constraints = vec![Constraint::Min(0)];

        if !in_progress.is_empty() {
            constraints.push(Constraint::Length(2 + in_progress.len() as u16));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints.clone())
            .split(area);

        let table = Table::new(items)
            .header(
                Row::new(vec![
                    "VERSION".to_owned(),
                    "STATUS".to_owned(),
                    format!("SIZE [{}]", 0),
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

        if !in_progress.is_empty() {
            let mut progress_bars_constraints = Vec::new();

            // + 2 to account for top and bottom borders
            for _ in 0..in_progress.len() + 2 {
                progress_bars_constraints.push(Constraint::Length(1));
            }

            f.render_widget(
                Block::default()
                    .title("PROGRESS")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL),
                chunks[1],
            );

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(progress_bars_constraints)
                .split(chunks[1]);

            for (i, progress_item) in in_progress.iter().enumerate() {
                let label = progress_item.label();
                let ratio = progress_item.ratio().unwrap_or(1.0);

                let gauge = Gauge::default().ratio(ratio).label(label).gauge_style(
                    Style::default()
                        .fg(Color::Green)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                );

                // + 1 offset for upper border
                f.render_widget(
                    gauge,
                    chunks[i + 1].inner(&Margin {
                        horizontal: 1,
                        vertical: 0,
                    }),
                )
            }
        }
    }
}
