use crate::balapatch::apk::zipalign::ZipAlign;
use crate::balapatch::tui::progress::{create_spinner, GLOBAL_MP};
use crate::balapatch::tui::select_file::select_path_from_current_dir;
use crate::balapatch::{adb, balatro};
use adb_client::ADBServer;
use balapatch_derive::EnumChoice;
use indicatif::ProgressBar;
use inquire::error::InquireResult;
use inquire::ui::{Attributes, Color, RenderConfig, Styled};
use inquire::validator::{StringValidator, Validation};
use inquire::{CustomUserError, InquireError, MultiSelect, Select, Text};
use std::clone::Clone;
use std::fmt::{Debug, Display, Formatter};

trait Variants<T: 'static> {
    const VARIANTS: &'static [T];
}

#[derive(Debug, Copy, Clone, EnumChoice)]
#[allow(clippy::upper_case_acronyms)]
enum BalapatchCommands {
    ADB,
    Balatro,
}

#[derive(Debug, Copy, Clone, EnumChoice)]
enum AdbCommands {
    Connect,
    Disconnect,
    List,
    Check,
}

#[derive(Debug, Copy, Clone, EnumChoice)]
enum BalatroCommands {
    Check,
    Validate,
    Pull,
    Unpack,
    Mod,
}

impl Variants<BalapatchCommands> for BalapatchCommands {
    const VARIANTS: &'static [BalapatchCommands] = &[Self::ADB, Self::Balatro];
}

impl Variants<AdbCommands> for AdbCommands {
    const VARIANTS: &'static [AdbCommands] =
        &[Self::Connect, Self::Disconnect, Self::List, Self::Check];
}

impl Variants<BalatroCommands> for BalatroCommands {
    const VARIANTS: &'static [BalatroCommands] = &[
        Self::Check,
        Self::Pull,
        Self::Unpack,
        Self::Validate,
        Self::Mod,
    ];
}

fn enum_choice<E: Display + Debug + Copy + Clone + Variants<E> + 'static>(
    msg: &str,
) -> InquireResult<E> {
    let answer: E = Select::new(msg, E::VARIANTS.to_vec()).prompt()?;

    Ok(answer)
}

fn balapatch_inquire_style() -> RenderConfig<'static> {
    let style: Styled<String> = Styled::default()
        .with_fg(Color::LightMagenta)
        .with_attr(Attributes::BOLD);

    let render_cfg = RenderConfig::default()
        .with_unselected_checkbox(style.clone().with_content("{ }"))
        .with_selected_checkbox(style.clone().with_content("{X}"));
    // .with_prompt_prefix(style.clone().with_content("===>"))
    // .with_answered_prompt_prefix(
    //     style
    //         .clone()
    //         .with_bg(Color::LightGreen)
    //         .with_fg(Color::LightGreen)
    //         .with_content(">>"),
    // )
    // .with_highlighted_option_prefix(style.clone().with_content(">"));

    render_cfg
}

pub async fn balapatch() -> InquireResult<()> {
    inquire::set_global_render_config(balapatch_inquire_style());
    let mut adb_server = ADBServer::default();
    let init_actions = enum_choice::<BalapatchCommands>("Choose an action:")?;

    match init_actions {
        BalapatchCommands::ADB => {
            let adb_actions = enum_choice::<AdbCommands>("Available ADB actions:")?;

            match adb_actions {
                AdbCommands::Connect => {
                    crate::balapatch::tui::mode_select::choose_connection_mode()?;
                }
                AdbCommands::Disconnect => {
                    balatro_adb_disconnect(&mut adb_server)?;
                }
                AdbCommands::List => {
                    balatro_adb_list(&mut adb_server)?;
                }
                AdbCommands::Check => {
                    if adb_server.get_device().is_ok() {
                        println!("Found a valid device!");
                    } else {
                        println!("Couldn't find a valid device");
                    }
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
                BalatroCommands::Mod => {
                    // TODO: Actually implement the patcher with lovely :3
                    balatro_unpack(adb_server).await?;
                }
                BalatroCommands::Validate => {
                    balatro_validate(adb_server).await?;
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

    let (custom_apk_path, change_output_dir, verbose): (bool, bool, bool) = {
        (
            opts.iter().any(|s| *s == "APK Path"),
            opts.iter().any(|s| *s == "Output Directory"),
            opts.iter().any(|s| *s == "Verbose"),
        )
    };

    let spinner = create_spinner("Preparing to unpack Balatro...");

    let apk_path = if custom_apk_path {
        Text::new("Please input the path to the Balatro APK:").prompt()?
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

#[derive(Clone)]
struct AlignmentValidator;

impl StringValidator for AlignmentValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        if input.chars().all(|a| a.is_ascii_digit()) {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("Fuck".into()))
        }
    }
}

pub async fn balatro_validate(mut adb_server: ADBServer) -> Result<(), InquireError> {
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

    let spinner = create_spinner("Preparing to validate Balatro...");
    let apk_path = if change_apk_path {
        Text::new("Please input the path to the Balatro APK:").prompt()?
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
    let alignment = Text::new("Alignment:")
        .with_validator(AlignmentValidator)
        .prompt()?;
    let zipalign = ZipAlign::new(
        apk_file.into(),
        Some(out_dir.into()),
        alignment.parse::<u64>().expect("Failed to parse alignment"),
    );

    if zipalign.verify_zip(verbose).is_ok() {
        spinner.finish_with_message("APK is aligned correctly");
    } else {
        spinner.finish_with_message("APK is not aligned correctly");
    }

    Ok(())
}
