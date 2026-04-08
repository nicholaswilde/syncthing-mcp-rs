#[cfg(test)]
mod tests {
    use crate::config::AppConfig;
    use crate::mcp::server::McpServer;
    use crate::tools::create_registry;
    use serde_json::json;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_server_run_loop() {
        let registry = create_registry();
        let config = AppConfig::default();
        let (server, rx) = McpServer::new(registry, config);

        let (mut client_writer, server_reader) = tokio::io::duplex(1024);
        let (server_writer, mut client_reader) = tokio::io::duplex(1024);

        let server_handle = tokio::spawn(async move {
            server.run(server_reader, server_writer, rx).await.unwrap();
        });

        // Send a request
        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        });
        let mut req_bytes = serde_json::to_vec(&req).unwrap();
        req_bytes.push(b'\n');
        client_writer.write_all(&req_bytes).await.unwrap();

        // Read response
        let mut buf = [0u8; 1024];
        let n = client_reader.read(&mut buf).await.unwrap();
        let resp: serde_json::Value = serde_json::from_slice(&buf[..n]).unwrap();
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");

        // Send an empty line (should be ignored)
        client_writer.write_all(b"\n").await.unwrap();

        // Send an invalid JSON line (should be ignored by the request parser but might log error)
        client_writer.write_all(b"not json\n").await.unwrap();

        // Shutdown server
        // We can't easily shutdown from here unless we drop the writers or use the shutdown signal
        // server.stop() is not accessible because server was moved to spawn.
        // But we can drop client_writer which should cause reader.next_line() to return None and loop to break.
        drop(client_writer);

        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_server_shutdown_signal() {
        let _ = tracing_subscriber::fmt::try_init();
        let registry = create_registry();
        let config = AppConfig::default();
        let (server, rx) = McpServer::new(registry, config);

        let (_client_writer, server_reader) = tokio::io::duplex(1024);
        let (server_writer, _client_reader) = tokio::io::duplex(1024);

        let server_clone = server.clone();
        let server_handle = tokio::spawn(async move {
            server_clone
                .run(server_reader, server_writer, rx)
                .await
                .unwrap();
        });

        // Sleep briefly to ensure server is in the select! loop
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Trigger shutdown
        server.stop();

        // Wait for server to shutdown with a timeout
        tokio::time::timeout(tokio::time::Duration::from_secs(5), server_handle)
            .await
            .expect("Server failed to shutdown within 5 seconds")
            .unwrap();
    }
}
