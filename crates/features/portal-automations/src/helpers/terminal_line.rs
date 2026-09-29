const ESCAPE: char = '\u{1b}';
const BELL: char = '\u{7}';

pub fn last_line(text: &str) -> Option<String> {
    without_escapes(text)
        .replace("\r\n", "\n")
        .split('\n')
        .map(overwritten)
        .rfind(|line| !line.trim().is_empty())
        .map(|line| line.trim_end().to_string())
}

fn overwritten(line: &str) -> String {
    let mut screen: Vec<char> = Vec::new();
    for segment in line.split('\r') {
        let written: Vec<char> = segment.chars().collect();
        let kept = screen.get(written.len()..).unwrap_or_default().to_vec();
        screen = written.into_iter().chain(kept).collect();
    }
    screen.into_iter().collect()
}

fn without_escapes(text: &str) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut kept = String::with_capacity(text.len());
    let mut index = 0;
    while index < characters.len() {
        match escape_length(&characters[index..]) {
            Some(length) => index += length,
            None => {
                kept.push(characters[index]);
                index += 1;
            }
        }
    }
    kept
}

fn escape_length(characters: &[char]) -> Option<usize> {
    if characters.first() != Some(&ESCAPE) {
        return None;
    }
    let next = *characters.get(1)?;
    let operating = (next == ']')
        .then(|| operating_command_length(characters))
        .flatten();
    let sequence = (next == '[')
        .then(|| control_sequence_length(characters))
        .flatten();
    operating
        .or(sequence)
        .or_else(|| (('@'..='Z').contains(&next) || ('\\'..='_').contains(&next)).then_some(2))
}

fn operating_command_length(characters: &[char]) -> Option<usize> {
    let mut index = 2;
    while let Some(character) = characters.get(index) {
        match *character {
            BELL => return Some(index + 1),
            ESCAPE => return (characters.get(index + 1) == Some(&'\\')).then_some(index + 2),
            _ => index += 1,
        }
    }
    None
}

fn control_sequence_length(characters: &[char]) -> Option<usize> {
    let mut index = 2;
    while characters
        .get(index)
        .is_some_and(|character| ('0'..='?').contains(character))
    {
        index += 1;
    }
    while characters
        .get(index)
        .is_some_and(|character| (' '..='/').contains(character))
    {
        index += 1;
    }
    characters
        .get(index)
        .is_some_and(|character| ('@'..='~').contains(character))
        .then_some(index + 1)
}
