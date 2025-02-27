use crate::balapatch::adb::{ConnectionMode, get_adb_connection};
use crate::balapatch::tui::progress::create_spinner;
use crate::balapatch::tui::adb_wireless_input::adb_wireless_input;
use adb_client::ADBServer;
use anyhow::{Context, Error};
use std::net::{Ipv4Addr, SocketAddrV4};
use tracing::info;

pub fn adb_connect_wireless() -> Result<(), Error> {
    let info = adb_wireless_input()?;

    let spinner = create_spinner("Establishing wireless connection...");
    let connection = get_adb_connection(ConnectionMode::Wireless((info.0.addr, info.0.port)))?;

    match connection {
        conn if conn.is_left() => {
            let mut server: ADBServer = conn.get_left().unwrap();
            server
                .pair(
                    SocketAddrV4::new(
                        Ipv4Addr::new(
                            info.0.addr[0],
                            info.0.addr[1],
                            info.0.addr[2],
                            info.0.addr[3],
                        ),
                        info.0.port,
                    ),
                    info.1,
                )
                .context("Failed to pair with ADB device")?;

            spinner.finish_with_message("Wireless connection established");
            info!(
                "Successfully connected to wireless ADB device:\n=====> {}",
                server.get_device()?.identifier
            );
        }
        _ => unreachable!(),
    }
    Ok(())
}