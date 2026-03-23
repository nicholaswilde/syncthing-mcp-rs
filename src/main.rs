#[tokio::main]
async fn main() -> anyhow::Result<()> {
    syncthing_mcp_rs::run().await
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main() {
        // main() is now async and runs the server, difficult to test directly
    }
}
