use crate::app::{AppState, AppView, Drawable, ViewType};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    symbols::DOT,
    text::{Span, Spans, Text},
    widgets::{
        canvas::{Canvas, Line, Map, MapResolution, Rectangle},
        BorderType,
    },
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Clear, Dataset, Gauge, LineGauge, List,
        ListItem, Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};

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
        app: &mut AppState,
    ) -> Option<Rect> {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
            .direction(Direction::Horizontal)
            .split(area);

        let pop_style = Style::default().fg(Color::Green);
        let online_stype = Style::default().fg(Color::Red);
        let offline_stype =
            online_stype.add_modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT);

        let servers = app.servers.read();

        let rows = servers.values().map(|s| {
            let style = if s.offline {
                offline_stype
            } else if s.data.players == 0 {
                online_stype
            } else {
                pop_style
            };

            Row::new(vec![
                s.data.name.clone(),
                s.data.players.to_string(),
                s.data.build.to_string(),
            ])
            .style(style)
        });
        let table = Table::new(rows)
            .header(
                Row::new(vec!["SERVER", "Location", "Status"])
                    .style(Style::default().fg(Color::Yellow))
                    .bottom_margin(1),
            )
            .block(Block::default().title("Servers").borders(Borders::ALL))
            .widths(&[
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(10),
            ])
            .highlight_style(Style::default().bg(Color::Yellow));

        f.render_widget(table, chunks[0]);

        let map = Canvas::default()
            .block(Block::default().title("World").borders(Borders::ALL))
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::White,
                    resolution: MapResolution::High,
                });
                ctx.layer();

                // let user_location = {
                //     let servers = app.servers.read();
                //     let locations = servers.locations.read();

                //     if let Some(location) = locations.get(&IP::Local) {
                //         ctx.print(location.longitude, location.latitude, "X", Color::Red);
                //         Some(*location)
                //     } else {
                //         None
                //     }
                // };

                // if let Some(user_location) = user_location {
                //     for sv in app.servers.read().values() {
                //         // if let ServerLocation::Resolved(location) = sv.location {
                //         //     ctx.draw(&Line {
                //         //         x1: user_location.longitude,
                //         //         y1: user_location.latitude,
                //         //         x2: location.longitude,
                //         //         y2: location.latitude,
                //         //         color: Color::Yellow,
                //         //     });
                //         // }
                //     }
                // }

                // for sv in servers.values() {
                //     // if let ServerLocation::Resolved(location) = sv.location {
                //     //     let color = if sv.data.players != 0 {
                //     //         Color::Green
                //     //     } else {
                //     //         Color::Red
                //     //     };
                //     //     ctx.print(location.longitude, location.latitude, "S", color);
                //     // }
                // }
            })
            .marker(symbols::Marker::Braille)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);
        f.render_widget(map, chunks[1]);

        None
    }
}
