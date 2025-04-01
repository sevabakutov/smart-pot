mod private {
    use crate::core::{Result, SmartPotError};
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use esp_idf_hal::gpio::{AnyIOPin, IOPin, InputOutput, InputPin, OutputPin, PinDriver};
    use esp_idf_hal::peripheral::Peripheral;
    use one_wire_bus::OneWire;
    use serde::Serialize;

    pub type OneWireType<T> = OneWire<PinDriver<'static, T, InputOutput>>;

    pub trait TryIntoPin {
        /// Unsafe. Try to convert `i32` to GPIO pin using available pins
        ///
        /// # Safety
        /// Caller must ensure that:
        /// - The provided `i32` corresponds to a GPIO that hasn't already been moved or initialized
        /// - The `Pins` object must still contain ownership of the corresponding pin
        /// - The same GPIO must not be taken twice (Rust will panic or UB)
        unsafe fn try_into_gpio_pin(&self, p: &mut esp_idf_hal::gpio::Pins) -> Result<AnyIOPin>;
    }

    impl TryIntoPin for i32 {
        unsafe fn try_into_gpio_pin(&self, p: &mut esp_idf_hal::gpio::Pins) -> Result<AnyIOPin> {
            match *self {
                0 => Ok(p.gpio0.clone_unchecked().downgrade()),
                1 => Ok(p.gpio1.clone_unchecked().downgrade()),
                2 => Ok(p.gpio2.clone_unchecked().downgrade()),
                3 => Ok(p.gpio3.clone_unchecked().downgrade()),
                4 => Ok(p.gpio4.clone_unchecked().downgrade()),
                5 => Ok(p.gpio5.clone_unchecked().downgrade()),
                12 => Ok(p.gpio12.clone_unchecked().downgrade()),
                13 => Ok(p.gpio13.clone_unchecked().downgrade()),
                14 => Ok(p.gpio14.clone_unchecked().downgrade()),
                15 => Ok(p.gpio15.clone_unchecked().downgrade()),
                16 => Ok(p.gpio16.clone_unchecked().downgrade()),
                17 => Ok(p.gpio17.clone_unchecked().downgrade()),
                18 => Ok(p.gpio18.clone_unchecked().downgrade()),
                19 => Ok(p.gpio19.clone_unchecked().downgrade()),
                21 => Ok(p.gpio21.clone_unchecked().downgrade()),
                22 => Ok(p.gpio22.clone_unchecked().downgrade()),
                23 => Ok(p.gpio23.clone_unchecked().downgrade()),
                25 => Ok(p.gpio25.clone_unchecked().downgrade()),
                26 => Ok(p.gpio26.clone_unchecked().downgrade()),
                27 => Ok(p.gpio27.clone_unchecked().downgrade()),
                32 => Ok(p.gpio32.clone_unchecked().downgrade()),
                33 => Ok(p.gpio33.clone_unchecked().downgrade()),
                _ => Err(SmartPotError::PinParseError(format!(
                    "GPIO {} is not supported for DHT/DS18B20",
                    self
                ))),
            }
        }
    }
    pub struct DhtConfig {
        pub pin: AnyIOPin,
        pub dht_type: DhtType,
    }

    impl DhtConfig {
        pub fn new(pin: AnyIOPin, dht_type: DhtType) -> Self {
            DhtConfig { pin, dht_type }
        }
    }
    pub enum DhtType {
        Dht11,
        Dht22,
    }

    #[derive(Debug, Serialize)]
    pub struct TemperatureWithHumidity {
        pub temperature: f32,
        pub humidity: f32,
    }

    #[derive(Debug, Serialize)]
    pub enum Telemetry {
        Temperature(f32),
        TemperatureWithHumidity(TemperatureWithHumidity),
    }

    #[derive(Debug, Serialize)]
    pub struct TemperatureSensorData {
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub telemetry: Telemetry,
    }

    pub trait Sensor {
        type Pin: InputPin + OutputPin;

        fn get_name(&self) -> String;

        fn read_data(&mut self) -> Result<TemperatureSensorData>;
    }
}

crate::mod_interface! {
    layer ds18b20;
    layer dht_sensor;
    layer wifi;
    layer board;

    own use {
        Sensor,
        OneWireType,
        DhtType,
        DhtConfig,
        TryIntoPin
    };

    orphan use {
        TemperatureSensorData,
        Telemetry,
    };
}
