use crossterm::event::KeyCode;

use crate::datatypes::hotkey::HotKey;
use crate::views::HotKeys;

use super::Versions;

impl HotKeys for Versions {
    fn hotkeys(&self) -> Vec<HotKey> {
        let mut hotkeys = vec![
            HotKey {
                description: "Refresh version list",
                key: KeyCode::F(5),
                modifiers: None,
            },
            HotKey {
                description: "Install selected version",
                key: KeyCode::Char('i'),
                modifiers: None,
            },
            HotKey {
                description: "Run selected version (installs if needed)",
                key: KeyCode::Enter,
                modifiers: None,
            },
        ];

        //hotkeys.append(&mut self.selection.hotkeys());

        hotkeys
    }
}
