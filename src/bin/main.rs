// main.rs

use core::pin::pin;
use core::time::Duration;

use log::*;
use anyhow::Result;

use embassy_futures::select::{select, Either};

use esp_idf_svc::hal::task;
use esp_idf_svc::sys::link_patches;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::timer::{EspTimerService, EspAsyncTimer};

use embedded_svc::mqtt::client::{QoS, Event, Client};
use esp_idf_sys::EspError;

use smart_pot::core::{
    esp::Board,
    IoTHub,
    generate_sas_token,
};

// Environment vars or constants
const SSID: &str = env!("WIFI_SSID");
const PASS: &str = env!("WIFI_PASS");
const HUB_NAME: &str = env!("IOT_HUB_NAME");  // e.g. "my-hub"
const DEVICE_ID: &str = env!("DEVICE_ID");    // e.g. "my-device"
const SHARED_ACCESS_KEY: &str = env!("SHARED_ACCESS_KEY");

#[allow(unused)]
#[no_mangle]
extern "C" fn app_main() {
    link_patches();
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
    // 1. Initialize board (Wi-Fi, etc.)
    let mut timer_svc = EspTimerService::new()?;
    let mut timer = timer_svc.timer()?;

    let mut board = loop {
        match Board::init_board(SSID, PASS).await {
            Ok(board) => break board,
            Err(e) => {
                error!("Error initializing board: {e:?}");
            }
        }
        // Retry every 5 seconds
        timer.after(Duration::from_secs(5)).await?;
    };

    let sensor = &mut board.ds18b20_sensors[0];

    // 2. Generate SAS token (valid for 1 hour)
    let expiry_unix_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() + 3600;

    let sas_token = generate_sas_token(HUB_NAME, DEVICE_ID, SHARED_ACCESS_KEY, expiry_unix_ts);
    info!("SAS token generated. Expires at {expiry_unix_ts}");

    // 3. Create IoTHub client
    let mut iothub = IoTHub::new(HUB_NAME, DEVICE_ID, &sas_token)?;
    info!("IoTHub client created. Connecting...");

    // 4. Run both tasks concurrently:
    run(&mut iothub, &mut timer, sensor).await?;

    Ok(())
}

/// Spawns two tasks via `select(...)`:
///  - Inbound messages listener
///  - Outbound telemetry publisher
async fn run(
    iothub: &mut IoTHub,
    timer: &mut EspAsyncTimer,
    sensor: &mut smart_pot::sensors::DS18B20,
) -> Result<(), EspError> {
    let topic = format!("devices/{}/messages/events/", DEVICE_ID);

    // Construct the two tasks (futures)
    let inbound = inbound_messages_task(&mut iothub.connection);
    let outbound = telemetry_task(&mut iothub, sensor, timer, &topic);

    // Run both concurrently. `select` returns once either finishes/errors.
    let res = select(pin!(inbound), pin!(outbound)).await;

    match res {
        Either::First((Ok(()), _)) => {
            info!("Inbound messages task finished. Outbound task is canceled.");
            Ok(())
        }
        Either::First((Err(e), _)) => {
            error!("Inbound messages task failed: {e:?}");
            Err(e)
        }
        Either::Second((Ok(()), _)) => {
            info!("Outbound telemetry task finished. Inbound task is canceled.");
            Ok(())
        }
        Either::Second((Err(e), _)) => {
            error!("Outbound telemetry task failed: {e:?}");
            Err(e)
        }
    }
}

/// Task #1: Continuously listen for inbound MQTT messages.
async fn inbound_messages_task(
    hub: &mut IoTHub
) -> Result<(), EspError> {
    info!("Starting inbound messages task...");
    while let Ok(event) = hub.connection.next().await {
        match event {
            Event::Received(msg) => {
                info!("[MQTT Inbound] Payload = {}", msg.payload());
            }
            other => {
                info!("[MQTT Event] {:?}", other);
            }
        }
    }

    info!("MQTT connection closed (inbound task)");
    Ok(())
}

/// Task #2: Read sensor data and publish to Azure IoT Hub at intervals
async fn telemetry_task(
    hub: &mut IoTHub,
    sensor: &mut smart_pot::sensors::DS18B20,
    timer: &mut EspAsyncTimer,
    topic: &str,
) -> Result<(), EspError> {
    info!("Starting outbound telemetry task...");

    loop {
        let temp = sensor.read_temperature();
        info!("Sensor reading: {:?}", temp);

        let payload = serde_json::to_string(&data)
        .map_err(|e| {
            error!("JSON serialization error: {:?}", e);
            SmartPotError::ParsingError(e.to_string())
        })?;

        hub.send_message(topic, payload).await?;
        info!("Published temperature={temp} to {topic}");

        timer.after(Duration::from_secs(5)).await?;
    }
}
