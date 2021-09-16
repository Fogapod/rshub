use std::convert::TryFrom;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

#[derive(Debug)]
pub enum UserInput {
    // hardcoded global inputs
    Help,
    Quit,

    // directions
    Up,
    Down,
    Left,
    Right,
    // gotos
    Top,
    Bottom,
    // actions
    Back,
    Enter,
    Delete,
    // misc
    Tab,
    Refresh,
    // custom
    Char(char),
}

impl TryFrom<&Event> for UserInput {
    type Error = ();

    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(key) => match key {
                KeyEvent {
                    code: KeyCode::Char('c' | 'C'),
                    modifiers: KeyModifiers::CONTROL,
                }
                | KeyEvent {
                    code: KeyCode::Char('q' | 'Q'),
                    ..
                } => Ok(Self::Quit),
                KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                } => Ok(Self::Char(*c)),
                KeyEvent {
                    code: KeyCode::Left,
                    ..
                } => Ok(Self::Left),
                KeyEvent {
                    code: KeyCode::Right,
                    ..
                } => Ok(Self::Right),
                KeyEvent {
                    code: KeyCode::Up, ..
                } => Ok(Self::Up),
                KeyEvent {
                    code: KeyCode::Down,
                    ..
                } => Ok(Self::Down),
                KeyEvent {
                    code: KeyCode::Home,
                    ..
                } => Ok(Self::Top),
                KeyEvent {
                    code: KeyCode::End, ..
                } => Ok(Self::Bottom),
                KeyEvent {
                    code: KeyCode::Esc, ..
                } => Ok(Self::Back),
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                } => Ok(Self::Enter),
                KeyEvent {
                    code: KeyCode::Delete | KeyCode::Backspace,
                    ..
                } => Ok(Self::Delete),
                KeyEvent {
                    code: KeyCode::Tab, ..
                } => Ok(Self::Tab),
                KeyEvent {
                    code: KeyCode::F(1),
                    ..
                } => Ok(Self::Help),
                KeyEvent {
                    code: KeyCode::F(5),
                    ..
                } => Ok(Self::Refresh),
                _ => Err(()),
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => Ok(Self::Up),
                MouseEventKind::ScrollDown => Ok(Self::Down),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}
