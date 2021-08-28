use parking_lot::RwLock;

use crate::datatypes::installation::Installation;

pub struct InstallationsState {
    pub items: RwLock<Vec<Installation>>,
}

impl Default for InstallationsState {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallationsState {
    pub fn new() -> Self {
        Self {
            items: RwLock::new(Vec::new()),
        }
    }
}
