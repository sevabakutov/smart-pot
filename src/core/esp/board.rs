mod private {
    use crate::core::esp::{wifi, DhtConfig, DhtSensor, Ds18B20Sensor, Sensor};
    use crate::core::{Result, SmartPotError};
    use esp_idf_hal::gpio::{AnyIOPin, PinDriver};
    use esp_idf_hal::modem::Modem;
    use esp_idf_svc::sntp::{EspSntp, SyncStatus};
    use esp_idf_svc::{
        eventloop::EspSystemEventLoop,
        nvs::EspDefaultNvsPartition,
        timer::EspTaskTimerService,
        wifi::{AsyncWifi, EspWifi},
    };
    use std::{cell::RefCell, rc::Rc, time::Duration};

    pub struct Board<'a> {
        pub wifi: AsyncWifi<EspWifi<'a>>,
        pub sensors: Vec<Box<dyn Sensor<Pin = AnyIOPin>>>,
    }

    impl Board<'_> {
        pub async fn init_board<'a>(
            ds18b20_pin: Option<AnyIOPin>,
            dht_config: Option<DhtConfig>,
            wifi_modem: Modem,
            wifi_ssid: &'static str,
            wifi_password: &'static str,
        ) -> Result<Board<'a>> {
            let sysloop = EspSystemEventLoop::take()?;
            let timer_service = EspTaskTimerService::new()?;
            let nvs = Some(EspDefaultNvsPartition::take()?);

            let wifi = wifi(
                wifi_ssid,
                wifi_password,
                wifi_modem,
                &sysloop,
                nvs,
                &timer_service,
            )
            .await?;

            let ntp = EspSntp::new_default()?;
            let mut sensors: Vec<Box<dyn Sensor<Pin = AnyIOPin>>> = Vec::new();

            if let Some(ds_pin) = ds18b20_pin {
                let ds_driver = PinDriver::input_output_od(ds_pin)?;
                let one_wire_bus = one_wire_bus::OneWire::new(ds_driver).map_err(|e| {
                    SmartPotError::OneWireError(format!("OneWire initializing error: {e:?}"))
                })?;
                let onewire_ref = Rc::from(RefCell::from(one_wire_bus));
                let ds18b20_sensors = Ds18B20Sensor::find_all(onewire_ref.clone())?;
                let ds18b20_sensors = ds18b20_sensors
                    .into_iter()
                    .map(|s| s as Box<dyn Sensor<Pin = AnyIOPin>>)
                    .collect::<Vec<Box<dyn Sensor<Pin = AnyIOPin>>>>();
                sensors.extend(ds18b20_sensors);
            }

            if let Some(config) = dht_config {
                let dht_driver = PinDriver::input_output_od(config.pin)?;
                let dht_sensor = Box::new(DhtSensor::new(dht_driver, config.dht_type));

                sensors.push(dht_sensor);
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
