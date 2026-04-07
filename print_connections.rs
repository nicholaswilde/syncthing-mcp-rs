use reqwest::Client;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let url = env::var("SYNCTHING_URL").unwrap();
    let key = env::var("SYNCTHING_API_KEY").unwrap();
    let client = Client::builder().danger_accept_invalid_certs(true).build()?;
    let res = client.get(&format!("{}/rest/system/connections", url))
        .header("X-API-Key", key)
        .send()
        .await?;
    println!("{}", res.text().await?);
    Ok(())
}
