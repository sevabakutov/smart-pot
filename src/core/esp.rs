mod private {
    use crate::core::Result;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use one_wire_bus::OneWire;
    use serde::Serialize;
    use std::cell::RefCell;
    use std::rc::Rc;
    use esp_idf_hal::gpio::{InputOutput, InputPin, OutputPin, PinDriver};

    pub type OneWireType<T> = OneWire<PinDriver<'static, T, InputOutput>>;

    /// # TemperatureSensorData
    ///
    /// Data from temperature sensor of ESP32.
    #[derive(Debug, Serialize)]
    pub struct TemperatureSensorData {
        #[serde(with = "ts_seconds")]
        pub timestamp: DateTime<Utc>,
        pub temperature: f32,
    }

    /// # Telemetry
    ///
    /// All device telemtry.
    #[derive(Debug)]
    pub struct Telemetry {
        pub temperature: Option<TemperatureSensorData>,
    }

    pub trait Sensor {
        type Pin: InputPin + OutputPin;

        fn find_all(
            one_wire_bus: Rc<RefCell<OneWireType<Self::Pin>>>
        ) -> Result<Vec<Box<Self>>>
        where
            Self: Sized;

        fn read_temperature(&self) -> Result<TemperatureSensorData>;
    }
}

crate::mod_interface! {
    layer ds18b20;
    layer wifi;
    layer board;

    own use {
        Sensor,
        OneWireType
    };

    orphan use {
        TemperatureSensorData,
        Telemetry,
    };
}