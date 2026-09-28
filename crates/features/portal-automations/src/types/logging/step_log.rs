use super::Rendered;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StepLog {
    pub values: Vec<Rendered>,
    pub lines: Vec<String>,
    pub values_dropped: usize,
    pub lines_dropped: usize,
}

impl StepLog {
    pub const MOST_VALUES: usize = 20;
    pub const MOST_LINES: usize = 20;
    pub const LONGEST: usize = 300;
    pub const MOST_BYTES_PER_RUN: usize = 256 * 1024;

    pub fn push_value(&mut self, template: &str, value: &str) {
        if self.values.len() >= Self::MOST_VALUES {
            self.values_dropped += 1;
            return;
        }
        self.values.push(Rendered {
            template: Self::shortened(template, Self::LONGEST),
            value: Self::shortened(value, Self::LONGEST),
        });
    }

    pub fn push_line(&mut self, line: impl Into<String>) {
        if self.lines.len() >= Self::MOST_LINES {
            self.lines_dropped += 1;
            return;
        }
        self.lines
            .push(Self::shortened(&line.into(), Self::LONGEST));
    }

    pub fn extend_lines(&mut self, lines: Vec<String>) {
        for line in lines {
            self.push_line(line);
        }
    }

    pub fn bytes(&self) -> usize {
        self.values
            .iter()
            .map(|value| value.template.len() + value.value.len())
            .chain(self.lines.iter().map(String::len))
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty() && self.lines.is_empty()
    }

    pub fn masked(self, mask: impl Fn(&str) -> String) -> StepLog {
        StepLog {
            values: self
                .values
                .into_iter()
                .map(|value| Rendered {
                    template: mask(&value.template),
                    value: mask(&value.value),
                })
                .collect(),
            lines: self.lines.iter().map(|line| mask(line)).collect(),
            ..self
        }
    }

    pub fn dropped(self) -> StepLog {
        StepLog {
            values: Vec::new(),
            lines: Vec::new(),
            values_dropped: self.values_dropped + self.values.len(),
            lines_dropped: self.lines_dropped + self.lines.len(),
        }
    }

    pub fn shortened(text: &str, longest: usize) -> String {
        if text.len() <= longest {
            return text.to_string();
        }
        let mut end = longest;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &text[..end])
    }
}
