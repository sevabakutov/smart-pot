// iot_hub.rs

mod private {
    use std::time::Duration;

    use crate::core::Result;
    use crate::core::SmartPotError;

    use esp_idf_svc::mqtt::client::{
        EspAsyncMqttClient, EspAsyncMqttConnection, MqttClientConfiguration, MqttProtocolVersion,
    };
    use esp_idf_sys::{esp_tls_init_global_ca_store, esp_tls_set_global_ca_store, ESP_OK};

    const AZURE_IOT_CA_CERT: &[u8] = include_bytes!("../../../DigiCertGlobalRootG2.pem");
    ///
    /// A simple Azure IoT Hub MQTT wrapper.
    pub struct IoTHub {
        pub client: EspAsyncMqttClient,
        pub connection: EspAsyncMqttConnection,
    }

    impl IoTHub {
        /// Creates a new Mqtt Client with SAS auth.
        pub fn new(hub_name: &str, device_id: &str, sas_token: &str) -> Result<Self> {
            let status_code = unsafe { esp_tls_init_global_ca_store() };

            if status_code != ESP_OK {
                return Err(SmartPotError::CAError(
                    "Failed to initialize ca store. Status code: {status_code}".to_string(),
                ));
            }
            let cstr = std::ffi::CString::new(AZURE_IOT_CA_CERT)
                .expect("Certificate string contained internal NUL byte");

            let status_code = unsafe {
                esp_tls_set_global_ca_store(
                    cstr.as_ptr() as *const u8,
                    cstr.as_bytes_with_nul().len() as u32,
                )
            };

            if status_code != ESP_OK {
                return Err(SmartPotError::CAError(format!(
                    "failed to set global ca store (err={})",
                    status_code
                )));
            }

            let broker_url = format!("mqtts://{}.azure-devices.net:8883", hub_name);
            let username =
                format!("{hub_name}.azure-devices.net/{device_id}/?api-version=2021-06-30");

            let mqtt_config = MqttClientConfiguration {
                protocol_version: Some(MqttProtocolVersion::V3_1_1),
                client_id: Some(device_id),
                username: Some(&username),
                password: Some(sas_token),

                use_global_ca_store: true,
                crt_bundle_attach: None,

                keep_alive_interval: Some(Duration::from_secs(60)),
                reconnect_timeout: Some(Duration::from_secs(5)),
                disable_clean_session: false,
                ..Default::default()
            };

            let (client, connection) = EspAsyncMqttClient::new(&broker_url, &mqtt_config)?;

            Ok(Self { client, connection })
        }
    }
}

crate::mod_interface! {
    orphan use {
        IoTHub
    };
}
