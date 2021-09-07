use std::fmt;

use crossterm::event::{KeyCode, KeyModifiers};

// TODO: less code duplication by associating this with inputs.rs somehow
#[derive(Debug, Clone)]
pub struct HotKey {
    pub description: &'static str,
    pub key: KeyCode,
    pub modifiers: Option<KeyModifiers>,
}

impl fmt::Display for HotKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(modifiers) = self.modifiers {
            write!(f, "{:?}+", modifiers)?;
        }
        write!(
            f,
            "{}",
            match self.key {
                KeyCode::Backspace => "Backspace".to_owned(),
                KeyCode::Enter => "Enter".to_owned(),
                KeyCode::Left => "Left".to_owned(),
                KeyCode::Right => "Right".to_owned(),
                KeyCode::Up => "Up".to_owned(),
                KeyCode::Down => "Down".to_owned(),
                KeyCode::Home => "Home".to_owned(),
                KeyCode::End => "End".to_owned(),
                KeyCode::PageUp => "PageUp".to_owned(),
                KeyCode::PageDown => "PageDown".to_owned(),
                KeyCode::Tab => "Tab".to_owned(),
                KeyCode::BackTab => "BackTab".to_owned(),
                KeyCode::Delete => "Delete".to_owned(),
                KeyCode::Insert => "Insert".to_owned(),
                KeyCode::F(i) => format!("F{}", i),
                KeyCode::Char(c) => c.to_uppercase().to_string(),
                KeyCode::Null => "Null".to_owned(),
                KeyCode::Esc => "Esc".to_owned(),
            }
        )
    }
}

pub struct HelpState {
    pub view_name: String,
    pub global_hotkeys: Vec<HotKey>,
    pub local_hotkeys: Vec<HotKey>,
}

impl HelpState {
    pub fn new() -> Self {
        Self {
            view_name: "".to_owned(),
            global_hotkeys: vec![
                HotKey {
                    description: "Display help in current context",
                    key: KeyCode::F(1),
                    modifiers: None,
                },
                HotKey {
                    description: "Close help screen",
                    key: KeyCode::Esc,
                    modifiers: None,
                },
                HotKey {
                    description: "Quit app",
                    key: KeyCode::Char('q'),
                    modifiers: None,
                },
            ],
            local_hotkeys: Vec::new(),
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.view_name = name.to_owned()
    }

    pub fn set_hotkeys(&mut self, hotkeys: &[HotKey]) {
        self.local_hotkeys = hotkeys.to_vec();
    }
}
