use adb_client::{ADBDeviceExt, ADBServer, DeviceState, RustADBError};
use anyhow::Context;
use std::{
    fmt::Display,
    fs::File,
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
};

use crate::utils::StringBuf;

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
        DeviceState::Sideload => "In Sideload Mode",
        DeviceState::Rescue => "In Rescue Mode",
    };

    formatted_value.to_string()
}

pub fn get_adb_server(ipv4_addr: Option<[u8; 4]>, port: Option<u16>) -> ADBServer {
    match (ipv4_addr, port) {
        (Some(addr), Some(p)) => ADBServer::new(SocketAddrV4::new(
            Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]),
            p,
        )),
        _ => ADBServer::default(),
    }
}

pub fn check_adb_file_exists(server: &mut ADBServer, file_path: &str) -> anyhow::Result<bool> {
    Ok(server
        .get_device()
        .expect("Could not connect to device")
        .shell_command(&["test", file_path], &mut std::io::stdout())
        .is_ok())
}

pub fn pull_adb_file(
    server: &mut ADBServer,
    file_path: &str,
    target_path: &str,
) -> anyhow::Result<()> {
    let mut device = &mut server
        .get_device()
        .context("Failed to connect to ADB device")?;
    let local_path = Path::new(target_path);

    if let Some(parent_dir) = local_path.parent() {
        std::fs::create_dir_all(parent_dir).context("Failed to create parent directory")?;
    }

    let mut file = File::create(local_path).context("Failed to create local file")?;

    device
        .pull(&target_path, &mut file)
        .context("Failed to pull file from ADB")?;

    Ok(())
}

pub fn install_apk(server: &mut ADBServer, apk_path: &str) -> anyhow::Result<(), RustADBError> {
    let mut device = server.get_device()?;
    let apk_path = Path::new(apk_path);

    if !apk_path.exists() {
        return Err(RustADBError::ADBRequestFailed("fuck".to_string()));
    } else {
        device.install(apk_path)?;
    }

    Ok(())
}

pub fn pull_app_apks(
    server: &mut ADBServer,
    app_id: &str,
    output_dir: &str,
    verbose: bool,
    all: bool,
) -> anyhow::Result<()> {
    let (installed, paths) = check_balatro_install(server)?;

    if !installed {
        return Err(anyhow::anyhow!("Balatro is not currently installed"));
    }

    let mut device = server.get_device()?;

    if server.get_device().is_ok() {
        println!("Found valid connected device");
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
        let mut output_file = std::fs::File::create(&output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;

        let pull_status = device.pull(&path, &mut output_file).with_context(|| {
            format!(
                "Failed to pull APK from {} to {}",
                path,
                output_path.display()
            )
        });

        if pull_status.is_ok() {
            println!("Successfully pulled APK to host device");
            println!("APK =======> {}", filename);
            println!("Dest ======> {}\n", output_path.display());
        }
    }

    Ok(())
}

pub fn disconnect_all_devices(server: &mut ADBServer) -> anyhow::Result<()> {
    server
        .kill()
        .context("Failed to disconnect all ADB devices")?;

    Ok(())
}

pub fn list_devices(server: &mut ADBServer) -> anyhow::Result<()> {
    let devices = server.devices().context("Failed to list ADB devices")?;

    if devices.is_empty() {
        println!("No connected devices found.");
    } else {
        println!("Connected devices:");
        for device in devices {
            println!(
                "Identifier => {}\nState      => {}\n",
                device.identifier.split('.').next().unwrap(),
                format_device_state(&device.state)
            );
        }
    }

    Ok(())
}

pub fn check_balatro_install(server: &mut ADBServer) -> anyhow::Result<(bool, Vec<String>)> {
    let mut output = StringBuf::new();
    let mut device = server
        .get_device()
        .context("Failed to connect to ADB device")?;

    device
        .shell_command(
            &["pm", "path", "com.playstack.balatro.android"],
            &mut output,
        )
        .context("Failed to find Balatro")?;

    let output_str = output.as_string()?;
    let paths: Vec<String> = output_str
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_start_matches("package:").to_string())
        .collect();

    Ok((!paths.is_empty(), paths))
}
