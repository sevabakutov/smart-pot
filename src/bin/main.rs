// main.rs

use core::pin::pin;
use core::time::Duration;

use log::*;

use smart_pot::core::error::SmartPotError;
use smart_pot::core::esp::board::Board;
use smart_pot::core::azure::IoTHub;
use smart_pot::core::esp::Sensor;

use embassy_futures::select::{select, Either};

use esp_idf_svc::hal::task;
use esp_idf_svc::sys::link_patches;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::timer::{EspTimerService, EspAsyncTimer};

use esp_idf_hal::gpio::AnyIOPin;

use std::sync::Arc;
use embassy_sync::mutex::Mutex;
use embedded_svc::mqtt::client::Event;

use smart_pot::core::{
    Result,
    azure::generate_sas_token,
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
    let timer_svc = EspTimerService::new()?;
    let mut timer = timer_svc.timer_async()?;

    let board = loop {
        match Board::init_board(SSID, PASS).await {
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
        .as_secs() + 3600;

    let sas_token = generate_sas_token(HUB_NAME, DEVICE_ID, SHARED_ACCESS_KEY, expiry_unix_ts);
    info!("SAS token generated. Expires at {expiry_unix_ts}");

    // 3. Create IoTHub client
    let iothub = Arc::new(Mutex::new(IoTHub::new(HUB_NAME, DEVICE_ID, &sas_token)?));
    info!("IoTHub client created. Connecting...");

    // 4. Run both tasks concurrently:
    run(iothub, &mut timer, &board.sensors).await?;

    Ok(())
}

/// Spawns two tasks via `select(...)`:
///  - Inbound messages listener
///  - Outbound telemetry publisher
async fn run(
    iothub: Arc<Mutex<IoTHub>>,
    timer: &mut EspAsyncTimer,
    sensors: &[Box<dyn Sensor<Pin=AnyIOPin>>],
) -> Result<()> {
    let topic = format!("devices/{}/messages/events/", DEVICE_ID);

    // Construct the two tasks (futures)
    let inbound = inbound_messages_task(iothub.clone());
    let outbound = telemetry_task(iothub.clone(), sensors, timer, &topic);

    // Run both concurrently. `select` returns once either finishes/errors.
    // TODO: use join instead of select.
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

/// Task #1: Continuously listen for inbound MQTT messages.
async fn inbound_messages_task(
    hub: Arc<Mutex<IoTHub>>
) -> Result<()> {
    info!("Starting inbound messages task...");

    while let Ok(event) = hub.connection.next().await {
        match event {
            Event::Received(msg) => {
                info!("[MQTT Inbound] Payload = {}", msg.payload());
            }
            other => {}
        }
    }

    info!("MQTT connection closed (inbound task)");
    Ok(())
}

/// Task #2: Read sensor data and publish to Azure IoT Hub at intervals
async fn telemetry_task(
    hub: Arc<Mutex<IoTHub>>,
    sensors: &[Box<dyn Sensor<Pin=AnyIOPin>>],
    timer: &mut EspAsyncTimer,
    topic: &str,
) -> Result<()> {
    info!("Starting outbound telemetry task...");

    loop {
        for (index, sensor) in sensors.iter().enumerate() {
            let sensor_data = sensor.read_temperature().map_err(|e| {
                error!("Sensor #{} read error: {:?}", index, e);
                e
            })?;
            info!("Sensor #{} => {:?}", index, sensor_data);

            let payload = serde_json::to_string(&sensor_data).map_err(|e| {
                error!("JSON serialization error: {:?}", e);
                SmartPotError::ParsingError(e.to_string())
            })?;

            {
                let mut locked_hub = hub.lock().await;
                locked_hub.send_message(topic, &payload).await?;
            }

            info!("Published sensor #{} data to {topic}", index);
        }

        timer.after(Duration::from_secs(5)).await?;
    }
}
