// iot_hub.rs

mod private {
    use crate::core::Result;

    use std::time::Duration;

    use esp_idf_svc::mqtt::client::{
        EspAsyncMqttClient, MqttClientConfiguration, EspAsyncMqttConnection, MqttProtocolVersion
    };
    // use esp_idf_sys::esp_tls_init_global_ca_store;
    use embedded_svc::mqtt::client::QoS;
    use esp_idf_sys::EspError;
    

    /// # IoTHub
    /// 
    /// A simple Azure IoT Hub MQTT wrapper.
    pub struct IoTHub {
        pub client: EspAsyncMqttClient,
        pub connection: EspAsyncMqttConnection,
    }

    impl IoTHub {
        /// Creates a new IoTHub client with SAS auth
        pub fn new(
            hub_name: &str,
            device_id: &str,
            sas_token: &str,
        ) -> Result<Self> {
            // Initialize the global CA store (required for SSL)
            // let code = esp_tls_init_global_ca_store();
            // if code != 0 {
            //     return Err(SmartPotError::EspError(EspError::from_non_zero(NonZero::new(code).unwrap())))
            // }

            let broker_url = format!("ssl://{}.azure-devices.net:8883", hub_name);

            // Azure requires:
            //   - client_id = device_id
            //   - username = "{hub_name}.azure-devices.net/{device_id}/?api-version=2021-06-30"
            let username = format!("{hub_name}.azure-devices.net/{device_id}/?api-version=2021-06-30");

            let mqtt_config = MqttClientConfiguration {
                protocol_version: Some(MqttProtocolVersion::V3_1_1),
                client_id: Some(device_id),
                username: Some(&username),
                password: Some(sas_token),

                // use_global_ca_store: true,
                // crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

                keep_alive_interval: Some(Duration::from_secs(60)),
                reconnect_timeout: Some(Duration::from_secs(5)),
                disable_clean_session: false,
                ..Default::default()
            };

            let (client, connection) = EspAsyncMqttClient::new(&broker_url, &mqtt_config)?;

            Ok(Self { client, connection })
        }

        /// Sends a message to Azur.
        pub async fn send_message(
            &mut self,
            topic: &str,
            payload: &str
        ) -> std::result::Result<(), EspError> {
            let publish_result = self.client
                .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
                .await
                .map(|_publish_id| ());

            publish_result
        }
    }
}

crate::mod_interface! {
    orphan use {
        IoTHub
    };
}
