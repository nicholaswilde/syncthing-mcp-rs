#[cfg(test)]
mod tests {
    use crate::api::models::ConnectionsResponse;

    #[test]
    fn test_parse_connections_response() {
        let json = r#"{
            "total": {
                "inBytesTotal": 1000,
                "outBytesTotal": 2000
            },
            "connections": {
                "DEVICE-123": {
                    "at": "2026-04-05T12:00:00Z",
                    "inBytesTotal": 500,
                    "outBytesTotal": 1000,
                    "address": "192.168.1.100:22000",
                    "clientVersion": "v1.20.0",
                    "connected": true,
                    "type": "TCP (Client)",
                    "paused": false,
                    "crypto": "TLS1.3",
                    "isLocal": true,
                    "mac": "00:11:22:33:44:55"
                }
            }
        }"#;

        let response: ConnectionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total.in_bytes_total, 1000);
        assert_eq!(response.total.out_bytes_total, 2000);
        
        let conn = response.connections.get("DEVICE-123").unwrap();
        assert_eq!(conn.in_bytes_total, 500);
        assert_eq!(conn.connection_type.as_deref(), Some("TCP (Client)"));
        assert_eq!(conn.crypto.as_deref(), Some("TLS1.3"));
        assert_eq!(conn.is_local, Some(true));
        assert_eq!(conn.mac.as_deref(), Some("00:11:22:33:44:55"));
    }
}
