use std::path::PathBuf;
use smart_pot::{config::Settings, core::Result};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let path = PathBuf::from("Config.toml");
    let settings = Settings::new(&path)?;

    println!("Final configuration:\n{:#?}", settings);

    Ok(())
}