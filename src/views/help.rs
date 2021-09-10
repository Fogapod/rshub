use std::io;
use std::sync::Arc;

use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::AppAction;
use crate::datatypes::hotkey::HotKey;
use crate::input::UserInput;
use crate::states::AppState;
use crate::views::{AppView, Draw, HotKeys, Input, Name};

pub struct Help {}

impl AppView for Help {}

#[async_trait::async_trait]
impl Name for Help {
    fn name(&self) -> String {
        "Help Screen".to_owned()
    }
}

#[async_trait::async_trait]
impl HotKeys for Help {
    fn hotkeys(&self) -> Vec<HotKey> {
        vec![HotKey {
            description: "Close help",
            key: KeyCode::Esc,
            modifiers: None,
        }]
    }
}

#[async_trait::async_trait]
impl Input for Help {
    async fn on_input(&mut self, input: &UserInput, _: Arc<AppState>) -> Option<AppAction> {
        match input {
            UserInput::Back => Some(AppAction::CloseView),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl Draw for Help {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: Arc<AppState>,
    ) {
        let help = app.help.lock().unwrap();

        let list_length = (help.global_hotkeys.len() + help.local_hotkeys.len()) as u16
        + 2  // outer border
        + 1  // GLOBAL label
        + 1; // LOCAL label
        let vertical_margin = if list_length < area.height {
            (area.height - list_length) / 2
        } else {
            0
        };

        // border
        f.render_widget(
            Block::default()
                .title(help.view_name.clone())
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL),
            area,
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(vertical_margin),
                Constraint::Length(1), // GLOBAL label
                Constraint::Length(help.global_hotkeys.len() as u16),
                Constraint::Length(1), // LOCAL label
                Constraint::Length(help.local_hotkeys.len() as u16),
            ])
            .split(area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }));

        // labels
        f.render_widget(
            Block::default()
                .title(Span::styled(
                    "GLOBAL",
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center),
            chunks[1],
        );
        f.render_widget(
            Block::default()
                .title(Span::styled(
                    "LOCAL",
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center),
            chunks[3],
        );

        // containers for lists
        let chunks_global = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);
        let chunks_local = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[4]);

        // left and right parts of globals
        f.render_widget(
            Paragraph::new(
                help.global_hotkeys
                    .iter()
                    .map(|h| format!("{} :", h.description))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .alignment(Alignment::Right)
            .wrap(Wrap { trim: true }),
            chunks_global[0],
        );
        f.render_widget(
            Paragraph::new(
                help.global_hotkeys
                    .iter()
                    .map(|h| format!(" {}", h))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
            chunks_global[1],
        );

        // left and right parts of locals
        f.render_widget(
            Paragraph::new(
                help.local_hotkeys
                    .iter()
                    .map(|h| format!("{} :", h.description))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .alignment(Alignment::Right)
            .wrap(Wrap { trim: true }),
            chunks_local[0],
        );
        f.render_widget(
            Paragraph::new(
                help.local_hotkeys
                    .iter()
                    .map(|h| format!(" {}", h))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false }),
            chunks_local[1],
        );
    }
}
