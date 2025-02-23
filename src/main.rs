#![allow(unused)]

mod adb;
mod cli;
mod utils;
mod zipalign;

use std::{
    fs::{File, read_to_string},
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
};

use adb::get_adb_server;
use adb_client::ADBServer;
use anyhow::Context;
use clap::Parser;
use cli::{
    clap_cli::{AdbSubcommands, BalatroSubcommands, Cli, Commands},
    prompts::select_file::select_path_from_current_dir,
};

fn main() -> anyhow::Result<()> {
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
                if adb::check_balatro_install(&mut adb_server)?.0 {
                    println!("Balatro is correctly installed on the connected device");
                } else {
                    println!(
                        "Could not find a valid installation of Balatro on the connected device"
                    );
                }
            }
            BalatroSubcommands::Pull { out, all, verbose } => {
                let apks_out = match &out {
                    Some(folder) => folder,
                    None => "balatro-apks",
                };
                std::fs::create_dir_all(apks_out)?;

                if adb::check_balatro_install(&mut adb_server)?.0 {
                    adb::pull_app_apks(
                        &mut adb_server,
                        "com.playstack.balatro.android",
                        apks_out,
                        verbose,
                        all,
                    )?;
                    println!("Successfully pulled Balatro APKs to {}", apks_out);
                } else {
                    println!(
                        "Could not find a valid installation of Balatro on the connected device"
                    );
                }
            }
        },
    }

    Ok(())
}
