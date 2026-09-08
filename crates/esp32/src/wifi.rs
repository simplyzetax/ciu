use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    sys,
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

pub fn mac_address() -> anyhow::Result<[u8; 6]> {
    let mut mac = [0u8; 6];

    let result =
        unsafe { sys::esp_wifi_get_mac(sys::wifi_interface_t_WIFI_IF_STA, mac.as_mut_ptr()) };

    if result != sys::ESP_OK {
        anyhow::bail!("failed to get Wi-Fi MAC address: {result}");
    }

    Ok(mac)
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
