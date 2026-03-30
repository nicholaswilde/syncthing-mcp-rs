use crate::api::SyncThingClient;
use crate::config::AppConfig;
use crate::error::Error;
use crate::mcp::{Message, Notification, Request, Response, ResponseError};
use crate::tools::ToolRegistry;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, Request as HttpRequest, StatusCode},
    middleware::{self, Next},
    response::{Response as HttpResponse, Sse, sse::Event},
    routing::{get, post},
};
use dashmap::DashMap;
use futures::stream::Stream;
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};
use tokio::sync::{mpsc, watch};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

/// Represents an active MCP session over SSE.
pub struct Session {
    /// Sender for messages to the client.
    pub tx: mpsc::Sender<Message>,
}

/// The Model Context Protocol (MCP) server for SyncThing.
#[derive(Clone)]
pub struct McpServer {
    /// The registry of available tools.
    pub registry: Arc<Mutex<ToolRegistry>>,
    /// The application configuration.
    pub config: AppConfig,
    /// A sender for sending notifications to the client.
    pub notification_tx: mpsc::Sender<Notification>,
    /// Active SSE sessions.
    pub sessions: Arc<DashMap<String, Session>>,
    /// Shutdown signal.
    pub shutdown_tx: Arc<watch::Sender<bool>>,
}

/// Query parameters for session-based messages.
#[derive(Deserialize)]
pub struct SessionQuery {
    /// The unique session identifier.
    pub session_id: String,
}

impl McpServer {
    /// Creates a new MCP server with the given registry and configuration.
    pub fn new(registry: ToolRegistry, config: AppConfig) -> (Self, mpsc::Receiver<Notification>) {
        let (tx, rx) = mpsc::channel(100);
        let (shutdown_tx, _) = watch::channel(false);
        (
            Self {
                registry: Arc::new(Mutex::new(registry)),
                config,
                notification_tx: tx,
                sessions: Arc::new(DashMap::new()),
                shutdown_tx: Arc::new(shutdown_tx),
            },
            rx,
        )
    }

    /// Stops the server.
    pub fn stop(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    /// Returns an axum router for the MCP HTTP/SSE transport.
    pub fn router(&self) -> Router {
        let router = Router::new()
            .route("/sse", get(sse_handler))
            .route("/message", post(message_handler))
            .with_state(self.clone());

        if self.config.http_server.api_key.is_some() {
            tracing::info!("HTTP authentication enabled");
            router.layer(middleware::from_fn_with_state(
                self.clone(),
                auth_middleware,
            ))
        } else {
            router
        }
    }

    /// Runs the server on standard input/output.
    pub async fn run_stdio(&self, rx: mpsc::Receiver<Notification>) -> anyhow::Result<()> {
        self.run(stdin(), stdout(), rx).await
    }

    /// Runs the server with the given reader and writer.
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
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            if *shutdown_rx.borrow() {
                break;
            }

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
                        let out = serde_json::to_string(&Message::Notification(n.clone()))? + "\n";
                        writer.write_all(out.as_bytes()).await?;
                        writer.flush().await?;

                        // Only notify active SSE sessions if any exist
                        if !self.sessions.is_empty() {
                            let sessions = self.sessions.clone();
                            tokio::spawn(async move {
                                for session in sessions.iter() {
                                    let _ = session.tx.send(Message::Notification(n.clone())).await;
                                }
                            });
                        }
                    } else {
                        // rx closed
                        break;
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Handles an incoming MCP request.
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
                    handler(&client, &self.config, args).await
                } else {
                    Err(Error::Internal(format!("Tool not found: {}", tool_name)))
                }
            }
            _ => Err(Error::Internal(format!("Method not found: {}", req.method))),
        }
    }
}

async fn sse_handler(
    State(server): State<McpServer>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel(100);

    tracing::info!("New SSE session established: {}", session_id);
    server.sessions.insert(session_id.clone(), Session { tx });

    let endpoint_url = format!("/message?session_id={}", session_id);

    let initial_event = Event::default().event("endpoint").data(endpoint_url);

    // Use a guard to remove the session when the stream is dropped
    struct SessionGuard {
        session_id: String,
        sessions: Arc<DashMap<String, Session>>,
    }

    impl Drop for SessionGuard {
        fn drop(&mut self) {
            tracing::info!("SSE session closed: {}", self.session_id);
            self.sessions.remove(&self.session_id);
        }
    }

    let guard = Arc::new(SessionGuard {
        session_id: session_id.clone(),
        sessions: server.sessions.clone(),
    });

    let stream = ReceiverStream::new(rx)
        .map(move |msg| {
            let _guard = &guard;
            let json = serde_json::to_string(&msg).unwrap_or_default();
            Event::default().event("message").data(json)
        })
        .map(Ok);

    // Chain the initial 'endpoint' event with the message stream
    let full_stream = tokio_stream::once(Ok(initial_event)).chain(stream);

    Sse::new(full_stream).keep_alive(axum::response::sse::KeepAlive::default())
}

async fn message_handler(
    State(server): State<McpServer>,
    Query(query): Query<SessionQuery>,
    Json(message): Json<Message>,
) -> Result<Json<Value>, (axum::http::StatusCode, String)> {
    tracing::debug!("Received HTTP message for session: {}", query.session_id);
    let _session = server.sessions.get(&query.session_id).ok_or_else(|| {
        tracing::warn!("Session not found: {}", query.session_id);
        (
            axum::http::StatusCode::NOT_FOUND,
            "Session not found".to_string(),
        )
    })?;

    match message {
        Message::Request(req) => {
            let id = req.id.clone();
            let method = req.method.clone();
            tracing::info!("Handling HTTP request: {}", method);
            let response = server.handle_request(req).await;

            let json_resp = match response {
                Ok(result) => {
                    tracing::debug!("HTTP request successful: {}", method);
                    Response {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(result),
                        error: None,
                    }
                }
                Err(e) => {
                    tracing::error!("HTTP request failed: {}: {}", method, e);
                    Response {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: None,
                        error: Some(ResponseError::from(e)),
                    }
                }
            };

            // In HTTP transport, responses are sent back in the HTTP response body
            Ok(Json(serde_json::to_value(json_resp).unwrap()))
        }
        Message::Notification(n) => {
            // Notifications from client to server (if any)
            tracing::info!("Received notification from HTTP client: {}", n.method);
            Ok(Json(serde_json::json!({"status": "received"})))
        }
        _ => {
            tracing::warn!("Unsupported message type received over HTTP");
            Err((
                axum::http::StatusCode::BAD_REQUEST,
                "Unsupported message type".to_string(),
            ))
        }
    }
}

async fn auth_middleware(
    State(server): State<McpServer>,
    headers: HeaderMap,
    request: HttpRequest<axum::body::Body>,
    next: Next,
) -> Result<HttpResponse, StatusCode> {
    if let Some(expected_key) = &server.config.http_server.api_key {
        let auth_header = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                tracing::warn!("Missing authorization header");
                StatusCode::UNAUTHORIZED
            })?;

        if !auth_header.starts_with("Bearer ") {
            tracing::warn!("Invalid authorization header format");
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];
        if token != expected_key {
            tracing::warn!("Invalid API key provided");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    Ok(next.run(request).await)
}
