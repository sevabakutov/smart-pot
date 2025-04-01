//!
//!  Board module
//!

mod private {
    use crate::core::esp::{wifi, Bh1750, DhtConfig, DhtSensor, Ds18B20Sensor, Sensor};
    use crate::core::{Result, SmartPotError};
    use esp_idf_hal::gpio::{AnyIOPin, PinDriver};
    use esp_idf_hal::i2c::I2cDriver;
    use esp_idf_hal::modem::Modem;
    use esp_idf_svc::sntp::{EspSntp, SyncStatus};
    use esp_idf_svc::{
        eventloop::EspSystemEventLoop,
        nvs::EspDefaultNvsPartition,
        timer::EspTaskTimerService,
        wifi::{AsyncWifi, EspWifi},
    };
    use std::{cell::RefCell, rc::Rc, time::Duration};

    /// # Board
    ///
    /// Represents a board with supported sensors and a Wi-Fi module. This struct
    /// encapsulates the sensors connected to the board as well as the Wi-Fi interface.
    ///
    /// ## Fields:
    /// - `wifi`:  
    ///   An instance of the `AsyncWifi` struct, responsible for managing the Wi-Fi connection.
    /// - `sensors`:  
    ///   A vector containing all sensors connected to the board, each implementing the `Sensor` trait.
    ///
    /// ## Usage
    ///
    /// `Board` obtained by calling the `init_board()` function.
    pub struct Board {
        pub wifi: AsyncWifi<EspWifi<'static>>,
        pub sensors: Vec<Box<dyn Sensor<'static>>>,
    }

    impl Board {
        /// Initializes the board with connected sensors and establishes the Wi-Fi connection.
        ///
        /// This method configures the Wi-Fi connection, synchronizes NTP, and sets up sensors including
        /// DS18B20, DHT, and BH1750 sensors if provided. I
        ///
        /// # Parameters:
        /// - `ds18b20_pins`:  
        ///   A vector of `AnyIOPin` pins connected to DS18B20 sensors (OneWire bus).
        /// - `dht_configs`:  
        ///   A vector of configuration structs for DHT sensors. Each config contains the pin number
        ///   and sensor type (DHT11, DHT22).
        /// - `bh1750_i2c`:  
        ///   An optional `I2cDriver` for the BH1750 light sensor. If `None`, no BH1750 sensor is initialized.
        /// - `wifi_modem`:  
        ///   The modem used to manage the Wi-Fi connection.
        /// - `wifi_ssid`:  
        ///   The SSID of the Wi-Fi network.
        /// - `wifi_password`:  
        ///   The password for the Wi-Fi network.
        ///
        /// # Returns:
        /// - `Result<Board>`:  
        ///   A result containing the fully initialized `Board` instance if successful, or an error.
        ///
        /// # Errors:
        /// This method may return errors in the following cases:
        /// - Failed to initialize OneWire for DS18B20 sensors.
        /// - Failed to configure any of the sensors.
        /// - Failed to establish the Wi-Fi connection or synchronize NTP.
        pub async fn init_board(
            ds18b20_pins: Vec<AnyIOPin>,
            dht_configs: Vec<DhtConfig>,
            bh1750_i2c: Option<I2cDriver<'static>>,
            wifi_modem: Modem,
            wifi_ssid: &str,
            wifi_password: &str,
        ) -> Result<Board> {
            let sysloop = EspSystemEventLoop::take()?;
            let timer_service = EspTaskTimerService::new()?;
            let nvs = Some(EspDefaultNvsPartition::take()?);

            let wifi: AsyncWifi<EspWifi<'_>> = wifi(
                wifi_ssid,
                wifi_password,
                wifi_modem,
                &sysloop,
                nvs,
                &timer_service,
            )
            .await?;

            let ntp = EspSntp::new_default()?;
            let mut sensors: Vec<Box<dyn Sensor>> = Vec::new();

            for ds in ds18b20_pins {
                let ds_driver = PinDriver::input_output_od(ds)?;
                let one_wire_bus = one_wire_bus::OneWire::new(ds_driver)
                    .map_err(|e| SmartPotError::OneWireError(e.into()))?;
                let onewire_ref = Rc::from(RefCell::from(one_wire_bus));
                let ds18b20_sensors = Ds18B20Sensor::find_all(onewire_ref.clone())?;
                let ds18b20_sensors = ds18b20_sensors
                    .into_iter()
                    .map(|s| s as Box<dyn Sensor>)
                    .collect::<Vec<Box<dyn Sensor>>>();
                sensors.extend(ds18b20_sensors);
            }

            for dht in dht_configs {
                let dht_driver = PinDriver::input_output_od(dht.pin)?;
                let dht_sensor = Box::new(DhtSensor::new(dht_driver, dht.dht_type));

                sensors.push(dht_sensor);
            }

            if let Some(i2c) = bh1750_i2c {
                let bh = Box::new(Bh1750::new(i2c, bh1750::Resolution::High));
                sensors.push(bh);
            }
            while ntp.get_sync_status() != SyncStatus::Completed {
                std::thread::sleep(Duration::from_millis(20));
            }

            Ok(Board { wifi, sensors })
        }
    }
}

crate::mod_interface! {
    own use {
        Board
    };
}
