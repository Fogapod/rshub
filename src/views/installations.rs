use std::io;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Borders, ListState},
    Frame,
};

use crate::app::ActionResult;

use crate::input::UserInput;
use crate::states::{AppState, StatelessList};
use crate::views::{Drawable, InputProcessor};

pub struct InstallationView {
    state: StatelessList<ListState>,
}

impl InstallationView {
    pub fn new() -> Self {
        Self {
            state: StatelessList::new(ListState::default()),
        }
    }
}

impl InputProcessor for InstallationView {
    fn on_input(&mut self, input: &UserInput, app: &AppState) -> ActionResult {
        self.state.on_input(input, app.commits.count())
    }
}

impl Drawable for InstallationView {
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>, area: Rect, _: &AppState) {
        let chunks = Layout::default()
            .constraints(vec![Constraint::Percentage(100)])
            .split(area);

        let block = Block::default().borders(Borders::ALL).title("WIP");

        f.render_widget(block, chunks[0]);
    }
}
