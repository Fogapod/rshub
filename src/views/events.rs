use std::io;
use std::sync::Arc;

use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders},
    Frame,
};

use crate::states::app::AppState;
use crate::states::events::AppEvent;
use crate::views::Drawable;

pub struct EventsView {}

#[async_trait::async_trait]
impl Drawable for EventsView {
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: Arc<AppState>,
    ) {
        if let Some(event) = &app.events.read().await.current_event {
            let mut style = Style::default().add_modifier(Modifier::BOLD);
            let mut border_style = Style::default();

            match event {
                AppEvent::Error(_) => {
                    style = style.bg(Color::Red);
                    border_style = border_style.bg(Color::Red);
                }
                AppEvent::Event(_) => {
                    style = style.bg(Color::Green).fg(Color::Black);
                    border_style = border_style.bg(Color::Green).fg(Color::Black);
                }
            };

            f.render_widget(
                Block::default()
                    .title(Span::styled(format!(" {} ", event), style))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::TOP)
                    .border_style(border_style),
                area,
            );
        }
    }
}
