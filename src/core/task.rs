mod private {
    use std::time::Duration;

    use log::*;

    use crate::core::esp::Sensor;
    use crate::core::Result;
    use embedded_svc::mqtt::client::QoS;
    use esp_idf_hal::delay::FreeRtos;
    use esp_idf_svc::mqtt::client::EspAsyncMqttClient;
    use esp_idf_svc::mqtt::client::EspAsyncMqttConnection;
    use esp_idf_svc::timer::EspAsyncTimer;

    /// cloud-to-device messages.
    pub async fn inbound_messages_task(connection: &mut EspAsyncMqttConnection) -> Result<()> {
        info!("Starting inbound messages task...");

        while let Ok(event) = connection.next().await {
            info!("[Queue] Event: {}", event.payload());
        }

        info!("MQTT connection closed (inbound task)");
        Ok(())
    }

    /// device-to-cloud messages.
    pub async fn telemetry_task(
        client: &mut EspAsyncMqttClient,
        sensors: &mut [Box<dyn Sensor<'_>>],
        timer: &mut EspAsyncTimer,
        topic: &str,
    ) -> Result<()> {
        info!("Starting outbound telemetry task...");

        loop {
            for sensor in sensors.iter_mut() {
                let mut sensor_data = None;
                for i in 1..=3 {
                    match sensor.read_data() {
                        Ok(data) => {
                            sensor_data = Some(data);
                            break;
                        }
                        Err(e) => {
                            warn!(
                                "Retry #{i}: Sensor #{} read error: {:?}",
                                sensor.get_name(),
                                e
                            );
                            FreeRtos::delay_ms(100);
                        }
                    }
                }

                let sensor_data = match sensor_data {
                    Some(data) => data,
                    None => {
                        error!("Sensor #{} failed to read after retries", sensor.get_name());
                        continue;
                    }
                };

                info!("Sensor #{} => {:?}", sensor.get_name(), sensor_data);

                let payload = match serde_json::to_string(&sensor_data) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("JSON serialization error: {:?}", e);
                        continue;
                    }
                };
                match client
                    .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
                    .await
                {
                    Ok(_) => {
                        info!("Published sensor #{} data to {topic}", sensor.get_name());
                    }
                    Err(e) => {
                        error!("Publishing #{} error: {:?}", sensor.get_name(), e);
                    }
                }
            }

            timer.after(Duration::from_secs(5)).await?;
        }
    }
}

crate::mod_interface! {
    own use {
        telemetry_task,
        inbound_messages_task
    };
}
