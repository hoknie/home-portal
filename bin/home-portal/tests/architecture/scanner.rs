pub struct Scanned {
    pub code: String,
    pub comment_lines: Vec<usize>,
}

pub fn scan(source: &str) -> Scanned {
    let characters: Vec<char> = source.chars().collect();
    let mut scanner = Scanner {
        characters,
        position: 0,
        line: 1,
        code: String::with_capacity(source.len()),
        comment_lines: Vec::new(),
    };
    scanner.run();
    Scanned {
        code: scanner.code,
        comment_lines: scanner.comment_lines,
    }
}

struct Scanner {
    characters: Vec<char>,
    position: usize,
    line: usize,
    code: String,
    comment_lines: Vec<usize>,
}

impl Scanner {
    fn run(&mut self) {
        while let Some(current) = self.peek(0) {
            match current {
                '/' if self.peek(1) == Some('/') => self.line_comment(),
                '/' if self.peek(1) == Some('*') => self.block_comment(),
                '"' => self.quoted(),
                'r' | 'b' | 'c' if self.at_word_start() && self.raw_prefix().is_some() => {
                    let hashes = self.raw_prefix().unwrap_or_default();
                    self.raw(hashes);
                }
                '\'' => self.quote_or_lifetime(),
                _ => self.keep(),
            }
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.characters.get(self.position + offset).copied()
    }

    fn at_word_start(&self) -> bool {
        self.position == 0
            || !self.characters[self.position - 1].is_alphanumeric()
                && self.characters[self.position - 1] != '_'
    }

    fn raw_prefix(&self) -> Option<usize> {
        let mut offset = match (self.peek(0), self.peek(1)) {
            (Some('r'), _) => 1,
            (Some('b' | 'c'), Some('r')) => 2,
            _ => return None,
        };
        let mut hashes = 0;
        while self.peek(offset) == Some('#') {
            hashes += 1;
            offset += 1;
        }
        (self.peek(offset) == Some('"')).then_some(hashes)
    }

    fn keep(&mut self) {
        let current = self.characters[self.position];
        self.code.push(current);
        if current == '\n' {
            self.line += 1;
        }
        self.position += 1;
    }

    fn blank(&mut self) {
        let current = self.characters[self.position];
        if current == '\n' {
            self.code.push('\n');
            self.line += 1;
        } else {
            self.code.push(' ');
        }
        self.position += 1;
    }

    fn line_comment(&mut self) {
        self.comment_lines.push(self.line);
        while self.peek(0).is_some_and(|current| current != '\n') {
            self.blank();
        }
    }

    fn block_comment(&mut self) {
        self.comment_lines.push(self.line);
        self.blank();
        self.blank();
        let mut depth = 1;
        while depth > 0 && self.peek(0).is_some() {
            if self.peek(0) == Some('/') && self.peek(1) == Some('*') {
                depth += 1;
                self.blank();
            } else if self.peek(0) == Some('*') && self.peek(1) == Some('/') {
                depth -= 1;
                self.blank();
            }
            self.blank();
        }
    }

    fn quoted(&mut self) {
        self.keep();
        while let Some(current) = self.peek(0) {
            if current == '\\' {
                self.blank();
                if self.peek(0).is_some() {
                    self.blank();
                }
            } else if current == '"' {
                self.keep();
                return;
            } else {
                self.blank();
            }
        }
    }

    fn raw(&mut self, hashes: usize) {
        while self.peek(0) != Some('"') {
            self.keep();
        }
        self.keep();
        while self.peek(0).is_some() {
            if self.peek(0) == Some('"')
                && (1..=hashes).all(|offset| self.peek(offset) == Some('#'))
            {
                for _ in 0..=hashes {
                    self.keep();
                }
                return;
            }
            self.blank();
        }
    }

    fn quote_or_lifetime(&mut self) {
        let escaped = self.peek(1) == Some('\\');
        let single = self.peek(2) == Some('\'');
        if !escaped && !single {
            self.keep();
            return;
        }
        self.keep();
        while let Some(current) = self.peek(0) {
            if current == '\\' {
                self.blank();
                self.blank();
            } else if current == '\'' {
                self.keep();
                return;
            } else {
                self.blank();
            }
        }
    }
}

#[test]
fn a_url_inside_a_string_is_not_a_comment() {
    assert!(
        scan(r#"let a = "http://127.0.0.1:8080";"#)
            .comment_lines
            .is_empty()
    );
}

#[test]
fn a_raw_string_holding_slashes_is_not_a_comment() {
    assert!(
        scan(r###"let a = r#"// not a comment "quoted""#;"###)
            .comment_lines
            .is_empty()
    );
    assert!(scan(r###"let a = br"//";"###).comment_lines.is_empty());
}

#[test]
fn a_slash_character_and_a_lifetime_are_not_comments() {
    assert!(
        scan("let a = '/'; let b = '\\''; fn f<'a>(x: &'a str) {}")
            .comment_lines
            .is_empty()
    );
}

#[test]
fn an_escaped_quote_does_not_end_a_string() {
    assert!(
        scan(r#"let a = "say \"//\" now";"#)
            .comment_lines
            .is_empty()
    );
}

#[test]
fn line_doc_and_block_comments_are_found_on_their_lines() {
    let source = "fn a() {}\n/// doc\n//! inner\nlet b = 1; // tail\n/* block */\n";
    assert_eq!(scan(source).comment_lines, vec![2, 3, 4, 5]);
}

#[test]
fn string_contents_are_blanked_in_the_code_view() {
    let scanned = scan(r##"let a = "#[allow(x)]";"##);
    assert!(!scanned.code.contains("allow"));
}
