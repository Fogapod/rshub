use crossterm::event::KeyCode;

use crate::datatypes::hotkey::HotKey;
use crate::views::HotKeys;

use super::tab::Tab;
use super::Tabs;

impl HotKeys for Tabs {
    fn hotkeys(&self) -> Vec<HotKey> {
        let mut hotkeys = vec![
            HotKey {
                description: "Go to next tab",
                key: KeyCode::Tab,
                modifiers: None,
            },
            HotKey {
                description: "Go Servers tab",
                key: KeyCode::Char('s'),
                modifiers: None,
            },
            HotKey {
                description: "Go versions tab",
                key: KeyCode::Char('v'),
                modifiers: None,
            },
            HotKey {
                description: "Go Commits tab",
                key: KeyCode::Char('c'),
                modifiers: None,
            },
        ];

        // hotkeys.append(&mut match self.selected_tab() {
        //     Tab::Servers => self.view_servers.hotkeys(),
        //     Tab::Versions => self.view_versions.hotkeys(),
        //     Tab::Commits => self.view_commits.hotkeys(),
        // });

        hotkeys
    }
}
