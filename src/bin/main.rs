use esp_idf_hal::delay::Ets;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use smart_pot::core::{esp::Board, Result};

static SSID: &str = "kievskaya_hunta";
static PASS: &str = "fourtyeighteen";

#[tokio::main]
async fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut board = loop {
        match Board::init_board(SSID, PASS) {
            Ok(board) => break board,
            Err(e) => {
                log::error!("Error while initialize board:{e:?}")
            }
        }
        Ets::delay_ms(5000);
    };
    let sensor = &mut board.ds18b20_sensors[0];

    loop {
        let temp = sensor.read_temperature();
        log::info!("{temp:?}");

        Ets::delay_ms(600);
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
