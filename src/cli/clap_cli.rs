use anyhow::Result as Anyhow;
use clap::builder::TypedValueParser;
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Error, Parser, Subcommand};

#[derive(Clone)]
struct SwitchValueParser;

#[derive(Clone)]
struct Ipv4Parser;

impl TypedValueParser for SwitchValueParser {
    type Value = bool;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        Ok(!value.is_empty())
    }
}

impl TypedValueParser for Ipv4Parser {
    type Value = [u8; 4];

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value_str = value
            .to_str()
            .ok_or_else(|| {
                Error::raw(
                    clap::error::ErrorKind::InvalidUtf8,
                    "Invalid UTF-8 sequence in IP address",
                )
            })
            .expect("Fuck");

        let parts: Vec<&str> = value_str.split('.').collect();
        if parts.len() != 4 {
            return Err(Error::raw(
                clap::error::ErrorKind::InvalidValue,
                "IPv4 address must have exactly 4 octets",
            ));
        }

        let mut octets = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            octets[i] = part
                .parse::<u8>()
                .map_err(|_| {
                    Error::raw(
                        clap::error::ErrorKind::InvalidValue,
                        format!("Invalid octet: {}", part),
                    )
                })
                .expect("fuck");
        }

        Ok(octets)
    }
}

fn switch_value_parser() -> impl TypedValueParser<Value = bool> {
    SwitchValueParser
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// ADB device management commands
    Adb {
        #[command(subcommand)]
        command: AdbSubcommands,
    },

    Balatro {
        #[command(subcommand)]
        command: BalatroSubcommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum BalatroSubcommands {
    /// Check if Balatro is installed on connected device
    CheckBalatro,
}

#[derive(Subcommand, Debug)]
pub enum AdbSubcommands {
    /// Connect to an ADB device
    Connect {
        /// IP address of the device
        #[arg(short = 'i', long, value_parser = Ipv4Parser)]
        ip_addr: [u8; 4],

        /// Port number for ADB connection
        #[arg(short = 'p', long)]
        port: u16,

        /// ADB pairing PIN
        #[arg(short = 'c', long)]
        adb_pin: u32,
    },

    /// Disconnect all ADB devices
    Disconnect,

    /// List all connected ADB devices
    ListDevices,
}
