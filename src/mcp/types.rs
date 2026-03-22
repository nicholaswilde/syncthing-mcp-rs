use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
    #[serde(rename = "null")]
    Null,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl From<crate::error::Error> for ResponseError {
    fn from(err: crate::error::Error) -> Self {
        use crate::error::Error;
        let (code, message) = match err {
            Error::Unauthorized(m) => (-32001, format!("Unauthorized: {}", m)),
            Error::Forbidden(m) => (-32002, format!("Forbidden: {}", m)),
            Error::NotFound(m) => (-32003, format!("Not Found: {}", m)),
            Error::Network(m) => (-32004, format!("Network Error: {}", m)),
            Error::SyncThing(m) => (-32005, format!("SyncThing Error: {}", m)),
            Error::ValidationError(m) => (-32602, format!("Validation Error: {}", m)),
            _ => (-32000, err.to_string()),
        };

        ResponseError {
            code,
            message,
            data: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}
