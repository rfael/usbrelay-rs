use std::fmt;

use anyhow::{bail, Context};
use enum_repr::EnumRepr;
use hidapi_rusb::{HidApi, HidDevice};

const USB_RELAY_VID: u16 = 0x16c0;
const USB_RELAY_PID: u16 = 0x05df;

const SERIAL_NUMBER_SIZE: usize = 5;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UsbRelayState {
    On,
    Off,
}

pub struct UsbRelayBoard {
    hid_device: HidDevice,
    serial_number: String,
    relay_states: Vec<UsbRelayState>,
}

#[EnumRepr(type = "u8")]
enum UsbRelayCommand {
    ReadFeatures = 0x01,
    SetSerialNumber = 0xfa,
    TurnOff = 0xfd,
    TurnOn = 0xff,
}

impl UsbRelayBoard {
    pub fn find_relays() -> anyhow::Result<Vec<Self>> {
        let hid_api = HidApi::new()?;

        let relays_info = hid_api
            .device_list()
            .filter(|d| d.vendor_id() == USB_RELAY_VID && d.product_id() == USB_RELAY_PID);

        let mut usb_relays = Vec::new();

        for relay_info in relays_info {
            let product = relay_info
                .product_string()
                .context("Can not read product string")?;

            if !product.starts_with("USBRelay") {
                bail!("Product {product} unsupported")
            }

            let relay_count = product.trim_start_matches("USBRelay");
            let relay_count = relay_count.parse::<usize>()?;
            if relay_count > 8 {
                bail!("Up to 8 relays supported");
            }

            let hid_device = relay_info.open_device(&hid_api)?;
            let (serial_number, states) = Self::read_features(&hid_device)?;

            let mut relay_states = Vec::new();
            for index in 0..relay_count {
                let relay_state = states & (0x01 << index);
                let relay_state = if relay_state > 0 {
                    UsbRelayState::On
                } else {
                    UsbRelayState::Off
                };
                relay_states.push(relay_state);
            }

            let usb_relay = Self {
                hid_device,
                serial_number,
                relay_states,
            };

            usb_relays.push(usb_relay);
        }

        Ok(usb_relays)
    }

    pub fn serial_number(&self) -> &str {
        &self.serial_number
    }

    fn read_features(hid_device: &HidDevice) -> anyhow::Result<(String, u8)> {
        let mut buf = [0u8; 9];

        buf[0] = UsbRelayCommand::ReadFeatures as u8;
        let rb = hid_device.get_feature_report(&mut buf)?;

        if rb != buf.len() {
            bail!("Can not read feature report")
        }

        let aux = Vec::from(&buf[0..SERIAL_NUMBER_SIZE]);
        let serial_number = String::from_utf8(aux)?;
        let states = buf[7];

        log::debug!("Relay {serial_number} state: {states:08b}");

        Ok((serial_number, states))
    }

    pub fn set_state(&mut self, relay_index: u8, state: UsbRelayState) -> anyhow::Result<()> {
        if relay_index as usize >= self.relay_states.len() {
            bail!(
                "Invalid relay index {relay_index}, board {} has only {} relays",
                self.serial_number,
                self.relay_states.len()
            )
        }

        let mut buf = [0u8; 9];
        let command = match state {
            UsbRelayState::On => UsbRelayCommand::TurnOn,
            UsbRelayState::Off => UsbRelayCommand::TurnOff,
        };
        buf[1] = command as u8;
        buf[2] = relay_index + 1;

        let wb = self.hid_device.write(&buf)?;
        if wb != buf.len() {
            bail!("Not all bytes has been written to relay")
        }

        self.relay_states[relay_index as usize] = state;

        Ok(())
    }
}

impl fmt::Display for UsbRelayBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serial_number)?;

        for (index, state) in self.relay_states.iter().enumerate() {
            write!(f, " {index}:{state:?}")?;
        }

        Ok(())
    }
}

impl fmt::Display for UsbRelayState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = match self {
            UsbRelayState::On => "on",
            UsbRelayState::Off => "off",
        };

        f.write_str(state)
    }
}
