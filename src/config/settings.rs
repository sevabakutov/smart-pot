

mod private {
    use std::path::PathBuf;

    use config::{Config, Environment, File};
    use serde::Deserialize;

    use crate::core::{Result, SmartPotError};

    #[derive(Debug, Deserialize)]
    pub struct DeviceSettings {
        id: String
    }

    #[derive(Debug, Deserialize)]
    pub struct HubSettings {
        pub host: String,
        pub port: u16
    }

    #[derive(Debug, Deserialize)]
    pub struct Settings {
        pub hub: HubSettings,
        pub device: DeviceSettings
    }

    impl Settings {
        pub fn new(path: &PathBuf) -> Result<Self> {
            Config::builder()
                .add_source(File::from(path.as_path()).required(true))
                .add_source(Environment::with_prefix("CONF").separator("__"))
                .build()?
                .try_deserialize()
                .map_err(|err| SmartPotError::ConfigError(err))
        }
    }
}

crate::mod_interface! {
    orphan use {
        Settings
    };
}