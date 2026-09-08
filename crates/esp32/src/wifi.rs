use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    sys,
    wifi::{ClientConfiguration, Configuration, EspWifi, WifiDeviceId},
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

pub fn station_mac(wifi: &EspWifi<'_>) -> anyhow::Result<[u8; 6]> {
    Ok(wifi.get_mac(WifiDeviceId::Sta)?)
}

fn set_channel(channel: u8) -> anyhow::Result<()> {
    if !(1..=13).contains(&channel) {
        anyhow::bail!("invalid Wi-Fi channel: {channel}");
    }

    let result = unsafe {
        sys::esp_wifi_set_channel(channel, sys::wifi_second_chan_t_WIFI_SECOND_CHAN_NONE)
    };

    if result != sys::ESP_OK {
        anyhow::bail!("failed to set Wi-Fi channel: {result}");
    }

    Ok(())
}
