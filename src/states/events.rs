use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;

use crate::config::AppConfig;

use crate::states::app::{AppState, TaskResult};

#[derive(Debug)]
pub enum AppEvent {
    Event(String),
    Error(anyhow::Error),
}

impl fmt::Display for AppEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Event(text) => write!(f, "{}", text),
            Self::Error(err) => write!(f, "{:#}", err),
        }
    }
}

pub struct EventsState {
    pub current_event: Option<AppEvent>,
    events: mpsc::UnboundedSender<AppEvent>,
    events_recv: Option<mpsc::UnboundedReceiver<AppEvent>>,
}

impl EventsState {
    pub async fn new(_: &AppConfig) -> Self {
        let (events, events_recv) = mpsc::unbounded_channel();

        Self {
            current_event: None,
            events,
            events_recv: Some(events_recv),
        }
    }

    pub async fn run(&mut self, app: Arc<AppState>) {
        app.watch_task(tokio::spawn(Self::event_task(
            self.events_recv.take().expect("event_recv taken"),
            app.clone(),
        )))
        .await;
    }

    pub async fn event(&self, message: &str) {
        log::debug!("event: {}", message);

        if let Err(err) = self.events.send(AppEvent::Event(message.to_owned())) {
            log::error!("error sending event: {}", err);
        }
    }

    pub async fn error(&self, err: anyhow::Error) {
        log::debug!("error: {:#}", &err);

        if let Err(err) = self.events.send(AppEvent::Error(err)) {
            log::error!("error sending error: {}", err);
        }
    }

    async fn event_task(
        mut events_recv: mpsc::UnboundedReceiver<AppEvent>,
        app: Arc<AppState>,
    ) -> TaskResult {
        let min_interval = Duration::from_secs(1);
        let last_event = Arc::new(Mutex::new(Instant::now() - min_interval));

        while let Some(event) = events_recv.recv().await {
            let duration_since_last = Instant::now().duration_since(*last_event.lock().await);

            if duration_since_last < min_interval {
                sleep(min_interval - duration_since_last).await;
            }

            let mut lock = last_event.lock().await;
            *lock = Instant::now();

            app.events.write().await.current_event = Some(event);

            let events = app.events.clone();
            let last_event = last_event.clone();

            tokio::spawn(async move {
                let duration = Duration::from_secs(10);

                sleep(duration).await;

                // event was updated, let other task take care of it
                if last_event.lock().await.elapsed() < duration {
                    return;
                }

                events.write().await.current_event = None;
            });
        }

        log::info!("event channel closed");

        Ok(())
    }
}
