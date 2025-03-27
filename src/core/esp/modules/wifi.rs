mod private {
    use crate::core::Result;
    use heapless::String;
    use std::str::FromStr;

    use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
    use esp_idf_hal::peripheral::Peripheral;
    use esp_idf_svc::{
        eventloop::EspSystemEventLoop,
        nvs::{EspNvsPartition, NvsDefault},
        ping::EspPing,
        timer::{EspTimerService, Task},
        wifi::{AsyncWifi, EspWifi},
    };

    pub async fn wifi(
        ssid: &'static str,
        password: &'static str,
        modem: impl Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
        sysloop: &EspSystemEventLoop,
        nvs: Option<EspNvsPartition<NvsDefault>>,
        timer_service: &EspTimerService<Task>,
    ) -> Result<AsyncWifi<EspWifi<'static>>> {
        let mut wifi = AsyncWifi::wrap(
            EspWifi::new(modem, sysloop.clone(), nvs)?,
            sysloop.clone(),
            timer_service.clone(),
        )?;

        connect(&mut wifi, ssid, password).await?;

        let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

        log::info!("Wifi DHCP info: {:?}", ip_info);

        EspPing::default().ping(
            ip_info.subnet.gateway,
            &esp_idf_svc::ping::Configuration::default(),
        )?;
        Ok(wifi)
    }

    async fn connect(
        wifi: &mut AsyncWifi<EspWifi<'static>>,
        ssid: &'static str,
        password: &'static str,
    ) -> Result<()> {
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid: String::<32>::from_str(ssid).expect("Invalid wifi SSID"),
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: String::from_str(password).expect("Invalid wifi password"),
            channel: None,
            pmf_cfg: esp_idf_svc::wifi::PmfConfiguration::NotCapable,
            scan_method: esp_idf_svc::wifi::ScanMethod::FastScan,
        });

        wifi.set_configuration(&wifi_configuration)?;

        wifi.start().await?;
        wifi.connect().await?;
        wifi.wait_netif_up().await?;

        Ok(())
    }
}

crate::mod_interface! {
    orphan use {
        wifi
    };
}
