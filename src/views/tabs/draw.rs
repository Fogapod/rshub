use std::io;
use std::sync::Arc;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::Spans,
    widgets::BorderType,
    widgets::{Block, Tabs as TuiTabs},
    Frame,
};

use futures::stream::{self, StreamExt};

use crate::states::AppState;
use crate::views::Draw;

use super::tab::Tab;
use super::Tabs;

#[async_trait::async_trait]
impl Draw for Tabs {
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
            TuiTabs::new(tabs)
                .block(Block::default().border_type(BorderType::Plain))
                .highlight_style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .divider(DOT)
                .select(self.selection.selected().unwrap_or_default()),
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
