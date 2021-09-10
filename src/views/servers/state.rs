pub struct State {
    pub items: Vec<Server>,
}

impl State {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}
