use hidapi_rusb::HidApi;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let api = HidApi::new()?;

    let (vid, pid) = (0x16c0, 0x05df);
    let device = api.open(vid, pid)?;
    log::info!("Device {vid:x}:{pid:x} opened");

    Ok(())
}
