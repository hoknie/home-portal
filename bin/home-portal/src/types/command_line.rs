use clap::Parser;
use clap::builder::StyledStr;
use portal_config::CONFIGURATION_VARIABLE;
use portal_web::WEB_VARIABLE;

use super::command::Command;
use crate::boot::ADDRESS_VARIABLE;
use crate::cli::{examples, palette, sections};

pub const NAME: &str = "home-portal";
pub const ABOUT: &str = "The status of your home services, and the place to manage them";
pub const CONFIGURATION_MEANING: &str =
    "the configuration file [default: ~/.config/home-portal/home-portal.toml]";
pub const ADDRESS_MEANING: &str = "ip:port to listen on, instead of [network]";
pub const WEB_MEANING: &str = "the interface folder, instead of web/ beside the binary";

#[derive(Debug, Parser)]
#[command(
    name = NAME,
    version,
    about = ABOUT,
    styles = palette(),
    after_help = CommandLine::examples(),
    subcommand_value_name = "COMMAND"
)]
pub struct CommandLine {
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl CommandLine {
    pub fn examples() -> StyledStr {
        sections(&[
            examples(
                "Examples",
                &[
                    ("home-portal", "start the portal"),
                    ("home-portal password-hash", "hash a password for [[users]]"),
                    ("home-portal probe media", "probe the service `media` once"),
                    (
                        "home-portal probe tcp://nas.lan:22 --kind tcp",
                        "check that a port accepts connections",
                    ),
                    (
                        "home-portal proxy render > caddy.json",
                        "print the Caddy configuration",
                    ),
                ],
            ),
            examples(
                "Environment",
                &[
                    (CONFIGURATION_VARIABLE, CONFIGURATION_MEANING),
                    (ADDRESS_VARIABLE, ADDRESS_MEANING),
                    (WEB_VARIABLE, WEB_MEANING),
                ],
            ),
        ])
    }
}
