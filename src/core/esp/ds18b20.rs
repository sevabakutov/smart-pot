mod private {
    use crate::core::esp::{OneWireType, Sensor};
    use crate::core::SensorData;
    use crate::core::Telemetry;
    use crate::core::{Result, SmartPotError};
    use esp_idf_hal::gpio::{InputPin, OutputPin};
    use esp_idf_sys::EspError;
    use std::cell::RefCell;
    use std::rc::Rc;
    /// # TemperatureSensor
    ///
    /// Structure to interact with temperature sensor of ESP32.
    pub struct Ds18B20Sensor<T>
    where
        T: InputPin + OutputPin,
    {
        one_wire_bus: Rc<RefCell<OneWireType<T>>>,
        ds_address: ds18b20::Ds18b20,
    }

    impl<T> Ds18B20Sensor<T>
    where
        T: InputPin + OutputPin,
    {
        pub fn find_all(one_wire_bus: Rc<RefCell<OneWireType<T>>>) -> Result<Vec<Box<Self>>>
        where
            Self: Sized,
        {
            let mut search_state = None;

            let mut delay = esp_idf_hal::delay::Delay::new_default();
            let mut ds_sensors: Vec<ds18b20::Ds18b20> = Vec::new();

            while let Some((device_address, state)) = one_wire_bus
                .borrow_mut()
                .device_search(search_state.as_ref(), false, &mut delay)
                .map_err(|e| {
                    SmartPotError::OneWireError(format!("Error while searching devices: {e:?}"))
                })?
            {
                search_state = Some(state);

                if device_address.family_code() != ds18b20::FAMILY_CODE {
                    continue;
                }

                log::trace!("Found ds18b20: {:?}", device_address);
                let sensor = ds18b20::Ds18b20::new::<EspError>(device_address).map_err(|e| {
                    SmartPotError::OneWireError(format!(
                        "Error while getting {device_address:?} device: {e:?}"
                    ))
                })?;

                ds_sensors.push(sensor);
            }

            let sensors = ds_sensors
                .into_iter()
                .map(|s| {
                    Box::new(Ds18B20Sensor {
                        one_wire_bus: one_wire_bus.clone(),
                        ds_address: s,
                    })
                })
                .collect::<Vec<Box<Ds18B20Sensor<T>>>>();
            Ok(sensors)
        }
    }

    impl<T> Sensor<'_> for Ds18B20Sensor<T>
    where
        T: InputPin + OutputPin,
    {
        fn get_name(&self) -> String {
            "Ds18B20".to_string()
        }

        fn read_data(&mut self) -> Result<SensorData> {
            let mut delay = esp_idf_hal::delay::Delay::new_default();

            ds18b20::start_simultaneous_temp_measurement(
                &mut self.one_wire_bus.borrow_mut(),
                &mut delay,
            )
            .map_err(|e| {
                SmartPotError::OneWireError(format!(
                    "Error while starting temperature measurment: {e:?}"
                ))
            })?;

            ds18b20::Resolution::Bits12.delay_for_measurement_time(&mut delay);

            let sensor_data = self
                .ds_address
                .read_data(&mut self.one_wire_bus.borrow_mut(), &mut delay)
                .map_err(|e| {
                    SmartPotError::OneWireError(format!("Error while reading temperature: {e:?}"))
                })?;

            Ok(SensorData {
                timestamp: chrono::Utc::now(),
                telemetry: Telemetry::Temperature(sensor_data.temperature),
            })
        }
    }
}

crate::mod_interface! {
    orphan use {
        Ds18B20Sensor,
    };
}
