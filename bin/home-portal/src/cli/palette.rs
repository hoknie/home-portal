use std::fmt::Write;

use clap::builder::styling::{AnsiColor, Style};
use clap::builder::{StyledStr, Styles};

pub const HEADER: Style = AnsiColor::Green.on_default().bold();
pub const LITERAL: Style = AnsiColor::Cyan.on_default().bold();
pub const PLACEHOLDER: Style = Style::new().dimmed();
pub const ERROR: Style = AnsiColor::Red.on_default().bold();
pub const SUCCESS: Style = AnsiColor::Green.on_default().bold();
pub const WARNING: Style = AnsiColor::Yellow.on_default().bold();
pub const ALERT: Style = AnsiColor::Magenta.on_default().bold();
pub const EMPHASIS: Style = Style::new().bold();
pub const MUTED: Style = Style::new().dimmed();

pub fn palette() -> Styles {
    Styles::styled()
        .header(HEADER)
        .usage(HEADER)
        .literal(LITERAL)
        .placeholder(PLACEHOLDER)
        .error(ERROR)
        .valid(SUCCESS)
        .invalid(WARNING)
}

pub fn examples(heading: &str, lines: &[(&str, &str)]) -> StyledStr {
    let width = lines
        .iter()
        .map(|(example, _)| example.chars().count())
        .max()
        .unwrap_or(0);
    let mut text = StyledStr::new();
    let _ = write!(text, "{HEADER}{heading}:{HEADER:#}");
    for (example, meaning) in lines {
        let _ = write!(text, "\n  {LITERAL}{example:<width$}{LITERAL:#}  {meaning}");
    }
    text
}

pub fn sections(parts: &[StyledStr]) -> StyledStr {
    let mut text = StyledStr::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            let _ = write!(text, "\n\n");
        }
        let _ = write!(text, "{}", part.ansi());
    }
    text
}
