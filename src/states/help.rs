use crossterm::event::KeyCode;

use crate::datatypes::hotkey::HotKey;

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
