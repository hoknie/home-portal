use clap::ValueEnum;

pub const RENDER_HELP: &str = "print the Caddy configuration as JSON, without contacting Caddy";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProxyAction {
    #[value(help = RENDER_HELP)]
    Render,
}
