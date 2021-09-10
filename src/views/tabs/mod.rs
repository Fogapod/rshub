mod draw;
mod hotkeys;
mod input;
mod tab;

use tui::widgets::ListState;

use crate::states::StatelessList;
use crate::views::{AppView, Name};

use tab::Tab;

pub struct Tabs {
    selection: StatelessList<ListState>,
}

impl Tabs {
    pub fn new() -> Self {
        let mut selection = StatelessList::new(ListState::default(), true);

        selection.select_first(Tab::tab_count());

        Self { selection }
    }

    fn selected_tab(&self) -> Tab {
        *Tab::all()
            .get(self.selection.selected().unwrap_or_default())
            .unwrap_or(&Tab::Servers)
    }

    fn select_tab(&mut self, tab: Tab) {
        self.selection.select_index(tab.into());
    }
}

impl AppView for Tabs {}

impl Name for Tabs {
    fn name(&self) -> String {
        format!(
            "Tab: {}",
            match self.selected_tab() {
                Tab::Servers => "servers",
                Tab::Versions => "versions",
                Tab::Commits => "commits",
            }
        )
    }
}
