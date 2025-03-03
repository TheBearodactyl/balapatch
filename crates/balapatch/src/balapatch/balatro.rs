use std::path::PathBuf;
use {
    crate::balapatch::{
        adb, apk,
        tui::progress::{self, create_bytes_progress},
        utils::string_buf::StringBuf,
    },
    adb_client::{ADBDeviceExt, ADBServer},
    anyhow::{Context, Error},
    indicatif::{ProgressBar, ProgressStyle},
    rayon::prelude::*,
    std::sync::{Arc, Mutex},
    tracing::info,
};

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

/// Pulls the APK files of the Balatro application from a connected ADB device to a specified output directory.
///
/// # Parameters
///
/// - `adb_server`: A mutable reference to an `ADBServer` instance used to communicate with the ADB device.
/// - `out`: An optional reference to a `String` specifying the output directory where the APKs will be saved.
///   If `None`, defaults to "balapatch/balatro-apks".
/// - `all`: An optional `bool` indicating whether to pull all APK splits. Defaults to `false` if `None`.
/// - `verbose`: A `bool` that, if `true`, enables verbose output during the APK pulling process.
///
/// # Returns
///
/// Returns a `Result` which is:
/// - `Ok(())` if the APKs are successfully pulled or if the Balatro application is not installed.
/// - An `Error` if there is a failure in creating the output directory or pulling the APKs.
pub fn pull_balatro(
    adb_server: &mut ADBServer,
    out: &Option<String>,
    all: Option<bool>,
    verbose: bool,
) -> Result<(), Error> {
    let pb = progress::create_spinner("Checking for Balatro installation...");

    let apks_out = match &out {
        Some(folder) => folder,
        None => "balapatch/balatro-apks",
    };

    pb.set_message("Creating output directory...");
    std::fs::create_dir_all(apks_out).context("Failed to create output directory")?;

    let (installed, paths) = check_balatro_install(adb_server)?;
    pb.finish_and_clear();

    if installed {
        let mp = progress::GLOBAL_MP.clone();
        let main_pb = mp.add(ProgressBar::new(paths.len() as u64));

        main_pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} {bar:40} {pos}/{len}")
                .expect("Progress style error"),
        );

        main_pb.set_message("Pulling APKs...");
        let all = all.unwrap_or(false);

        // Wrap ADBServer in an Arc<Mutex> for thread-safe shared access
        let adb_server = Arc::new(Mutex::new(adb_server));

        // Track whether we've pulled `base.apk` and should stop
        let pulled_base_apk = Arc::new(Mutex::new(false));

        // Parallel APK pulling using Rayon
        let pull_results: Result<(), Error> =
            paths.par_iter().enumerate().try_for_each(|(idx, path)| {
                if !all && *pulled_base_apk.lock().unwrap() {
                    return Ok(()); // Skip if we've already pulled `base.apk` and `all` is false
                }

                let mp = progress::GLOBAL_MP.clone();
                let _file_pb = mp.insert(idx, create_bytes_progress("Pulling APK", 0));

                // Lock the ADBServer for thread-safe access
                let mut adb_server = adb_server.lock().unwrap();

                adb::pull_app_apks(
                    &mut adb_server,
                    "com.playstack.balatro.android",
                    apks_out,
                    verbose,
                    all,
                )?;

                // Check if the pulled APK is `base.apk` and `all` is false
                if !all && path.ends_with("base.apk") {
                    // Set the progress bar to 1 step and mark `base.apk` as pulled
                    main_pb.set_length(1);
                    main_pb.inc(1);
                    *pulled_base_apk.lock().unwrap() = true;
                }

                main_pb.inc(1);
                Ok(())
            });

        pull_results?; // Propagate any errors from parallel execution
        main_pb.finish_with_message("All APKs pulled");
    }

    Ok(())
}

pub async fn unpack_balatro(balatro_path: &str, out_path: &str) -> anyhow::Result<()> {
    let apktool_path = crate::balapatch::apk::apktool::has_apktool()
        .await
        .unwrap_or(PathBuf::from("./balapatch/apktool.jar"));

    if apk::apktool::get_apktool().await.is_ok()
        && crate::balapatch::utils::misc::return_java_install().0
    {
        let output = std::process::Command::new(
            crate::balapatch::utils::misc::return_java_install()
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
            info!("Balatro unpacked successfully!");
        } else {
            info!(
                "Apktool exited with non-zero status: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    Ok(())
}
