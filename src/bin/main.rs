use std::path::PathBuf;
use smart_pot::{
    config::Settings, 
    core::{IoTHub, Result}
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let path = PathBuf::from("Config.toml");
    let settings = Settings::new(&path)?;

    let mut hub = IoTHub::from_settings(settings);

    // TODO: Remove from here.
    tokio::spawn(async move {
        loop {
            if let Err(e) = hub.eventloop().poll().await {
                eprintln!("Error in MQTT event loop: {:?}", e);
            }
        }
    });

    Ok(())
}