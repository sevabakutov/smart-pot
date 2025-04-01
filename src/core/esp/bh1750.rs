mod private {
    use crate::core::esp::Sensor;
    use crate::core::{Result, SensorData, SmartPotError, Telemetry};
    use bh1750::*;
    use esp_idf_hal::delay::Delay;
    use esp_idf_hal::i2c::I2cDriver;
    pub struct Bh1750<'a> {
        sensor: bh1750::BH1750<I2cDriver<'a>, Delay>,
        resolution: Resolution,
    }

    impl<'a> Bh1750<'a> {
        pub fn new(i2c_driver: I2cDriver<'a>, resolution: Resolution) -> Self {
            let delay = esp_idf_hal::delay::Delay::new_default();

            let sensor = bh1750::BH1750::new(i2c_driver, delay, false);
            Bh1750 { sensor, resolution }
        }
    }

    impl<'a> Sensor<'a> for Bh1750<'a> {
        fn get_name(&self) -> String {
            "Bh1750 light sensor".to_string()
        }

        fn read_data(&mut self) -> Result<SensorData> {
            let data = self
                .sensor
                .get_one_time_measurement(self.resolution)
                .map_err(|err| {
                    SmartPotError::I2cError(format!("Failed to get light sensor data: {err:?}"))
                })?;

            Ok(SensorData {
                timestamp: chrono::Utc::now(),
                telemetry: Telemetry::LightValue(data),
            })
        }
    }
}

crate::mod_interface! {
    orphan use {
        private::Bh1750,
    };
}
