// iot_hub.rs

mod private {
    use std::time::Duration;

    use crate::core::Result;
    use crate::core::SmartPotError; 

    use esp_idf_svc::mqtt::client::{
        EspAsyncMqttClient, 
        MqttClientConfiguration, 
        EspAsyncMqttConnection, 
        MqttProtocolVersion
    };
    use esp_idf_sys::{
        esp_tls_init_global_ca_store, 
        esp_tls_set_global_ca_store,
        esp_crt_bundle_attach, 
        ESP_OK
    };

    const AZURE_IOT_CA_CERT: &[u8] = 
"-----BEGIN CERTIFICATE-----
MIIDjjCCAnagAwIBAgIQAzrx5qcRqaC7KGSxHQn65TANBgkqhkiG9w0BAQsFADBh
MQswCQYDVQQGEwJVUzEVMBMGA1UEChMMRGlnaUNlcnQgSW5jMRkwFwYDVQQLExB3
d3cuZGlnaWNlcnQuY29tMSAwHgYDVQQDExdEaWdpQ2VydCBHbG9iYWwgUm9vdCBH
MjAeFw0xMzA4MDExMjAwMDBaFw0zODAxMTUxMjAwMDBaMGExCzAJBgNVBAYTAlVT
MRUwEwYDVQQKEwxEaWdpQ2VydCBJbmMxGTAXBgNVBAsTEHd3dy5kaWdpY2VydC5j
b20xIDAeBgNVBAMTF0RpZ2lDZXJ0IEdsb2JhbCBSb290IEcyMIIBIjANBgkqhkiG
9w0BAQEFAAOCAQ8AMIIBCgKCAQEAuzfNNNx7a8myaJCtSnX/RrohCgiN9RlUyfuI
2/Ou8jqJkTx65qsGGmvPrC3oXgkkRLpimn7Wo6h+4FR1IAWsULecYxpsMNzaHxmx
1x7e/dfgy5SDN67sH0NO3Xss0r0upS/kqbitOtSZpLYl6ZtrAGCSYP9PIUkY92eQ
q2EGnI/yuum06ZIya7XzV+hdG82MHauVBJVJ8zUtluNJbd134/tJS7SsVQepj5Wz
tCO7TG1F8PapspUwtP1MVYwnSlcUfIKdzXOS0xZKBgyMUNGPHgm+F6HmIcr9g+UQ
vIOlCsRnKPZzFBQ9RnbDhxSJITRNrw9FDKZJobq7nMWxM4MphQIDAQABo0IwQDAP
BgNVHRMBAf8EBTADAQH/MA4GA1UdDwEB/wQEAwIBhjAdBgNVHQ4EFgQUTiJUIBiV
5uNu5g/6+rkS7QYXjzkwDQYJKoZIhvcNAQELBQADggEBAGBnKJRvDkhj6zHd6mcY
1Yl9PMWLSn/pvtsrF9+wX3N3KjITOYFnQoQj8kVnNeyIv/iPsGEMNKSuIEyExtv4
NeF22d+mQrvHRAiGfzZ0JFrabA0UWTW98kndth/Jsw1HKj2ZL7tcu7XUIOGZX1NG
Fdtom/DzMNU+MeKNhJ7jitralj41E6Vf8PlwUHBHQRFXGU7Aj64GxJUTFy8bJZ91
8rGOmaFvE7FBcf6IKshPECBV1/MUReXgRPTqh5Uykw7+U0b6LJ3/iyK5S9kJRaTe
pLiaWN0bfVKfjllDiIGknibVb63dDcY3fe0Dkhvld1927jyNxF1WW6LZZm6zNTfl
MrY=
-----END CERTIFICATE-----"

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

            let status_code = unsafe { esp_tls_init_global_ca_store() };

            if status_code != ESP_OK {
                log::error!("Failed to initialize ca store. Status code: {status_code}");
                return Err(SmartPotError::CAError("Не работает нихуя."));
            }

            let status_code = unsafe {
                esp_tls_set_global_ca_store(
                    AZURE_IOT_CA_CERT.as_ptr(),
                    AZURE_IOT_CA_CERT.len() as u32,
                )
            };

            if status_code != ESP_OK {
                return Err(SmartPotError::CAError(
                    format!("failed to set global ca store (err={})", code)
                ));
            }

            let broker_url = format!("mqtts://{}.azure-devices.net:8883", hub_name);
            let username = format!("{hub_name}.azure-devices.net/{device_id}/?api-version=2021-06-30");

            let mqtt_config = MqttClientConfiguration {
                protocol_version: Some(MqttProtocolVersion::V3_1_1),
                client_id: Some(device_id),
                username: Some(&username),
                password: Some(sas_token),

                use_global_ca_store: true,
                crt_bundle_attach: Some(esp_crt_bundle_attach),

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
