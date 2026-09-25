pub const OPEN: &str = "{{";
pub const CLOSE: &str = "}}";

pub fn placeholders_of(template: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find(OPEN) {
        let after = &rest[start + OPEN.len()..];
        match placeholder_at(after) {
            Some(name) => {
                found.push(name);
                rest = &after[name.len() + CLOSE.len()..];
            }
            None => rest = &rest[start + 1..],
        }
    }
    found
}

pub fn render<'a>(template: &str, lookup: impl Fn(&str) -> Option<&'a str>) -> String {
    let mut rendered = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find(OPEN) {
        let after = &rest[start + OPEN.len()..];
        match placeholder_at(after).and_then(|name| lookup(name).map(|value| (name, value))) {
            Some((name, value)) => {
                rendered.push_str(&rest[..start]);
                rendered.push_str(value);
                rest = &after[name.len() + CLOSE.len()..];
            }
            None => {
                rendered.push_str(&rest[..start + 1]);
                rest = &rest[start + 1..];
            }
        }
    }
    rendered.push_str(rest);
    rendered
}

pub fn unknown_placeholders(template: &str, allowed: &[&str]) -> Vec<String> {
    placeholders_of(template)
        .into_iter()
        .filter(|name| !allowed.contains(name))
        .map(str::to_string)
        .collect()
}

fn placeholder_at(text: &str) -> Option<&str> {
    let end = text.find(CLOSE)?;
    let name = &text[..end];
    let valid = !name.is_empty()
        && name.split('.').all(|part| {
            part.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        });
    valid.then_some(name)
}
