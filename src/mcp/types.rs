use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An MCP message, which can be a request, a response, or a notification.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Message {
    /// A request message.
    Request(Request),
    /// A response message.
    Response(Response),
    /// A notification message.
    Notification(Notification),
}

/// An MCP request message.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    /// The JSON-RPC version (usually "2.0").
    pub jsonrpc: String,
    /// The request ID.
    pub id: RequestId,
    /// The method name.
    pub method: String,
    /// Optional parameters for the method.
    pub params: Option<Value>,
}

/// A request identifier, which can be a string, a number, or null.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    /// A string ID.
    String(String),
    /// A numeric ID.
    Number(i64),
    /// A null ID.
    #[serde(rename = "null")]
    Null,
}

/// An MCP response message.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    /// The JSON-RPC version (usually "2.0").
    pub jsonrpc: String,
    /// The ID of the request being responded to.
    pub id: RequestId,
    /// The result of the request, if successful.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// The error that occurred, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// An MCP response error.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseError {
    /// The error code.
    pub code: i32,
    /// The error message.
    pub message: String,
    /// Optional additional data about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl From<crate::error::Error> for ResponseError {
    fn from(err: crate::error::Error) -> Self {
        use crate::error::Error;
        let diagnostic = err.diagnose();
        let (code, message) = match err {
            Error::Unauthorized(m) => (-32001, format!("Unauthorized: {}", m)),
            Error::Forbidden(m) => (-32002, format!("Forbidden: {}", m)),
            Error::NotFound(m) => (-32003, format!("Not Found: {}", m)),
            Error::Network(m) => (-32004, format!("Network Error: {}", m)),
            Error::SyncThing(m) => (-32005, format!("SyncThing Error: {}", m)),
            Error::ValidationError(m) => (-32602, format!("Validation Error: {}", m)),
            Error::Context(e, ctx) => {
                let inner = ResponseError::from(*e);
                (inner.code, format!("{}: {}", ctx, inner.message))
            }
            _ => (-32000, err.to_string()),
        };

        // Truncate very long technical messages for better AI readability,
        // but keep the important parts.
        let concise_message = if message.len() > 500 {
            format!("{}... (truncated)", &message[..490])
        } else {
            message
        };

        ResponseError {
            code,
            message: concise_message,
            data: Some(serde_json::to_value(diagnostic).unwrap_or_default()),
        }
    }
}

/// An MCP notification message.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    /// The JSON-RPC version (usually "2.0").
    pub jsonrpc: String,
    /// The method name.
    pub method: String,
    /// Optional parameters for the notification.
    pub params: Option<Value>,
}
