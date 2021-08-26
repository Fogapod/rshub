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
pub struct CommitView {}

impl CommitView {
    pub fn new() -> Self {
        Self {}
    }
}

impl AppView for CommitView {
    fn view_type(&self) -> ViewType {
        ViewType::Commits
    }
}

impl Drawable for CommitView {
    fn draw(
        &mut self,
        f: &mut Frame<CrosstermBackend<io::Stdout>>,
        area: Rect,
        _: &mut AppState,
    ) -> Option<Rect> {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(area);
        let colors = [
            Color::Reset,
            Color::Black,
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::Gray,
            Color::DarkGray,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
            Color::White,
        ];
        let items: Vec<Row> = colors
            .iter()
            .map(|c| {
                let cells = vec![
                    Cell::from(Span::raw(format!("{:?}: ", c))),
                    Cell::from(Span::styled("Foreground", Style::default().fg(*c))),
                    Cell::from(Span::styled("Background", Style::default().bg(*c))),
                ];
                Row::new(cells)
            })
            .collect();
        let table = Table::new(items)
            .block(Block::default().title("Colors").borders(Borders::ALL))
            .widths(&[
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ]);
        f.render_widget(table, chunks[0]);

        None
    }
}
