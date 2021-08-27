use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::datatypes::commit::Commit;
use crate::input::UserInput;
use crate::states::AppState;
use crate::views::{ActionResult, AppView, Drawable, StatelessList, ViewType};

pub struct CommitView {
    // FIXME: dynamic commits somehow
    loaded: bool,

    state: StatelessList,
}

impl CommitView {
    pub fn new() -> Self {
        Self {
            loaded: false,
            state: StatelessList::new(),
        }
    }
}

impl AppView for CommitView {
    fn view_type(&self) -> ViewType {
        ViewType::Commits
    }

    fn on_input(&mut self, input: &UserInput, app: &AppState) -> ActionResult {
        match input {
            UserInput::Up => {
                self.state.previous(app.commits.commits.read().len());
                ActionResult::Stop
            }
            UserInput::Down => {
                self.state.next(app.commits.commits.read().len());
                ActionResult::Stop
            }
            UserInput::Back => {
                self.state.unselect();
                ActionResult::Stop
            }
            _ => ActionResult::Continue,
        }
    }
}

impl Drawable for CommitView {
    fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    ) -> Option<Rect> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(area);

        let commits = app.commits.commits.read();

        let items = commits.values().map(|c| Row::new(vec![c.sha.clone()]));

        let list = Table::new(items)
            .block(Block::default().borders(Borders::ALL).title("hashes"))
            .widths(&[Constraint::Percentage(100)])
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(list, chunks[0], &mut self.state.state);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(chunks[1]);

        let selected: Option<Commit> = None;

        let text = if let Some(selected) = selected {
            Text::from(format!("author: {}", selected.author.name))
        } else {
            Text::from(format!("author: {}", "Doobly"))
        };

        let par = Paragraph::new(text)
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::ALL).title("author"))
            .wrap(Wrap { trim: true });

        f.render_widget(par, chunks[0]);
        f.render_widget(
            Block::default().borders(Borders::ALL).title("info"),
            chunks[1],
        );

        drop(commits);

        if !self.loaded {
            app.commits.load();
            self.loaded = true;
        }

        None
    }
}
