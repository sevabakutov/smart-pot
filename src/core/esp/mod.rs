mod modules;

mod private {
    use super::modules::{ds18b20::Ds18B20Sensor, wifi::wifi};
    use crate::core::{Result, SmartPotError};
    use esp_idf_hal::{
        gpio::{AnyIOPin, IOPin, PinDriver},
        prelude::Peripherals,
    };
    use esp_idf_svc::{
        eventloop::EspSystemEventLoop,
        nvs::EspDefaultNvsPartition,
        timer::EspTaskTimerService,
        wifi::{AsyncWifi, EspWifi},
    };

    use esp_idf_svc::sntp::{EspSntp, SyncStatus};
    use std::{cell::RefCell, rc::Rc, time::Duration};

    pub struct Board<'a> {
        pub wifi: AsyncWifi<EspWifi<'a>>,
        pub ds18b20_sensors: Vec<Ds18B20Sensor<'a, AnyIOPin>>,
    }

    impl Board<'_> {
        pub async fn init_board<'a>(
            wifi_ssid: &'static str,
            wifi_password: &'static str,
        ) -> Result<Board<'a>> {
            let peripherals: Peripherals = Peripherals::take()?;
            let sysloop = EspSystemEventLoop::take()?;
            let timer_service = EspTaskTimerService::new()?;
            let nvs = Some(EspDefaultNvsPartition::take()?);
            let modem = peripherals.modem;

            let wifi = wifi(
                wifi_ssid,
                wifi_password,
                modem,
                &sysloop,
                nvs,
                &timer_service,
            )
            .await?;

            let ntp = EspSntp::new_default()?;

            let ds_pin = PinDriver::input_output_od(peripherals.pins.gpio16.downgrade())?;
            let one_wire_bus = one_wire_bus::OneWire::new(ds_pin).map_err(|e| {
                SmartPotError::OneWireError(format!("OneWire initializing error: {e:?}"))
            })?;

            let onewire_ref = Rc::from(RefCell::from(one_wire_bus));
            let ds18b20_sensors = Ds18B20Sensor::find_all(onewire_ref.clone())?;

            while ntp.get_sync_status() != SyncStatus::Completed {
                std::thread::sleep(Duration::from_millis(20));
            }

            Ok(Board {
                wifi,
                ds18b20_sensors,
            })
        }
    }
}

crate::mod_interface! {
    orphan use {
        Board,
        modules::ds18b20::{Ds18B20Sensor, TemperatureSensorData}
    };
}
