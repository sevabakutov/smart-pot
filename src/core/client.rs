
mod private {
    use crate::*;
    use constants::ACCESS_KEY;
    use telemtry::Telemetry;

    #[derive(Debug)]
    pub struct IoTHub {
        client: IoTHubClient
    }

    impl IoTHub {
        /// Gets already registered device and returns IoTHub.
        pub async fn new() -> Result<Self> {
            let iothub_hostname = std::env::var("IOTHUB_HOSTNAME")?;
            let device_id = std::env::var("DEVICE_ID")?;
            let token_source = DeviceKeyTokenSource::new(
                iothub_hostname.as_str(),
                device_id.as_str(),
                ACCESS_KEY,
            ).unwrap();

            let client = IoTHubClient::new(&iothub_hostname, device_id, token_source)
                .await
                .map_err(|error| SmartPotError::InitializationError(error.to_string()))?;

            Ok(Self { client })
        }

        /// Register device and returns IoTHub.
        pub async fn register_device() -> Result<Self> {
            let scope_id = std::env::var("DPS_SCOPE_ID")?;
            let device_id = std::env::var("DPS_DEVICE_ID")?;
            let device_key = std::env::var("DPS_DEVICE_KEY")?;
        
            let client = IoTHubClient::from_provision_service(&scope_id, device_id, &device_key, 5)
                .await
                .map_err(|error| SmartPotError::InitializationError(error.to_string()))?;

            Ok(Self { client })
        }

        pub async fn get_telemtry(&self) -> Result<Telemetry> {
            unimplemented!()
        }

        pub async fn send_telemetry(&self) -> Result<()> {
            unimplemented!()
        }
    }
}

crate::mod_interface! {
    
}