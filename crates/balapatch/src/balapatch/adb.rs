use crate::balapatch::balatro;
use crate::balapatch::utils::misc::Either;
use adb_client::{ADBServer, ADBUSBDevice, DeviceState};
use anyhow::{Context, Result, anyhow};
use std::{
    fs::File,
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
};
use tracing::info;

fn format_device_state(state: &DeviceState) -> String {
    let formatted_value: &str = match state {
        DeviceState::Offline => "Offline",
        DeviceState::Device => "Device",
        DeviceState::NoDevice => "No Device",
        DeviceState::Authorizing => "Authorizing",
        DeviceState::Unauthorized => "Unauthorized",
        DeviceState::Connecting => "Connecting",
        DeviceState::NoPerm => "No Permission",
        DeviceState::Detached => "Detached",
        DeviceState::Bootloader => "In Bootloader",
        DeviceState::Host => "Host",
        DeviceState::Recovery => "In Recovery Mode",
        DeviceState::Sideload => "In Sideloading Mode",
        DeviceState::Rescue => "In Rescue Mode",
    };

    formatted_value.to_string()
}

pub type WirelessIp = ([u8; 4], u16);
pub type WiredIds = (u16, u16);

#[allow(unused)]
#[derive(Debug, PartialEq)]
pub enum ConnectionMode {
    Wireless(WirelessIp),
    Wired(WiredIds),
}

pub fn get_adb_connection(connection: ConnectionMode) -> Result<Either<ADBServer, ADBUSBDevice>> {
    match connection {
        ConnectionMode::Wireless(ip_addr) => {
            let server = ADBServer::new(SocketAddrV4::new(
                Ipv4Addr::new(ip_addr.0[0], ip_addr.0[1], ip_addr.0[2], ip_addr.0[3]),
                ip_addr.1,
            ));
            Ok(Either::new_left(server))
        }
        ConnectionMode::Wired(device_ids) => {
            if device_ids == (0, 0) {
                return Err(anyhow!("No connected USB devices found"));
            }

            let usb_device = ADBUSBDevice::new(device_ids.0, device_ids.1)
                .context("Failed to create USB device connection")?;

            Ok(Either::new_right(usb_device))
        }
    }
}

// pub fn check_adb_file_exists(server: &mut ADBServer, file_path: &str) -> Result<bool> {
//     Ok(server
//         .get_device()?
//         .shell_command(&["test", file_path], &mut std::io::stdout())
//         .is_ok())
// }

// pub fn pull_adb_file(server: &mut ADBServer, file_path: &str, target_path: &str) -> Result<()> {
//     let mut device = &mut server
//         .get_device()
//         .context("Failed to connect to ADB device")?;
//     let local_path = Path::new(target_path);
//
//     if let Some(parent_dir) = local_path.parent() {
//         std::fs::create_dir_all(parent_dir).context("Failed to create parent directory")?;
//     }
//
//     let mut file = File::create(local_path).context("Failed to create local file")?;
//
//     device
//         .pull(&target_path, &mut file)
//         .context("Failed to pull file from ADB")?;
//
//     Ok(())
// }

pub fn pull_app_apks(
    server: &mut ADBServer,
    _app_id: &str,
    output_dir: &str,
    _verbose: bool,
    all: bool,
) -> Result<()> {
    let (installed, paths) = balatro::check_balatro_install(server)?;

    if !installed {
        return Err(anyhow::anyhow!("Balatro is not currently installed"));
    }

    let mut device = server.get_device()?;

    if server.get_device().is_ok() {
        info!("Found valid connected device");
    }

    for path in paths {
        let filename = Path::new(&path)
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename in path: {}", path))?;

        if !all && !filename.eq("base.apk") {
            break;
        }

        let output_path = Path::new(output_dir).join(filename);
        let mut output_file = File::create(&output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;

        let pull_status = device.pull(&path, &mut output_file).with_context(|| {
            format!(
                "Failed to pull APK from {} to {}",
                path,
                output_path.display()
            )
        });

        if pull_status.is_ok() {
            info!("Successfully pulled APK to host device");
            info!("APK =======> {}", filename);
            info!("Dest ======> {}\n", output_path.display());
        }
    }

    Ok(())
}

pub fn disconnect_all_devices(server: &mut ADBServer) -> Result<()> {
    server
        .kill()
        .context("Failed to disconnect all ADB devices")?;

    Ok(())
}

pub fn list_devices(server: &mut ADBServer) -> Result<()> {
    let devices = server.devices().context("Failed to list ADB devices")?;

    if devices.is_empty() {
        info!("No connected devices found.");
    } else {
        println!("Connected devices:");
        for device in devices {
            info!(
                "Identifier => {}\nState      => {}\n",
                device.identifier.split('.').next().unwrap(),
                format_device_state(&device.state)
            );
        }
    }

    Ok(())
}
