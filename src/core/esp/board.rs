mod private {
    use crate::core::{Result, SmartPotError};
    use crate::core::esp::{
        Sensor,
        wifi,
        Ds18B20Sensor
    };
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
        pub sensors: Vec<Box<dyn Sensor<Pin = AnyIOPin>>>
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
            let ds18b20_sensors = ds18b20_sensors
                .into_iter()
                .map(|s| s as Box<dyn Sensor<Pin=AnyIOPin>>)
                .collect::<Vec<Box<dyn Sensor<Pin=AnyIOPin>>>>();

            let sensors_size = ds18b20_sensors.len();
            let mut sensors: Vec<Box<dyn Sensor<Pin=AnyIOPin>>> = Vec::with_capacity(sensors_size);
                
            sensors.extend(ds18b20_sensors);

            while ntp.get_sync_status() != SyncStatus::Completed {
                std::thread::sleep(Duration::from_millis(20));
            }

            Ok(Board {
                wifi,
                sensors,
            })
        }
    }
}

crate::mod_interface! {
    own use {
        Board
    };
}
