use clap::{Error, Parser, Subcommand};
use clap::builder::TypedValueParser;
use clap::error::ErrorKind;

pub mod program;
pub mod subcommands;

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
    ) -> Result<Self::Value, Error> {
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
    ) -> Result<Self::Value, Error> {
        let value_str = value
            .to_str()
            .ok_or_else(|| {
                Error::raw(
                    ErrorKind::InvalidUtf8,
                    "Invalid UTF-8 sequence in IP address",
                )
            })
            .expect("Fuck");

        let parts: Vec<&str> = value_str.split('.').collect();
        if parts.len() != 4 {
            return Err(Error::raw(
                ErrorKind::InvalidValue,
                "IPv4 address must have exactly 4 octets",
            ));
        }

        let mut octets = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            octets[i] = part
                .parse::<u8>()
                .map_err(|_| {
                    Error::raw(ErrorKind::InvalidValue, format!("Invalid octet: {}", part))
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
    Check,

    /// Pulls the apks associated with the installation of
    /// Balatro to the CLI specified out directory
    Pull {
        /// An optional output directory for the apks
        /// (defaults to `balatro-apks`)
        #[arg(short = 'o', long)]
        out: Option<String>,

        /// Pulls all apks instead of just `base.apk`
        #[arg(short = 'a', long)]
        all: bool,

        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
    },

    Unpack {
        /// Path to the apk file
        #[arg(short = 'a', long)]
        apk_path: Option<String>,

        /// Output directory for unpacked files
        #[arg(short = 'o', long)]
        out: Option<String>,

        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum AdbSubcommands {
    /// Connect to an ADB device
    Connect,

    /// Disconnect all ADB devices
    Disconnect,

    /// List all connected ADB devices
    ListDevices,
}