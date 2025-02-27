use crate::balapatch::tui::progress::{create_spinner, GLOBAL_MP};
use crate::balapatch::tui::select_file::select_path_from_current_dir;
use crate::balapatch::{adb, balatro};
use adb_client::ADBServer;
use balapatch_derive::EnumChoice;
use indicatif::ProgressBar;
use inquire::error::InquireResult;
use inquire::{InquireError, MultiSelect, Select};
use std::clone::Clone;
use std::fmt::{Debug, Display, Formatter};

trait Variants<T: 'static> {
    const VARIANTS: &'static [T];
}

#[derive(Debug, Copy, Clone, EnumChoice)]
enum BalapatchCommands {
    Adb,
    Balatro,
}

#[derive(Debug, Copy, Clone, EnumChoice)]
enum AdbCommands {
    Connect,
    Disconnect,
    ListDevices,
}

#[derive(Debug, Copy, Clone, EnumChoice)]
enum BalatroCommands {
    Check,
    Pull,
    Unpack,
}

impl Variants<BalapatchCommands> for BalapatchCommands {
    const VARIANTS: &'static [BalapatchCommands] = &[Self::Adb, Self::Balatro];
}

impl Variants<AdbCommands> for AdbCommands {
    const VARIANTS: &'static [AdbCommands] = &[Self::Connect, Self::Disconnect, Self::ListDevices];
}

impl Variants<BalatroCommands> for BalatroCommands {
    const VARIANTS: &'static [BalatroCommands] = &[Self::Check, Self::Pull, Self::Unpack];
}

fn enum_choice<E: Display + Debug + Copy + Clone + Variants<E> + 'static>(
    msg: &str,
) -> InquireResult<E> {
    let answer: E = Select::new(msg, E::VARIANTS.to_vec()).prompt()?;

    Ok(answer)
}

pub async fn balapatch() -> InquireResult<()> {
    let mut adb_server = ADBServer::default();
    let init_actions = enum_choice::<BalapatchCommands>("Choose an action:")?;

    match init_actions {
        BalapatchCommands::Adb => {
            let adb_actions = enum_choice::<AdbCommands>("Available ADB actions:")?;

            match adb_actions {
                AdbCommands::Connect => {
                    crate::balapatch::tui::mode_select::choose_connection_mode()?;
                }
                AdbCommands::Disconnect => {
                    balatro_adb_disconnect(&mut adb_server)?;
                }
                AdbCommands::ListDevices => {
                    balatro_adb_list(&mut adb_server)?;
                }
            }
        }
        BalapatchCommands::Balatro => {
            let balatro_actions = enum_choice::<BalatroCommands>("You can do the following:")?;

            match balatro_actions {
                BalatroCommands::Check => {
                    balatro_check(&mut adb_server)?;
                }
                BalatroCommands::Pull => {
                    balatro_pull(&mut adb_server)?;
                }
                BalatroCommands::Unpack => {
                    balatro_unpack(adb_server).await?;
                }
            }
        }
    }

    Ok(())
}

pub fn balatro_adb_list(adb_server: &mut ADBServer) -> InquireResult<()> {
    let spinner = create_spinner("Listing devices...");
    adb::list_devices(adb_server).expect("Fuck");
    spinner.finish_with_message("Listed all connected devices");

    Ok(())
}

pub fn balatro_adb_disconnect(adb_server: &mut ADBServer) -> InquireResult<()> {
    let spinner = create_spinner("Disconnecting devices...");
    adb::disconnect_all_devices(adb_server).expect("Failed to disconnect devices...");
    spinner.finish_with_message("All devices disconnected");

    Ok(())
}

pub fn balatro_check(adb_server: &mut ADBServer) -> InquireResult<()> {
    let spinner = create_spinner("Checking Balatro installation...");
    if balatro::check_balatro_install(adb_server)
        .expect("Failed to check balatro install")
        .0
    {
        spinner.finish_with_message("Found a Balatro install on the target device :3");
    } else {
        spinner.finish_with_message("Could not find a valid Balatro installation :(");
    }

    Ok(())
}

pub fn balatro_pull(adb_server: &mut ADBServer) -> Result<(), InquireError> {
    let pull_opts = vec!["Verbose", "Pull All", "Change Out Directory"];

    let opts = MultiSelect::new(
        "Please select any custom options for pulling Balatro:",
        pull_opts.clone(),
    )
    .prompt()?;

    let (verbose, pull_all, change_out_directory): (bool, bool, bool) = {
        (
            opts.iter().any(|s| *s == "Verbose"),
            opts.iter().any(|s| *s == "Pull All"),
            opts.iter().any(|s| *s == "Change Out Directory"),
        )
    };

    let out_dir = if change_out_directory {
        select_path_from_current_dir("Please select a directory...")?
    } else {
        "balapatch/balatro_apks".to_string()
    };

    let spinner = create_spinner("Pulling Balatro APKs...");
    balatro::pull_balatro(adb_server, &Some(out_dir), Some(pull_all), verbose).expect("Fuck");
    spinner.finish_with_message("Finished pulling the APKs...");
    Ok(())
}

pub async fn balatro_unpack(mut adb_server: ADBServer) -> Result<(), InquireError> {
    let unpack_opts = vec!["APK Path", "Output Directory", "Verbose"];
    let opts = MultiSelect::new(
        "Please select any custom options for unpacking:",
        unpack_opts.clone(),
    )
    .prompt()?;

    let (change_apk_path, change_output_dir, verbose): (bool, bool, bool) = {
        (
            opts.iter().any(|s| *s == "APK Path"),
            opts.iter().any(|s| *s == "Output Directory"),
            opts.iter().any(|s| *s == "Verbose"),
        )
    };

    let spinner = create_spinner("Preparing to unpack Balatro...");

    let apk_path = if change_apk_path {
        select_path_from_current_dir("Please select the path that contains the pulled `base.apk`")?
    } else {
        "balapatch/balatro_apks".to_string()
    };

    let out_dir = if change_output_dir {
        select_path_from_current_dir("Please select a directory...")?
    } else {
        "balapatch/balatro_unpacked".to_string()
    };

    balatro::pull_balatro(&mut adb_server, &Some(apk_path.clone()), None, verbose)
        .expect("Failed to pull");

    let apk_file = format!("{}\\base.apk", apk_path);
    if apk_file.ends_with(".apk") {
        spinner.finish_with_message("Starting unpack process...");
        let unpack_pb = GLOBAL_MP.add(ProgressBar::new_spinner());
        unpack_pb.set_message("Unpacking Balatro...");
        balatro::unpack_balatro(apk_file.as_str(), &out_dir)
            .await
            .expect("Failed to unpack");
        unpack_pb.finish_with_message("Unpacking complete!");
    } else {
        spinner.finish_with_message("Invalid input file");
    }
    Ok(())
}
