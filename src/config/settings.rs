

mod private {
    use std::path::Path;

    use config::{Config, Environment, File};
    use serde::Deserialize;

    use crate::core::Result;

    #[derive(Debug, Deserialize, Clone)]
    pub struct DeviceSettings {
        id: String
    }

    impl DeviceSettings {
        pub fn id(&self) -> &String {
            &self.id
        } 
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct HubSettings {
        host: String,
        port: u16
    }

    impl HubSettings {
        pub fn host(&self) -> &String {
            &self.host
        }

        pub fn port(&self) -> u16 {
            self.port
        }
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct Settings {
        hub: HubSettings,
        device: DeviceSettings
    }

    impl Settings {
        pub fn new(path: &Path) -> Result<Self> {
            Config::builder()
                .add_source(File::from(path).required(true))
                .add_source(Environment::with_prefix("CONF").separator("__"))
                .build()?
                .try_deserialize()
                .map_err(Into::into)
        }

        pub fn hub(&self) -> &HubSettings {
            &self.hub
        }

        pub fn device(&self) -> &DeviceSettings {
            &self.device
        }
    }
}

crate::mod_interface! {
    orphan use {
        Settings
    };
}