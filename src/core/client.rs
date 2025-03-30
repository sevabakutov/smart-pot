// core.rs
mod private {
    use crate::*;
    use crate::core::{Result, SmartPotError};

    use std::time::Duration;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use urlencoding::encode;

    use esp_idf_svc::tls;
    use esp_idf_svc::mqtt::client::{
        EspAsyncMqttClient, MqttClientConfiguration, MqttConnection, MqttProtocolVersion
    };
    use embedded_svc::mqtt::client::Client;
    use esp_idf_sys::EspError;

    /// # IoTHub
    /// 
    /// A simple Azure IoT Hub MQTT wrapper.
    pub struct IoTHub {
        pub client: EspAsyncMqttClient,
        pub connection: MqttConnection,
    }

    impl IoTHub {
        /// Creates a new IoTHub client with SAS auth
        pub fn new(
            hub_name: &str,
            device_id: &str,
            sas_token: &str,
        ) -> Result<Self> {
            // Initialize the global CA store (required for SSL)
            tls::esp_tls_init_global_ca_store()?;

            let broker_url = format!("ssl://{}.azure-devices.net:8883", hub_name);

            // Azure requires:
            //   - client_id = device_id
            //   - username = "{hub_name}.azure-devices.net/{device_id}/?api-version=2021-06-30"
            let client_id = device_id.to_string();
            let username = format!("{hub_name}.azure-devices.net/{device_id}/?api-version=2021-06-30");

            let mqtt_config = MqttClientConfiguration {
                protocol_version: Some(MqttProtocolVersion::V3_1_1),
                client_id: Some(client_id),
                username: Some(username),
                password: Some(sas_token.to_string()),

                use_global_ca_store: true,
                crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

                keep_alive_interval: Some(Duration::from_secs(60)),
                reconnect_timeout: Some(Duration::from_secs(5)),
                clean_session: true,
                ..Default::default()
            };

            let (client, connection) = EspAsyncMqttClient::new(broker_url, &mqtt_config)?;

            Ok(Self { client, connection })
        }

        /// Sends a message to Azur.
        pub async fn send_message(
            &mut self,
            topic: &str,
            payload: &str
        ) -> std::result::Result<(), EspError> {
            self.client
                .publish(topic, embedded_svc::mqtt::client::QoS::AtLeastOnce, false, payload.as_bytes())
                .await
        }
    }

    /// Generate an Azure IoT SAS token
    pub fn generate_sas_token(
        hub_name: &str,
        device_id: &str,
        key: &str,
        expiry_unix_ts: u64,
    ) -> String {
        let to_sign = format!(
            "{}.azure-devices.net/devices/{}\n{}",
            hub_name,
            device_id,
            expiry_unix_ts
        );

        let key_bytes = STANDARD.decode(key).unwrap();

        let mut mac = Hmac::<Sha256>::new_from_slice(&key_bytes).unwrap();
        mac.update(to_sign.as_bytes());
        let signature = mac.finalize().into_bytes();

        let sig_base64 = STANDARD.encode(signature);

        format!(
            "SharedAccessSignature sr={}&sig={}&se={}",
            encode(&resource_uri),
            encode(&sig_base64),
            expiry_unix_ts
        )
    }
}

crate::mod_interface! {
    orphan use {
        IoTHub
    };
    own use {
        generate_sas_token
    };
}
