use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::geolocation::{Location, IP};

pub struct LocationsState {
    pub items: RwLock<HashMap<IP, Location>>,
    queue: mpsc::UnboundedSender<IP>,
    geo_provider: String,
}

impl LocationsState {
    pub async fn new(config: &AppConfig, queue: mpsc::UnboundedSender<IP>) -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
            queue,
            geo_provider: config.args.geo_provider.clone(),
        }
    }

    pub async fn resolve(&self, address: IP) -> Result<(), Box<dyn std::error::Error>> {
        {
            if self.items.read().await.get(&address).is_some() {
                return Ok(());
            }
        }

        self.queue.send(address)?;

        Ok(())
    }

    // pub fn get(&self, address: IP) -> Option<Location> {
    //     self.locations.read().get(&address).cloned()
    // }

    pub async fn location_fetch_task(
        locations: Arc<LocationsState>,
        mut rx: mpsc::UnboundedReceiver<IP>,
    ) {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("creating client");

        while let Some(ip) = rx.recv().await {
            log::info!("resolving location: {:?}", ip);
            let location = ip.fetch(&client, &locations.geo_provider).await.unwrap();

            locations.items.write().await.insert(ip, location);
        }
    }
}
