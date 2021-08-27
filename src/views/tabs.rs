use std::io;

use tui::layout::Rect;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::{Span, Spans, Text},
    widgets::BorderType,
    widgets::{Block, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::input::UserInput;
use crate::states::AppState;
use crate::views::{ActionResult, AppView, Drawable, StatefulList, ViewType};

enum Tab {
    Servers,
    Installations,
    Commits,
}

impl Tab {
    fn name(&self, app: &AppState) -> String {
        match self {
            Self::Servers => {
                let servers = &app.servers;

                format!("servers [{}]", servers.servers.read().len())
            }
            Self::Installations => {
                let servers = &app.servers;

                format!("installations [{}]", servers.servers.read().len())
            }
            Self::Commits => "commits".to_owned(),
        }
    }

    fn all() -> Vec<Self> {
        vec![Self::Servers {}, Self::Installations {}, Self::Commits {}]
    }
}

pub struct TabView {
    state: StatefulList<Tab>,
}

impl TabView {
    pub fn new() -> Self {
        let mut state = StatefulList::with_items(Tab::all());

        // select first item
        state.next(false);

        Self { state }
    }

    fn index_to_view(&self) -> ViewType {
        match self.state.selected() {
            Some(0) => ViewType::Servers,
            Some(1) => ViewType::Installations,
            Some(2) => ViewType::Commits,
            _ => ViewType::Servers,
        }
    }

    const fn view_to_index(view: ViewType) -> usize {
        match view {
            ViewType::Servers => 0,
            ViewType::Installations => 1,
            ViewType::Commits => 2,
            // TODO: REMOVE THIS!!! SEPARATE ENUM FOR TABS OR SOMETHING ELSE
            _ => 0,
        }
    }
}

impl AppView for TabView {
    fn view_type(&self) -> ViewType {
        ViewType::Tab
    }

    fn on_input(&mut self, input: &UserInput, _: &AppState) -> ActionResult {
        match input {
            UserInput::Char('q') => ActionResult::Exit,
            // Servers
            UserInput::Char('s') => {
                self.state
                    .select_index(Self::view_to_index(ViewType::Servers));
                ActionResult::ReplaceView(self.index_to_view())
            }
            // Installations
            UserInput::Char('i') => {
                self.state
                    .select_index(Self::view_to_index(ViewType::Installations));
                ActionResult::ReplaceView(self.index_to_view())
            }
            // Commits
            UserInput::Char('c') => {
                self.state
                    .select_index(Self::view_to_index(ViewType::Commits));
                ActionResult::ReplaceView(self.index_to_view())
            }

            UserInput::Tab => {
                self.state.next(true);
                ActionResult::ReplaceView(self.index_to_view())
            }
            _ => ActionResult::Continue,
        }
    }
}

impl Drawable for TabView {
    fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    ) -> Option<Rect> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
            .split(area);

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            // TODO: figure out how to give tabs higher priority, drawing on top of name
            .constraints([Constraint::Min(0), Constraint::Length(25)])
            .split(chunks[0]);

        let paragraph = Paragraph::new(Text::from(format!(
            // NOTE: space at the end to prevent italic text go off screen
            // normal space is getting trimmed, so have to use this weird one
            "{}-{}\u{00a0}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )))
        .alignment(Alignment::Right)
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        )
        .wrap(Wrap { trim: false });

        f.render_widget(paragraph, header_chunks[1]);

        let titles = self
            .state
            .items
            .iter()
            .map(|t| Spans::from(Span::styled(t.name(app), Style::default().fg(Color::Green))))
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().border_type(BorderType::Plain))
            .highlight_style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(DOT)
            .select(self.state.selected().unwrap_or(0));

        f.render_widget(tabs, header_chunks[0]);

        Some(chunks[1])
    }
}
