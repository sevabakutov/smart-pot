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

    pub struct Board {
        pub wifi: AsyncWifi<EspWifi<'static>>,
        pub sensors: Vec<Box<dyn Sensor<'static>>>,
    }

    impl Board {
        pub async fn init_board(
            ds18b20_pins: Vec<AnyIOPin>,
            dht_configs: Vec<DhtConfig>,
            bh1750_i2c: Option<I2cDriver<'static>>,
            wifi_modem: Modem,
            wifi_ssid: &'static str,
            wifi_password: &'static str,
        ) -> Result<Board> {
            let sysloop = EspSystemEventLoop::take()?;
            let timer_service = EspTaskTimerService::new()?;
            let nvs = Some(EspDefaultNvsPartition::take()?);

            let wifi: AsyncWifi<EspWifi<'static>> = wifi(
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
                let one_wire_bus = one_wire_bus::OneWire::new(ds_driver).map_err(|e| {
                    SmartPotError::OneWireError(format!("OneWire initializing error: {e:?}"))
                })?;
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
