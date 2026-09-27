use clap::Subcommand;
use clap::builder::StyledStr;

use super::probe_kind_choice::ProbeKindChoice;
use super::proxy_action::ProxyAction;
use crate::cli::examples;

pub const SERVE_ABOUT: &str = "Start the portal (what runs when no command is given)";
pub const PASSWORD_HASH_ABOUT: &str = "Print an argon2id hash for a [[users]] entry";
pub const PASSWORD_HASH_LONG_ABOUT: &str = "Print an argon2id hash for a [[users]] entry.\n\nOn a terminal the password is asked for without echo; otherwise the first line of standard input is read. Only the hash is written to standard output.";
pub const PROBE_ABOUT: &str = "Probe a service or URL once and explain the result";
pub const PROBE_LONG_ABOUT: &str = "Probe a service or URL once, with the same code as the portal's loop, and explain the result.\n\nExits 0 when the target is up or degraded, and 1 otherwise.";
pub const TARGET_HELP: &str = "A service id from the configuration, or a URL";
pub const KIND_HELP: &str = "How to probe [default: the service's setting, or http for a URL]";
pub const PROXY_ABOUT: &str = "Work with the Caddy reverse proxy";
pub const ACTION_HELP: &str = "What to do with the proxy";

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    #[command(about = SERVE_ABOUT)]
    Serve,
    #[command(
        about = PASSWORD_HASH_ABOUT,
        long_about = PASSWORD_HASH_LONG_ABOUT,
        after_help = Command::password_hash_examples()
    )]
    PasswordHash,
    #[command(
        about = PROBE_ABOUT,
        long_about = PROBE_LONG_ABOUT,
        after_help = Command::probe_examples()
    )]
    Probe {
        #[arg(value_name = "SERVICE|URL", help = TARGET_HELP)]
        target: String,
        #[arg(short, long, value_enum, value_name = "KIND", help = KIND_HELP)]
        kind: Option<ProbeKindChoice>,
    },
    #[command(about = PROXY_ABOUT, after_help = Command::proxy_examples())]
    Proxy {
        #[arg(value_enum, value_name = "ACTION", help = ACTION_HELP)]
        action: ProxyAction,
    },
}

impl Command {
    pub fn password_hash_examples() -> StyledStr {
        examples(
            "Examples",
            &[
                (
                    "home-portal password-hash",
                    "type the password, get the hash",
                ),
                (
                    "printf 'secret\\n' | home-portal password-hash",
                    "hash a password from a script",
                ),
            ],
        )
    }

    pub fn probe_examples() -> StyledStr {
        examples(
            "Examples",
            &[
                ("home-portal probe media", "the service with id `media`"),
                ("home-portal probe http://nas.lan:5000", "any URL over HTTP"),
                (
                    "home-portal probe tcp://nas.lan:22 --kind tcp",
                    "whether a port accepts connections",
                ),
                (
                    "home-portal probe icmp://router.lan -k icmp",
                    "whether a host answers ping",
                ),
            ],
        )
    }

    pub fn proxy_examples() -> StyledStr {
        examples(
            "Examples",
            &[
                ("home-portal proxy render", "print the configuration"),
                (
                    "home-portal proxy render > caddy.json",
                    "save it for `caddy run --config caddy.json`",
                ),
            ],
        )
    }
}
