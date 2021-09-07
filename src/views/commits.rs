use std::io;
use std::sync::Arc;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::AppAction;
use crate::input::UserInput;
use crate::states::help::HotKey;
use crate::states::{AppState, CommitState, StatelessList};
use crate::views::{Drawable, HotKeys, InputProcessor, Named};

pub struct CommitView {
    // TODO:
    //   - on 1st launch: fetch N latest commits, save latest hash
    //   - on 2nd launch: read latest hash and fetch newer commits
    loaded: bool,

    state: StatelessList<ListState>,
}

impl CommitView {
    pub fn new() -> Self {
        Self {
            loaded: false,
            state: StatelessList::new(ListState::default(), false),
        }
    }
}

#[async_trait::async_trait]
impl Named for CommitView {
    fn name(&self) -> String {
        "Recent Commit List".to_owned()
    }
}

impl HotKeys for CommitView {
    fn hotkeys(&self) -> Vec<HotKey> {
        self.state.hotkeys()
    }
}

#[async_trait::async_trait]
impl InputProcessor for CommitView {
    async fn on_input(&mut self, input: &UserInput, app: Arc<AppState>) -> Option<AppAction> {
        self.state.on_input(input, app.commits.read().await.count())
    }
}

#[async_trait::async_trait]
impl Drawable for CommitView {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(60), Constraint::Min(0)])
            .split(area);

        let commits = &app.commits.read().await.items;

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

        f.render_stateful_widget(list, chunks[0], &mut self.state.state);

        if let Some(i) = self.state.state.selected() {
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
                app.watch_task(tokio::spawn(CommitState::load(app.commits.clone())))
                    .await;
            }

            self.loaded = true;
        }
    }
}
