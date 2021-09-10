use std::io;
use std::sync::Arc;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Row, Table},
    Frame,
};

use bytesize::ByteSize;

use crate::datatypes::game_version::DownloadUrl;
use crate::datatypes::game_version::GameVersion;
use crate::datatypes::installation::InstallationKind;
use crate::states::AppState;
use crate::views::Draw;

use super::Versions;

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
impl Draw for Versions {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: Arc<AppState>,
    ) {
        let versions = &self.state.read().await.items;

        let mut total_size = 0;
        let mut in_progress = Vec::new();

        let items: Vec<Row> = versions
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
                            total_size += progress;

                            in_progress.push(Progress::Downloading {
                                version: i.version.clone(),
                                progress: *progress,
                                total: *total,
                            });
                            "downloading".to_owned()
                        }
                        InstallationKind::Installed { size, .. } => {
                            total_size += size;

                            "installed".to_owned()
                        }
                        InstallationKind::Unpacking => {
                            in_progress.push(Progress::Unpacking {
                                version: i.version.clone(),
                            });
                            "unpacking".to_owned()
                        }
                    },
                    match &i.kind {
                        InstallationKind::Installed { size } => ByteSize::b(*size).to_string(),
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
                    format!("SIZE [{}]", ByteSize::b(total_size)),
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

        f.render_stateful_widget(table, chunks[0], &mut self.selection.state);

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
