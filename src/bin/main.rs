// main.rs

use core::pin::pin;
use core::time::Duration;

use esp_idf_hal::i2c::config::Config;
use esp_idf_hal::i2c::I2cDriver;
use log::*;

use smart_pot::core::azure::{generate_sas_token, IoTHub};
use smart_pot::core::esp::board::Board;
use smart_pot::core::esp::{DhtConfig, DhtType, Sensor};
use smart_pot::core::task::{inbound_messages_task, telemetry_task};
use smart_pot::core::Result;

use embassy_futures::select::{select, Either};

use esp_idf_svc::hal::task;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::timer::{EspAsyncTimer, EspTimerService};

use esp_idf_hal::gpio::IOPin;
use esp_idf_hal::prelude::Peripherals;

// Environment vars or constants
const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");
const HUB_NAME: &str = env!("IOT_HUB_NAME");
const DEVICE_ID: &str = env!("DEVICE_ID");
const SHARED_ACCESS_KEY: &str = env!("SHARED_ACCESS_KEY");
const SCOPE_ID: &str = env!("SCOPE_ID");
const REGISTRATION_ID: &str = env!("REGISTRATION_ID");
const SHARED_DPS_ACCESS_KEY: &str = env!("SHARED_DPS_ACCESS_KEY");

fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let result = task::block_on(async_main());
    if let Err(e) = result {
        error!("Error: {:?}", e);
    } else {
        info!("Done!");
    }
}

/// Main async entry point
async fn async_main() -> Result<()> {
    let timer_svc = EspTimerService::new()?;
    let mut timer = timer_svc.timer_async()?;

    let mut board = loop {
        let peripherals: Peripherals = match Peripherals::take() {
            Ok(per) => per,
            Err(err) => {
                log::error!("Error while getting peripherals: {err}");
                timer.after(Duration::from_secs(5)).await?;
                continue;
            }
        };

        let modem = peripherals.modem;
        let ds_pins = vec![peripherals.pins.gpio16.downgrade()];
        let dht_configs = vec![
            DhtConfig::new(peripherals.pins.gpio17.downgrade(), DhtType::Dht11),
            DhtConfig::new(peripherals.pins.gpio5.downgrade(), DhtType::Dht22),
        ];
        let i2c = Some(I2cDriver::new(
            peripherals.i2c0,
            peripherals.pins.gpio21,
            peripherals.pins.gpio22,
            &Config::default(),
        )?);

        match Board::init_board(ds_pins, dht_configs, i2c, modem, SSID, PASS).await {
            Ok(board) => break board,
            Err(e) => {
                error!("Error initializing board: {e:?}");
            }
        }
        timer.after(Duration::from_secs(5)).await?;
    };

    let expiry_unix_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;

    // let sas_token = generate_sas_token(HUB_NAME, DEVICE_ID, SHARED_ACCESS_KEY, expiry_unix_ts);
    // info!("SAS token generated. Expires at {expiry_unix_ts}");

    // let iothub = IoTHub::new(HUB_NAME, DEVICE_ID, &sas_token)?;
    // info!("IoTHub client created. Connecting...");

    let sas_token = generate_sas_token_dps(
        ID_SCOPE, 
        REGISTRATION_ID, 
        SHARED_DPS_ACCESS_KEY, 
        expiry_unix_ts
    );
    info!("SAS token generated. Expires at {expiry_unix_ts}");

    let iothub = IoTHub::dps(HUB_NAME, DEVICE_ID, REGISTRATION_ID, ID_SCOPE, sas_token)?;
    info!("IoTHub client created. Connecting...");


    run(iothub, &mut timer, &mut board.sensors).await?;

    Ok(())
}

/// starts main async loop
async fn run(
    mut iothub: IoTHub,
    timer: &mut EspAsyncTimer,
    sensors: &mut [Box<dyn Sensor<'_>>],
) -> Result<()> {
    let topic = format!("devices/{}/messages/events/", DEVICE_ID);

    let inbound = inbound_messages_task(&mut iothub.connection);
    let outbound = telemetry_task(&mut iothub.client, sensors, timer, &topic);

    let res = select(pin!(inbound), pin!(outbound)).await;

    match res {
        Either::First(Ok(())) => {
            info!("Inbound messages task finished. Outbound task is canceled.");
            Ok(())
        }
        Either::First(Err(e)) => {
            error!("Inbound messages task failed: {e:?}");
            Err(e)
        }
        Either::Second(Ok(())) => {
            info!("Outbound telemetry task finished. Inbound task is canceled.");
            Ok(())
        }
        Either::Second(Err(e)) => {
            error!("Outbound telemetry task failed: {e:?}");
            Err(e)
        }
    }
}
