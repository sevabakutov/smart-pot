//!
//! File with structures for interaction with ESP32.
//!

mod private {
    use crate::core::{Result, SmartPotError};
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use esp_idf_hal::delay::Ets;
    use esp_idf_hal::gpio::{InputOutput, InputPin, OutputPin, PinDriver};
    use esp_idf_sys::EspError;
    use one_wire_bus::OneWire;
    use serde::Serialize;
    use std::cell::RefCell;
    use std::rc::Rc;
    pub type OneWireType<'a, T> = OneWire<PinDriver<'a, T, InputOutput>>;

    /// # TemperatureSensorData
    ///
    /// Data from temperature sensor of ESP32.
    #[derive(Debug, Serialize)]
    pub struct TemperatureSensorData {
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub temperature: f32,
    }

    /// # TemperatureSensor
    ///
    /// Structure to interact with temperature sensor of ESP32.
    pub struct Ds18B20Sensor<'a, T>
    where
        T: InputPin + OutputPin,
    {
        one_wire_bus: Rc<RefCell<OneWireType<'a, T>>>,
        ds_address: ds18b20::Ds18b20,
    }

    impl<'a, T: InputPin + OutputPin> Ds18B20Sensor<'a, T> {
        pub fn find_all(
            one_wire_bus: Rc<RefCell<OneWireType<'a, T>>>,
        ) -> Result<Vec<Ds18B20Sensor<'a, T>>> {
            let mut search_state = None;

            let mut delay = Ets {};
            let mut ds_sensors: Vec<ds18b20::Ds18b20> = vec![];
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
                .map(|s| Ds18B20Sensor {
                    one_wire_bus: one_wire_bus.clone(),
                    ds_address: s,
                })
                .collect();
            Ok(sensors)
        }

        pub fn read_temperature(&self) -> Result<TemperatureSensorData> {
            let mut delay = Ets {};

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

            Ok(TemperatureSensorData {
                timestamp: chrono::Utc::now(),
                temperature: sensor_data.temperature,
            })
        }
    }

    /// # Telemetry
    ///
    /// All device telemtry.
    #[derive(Debug)]
    pub struct Telemetry {
        pub temperature: Option<TemperatureSensorData>,
    }
}

crate::mod_interface! {
    orphan use {
        OneWireType,
        TemperatureSensorData,
        Ds18B20Sensor,
    };
}
