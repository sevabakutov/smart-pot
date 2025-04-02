mod private {
    use log::*;

    use std::time::Duration;
    use std::collections::HashMap;

    use serde::{Serialize, Deserialize};
    use esp_idf_svc::mqtt::client::{
        EspAsyncMqttClient, 
        EspAsyncMqttConnection, 
        MqttClientConfiguration,
        MqttProtocolVersion, 
        QoS, Event::Received, Event,
    };
    // use futures::StreamExt;

    use crate::core::SmartPotError;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct DeviceRegistration {
        #[serde(rename = "installationId")]
        pub installation_id: String,
        #[serde(rename = "expirationTime")]
        pub expiration_time: String,
        pub tags: Vec<String>,
        pub platform: String,
        #[serde(rename = "pushChannel")]
        pub push_channel: String,
        pub templates: HashMap<String, Template>,
        #[serde(rename = "secondaryTiles")]
        pub secondary_tiles: HashMap<String, SecondaryTile>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Template {
        pub body: String,
        pub tags: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub headers: Option<HashMap<String, String>>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SecondaryTile {
        #[serde(rename = "pushChannel")]
        pub push_channel: String,
        pub tags: Vec<String>,
        pub templates: HashMap<String, SecondaryTemplate>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct SecondaryTemplate {
        #[serde(rename = "bodyTemplate")]
        pub body_template: String,
        pub tags: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub headers: Option<HashMap<String, String>>,
    }

    #[derive(Debug, Deserialize)]
    pub struct RegistrationOperationStatus {
        #[serde(rename = "operationId")]
        pub operation_id: Option<String>,
        pub status: Option<String>,
        #[serde(rename = "registrationState")]
        pub registration_state: Option<RegistrationState>,
    }

    #[derive(Debug, Deserialize)]
    pub struct RegistrationState {
        #[serde(rename = "assignedHub")]
        pub assigned_hub: Option<String>,
        #[serde(rename = "deviceId")]
        pub device_id: Option<String>,
    }

    pub struct Dps {
        pub client: EspAsyncMqttClient,
        pub connection: EspAsyncMqttConnection
    }

    impl Dps {
        pub fn new(
            registration_id: &str,
            id_scope: &str,
            sas_token: &str
        ) -> Result<Self> {
            set_global_ca_store()?;

            let broker_url = "mqtts://global.azure-devices-provisioning.net:8883";
            let username = format!("{id_scope}/registrations/{registration_id}/api-version=2019-03-31");

            let mqtt_config = MqttClientConfiguration {
                protocol_version: Some(MqttProtocolVersion::V3_1_1),
                client_id: Some(registration_id),
                username: Some(&username),
                password: Some(sas_token),

                use_global_ca_store: true,
                crt_bundle_attach: None,

                keep_alive_interval: Some(Duration::from_secs(60)),
                reconnect_timeout: Some(Duration::from_secs(5)),
                disable_clean_session: false,
                ..Default::default()
            };

            let (client, connection) = EspAsyncMqttClient::new(broker_url, &mqtt_config)?;

            Ok(Self { client, connection })
        }

        pub async fn register_device(
            &self, 
            request_id: &str,
            device_registration: DeviceRegistration
        ) -> Result<DeviceRegistration> {
            let topic_filter = "$dps/registrations/res/#";
            let topic_name = format!("$dps/registrations/PUT/iotdps-register/?$rid={request_id}");

            loop {
                match self.client.subscribe(topic_filter, QoS::AtMostOnce).await {
                    Ok(_) => {
                        info!("Subscribed to topic \"{}\"", topic_filter);
                        break;
                    },
                    Err(e) => {
                        error!("Failed to subscribe to topic \"{}\": {:?}, retrying...", topic_filter, e);
                        timer.after(Duration::from_millis(500)).await?;
                    }
                }
            };

            // Just to give a chance of our connection to get even the first published message
            timer.after(Duration::from_millis(500)).await?;

            let payload = serde_json::to_string(&device_registration)?;
            self.client
                .publish(topic, QoS::AtMostOnce, false, payload.as_bytes())
                .await?;

            info!("Registration request published to {}", register_topic);

            let mut operation_id: Option<String> = None;

            while let Some(event) = self.connection.next().await {
                match event {
                    Received(msg) => {
                        let topic = msg.topic();
                        if topic.starts_with("$dps/registrations/res/") {
                            if let Some(after_res) = topic.strip_prefix("$dps/registrations/res/") {
                                let status_code_str = after_res.split('/').next().unwrap_or("");
                                let status_code = status_code_str.parse::<u16>().unwrap_or(0);
    
                                let payload_str = match std::str::from_utf8(msg.data()) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        error!("Error parsing DPS response as UTF-8: {:?}", e);
                                        continue;
                                    }
                                };
    
                                let parse_result: serde_json::Result<RegistrationOperationStatus> = serde_json::from_str(payload_str);
                                if parse_result.is_err() {
                                    error!("Failed to parse DPS JSON response: {:?}", parse_result.err());
                                }
                                let response = parse_result.unwrap_or_default();
    
                                match status_code {
                                    202 => {
                                        info!("DPS returned 202. Device is not yet assigned; need to poll operation status.");
                                        if let Some(op_id) = response.operationId {
                                            operation_id = Some(op_id.clone());
                                            let get_topic = format!(
                                                "$dps/registrations/GET/iotdps-get-operationstatus/?$rid={}&operationId={}",
                                                request_id,
                                                op_id
                                            );
                                            
                                            self.client
                                                .publish(&get_topic, QoS::AtMostOnce, false, &[])
                                                .await?;
    
                                            info!("Requested operation status: {}", get_topic);
                                        } else {
                                            error!("Got 202 but no operationId in the response JSON");
                                        }
                                    }
                                    200 => {
                                        info!("DPS returned 200. Checking if device is assigned.");
                                        if let Some(status) = response.status {
                                            if status.to_lowercase() == "assigned" {
                                                info!("Device is successfully assigned!");
                                                if let Some(state) = response.registrationState {
                                                    info!(
                                                        "AssignedHub: {:?}, DeviceId: {:?}",
                                                        state.assigned_hub, state.device_id
                                                    );
                                                }

                                                return Ok(());
                                            } else {
                                                error!("Received 200, but status = {}", status);

                                                return Err(SmartPotError::DPSError(
                                                    "Registration ended with 200 but status != assigned".to_string()
                                                ));
                                            }
                                        } else {
                                            error!("Received 200, but no status field in JSON");

                                            return Err(SmartPotError::DPSError(
                                                "No status in DPS response with 200 code".to_string()
                                            ));
                                        }
                                    }
                                    429 => {
                                        error!("Got 429 (Too Many Requests). Will retry...");

                                        timer.after(Duration::from_millis(500)).await?;
                                        if let Some(op_id) = &operation_id {
                                            let get_topic = format!(
                                                "$dps/registrations/GET/iotdps-get-operationstatus/?$rid={}&operationId={}",
                                                request_id,
                                                op_id
                                            );
                                            self.client
                                                .publish(&get_topic, QoS::AtMostOnce, false, &[])
                                                .await?;
                                        }
                                    }
                                    401 => {
                                        error!("Got 401 (Unauthorized). Check your SAS token or credentials.");

                                        return Err(SmartPotError::DPSError(
                                            "Unauthorized (401) from DPS".to_string()
                                        ));
                                    }
                                    other => {
                                        error!("DPS returned unexpected status code: {}", other);

                                        return Err(SmartPotError::DPSError(
                                            format!("DPS returned unexpected code: {}", other)
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            };

            Err(SmartPotError::DPSError(
                "DPS registration did not complete. No 200 status received.".to_string()
            ))
        }
    }
}

crate::mod_interface! {
    own use {
        DeviceRegistration,
        Template,
        SecondaryTile,
        SecondaryTemplate,
        RegistrationOperationStatus,
        RegistrationState,
    };

    orphan use {
        Dps
    };
}