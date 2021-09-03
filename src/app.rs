use std::io;
use std::sync::atomic::AtomicBool;
use std::{collections::HashMap, sync::Arc};

use tui::backend::CrosstermBackend;
use tui::terminal::Frame;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

use crate::config::AppConfig;
use crate::input::UserInput;
use crate::states::AppState;
use crate::views::{tabs::TabView, world::World, AppView, ViewType};

pub enum AppAction {
    OpenView(ViewType),
    CloseView,
    Exit,
}

pub struct App {
    views: HashMap<ViewType, Box<dyn AppView>>,
    view_stack: Vec<ViewType>,

    pub state: Arc<AppState>,

    pub stopped: bool,
    pub panicked: Arc<AtomicBool>,
}

impl App {
    pub async fn new(config: AppConfig) -> Self {
        let panic_bool = Arc::new(AtomicBool::new(false));

        let mut instance = Self {
            state: AppState::new(config, panic_bool.clone()).await,
            views: HashMap::new(),

            view_stack: vec![ViewType::Tab],

            stopped: false,
            panicked: panic_bool,
        };

        instance.register_view(ViewType::Tab, Box::new(TabView::new()));
        instance.register_view(ViewType::World, Box::new(World {}));

        instance
    }

    fn register_view(&mut self, tp: ViewType, view: Box<dyn AppView>) {
        self.views.insert(tp, view);
    }

    pub async fn draw(&mut self, f: &mut Frame<'_, CrosstermBackend<io::Stdout>>) {
        if let Some(tp) = self.view_stack.last() {
            if let Some(widget) = self.views.get_mut(tp) {
                // use tui::layout::{Alignment, Constraint, Direction, Layout};
                // use tui::style::{Color, Modifier, Style};
                // use tui::text::Span;
                // use tui::widgets::{Block, Borders};
                //
                // let chunks = Layout::default()
                //     .constraints(vec![Constraint::Min(0), Constraint::Length(1)])
                //     .direction(Direction::Vertical)
                //     .split(f.size());

                // f.render_widget(
                //     Block::default()
                //         .title(Span::styled(
                //             " DEBUG BUILD ",
                //             Style::default().add_modifier(Modifier::BOLD).bg(Color::Red),
                //         ))
                //         .title_alignment(Alignment::Center)
                //         .borders(Borders::TOP)
                //         .border_style(Style::default().bg(Color::Red)),
                //     chunks[1],
                // );

                widget.draw(f, f.size(), &self.state).await;
            }
        }
    }

    pub(crate) async fn on_input(&mut self, event: &Event) {
        let input = match event {
            Event::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Char('c' | 'C'),
                    modifiers: KeyModifiers::CONTROL,
                } => {
                    self.stop();
                    None
                }
                KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                } => Some(UserInput::Char(c)),
                KeyEvent {
                    code: KeyCode::Left,
                    ..
                } => Some(UserInput::Left),
                KeyEvent {
                    code: KeyCode::Right,
                    ..
                } => Some(UserInput::Right),
                KeyEvent {
                    code: KeyCode::Up, ..
                } => Some(UserInput::Up),
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => Some(UserInput::Down),
                KeyEvent {
                    code: KeyCode::Home,
                    ..
                } => Some(UserInput::Top),
                KeyEvent {
                    code: KeyCode::End, ..
                } => Some(UserInput::Bottom),
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => Some(UserInput::Back),
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => Some(UserInput::Enter),
                KeyEvent {
                    code: KeyCode::Delete | KeyCode::Backspace,
                    ..
                } => Some(UserInput::Delete),
                KeyEvent {
                    code: KeyCode::Tab, ..
                } => Some(UserInput::Tab),
                KeyEvent {
                    code: KeyCode::F(1),
                    ..
                } => Some(UserInput::Help),
                KeyEvent {
                    code: KeyCode::F(5),
                    ..
                } => Some(UserInput::Refresh),
                _ => None,
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => Some(UserInput::Up),
                MouseEventKind::ScrollDown => Some(UserInput::Down),
                _ => None,
            },
            _ => None,
        };

        if let Some(input) = input {
            if let Some(top_widget) = self.view_stack.last() {
                if let Some(widget) = self.views.get_mut(top_widget) {
                    if let Some(action) = widget.on_input(&input, self.state.clone()).await {
                        match action {
                            AppAction::OpenView(view) => {
                                self.view_stack.push(view);
                            }
                            AppAction::CloseView => {
                                self.view_stack.pop();
                            }
                            AppAction::Exit => {
                                self.stop();
                            }
                        }
                    }
                }
            }
        }
    }

    fn stop(&mut self) {
        self.stopped = true;
    }
}
