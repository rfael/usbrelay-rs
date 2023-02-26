use anyhow::{bail, Context};
use clap::{Parser, Subcommand, ValueEnum};
use usbrelay_rs::usbrelay::{UsbRelayBoard, UsbRelayState};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Set application verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbosity: u8,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List all available relays
    List,
    /// Set relay state
    Set {
        /// Relay serial number
        serial_number: String,
        /// Relay index
        index: u8,
        /// Desired relay state
        state: CommandSetStateValue,
    },
    /// Update relay serial number
    Update {
        /// Relay serial nuber to update
        serial_number: String,
        /// New serial number
        new_serial_number: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CommandSetStateValue {
    On,
    Off,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.verbosity {
        0 => simple_logger::init_with_level(log::Level::Warn),
        1 => simple_logger::init_with_level(log::Level::Info),
        2 => simple_logger::init_with_level(log::Level::Debug),
        _ => simple_logger::init_with_level(log::Level::Trace),
    }?;

    log::info!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    log::debug!("Debug logs are turned on");

    match cli.command {
        Command::List => list_relays(),
        Command::Set {
            serial_number,
            index,
            state,
        } => set_relay_state(&serial_number, index, state.into()),
        Command::Update {
            serial_number,
            new_serial_number,
        } => update_serial_number(&serial_number, &new_serial_number),
    }
}

fn list_relays() -> anyhow::Result<()> {
    let relays = UsbRelayBoard::find_relays()?;
    for r in relays.iter() {
        println!("{r}");
    }
    Ok(())
}

fn set_relay_state(serial_number: &str, index: u8, state: UsbRelayState) -> anyhow::Result<()> {
    log::debug!("Attempt to set {serial_number}:{index} {state}");
    let mut relays = UsbRelayBoard::find_relays()?
        .into_iter()
        .filter(|r| r.serial_number() == serial_number)
        .collect::<Vec<_>>();

    if relays.is_empty() {
        bail!("No such relay")
    }

    if relays.len() > 1 {
        bail!("More than one relay with {serial_number} connected")
    }

    let relay = relays.get_mut(0).context("Available relays list empty")?;
    log::info!("Setting relay {serial_number}:{index} {state}");
    relay.set_state(index, state)?;

    Ok(())
}

impl From<CommandSetStateValue> for UsbRelayState {
    fn from(value: CommandSetStateValue) -> Self {
        match value {
            CommandSetStateValue::On => UsbRelayState::On,
            CommandSetStateValue::Off => UsbRelayState::Off,
        }
    }
}

fn update_serial_number(serial_number: &str, new_serial_number: &str) -> anyhow::Result<()> {
    log::debug!("Attempt to update relay serial number {serial_number} -> {new_serial_number}");

    Ok(())
}
