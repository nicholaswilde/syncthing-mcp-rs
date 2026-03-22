use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::mcp::{Message, Notification, Request, Response, ResponseError};
use crate::tools::ToolRegistry;
use crate::error::Error;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct McpServer {
    pub registry: Arc<Mutex<ToolRegistry>>,
    pub config: AppConfig,
    pub notification_tx: mpsc::Sender<Notification>,
}

impl McpServer {
    pub fn new(registry: ToolRegistry, config: AppConfig) -> (Self, mpsc::Receiver<Notification>) {
        let (tx, rx) = mpsc::channel(100);
        (
            Self {
                registry: Arc::new(Mutex::new(registry)),
                config,
                notification_tx: tx,
            },
            rx,
        )
    }

    pub async fn run_stdio(&self, rx: mpsc::Receiver<Notification>) -> anyhow::Result<()> {
        self.run(stdin(), stdout(), rx).await
    }

    pub async fn run<R, W>(
        &self,
        reader: R,
        mut writer: W,
        mut rx: mpsc::Receiver<Notification>,
    ) -> anyhow::Result<()>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        let mut reader = BufReader::new(reader).lines();

        loop {
            tokio::select! {
                line = reader.next_line() => {
                    let line = line?;
                    if let Some(line) = line {
                        let input = line.trim();
                        if input.is_empty() {
                            continue;
                        }

                        if let Ok(Message::Request(req)) = serde_json::from_str::<Message>(input) {
                            tracing::debug!("Received request: {}", req.method);
                            let id = req.id.clone();
                            let response = self.handle_request(req).await;

                            let json_resp = match response {
                                Ok(result) => {
                                    tracing::debug!("Request successful");
                                    Response {
                                        jsonrpc: "2.0".to_string(),
                                        id,
                                        result: Some(result),
                                        error: None,
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Request failed: {}", e);
                                    Response {
                                        jsonrpc: "2.0".to_string(),
                                        id,
                                        result: None,
                                        error: Some(ResponseError::from(e)),
                                    }
                                }
                            };

                            let out = serde_json::to_string(&json_resp)? + "\n";
                            writer.write_all(out.as_bytes()).await?;
                            writer.flush().await?;
                        }
                    } else {
                        break;
                    }
                }
                notification = rx.recv() => {
                    if let Some(n) = notification {
                        let out = serde_json::to_string(&Message::Notification(n))? + "\n";
                        writer.write_all(out.as_bytes()).await?;
                        writer.flush().await?;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn handle_request(&self, req: Request) -> Result<Value, Error> {
        match req.method.as_str() {
            "initialize" => Ok(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": "syncthing-mcp-rs",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            "tools/list" => {
                let tools = {
                    let registry = self.registry.lock().unwrap();
                    registry.list_tools()
                };
                Ok(serde_json::json!({
                    "tools": tools
                }))
            }
            "tools/call" => {
                let tool_name = req
                    .params
                    .as_ref()
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");

                let args = req
                    .params
                    .as_ref()
                    .and_then(|p| p.get("arguments"))
                    .cloned();

                let instance_name = args
                    .as_ref()
                    .and_then(|a| a.get("instance"))
                    .and_then(|i| i.as_str());

                let instance_config = self.config.get_instance(instance_name)?;

                let client = SyncThingClient::new(instance_config.clone());

                let handler = {
                    let registry = self.registry.lock().unwrap();
                    registry.get_tool(tool_name).map(|t| t.handler.clone())
                };

                if let Some(handler) = handler {
                    handler(&client, &self.config, args)
                        .await
                } else {
                    Err(Error::Internal(format!("Tool not found: {}", tool_name)))
                }
            }
            _ => Err(Error::Internal(format!("Method not found: {}", req.method))),
        }
    }
}
