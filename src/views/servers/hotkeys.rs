use crossterm::event::KeyCode;

use crate::datatypes::hotkey::HotKey;
use crate::views::HotKeys;

use super::Servers;

impl HotKeys for Servers {
    fn hotkeys(&self) -> Vec<HotKey> {
        let mut hotkeys = vec![
            #[cfg(feature = "geolocation")]
            HotKey {
                description: "Show world map",
                key: KeyCode::Char('m'),
                modifiers: None,
            },
            HotKey {
                description: "Install game version for selected server",
                key: KeyCode::Char('i'),
                modifiers: None,
            },
            HotKey {
                description: "Connect to selected server (install if needed)",
                key: KeyCode::Enter,
                modifiers: None,
            },
        ];

        hotkeys.append(&mut self.selection.hotkeys());

        hotkeys
    }
}
