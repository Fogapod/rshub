use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::config::AppConfig;
use crate::constants::USER_AGENT;
use crate::datatypes::geolocation::{Location, IP};

pub struct LocationsState {
    pub items: HashMap<IP, Location>,
    queue: mpsc::UnboundedSender<IP>,
    geo_provider: String,
}

impl LocationsState {
    pub async fn new(
        config: &AppConfig,
        managed_tasks: &mut Vec<tokio::task::JoinHandle<()>>,
    ) -> Arc<RwLock<Self>> {
        let (tx, rx) = mpsc::unbounded_channel::<IP>();

        let instance = Arc::new(RwLock::new(Self {
            items: HashMap::new(),
            queue: tx,
            geo_provider: config.geo_provider.clone(),
        }));

        managed_tasks.push(tokio::task::spawn(Self::location_fetch_task(
            instance.clone(),
            rx,
        )));

        instance
    }

    pub async fn resolve(&mut self, ip: IP) -> Result<(), Box<dyn std::error::Error>> {
        {
            if self.items.get(&ip).is_some() {
                return Ok(());
            }
        }

        self.queue.send(ip)?;

        Ok(())
    }

    pub async fn location_fetch_task(
        locations: Arc<RwLock<Self>>,
        mut rx: mpsc::UnboundedReceiver<IP>,
    ) {
        let client = Arc::new(
            reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("creating client"),
        );

        let geo_provider = locations.read().await.geo_provider.clone();

        while let Some(ip) = rx.recv().await {
            log::debug!("resolving location: {:?}", ip);

            let client = client.clone();
            let locations = locations.clone();
            let geo_provider = geo_provider.clone();

            tokio::spawn(async move {
                let mut request = client.get(format!("{}/json", geo_provider));

                if let IP::Remote(ref ip) = ip {
                    request = request.query(&[("ip", ip)])
                }

                let location = request
                    .send()
                    .await
                    .unwrap()
                    .json::<Location>()
                    .await
                    .unwrap();

                log::debug!("resolved location: {:?} -> {:?}", ip, location);

                if location.is_valid() {
                    locations.write().await.items.insert(ip, location);
                } else {
                    log::warn!("bad location");
                }
            });
        }
    }
}
