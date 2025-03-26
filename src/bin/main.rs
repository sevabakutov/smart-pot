use std::path::PathBuf;
use log::{error, info};
use smart_pot::{
    config::Settings, 
    core::{IoTHub, Result}
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let path = PathBuf::from("Config.toml");
    let settings = Settings::new(&path)?;

    let (hub, mut eventloop) = IoTHub::from_settings(settings)?;

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(rumqttc::Event::Incoming(packet)) => {
                    info!("Incomming packet: {:?}", packet);
                },
                Ok(rumqttc::Event::Outgoing(packet)) => {
                    info!("Outcomming packet: {:?}", packet);
                },
                Err(e) => {
                    error!("Error in MQTT event loop: {:?}", e);
                }
            }
        }
    });

    Ok(())
}