use inquire::Select;
use inquire::error::InquireResult;
use std::fmt::{Display, Formatter};
use tracing::{error, info};

#[derive(Debug, Copy, Clone)]
pub enum ConnectMode {
    Wireless,
    Wired,
}

impl ConnectMode {
    const VARIANTS: &'static [ConnectMode] = &[Self::Wireless, Self::Wired];
}

impl Display for ConnectMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub fn choose_connection_mode() -> InquireResult<ConnectMode> {
    let ans: ConnectMode =
        Select::new("Connection Mode:", ConnectMode::VARIANTS.to_vec()).prompt()?;

    match ans {
        ConnectMode::Wired => {
            info!("Sadly, wired connection doesn't work at the moment :(");
            info!("Defaulting to wireless!");
            crate::balapatch::cli::subcommands::adb_connect::adb_connect_wireless().expect("fuck");

            Ok(ConnectMode::Wireless)
        }
        ConnectMode::Wireless => {
            crate::balapatch::cli::subcommands::adb_connect::adb_connect_wireless().expect("fuck");

            Ok(ConnectMode::Wireless)
        }
    }
}
