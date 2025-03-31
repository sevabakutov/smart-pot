mod private {
    use std::time::Duration;

    use log::*;

    use crate::core::esp::Sensor;
    use crate::core::Result;
    use crate::core::SmartPotError;

    use esp_idf_svc::mqtt::client::EspAsyncMqttClient;
    use esp_idf_svc::mqtt::client::EspAsyncMqttConnection;
    use esp_idf_svc::timer::EspAsyncTimer;

    use esp_idf_hal::gpio::AnyIOPin;

    use embedded_svc::mqtt::client::QoS;

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
        sensors: &[Box<dyn Sensor<Pin = AnyIOPin>>],
        timer: &mut EspAsyncTimer,
        topic: &str,
    ) -> Result<()> {
        info!("Starting outbound telemetry task...");

        loop {
            for (index, sensor) in sensors.iter().enumerate() {
                let sensor_data = match sensor.read_temperature() {
                    Ok(sensor) => sensor,
                    Err(e) => {
                        error!("Sensor #{} read error: {:?}", index, e);
                        continue;
                    }
                };

                info!("Sensor #{} => {:?}", index, sensor_data);

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
                        info!("Published sensor #{} data to {topic}", index);
                    }
                    Err(e) => {
                        error!("Publishing #{} error: {:?}", index, e);
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
