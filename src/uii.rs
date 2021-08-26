use crate::app::App;

use std::cmp::Ordering;
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

use crate::geolocation::IP;
use crate::types::{Server, ServerLocation};

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn draw_exit_view<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    draw(f, app);

    let paragraph = Paragraph::new(Text::from(
        "Shutting down threads\nThis might take a few seconds",
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });

    let area = centered_rect(100, 100, f.size());

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(f.size());

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        // TODO: figure out how to give tabs higher priority, drawing on top of name
        .constraints([Constraint::Min(0), Constraint::Length(25)])
        .split(chunks[0]);

    let paragraph = Paragraph::new(Text::from(format!(
        // NOTE: space at the end to prevent italic text go off screen
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

    let titles = app
        .tabs
        .tabs
        .iter()
        .map(|t| Spans::from(Span::styled(t.name(app), Style::default().fg(Color::Green))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().border_type(BorderType::Plain))
        //.style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_style(
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )
        .divider(DOT)
        .select(app.tabs.index);

    f.render_widget(tabs, header_chunks[0]);

    match app.tabs.index {
        0 => draw_servers(f, app, chunks[1]),
        1 => draw_installations(f, app, chunks[1]),
        2 => draw_commits(f, app, chunks[1]),
        _ => {}
    };
}

fn draw_servers<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);

    let servers = app.servers.read();

    let offline_servers = servers.servers.values().filter(|s| s.offline).count();

    let mut servers_to_be_sorted = servers.servers.values().collect::<Vec<&Server>>();
    // TODO: custom sorts by each field
    // TODO: search by pattern
    // https://stackoverflow.com/a/40369685
    servers_to_be_sorted.sort_by(|a, b| match a.data.players.cmp(&b.data.players).reverse() {
        Ordering::Equal => a.data.name.cmp(&b.data.name),
        other => other,
    });

    let rows = servers_to_be_sorted.iter().map(|s| {
        let style = if s.offline {
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT)
        } else if s.data.players == 0 {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };

        Row::new(vec![
            s.data.name.clone(),
            s.data.build.to_string(),
            s.data.map.clone(),
            s.data.players.to_string(),
        ])
        .style(style)
    });

    let table = Table::new(rows)
        .header(
            Row::new(vec!["NAME", "BUILD", "MAP", "PLAYERS"]).style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .title(Span::styled(
                    format!("SERVERS {}:{}", servers.count(), offline_servers),
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(45),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
        ])
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    drop(servers);
    let mut servers = app.servers.write();

    f.render_stateful_widget(table, chunks[0], &mut servers.state);
}

fn draw_installations<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .direction(Direction::Horizontal)
        .split(area);

    let pop_style = Style::default().fg(Color::Green);
    let online_stype = Style::default().fg(Color::Red);
    let offline_stype = online_stype.add_modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT);

    let servers = app.servers.read();

    let rows = servers.servers.values().map(|s| {
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

            let user_location = {
                let servers = app.servers.read();
                let locations = servers.locations.read();

                if let Some(location) = locations.get(&IP::Local) {
                    ctx.print(location.longitude, location.latitude, "X", Color::Red);
                    Some(*location)
                } else {
                    None
                }
            };

            if let Some(user_location) = user_location {
                for sv in app.servers.read().servers.values() {
                    if let ServerLocation::Resolved(location) = sv.location {
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

            for sv in servers.servers.values() {
                if let ServerLocation::Resolved(location) = sv.location {
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
    f.render_widget(map, chunks[1]);
}

fn draw_commits<B>(f: &mut Frame<B>, _app: &mut App, area: Rect)
where
    B: Backend,
{
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
}
