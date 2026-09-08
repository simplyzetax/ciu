use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    wifi::{ClientConfiguration, Configuration, EspWifi},
};

pub const WIFI_CHANNEL: u8 = 2;

pub fn start(modem: Modem<'static>, channel: u8) -> anyhow::Result<EspWifi<'static>> {
    let sysloop = EspSystemEventLoop::take()?;

    let mut wifi = EspWifi::new(modem, sysloop.clone(), None)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    wifi.start()?;

    set_channel(channel)?;

    Ok(wifi)
}

fn set_channel(channel: u8) -> anyhow::Result<()> {
    if !(1..=13).contains(&channel) {
        anyhow::bail!("invalid Wi-Fi channel: {channel}");
    }

    let result = unsafe {
        esp_idf_sys::esp_wifi_set_channel(
            channel,
            esp_idf_sys::wifi_second_chan_t_WIFI_SECOND_CHAN_NONE,
        )
    };

    if result != esp_idf_sys::ESP_OK {
        anyhow::bail!("failed to set Wi-Fi channel: {result}");
    }

    Ok(())
}
