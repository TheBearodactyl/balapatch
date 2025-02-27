use anyhow::anyhow;
use inquire::error::InquireResult;
use inquire::validator::{StringValidator, Validation};
use inquire::{CustomType, CustomUserError, Text};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Ipv4Port {
    pub addr: [u8; 4],
    pub port: u16,
}

impl Display for Ipv4Port {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}:{}",
            self.addr[0], self.addr[1], self.addr[2], self.addr[3], self.port
        )
    }
}

impl FromStr for Ipv4Port {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.find(':').is_some() {
            let colon_pos = s.find(':').expect("fuck.");
            let ip_str = &s[0..colon_pos];
            let port_str = &s[colon_pos + 1..];
            let ip_parts: Vec<&str> = ip_str.split('.').collect();

            if ip_parts.len() != 4 {
                return Err(anyhow!("Invalid IP address"));
            }

            for part in &ip_parts {
                if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
                    return Err(anyhow!("Invalid IP address"));
                }
            }

            let mut addr: [u8; 4] = [0, 0, 0, 0];

            for idx in 0..4 {
                if let Ok(num) = ip_parts[idx].parse::<u8>() {
                    addr[idx] = num;
                } else {
                    return Err(anyhow!("Invalid IP address"));
                }
            }

            // This doesn't work for some reason...?
            // if let Ok(port_num) = port_str.parse::<u32>() {
            //     if !(1..=65535).contains(&port_num) {
            //         return Err(anyhow!("Invalid port number"));
            //     }
            // } else {
            //     return Err(anyhow!("Invalid port format"));
            // }
            let port_num = port_str.parse::<u16>();

            if port_num.clone().is_ok() && !(1..=65535).contains(&port_num.clone()?) {
                return Err(anyhow!("Invalid port number"));
            }

            if port_num.is_err() {
                return Err(anyhow!("Invalid port format"));
            }

            Ok(Ipv4Port {
                addr,
                port: port_num?,
            })
        } else {
            Err(anyhow!("Missing port"))
        }
    }
}

// fn is_valid_ip(ip_str: &str) -> bool {
//     let parts = ip_str.split('.').collect::<Vec<&str>>();
//     if parts.len() != 4 {
//         return false;
//     }
// 
//     for part in parts {
//         if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
//             return false;
//         }
// 
//         match part.parse::<u32>() {
//             Ok(num) => {
//                 if !(0..=255).contains(&num) {
//                     return false;
//                 }
//             }
//             Err(_) => return false,
//         }
//     }
// 
//     true
// }
// 
// fn is_valid_port(port_str: &str) -> bool {
//     match port_str.parse::<u32>() {
//         Ok(port) => (1..=65535).contains(&port),
//         Err(_) => false,
//     }
// }

#[derive(Clone)]
struct PinValidator;

impl StringValidator for PinValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        if input.chars().all(|a| a.is_ascii_digit()) {
            if input.len() == 6 {
                Ok(Validation::Valid)
            } else {
                Ok(Validation::Invalid("Fuck".into()))
            }
        } else {
            Ok(Validation::Invalid("Fuck".into()))
        }
    }
}

pub fn adb_wireless_input() -> InquireResult<(Ipv4Port, String)> {
    let ipv4_in = CustomType::<Ipv4Port>::new(
        "Please input a valid IPv4 address along with a port number:\n",
    )
    .with_placeholder("127.0.0.1:42069")
    .with_formatter(&|i| format!("{i}"))
    .with_error_message("Not a valid IPv4 address")
    .with_help_message("Example: 127.0.0.1:42069")
    .prompt()?;

    let pin = Text::new("Please input the ADB pin:")
        .with_validator(PinValidator)
        .prompt()?;

    Ok((ipv4_in, pin))
}
