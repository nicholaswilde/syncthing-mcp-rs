use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::mcp::Notification;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};

/// Manages event polling for SyncThing instances.
#[derive(Clone)]
pub struct EventManager {
    /// The application configuration.
    pub config: AppConfig,
    /// A sender for sending notifications to the client.
    pub notification_tx: mpsc::Sender<Notification>,
    /// Last seen event ID for each instance.
    pub last_ids: Arc<DashMap<String, u64>>,
    /// Shutdown signal.
    pub shutdown_tx: Arc<watch::Sender<bool>>,
}

impl EventManager {
    /// Creates a new event manager.
    pub fn new(config: AppConfig, notification_tx: mpsc::Sender<Notification>) -> Self {
        let (shutdown_tx, _) = watch::channel(false);
        Self {
            config,
            notification_tx,
            last_ids: Arc::new(DashMap::new()),
            shutdown_tx: Arc::new(shutdown_tx),
        }
    }

    /// Stops the event manager.
    pub fn stop(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    /// Runs the event polling loop.
    pub async fn run(&self) -> anyhow::Result<()> {
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Create clients once before the loop to reuse connection pools and avoid repeated allocations
        let clients: Vec<(String, SyncThingClient)> = self
            .config
            .instances
            .iter()
            .map(|instance| {
                let instance_name = instance
                    .name
                    .clone()
                    .unwrap_or_else(|| "default".to_string());
                (instance_name, SyncThingClient::new(instance.clone()))
            })
            .collect();

        loop {
            if *shutdown_rx.borrow() {
                break;
            }

            for (instance_name, client) in &clients {
                let since = self.last_ids.get(instance_name).map(|r| *r);
                match client.get_events(since, Some(10)).await {
                    Ok(events) => {
                        for event in events {
                            // Only notify for configured events
                            if self.config.mcp_events.contains(&event.event_type) {
                                let notification = Notification {
                                    jsonrpc: "2.0".to_string(),
                                    method: "notifications/message".to_string(),
                                    params: Some(serde_json::json!({
                                        "instance": instance_name,
                                        "summary": event.summary(),
                                        "event": event,
                                    })),
                                };
                                let _ = self.notification_tx.send(notification).await;
                            }
                            self.last_ids.insert(instance_name.clone(), event.id);
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to get events for instance {}: {}",
                            instance_name,
                            e
                        );
                    }
                }
            }

            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(5)) => {},
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
