use super::argument_line::argument_of;
use super::{HeaderProblem, ScriptArgument};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptHeader {
    pub description: Option<String>,
    pub arguments: Vec<ScriptArgument>,
    pub problems: Vec<HeaderProblem>,
}

impl ScriptHeader {
    pub const MOST_LINES: usize = 64;
    pub const MOST_BYTES: usize = 8 * 1024;
    pub const MOST_ARGUMENTS: usize = 32;
    const DESCRIPTION: &'static str = "@description";
    const ARGUMENT: &'static str = "@arg";

    pub fn parse(text: &str) -> ScriptHeader {
        let mut header = ScriptHeader::default();
        for (index, line) in Self::bounded(text)
            .lines()
            .take(Self::MOST_LINES)
            .enumerate()
        {
            let number = index + 1;
            if number == 1 && line.starts_with("#!") {
                continue;
            }
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                continue;
            }
            let Some(comment) = Self::comment(trimmed) else {
                break;
            };
            header.read(number, comment.trim());
        }
        header
    }

    fn bounded(text: &str) -> &str {
        if text.len() <= Self::MOST_BYTES {
            return text;
        }
        let mut end = Self::MOST_BYTES;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        &text[..end]
    }

    fn comment(line: &str) -> Option<&str> {
        line.strip_prefix("//")
            .or_else(|| line.strip_prefix('#'))
            .map(|rest| rest.trim_start_matches(['#', '/']))
    }

    fn tagged<'a>(line: &'a str, tag: &str) -> Option<&'a str> {
        let rest = line.strip_prefix(tag)?;
        (rest.is_empty() || rest.starts_with(char::is_whitespace)).then(|| rest.trim())
    }

    fn read(&mut self, line: usize, comment: &str) {
        if let Some(text) = Self::tagged(comment, Self::DESCRIPTION) {
            if self.description.is_some() {
                self.problem(line, "is a second @description; only the first one counts");
            } else if text.is_empty() {
                self.problem(line, "is an @description without text");
            } else {
                self.description = Some(text.to_string());
            }
            return;
        }
        let Some(declaration) = Self::tagged(comment, Self::ARGUMENT) else {
            return;
        };
        match argument_of(declaration) {
            Ok(argument) => self.add(line, argument),
            Err(message) => self.problem(line, &format!("declares an argument that {message}")),
        }
    }

    fn add(&mut self, line: usize, argument: ScriptArgument) {
        if self.arguments.len() >= Self::MOST_ARGUMENTS {
            self.problem(
                line,
                &format!("declares more than {} arguments", Self::MOST_ARGUMENTS),
            );
            return;
        }
        if self
            .arguments
            .iter()
            .any(|known| known.name == argument.name)
        {
            self.problem(line, &format!("declares {} a second time", argument.name));
            return;
        }
        let optional_before = self
            .arguments
            .iter()
            .any(|known| !known.option && !known.required);
        if !argument.option && argument.required && optional_before {
            self.problem(
                line,
                &format!(
                    "declares the required {} after an optional positional argument",
                    argument.name
                ),
            );
            return;
        }
        self.arguments.push(argument);
    }

    fn problem(&mut self, line: usize, message: &str) {
        self.problems.push(HeaderProblem {
            line,
            message: format!("line {line} {message}"),
        });
    }
}
