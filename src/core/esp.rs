mod private {
    use crate::core::Result;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use esp_idf_hal::gpio::{AnyIOPin, InputOutput, InputPin, OutputPin, PinDriver};
    use one_wire_bus::OneWire;
    use serde::Serialize;

    pub type OneWireType<T> = OneWire<PinDriver<'static, T, InputOutput>>;

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
        DhtConfig
    };

    orphan use {
        TemperatureSensorData,
        Telemetry,
    };
}
