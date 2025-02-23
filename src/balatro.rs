use adb_client::{ADBDeviceExt, ADBServer};
use anyhow::{Context, Error};
use crate::{adb, apk};
use crate::utils::StringBuf;

/// Checks if the Balatro application is installed on the connected ADB device and retrieves its APK paths.
///
/// # Parameters
///
/// - `server`: A mutable reference to an `ADBServer` instance, which is used to communicate with the ADB device.
///
/// # Returns
///
/// Returns an `anyhow::Result` containing a tuple:
/// - A boolean indicating whether the Balatro application is installed.
/// - A vector of strings, each representing a path to an APK file of the Balatro application.
///
/// If the function fails to connect to the device or execute the shell command, it returns an error.
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

pub fn pull_balatro(mut adb_server: &mut ADBServer, out: &Option<String>, all: Option<bool>, verbose: bool) -> Result<(), Error> {
    let apks_out = match &out {
        Some(folder) => folder,
        None => "balapatch/balatro-apks",
    };

    std::fs::create_dir_all(apks_out)?;

    if check_balatro_install(&mut adb_server)?.0 {
        adb::pull_app_apks(
            &mut adb_server,
            "com.playstack.balatro.android",
            apks_out,
            verbose,
            all.unwrap_or(false),
        )?;
        println!("Successfully pulled Balatro APKs to {}", apks_out);
    } else {
        println!(
            "Could not find a valid installation of Balatro on the connected device"
        );
    }
    Ok(())
}

pub fn unpack_balatro(balatro_path: &str, out_path: &str) -> anyhow::Result<()> {
    println!(
        "{} -jar apktool.jar d {} -r -o {}",
        crate::utils::return_java_install()
            .1
            .unwrap()
            .join("bin")
            .join("java.exe")
            .to_str()
            .unwrap(),
        balatro_path,
        out_path
    );

    if apk::get_apktool().is_ok() && crate::utils::return_java_install().0 {
        let output = std::process::Command::new(
            crate::utils::return_java_install()
                .1
                .unwrap()
                .join("bin")
                .join("java.exe"),
        )
        .arg("-jar")
        .arg("balapatch/apktool.jar")
        .arg("d")
        .arg(balatro_path)
        .arg("-r")
        .arg("-o")
        .arg(out_path)
        .output()
        .expect("Failed to execute Apktool");

        if output.status.success() {
            println!("Balatro unpacked successfully!");
        } else {
            println!(
                "Apktool exited with non-zero status: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    Ok(())
}