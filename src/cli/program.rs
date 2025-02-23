use adb_client::ADBServer;
use std::net::{Ipv4Addr, SocketAddrV4};
use clap::Parser;
use anyhow::Context;
use crate::{adb, balatro};
use crate::cli::clap_cli::{AdbSubcommands, BalatroSubcommands, Cli, Commands};

pub fn program() -> anyhow::Result<()> {
    let argv: Cli = Cli::parse();
    let mut adb_server = ADBServer::default();

    match argv.command {
        Commands::Adb { command } => match command {
            AdbSubcommands::Connect {
                ip_addr,
                port,
                adb_pin,
            } => {
                adb_server
                    .pair(
                        SocketAddrV4::new(
                            Ipv4Addr::new(ip_addr[0], ip_addr[1], ip_addr[2], ip_addr[3]),
                            port,
                        ),
                        adb_pin.to_string(),
                    )
                    .context("Failed to pair with ADB device")?;

                println!("Successfully connected to device");
                println!(
                    "Connected devices: {:#?}",
                    adb_server.devices().context("Failed to list ADB devices")?
                );
            }

            AdbSubcommands::Disconnect => {
                adb::disconnect_all_devices(&mut adb_server)?;
                println!("Successfully disconnected all ADB devices");
            }

            AdbSubcommands::ListDevices => {
                adb::list_devices(&mut adb_server)?;
            }
        },
        Commands::Balatro { command } => match command {
            BalatroSubcommands::Check => {
                if balatro::check_balatro_install(&mut adb_server)?.0 {
                    println!("Balatro is correctly installed on the connected device");
                } else {
                    println!(
                        "Could not find a valid installation of Balatro on the connected device"
                    );
                }
            }
            BalatroSubcommands::Pull { out, all, verbose } => {
                balatro::pull_balatro(&mut adb_server, &out, Some(all), verbose)?;
            }
            BalatroSubcommands::Unpack {
                apk_path,
                out,
                verbose,
            } => {
                let apks_dir = apk_path.unwrap_or_else(|| "balapatch/balatro_apks".to_string());
                let out_dir = out.unwrap_or_else(|| "balapatch/balatro_unpacked".to_string());

                balatro::pull_balatro(&mut adb_server, &Some(apks_dir.clone()), None, verbose)?;

                let apk_file = format!("{}\\base.apk", apks_dir);

                if apk_file.ends_with(".apk") {
                    balatro::unpack_balatro(apk_file.as_str(), &out_dir)?;
                } else {
                    return Err(anyhow::anyhow!("Invalid input file: {}", apk_file))?;
                }
            }
        },
    }

    Ok(())
}