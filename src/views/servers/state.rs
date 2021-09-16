use tui::widgets::TableState;

use crate::datatypes::server::Server;
use crate::states::StatelessList;

pub struct State {
    pub items: Vec<Server>,
    pub selection: StatelessList<TableState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selection: StatelessList::new(TableState::default(), false),
        }
    }
}
