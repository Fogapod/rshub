use std::io;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::Color,
    symbols,
    widgets::canvas::{Canvas, Line, Map, MapResolution},
    widgets::{Block, Borders},
    Frame,
};

use crate::geolocation::IP;
use crate::states::AppState;
use crate::views::{AppView, Drawable, ViewType};

pub struct InstallationView {}

impl InstallationView {
    pub fn new() -> Self {
        Self {}
    }
}

impl AppView for InstallationView {
    fn view_type(&self) -> ViewType {
        ViewType::Installations
    }
}

impl Drawable for InstallationView {
    fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        app: &AppState,
    ) -> Option<Rect> {
        let chunks = Layout::default()
            .constraints(vec![Constraint::Percentage(100)])
            .split(area);

        let servers = app.servers.servers.read();

        let map = Canvas::default()
            .block(Block::default().title("World").borders(Borders::ALL))
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::White,
                    resolution: MapResolution::High,
                });
                ctx.layer();

                // acquire lock once instead of doing it 20 times ahead
                let locations = app.locations.locations.read();

                let user_location = {
                    if let Some(location) = locations.get(&IP::Local) {
                        ctx.print(location.longitude, location.latitude, "X", Color::Red);
                        Some(location)
                    } else {
                        None
                    }
                };

                if let Some(user_location) = user_location {
                    for sv in app.servers.servers.read().values() {
                        if let Some(location) = locations.get(&IP::Remote(sv.data.ip.clone())) {
                            ctx.draw(&Line {
                                x1: user_location.longitude,
                                y1: user_location.latitude,
                                x2: location.longitude,
                                y2: location.latitude,
                                color: Color::Yellow,
                            });
                        }
                    }
                }

                for sv in servers.values() {
                    if let Some(location) = locations.get(&IP::Remote(sv.data.ip.clone())) {
                        let color = if sv.data.players != 0 {
                            Color::Green
                        } else {
                            Color::Red
                        };
                        ctx.print(location.longitude, location.latitude, "S", color);
                    }
                }
            })
            .marker(symbols::Marker::Braille)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);
        f.render_widget(map, chunks[0]);

        None
    }
}
