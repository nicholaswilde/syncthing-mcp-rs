#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::mcp::types::{Message, RequestId, ResponseError};
    use serde_json::json;

    #[test]
    fn test_response_error_from_unauthorized() {
        let err = Error::Unauthorized("api key invalid".to_string());
        let resp_err = ResponseError::from(err);
        assert_eq!(resp_err.code, -32001);
        assert!(resp_err.message.contains("Unauthorized"));
    }

    #[test]
    fn test_response_error_from_forbidden() {
        let err = Error::Forbidden("access denied".to_string());
        let resp_err = ResponseError::from(err);
        assert_eq!(resp_err.code, -32002);
        assert!(resp_err.message.contains("Forbidden"));
    }

    #[test]
    fn test_response_error_from_not_found() {
        let err = Error::NotFound("folder not found".to_string());
        let resp_err = ResponseError::from(err);
        assert_eq!(resp_err.code, -32003);
        assert!(resp_err.message.contains("Not Found"));
    }

    #[test]
    fn test_response_error_from_network() {
        let err = Error::Network("connection refused".to_string());
        let resp_err = ResponseError::from(err);
        assert_eq!(resp_err.code, -32004);
        assert!(resp_err.message.contains("Network Error"));
    }

    #[test]
    fn test_response_error_from_syncthing() {
        let err = Error::SyncThing("internal error".to_string());
        let resp_err = ResponseError::from(err);
        assert_eq!(resp_err.code, -32005);
        assert!(resp_err.message.contains("SyncThing Error"));
    }

    #[test]
    fn test_response_error_from_validation() {
        let err = Error::ValidationError("invalid folder id".to_string());
        let resp_err = ResponseError::from(err);
        assert_eq!(resp_err.code, -32602);
        assert!(resp_err.message.contains("Validation Error"));
    }

    #[test]
    fn test_response_error_from_context() {
        let inner = Error::NotFound("config file".to_string());
        let err = Error::Context(Box::new(inner), "Failed to load".to_string());
        let resp_err = ResponseError::from(err);
        assert_eq!(resp_err.code, -32003);
        assert!(resp_err.message.contains("Failed to load"));
        assert!(resp_err.message.contains("Not Found"));
    }

    #[test]
    fn test_response_error_truncation() {
        let long_msg = "a".repeat(600);
        let err = Error::SyncThing(long_msg);
        let resp_err = ResponseError::from(err);
        assert!(resp_err.message.len() <= 500);
        assert!(resp_err.message.contains("... (truncated)"));
    }

    #[test]
    fn test_request_id_deserialization() {
        let s = json!("123");
        let id: RequestId = serde_json::from_value(s).unwrap();
        assert_eq!(id, RequestId::String("123".to_string()));

        let n = json!(123);
        let id: RequestId = serde_json::from_value(n).unwrap();
        assert_eq!(id, RequestId::Number(123));

        let null = json!(null);
        let id: RequestId = serde_json::from_value(null).unwrap();
        assert_eq!(id, RequestId::Null);
    }

    #[test]
    fn test_message_untagged_deserialization() {
        let req_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "test",
            "params": {}
        });
        let msg: Message = serde_json::from_value(req_json).unwrap();
        if let Message::Request(r) = msg {
            assert_eq!(r.method, "test");
        } else {
            panic!("Expected Request");
        }

        let resp_json = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "ok"
        });
        let msg: Message = serde_json::from_value(resp_json).unwrap();
        if let Message::Response(r) = msg {
            assert_eq!(r.result, Some(json!("ok")));
        } else {
            panic!("Expected Response");
        }

        let notify_json = json!({
            "jsonrpc": "2.0",
            "method": "notify",
            "params": {}
        });
        let msg: Message = serde_json::from_value(notify_json).unwrap();
        if let Message::Notification(n) = msg {
            assert_eq!(n.method, "notify");
        } else {
            panic!("Expected Notification");
        }
    }
}
