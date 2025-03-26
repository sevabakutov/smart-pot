//!
//! File with structures for interaction with ESP32.
//! 


mod private {
    use chrono::{DateTime, Utc};
    use chrono::serde::ts_seconds;
    use serde::Serialize;
    
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
    #[derive(Debug)]
    pub struct TemperatureSensor {}
    
    impl TemperatureSensor {
        async fn read(&self) -> TemperatureSensorData {
            unimplemented!()
        }
    }
    
    /// # Telemetry
    /// 
    /// All device telemtry.
    #[derive(Debug)]
    pub struct Telemetry {
        pub temperature: Option<TemperatureSensorData>
    }
}

crate::mod_interface! {
    
}