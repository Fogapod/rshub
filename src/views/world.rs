use std::io;
use std::sync::Arc;

use tui::layout::Rect;
use tui::{
    backend::CrosstermBackend,
    style::Color,
    symbols,
    widgets::canvas::{Canvas, Line, Map, MapResolution},
    widgets::{Block, Borders},
    Frame,
};

use crossterm::event::KeyCode;

use crate::app::AppAction;

use crate::datatypes::geolocation::IP;
use crate::input::UserInput;
use crate::states::help::HotKey;
use crate::states::AppState;
use crate::views::{AppView, Drawable, HotKeys, InputProcessor, Named};

pub struct World {}

#[async_trait::async_trait]
impl InputProcessor for World {
    async fn on_input(&mut self, input: &UserInput, _: Arc<AppState>) -> Option<AppAction> {
        match input {
            UserInput::Char('m' | 'M') | UserInput::Back => Some(AppAction::CloseView),
            _ => None,
        }
    }
}

impl AppView for World {}

impl Named for World {
    fn name(&self) -> String {
        "World Map".to_owned()
    }
}

impl HotKeys for World {
    fn hotkeys(&self) -> Vec<HotKey> {
        vec![
            HotKey {
                description: "Close map",
                key: KeyCode::Char('m'),
                modifiers: None,
            },
            HotKey {
                description: "Close map",
                key: KeyCode::Esc,
                modifiers: None,
            },
        ]
    }
}

#[async_trait::async_trait]
impl Drawable for World {
    // TODO: render selected with labels by default, all without labels
    // TODO: zoom and map navigation
    async fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: Arc<AppState>,
    ) {
        let locations = &app.locations.read().await.items;
        let servers = &app.servers.read().await.items;

        let map = Canvas::default()
            .block(Block::default().borders(Borders::ALL))
            .marker(symbols::Marker::Braille)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Blue,
                    resolution: MapResolution::High,
                });
                ctx.layer();

                if let Some(user_location) = locations.get(&IP::Local) {
                    for sv in servers {
                        if let Some(location) = locations.get(&sv.address.ip) {
                            ctx.draw(&Line {
                                x1: user_location.longitude,
                                y1: user_location.latitude,
                                x2: location.longitude,
                                y2: location.latitude,
                                color: Color::Yellow,
                            });
                        }
                    }

                    ctx.print(
                        user_location.longitude,
                        user_location.latitude,
                        "X",
                        Color::Red,
                    );
                }

                // separate loop to draw on top of lines
                for sv in servers {
                    if let Some(location) = locations.get(&sv.address.ip) {
                        let color = if sv.offline {
                            Color::Gray
                        } else if sv.players == 0 {
                            Color::Red
                        } else {
                            Color::Green
                        };
                        ctx.print(location.longitude, location.latitude, "O", color);
                    }
                }
            });

        // map (canvas specifically) panics with overflow if area is 0
        // area of size 0 could happen on wild terminal resizes
        // this check uses 2 instead of 0 because borders add 2 to each dimension
        if area.height > 2 && area.width > 2 {
            f.render_widget(map, area);
        }
    }
}
