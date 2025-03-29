mod private {
    use crate::{
        config::Settings,
        core::{Result, SmartPotError},
    };
    use log::info;
    use std::{sync::Arc, time::Duration};
    use esp_idf_svc::mqtt::client::EspAsyncMqttClient;
    
    pub struct IoTHub {
        pub client: EspAsyncMqttClient,
        settings: Settings,
    }

    impl IoTHub {
        pub fn new() -> Self {
            unimplemented!()
        }

        pub fn from_settings(settings: Settings) -> Result<(Self, EventLoop)> {
            let id = settings.device().id().clone();
            let host = settings.hub().host().clone();
            let port = settings.hub().port();

            unimplemented!()
        }

        pub async fn send(&self, message: &str) -> Result<()> {
            unimplemented!()
        }
    }
}

crate::mod_interface! {
    orphan use {
        IoTHub
    };
}
