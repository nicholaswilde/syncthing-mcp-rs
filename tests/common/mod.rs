use anyhow::Result;
use std::sync::{Arc, Mutex};
#[allow(dead_code)]
use syncthing_mcp_rs::api::SyncThingClient;
use syncthing_mcp_rs::config::{AppConfig, InstanceConfig};
use syncthing_mcp_rs::tools::ToolRegistry;
use testcontainers::core::{ContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use tokio::io::AsyncBufReadExt;

pub struct SyncThingContainer {
    _container: testcontainers::ContainerAsync<GenericImage>,
    pub host: String,
    pub port: u16,
    pub api_key: String,
}

impl SyncThingContainer {
    pub async fn new() -> Result<Self> {
        println!("🐳 Starting SyncThing container...");
        let api_key = "test-api-key".to_string();

        let image = GenericImage::new("syncthing/syncthing", "latest")
            .with_wait_for(WaitFor::seconds(10))
            .with_exposed_port(ContainerPort::Tcp(8384))
            .with_env_var("STGUIAPIKEY", &api_key);

        let container: testcontainers::ContainerAsync<GenericImage> = image.start().await?;

        // Pipe stdout logs
        let stdout = container.stdout(true);
        let stderr = container.stderr(true);
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("DOCKER STDOUT: {}", line);
            }
        });
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                println!("DOCKER STDERR: {}", line);
            }
        });

        let port = container.get_host_port_ipv4(8384).await?;
        let host = "localhost".to_string();

        println!("✅ SyncThing container started at http://{}:{}", host, port);

        // Wait for web server to be ready
        let config = InstanceConfig {
            url: format!("http://{}:{}", host, port),
            api_key: Some(api_key.clone()),
            ..Default::default()
        };
        let client = SyncThingClient::new(config);
        let mut ready = false;
        for _ in 0..60 {
            if client.get_system_status().await.is_ok() {
                ready = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        if !ready {
            return Err(anyhow::anyhow!("SyncThing web server timed out"));
        }

        Ok(Self {
            _container: container,
            host,
            port,
            api_key,
        })
    }

    pub fn client(&self) -> SyncThingClient {
        SyncThingClient::new(self.instance_config())
    }

    pub fn instance_config(&self) -> InstanceConfig {
        InstanceConfig {
            name: Some("test-instance".to_string()),
            url: format!("http://{}:{}", self.host, self.port),
            api_key: Some(self.api_key.clone()),
            ..Default::default()
        }
    }

    pub fn config(&self) -> AppConfig {
        AppConfig {
            host: self.host.clone(),
            port: self.port,
            api_key: Some(self.api_key.clone()),
            instances: vec![self.instance_config()],
            ..Default::default()
        }
    }
}

pub struct TestContext {
    #[allow(dead_code)]
    pub container: SyncThingContainer,
    pub config: AppConfig,
    pub client: SyncThingClient,
    pub registry: Arc<Mutex<ToolRegistry>>,
}

impl TestContext {
    pub async fn new() -> Result<Self> {
        let container = SyncThingContainer::new().await?;
        Ok(Self::from_container(container))
    }

    pub fn from_container(container: SyncThingContainer) -> Self {
        let config = container.config();
        let client = container.client();
        let registry = Arc::new(Mutex::new(syncthing_mcp_rs::tools::create_registry()));

        Self {
            container,
            config,
            client,
            registry,
        }
    }

    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let handler = {
            let reg = self.registry.lock().unwrap();
            reg.get_tool(name).map(|t| t.handler.clone())
        };

        if let Some(handler) = handler {
            handler(&self.client, &self.config, Some(args))
                .await
                .map_err(Into::into)
        } else {
            Err(anyhow::anyhow!("Tool not found: {}", name))
        }
    }
}
