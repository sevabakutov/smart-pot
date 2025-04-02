// iot_hub.rs

mod private {
    use std::time::Duration;

    use crate::core::Result;
    use crate::core::SmartPotError;

    use esp_idf_svc::mqtt::client::{
        EspAsyncMqttClient, 
        EspAsyncMqttConnection, 
        MqttClientConfiguration, 
        MqttProtocolVersion,
    };

    ///
    ///
    /// A simple Azure IoT Hub MQTT wrapper.
    pub struct IoTHub {
        pub client: EspAsyncMqttClient,
        pub connection: EspAsyncMqttConnection,
    }

    impl IoTHub {
        /// Creates a new Mqtt Client with SAS auth.7
        pub fn new(
            host_name: &str, 
            device_id: &str, 
            sas_token: &str,
        ) -> Result<Self> {
            set_global_ca_store()?;
            
            let broker_url = format!("mqtts://{}.azure-devices.net:8883", hub_name);
            let username = format!("{host_name}/{device_id}/?api-version=2021-06-30");
            let topic = format!("devices/{device_id}/messages/events/");

            let lwt = LwtConfiguration {
                topic: topic.as_str(),
                payload: &[1, 2, 3],
                qos: QoS::AtMostOnce,
                retain: true
            };

            let mqtt_config = MqttClientConfiguration {
                protocol_version: Some(MqttProtocolVersion::V3_1_1),
                client_id: Some(device_id),
                username: Some(&username),
                password: Some(sas_token),

                use_global_ca_store: true,
                crt_bundle_attach: None,
                lwt: Some(lwt),

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
