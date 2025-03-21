#[tokio::main]
async fn main() -> azure_iot_sdk::Result<()> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    Ok(())
}