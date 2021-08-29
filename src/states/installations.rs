use tokio::sync::RwLock;

use crate::datatypes::installation::Installation;

pub struct InstallationsState {
    pub items: RwLock<Vec<Installation>>,
}

impl InstallationsState {
    pub async fn new() -> Self {
        Self {
            items: RwLock::new(Vec::new()),
        }
    }

    pub async fn count(&self) -> usize {
        self.items.read().await.len()
    }
}
