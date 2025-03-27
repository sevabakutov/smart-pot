use std::{
    any::Any,
    str::{from_utf8, FromStr},
    sync::{Arc, Mutex},
};

use embedded_svc::{
    http::Method::Get,
    http::Method::Post,
    io::Read,
    wifi::{AuthMethod, ClientConfiguration, Configuration},
};
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{IOPin, InputPin, OutputPin, Pin},
    peripheral::PeripheralRef,
    units::*,
};
use esp_idf_hal::{
    gpio::PinDriver,
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver},
    peripheral::Peripheral,
    prelude::Peripherals,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::server::EspHttpServer,
    nvs::{EspDefaultNvsPartition, EspNvsPartition, NvsDefault},
    ping::EspPing,
    timer::{EspTaskTimerService, EspTimerService, Task},
    wifi::{AsyncWifi, EspWifi},
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

use smart_pot::{
    config::Settings,
    core::{telemetry::*, wifi::wifi, IoTHub, Result, SmartPotError},
};
use std::{cell::RefCell, rc::Rc};
static SSID: &str = "kievskaya_hunta";
static PASS: &str = "fourtyeighteen";

use esp_idf_hal::gpio::AnyIOPin;

struct Board<'a> {
    wifi: AsyncWifi<EspWifi<'a>>,
    ds18b20_sensors: Vec<Ds18B20Sensor<'a, AnyIOPin>>,
}

fn init_board<'a, T: InputPin + OutputPin>() -> Result<Board<'a>> {
    let peripherals: Peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = Some(EspDefaultNvsPartition::take()?);
    let modem = peripherals.modem;

    let wifi = wifi(SSID, PASS, modem, &sysloop, nvs, &timer_service)?;

    let ds_pin = PinDriver::input_output_od(peripherals.pins.gpio16.downgrade())?;
    let one_wire_bus = one_wire_bus::OneWire::new(ds_pin)
        .map_err(|e| SmartPotError::OneWireError(format!("OneWire initializing error: {e:?}")))?;

    let onewire_ref = Rc::from(RefCell::from(one_wire_bus));
    let ds18b20_sensors = Ds18B20Sensor::find_all(onewire_ref.clone())?;

    Ok(Board {
        wifi,
        ds18b20_sensors,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    //let path = PathBuf::from("Config.toml");

    //env_logger::init();
    //let settings = Settings::new(&path)?;

    let mut peripherals: Peripherals = Peripherals::take().unwrap();

    let ds_pin = PinDriver::input_output_od(peripherals.pins.gpio16).unwrap();

    let modem: &mut esp_idf_hal::modem::Modem = &mut peripherals.modem;
    loop {
        //      let board = init_board();
    }

    // let (hub, mut eventloop) = IoTHub::from_settings(settings)?;

    // tokio::spawn(async move {
    //     loop {
    //         match eventloop.poll().await {
    //             Ok(rumqttc::Event::Incoming(packet)) => {
    //                 info!("Incomming packet: {:?}", packet);
    //             },
    //             Ok(rumqttc::Event::Outgoing(packet)) => {
    //                 info!("Outcomming packet: {:?}", packet);
    //             },
    //             Err(e) => {
    //                 error!("Error in MQTT event loop: {:?}", e);
    //             }
    //         }
    //     }
    // });

    Ok(())
}
