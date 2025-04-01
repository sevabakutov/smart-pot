mod private {
    use crate::core::esp::private::TemperatureWithHumidity;
    use crate::core::esp::{DhtType, Sensor};
    use crate::core::{Result, SensorData, SmartPotError, Telemetry};
    use dht_sensor::*;
    use esp_idf_hal::gpio::{InputOutput, InputPin, OutputPin, PinDriver};

    type Driver<T> = PinDriver<'static, T, InputOutput>;

    pub struct DhtSensor<T>
    where
        T: InputPin + OutputPin,
    {
        pin_driver: Driver<T>,
        dht_type: DhtType,
    }

    impl<T> DhtSensor<T>
    where
        T: InputPin + OutputPin,
    {
        pub fn new(pin: Driver<T>, dht_type: DhtType) -> Self {
            DhtSensor {
                pin_driver: pin,
                dht_type,
            }
        }
    }

    impl<T> Sensor<'_> for DhtSensor<T>
    where
        T: InputPin + OutputPin,
    {
        fn get_name(&self) -> String {
            "DHT".to_string()
        }

        fn read_data(&mut self) -> Result<SensorData> {
            let mut delay = esp_idf_hal::delay::Delay::new_default();

            match self.dht_type {
                DhtType::Dht11 => match dht11::Reading::read(&mut delay, &mut self.pin_driver) {
                    Ok(data) => Ok(SensorData {
                        timestamp: chrono::Utc::now(),
                        telemetry: Telemetry::TemperatureWithHumidity(TemperatureWithHumidity {
                            temperature: data.temperature as f32,
                            humidity: data.relative_humidity as f32,
                        }),
                    }),
                    Err(err) => Err(SmartPotError::DhtError(format!(
                        "Error while reading dht data: {err:?}"
                    ))),
                },

                DhtType::Dht22 => match dht22::Reading::read(&mut delay, &mut self.pin_driver) {
                    Ok(data) => Ok(SensorData {
                        timestamp: chrono::Utc::now(),
                        telemetry: Telemetry::TemperatureWithHumidity(TemperatureWithHumidity {
                            temperature: data.temperature,
                            humidity: data.relative_humidity,
                        }),
                    }),
                    Err(err) => Err(SmartPotError::DhtError(format!(
                        "Error while reading dht data: {err:?}"
                    ))),
                },
            }
        }
    }
}

crate::mod_interface! {
    orphan use {
        private::DhtSensor,
    };
}
