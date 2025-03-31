// iot_hub.rs

mod private {
    use std::time::Duration;

    use crate::core::Result;

    use esp_idf_svc::mqtt::client::{
        EspAsyncMqttClient, 
        MqttClientConfiguration, 
        EspAsyncMqttConnection, 
        MqttProtocolVersion
    };

    /// # IoTHub
    /// 
    /// A simple Azure IoT Hub MQTT wrapper.
    pub struct IoTHub {
        pub client: EspAsyncMqttClient,
        pub connection: EspAsyncMqttConnection,
    }

    impl IoTHub {
        /// Creates a new Mqtt Client with SAS auth.
        pub fn new(
            hub_name: &str,
            device_id: &str,
            sas_token: &str,
        ) -> Result<Self> {
            let broker_url = format!("mqtts://{}.azure-devices.net:8883", hub_name);
            let username = format!("{hub_name}.azure-devices.net/{device_id}/?api-version=2021-06-30");

            let mqtt_config = MqttClientConfiguration {
                protocol_version: Some(MqttProtocolVersion::V3_1_1),
                client_id: Some(device_id),
                username: Some(&username),
                password: Some(sas_token),

                keep_alive_interval: Some(Duration::from_secs(60)),
                reconnect_timeout: Some(Duration::from_secs(5)),
                disable_clean_session: false,
                ..Default::default()
            };

            let (client, connection) = EspAsyncMqttClient::new(&broker_url, &mqtt_config)?;

            Ok(Self {
                client, 
                connection 
            })
        }
    }
}

crate::mod_interface! {
    orphan use {
        IoTHub
    };
}
