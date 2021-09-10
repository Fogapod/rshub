use crate::datatypes::commit::Commit;

pub struct State {
    pub items: Vec<Commit>,
}

impl State {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}
