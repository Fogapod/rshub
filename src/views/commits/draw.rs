use std::io;
use std::sync::Arc;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::states::AppState;
use crate::views::Draw;

use super::Commits;

#[async_trait::async_trait]
impl Draw for Commits {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: Arc<AppState>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(60), Constraint::Min(0)])
            .split(area);

        let commits = &self.state.read().await.items;

        let items: Vec<ListItem> = commits
            .iter()
            .map(|c| ListItem::new(c.title.clone()))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("latest commits")
                    .title_alignment(Alignment::Center),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, chunks[0], &mut self.selection.state);

        if let Some(i) = self.selection.state.selected() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(chunks[1]);

            let selected = &commits[i];

            f.render_widget(
                Paragraph::new(Text::from(format!("author: {}", selected.author.name)))
                    .alignment(Alignment::Left)
                    .block(Block::default().borders(Borders::ALL).title("author"))
                    .wrap(Wrap { trim: true }),
                chunks[0],
            );
            f.render_widget(
                Paragraph::new(Text::from(selected.message.clone()))
                    .alignment(Alignment::Left)
                    .block(Block::default().borders(Borders::ALL).title("info"))
                    .wrap(Wrap { trim: true }),
                chunks[1],
            );
        } else {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Length(1)])
                .split(chunks[1]);
            f.render_widget(
                Paragraph::new("select commit").alignment(Alignment::Center),
                chunks[1],
            );
        }

        if !self.loaded {
            if !app.config.offline {
                app.watch_task(tokio::spawn(self.load(Arc::clone(&app))))
                    .await;
            }

            self.loaded = true;
        }
    }
}
