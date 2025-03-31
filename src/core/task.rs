mod private {
    use std::time::Duration;

    use log::*;

    use crate::core::esp::Sensor;
    use crate::core::SmartPotError;
    use crate::core::Result;

    use esp_idf_svc::mqtt::client::EspAsyncMqttConnection;
    use esp_idf_svc::mqtt::client::EspAsyncMqttClient;
    use esp_idf_svc::timer::EspAsyncTimer;

    use esp_idf_hal::gpio::AnyIOPin;

    use embedded_svc::mqtt::client::QoS;

    /// cloud-to-device messages.
    pub async fn inbound_messages_task(
        connection: &mut EspAsyncMqttConnection
    ) -> Result<()> {
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
        sensors: &[Box<dyn Sensor<Pin=AnyIOPin>>],
        timer: &mut EspAsyncTimer,
        topic: &str,
    ) -> Result<()> {
        info!("Starting outbound telemetry task...");

        loop {
            for (index, sensor) in sensors.iter().enumerate() {
                let sensor_data = sensor.read_temperature().map_err(|e| {
                    error!("Sensor #{} read error: {:?}", index, e);
                    e
                })?;
                info!("Sensor #{} => {:?}", index, sensor_data);

                let payload = serde_json::to_string(&sensor_data).map_err(|e| {
                    error!("JSON serialization error: {:?}", e);
                    SmartPotError::ParsingError(e.to_string())
                })?;

                let _ = client
                    .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
                    .await?;
               
                info!("Published sensor #{} data to {topic}", index);
            }

            timer.after(Duration::from_secs(5)).await?;
        }
    }

}

crate::mod_interface!{
    own use {
        telemetry_task,
        inbound_messages_task
    };
}