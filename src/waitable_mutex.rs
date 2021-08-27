use std::sync::Arc;
use std::time::Duration;

use parking_lot::{Condvar, Mutex};

pub struct WaitableMutex<T> {
    data: (Mutex<T>, Condvar),
}

impl<T> WaitableMutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            data: (Mutex::new(value), Condvar::new()),
        }
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        *self.data.0.lock()
    }

    pub fn set(&self, value: T) {
        *self.data.0.lock() = value;

        self.data.1.notify_all();
    }

    pub fn wait_for(&self, duration: Duration) -> bool {
        self.data
            .1
            .wait_for(&mut self.data.0.lock(), duration)
            .timed_out()
    }
}

pub type SharedWaitableMutex<T> = Arc<WaitableMutex<T>>;
