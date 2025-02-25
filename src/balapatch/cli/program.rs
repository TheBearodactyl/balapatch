use crate::balapatch::adb::{get_adb_connection, ConnectionMode};
use crate::balapatch::cli::subcommands::adb_connect;
use crate::balapatch::cli::{AdbSubcommands, BalatroSubcommands, Cli, Commands};
use crate::balapatch::tui::progress::{create_spinner, GLOBAL_MP};
use adb_client::{ADBServer, ADBUSBDevice};
use anyhow::Context;
use clap::Parser;
use clap::builder::Str;
use indicatif::ProgressBar;
use std::fmt::format;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::process::Command;
use tracing::{debug, error, info};
use crate::balapatch::{adb, balatro};

pub async fn program() -> anyhow::Result<()> {
	let argv: Cli = Cli::parse();
	let mut adb_server = ADBServer::default();

	match argv.command {
		Commands::Adb { command } => match command {
			AdbSubcommands::Connect => {
				crate::balapatch::tui::mode_select::choose_connection_mode().expect("fuck");
			},

			AdbSubcommands::Disconnect => {
				let spinner = create_spinner("Disconnecting devices...");
				adb::disconnect_all_devices(&mut adb_server)?;
				spinner.finish_with_message("All devices disconnected");
				info!("Successfully disconnected all ADB devices");
			}

			AdbSubcommands::ListDevices => {
				let spinner = create_spinner("Listing devices...");
				adb::list_devices(&mut adb_server)?;
				spinner.finish_with_message("Devices listed");
			}
		},
		Commands::Balatro { command } => match command {
			BalatroSubcommands::Check => {
				let spinner = create_spinner("Checking Balatro installation...");
				if balatro::check_balatro_install(&mut adb_server)?.0 {
					spinner.finish_with_message("Balatro is installed");
					info!("Balatro is correctly installed on the connected device");
				} else {
					spinner.finish_with_message("Balatro not found");
					error!(
                        "Could not find a valid installation of Balatro on the connected device"
                    );
				}
			}
			BalatroSubcommands::Pull { out, all, verbose } => {
				let spinner = create_spinner("Pulling Balatro APKs...");
				balatro::pull_balatro(adb_server, &out, Some(all), verbose)?;
				spinner.finish_with_message("APKs pulled successfully");
			}
			BalatroSubcommands::Unpack {
				apk_path,
				out,
				verbose,
			} => {
				let spinner = create_spinner("Preparing to unpack Balatro...");
				let apks_dir = apk_path.unwrap_or_else(|| "balapatch/balatro_apks".to_string());
				let out_dir = out.unwrap_or_else(|| "balapatch/balatro_unpacked".to_string());

				balatro::pull_balatro(adb_server, &Some(apks_dir.clone()), None, verbose)?;

				let apk_file = format!("{}\\base.apk", apks_dir);

				if apk_file.ends_with(".apk") {
					spinner.finish_with_message("Starting unpacking process");
					let unpack_pb = GLOBAL_MP.add(ProgressBar::new_spinner());
					unpack_pb.set_message("Unpacking APK...");
					balatro::unpack_balatro(apk_file.as_str(), &out_dir).await?;
					unpack_pb.finish_with_message("Unpacking complete");
					info!("Balatro APK successfully unpacked to {}", out_dir);
				} else {
					spinner.finish_with_message("Invalid input file");
					error!("Invalid input file: {}", apk_file);
					Err(anyhow::anyhow!("Invalid input file: {}", apk_file))?;
				}
			}
		},
	}

	Ok(())
}
