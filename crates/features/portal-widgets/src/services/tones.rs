use serde_json::Value;

use crate::types::{Tone, ToneFields, ToneRule};

pub fn tone_for(fields: &ToneFields, rendered: &str) -> Tone {
    match fields.rule() {
        Ok((tone, ToneRule::Fixed)) => tone,
        Ok((_, ToneRule::Thresholds(thresholds))) => {
            leading_number(rendered).map_or(Tone::Neutral, |number| thresholds.tone_of(number))
        }
        Ok((_, ToneRule::Map(tones))) => tones.get(rendered.trim()).copied().unwrap_or_default(),
        Err(_) => Tone::Neutral,
    }
}

pub fn number_in(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => leading_number(text),
        _ => None,
    }
}

pub fn leading_number(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    let end = trimmed
        .char_indices()
        .take_while(|(index, character)| {
            character.is_ascii_digit()
                || *character == '.'
                || (*index == 0 && matches!(character, '-' | '+'))
        })
        .map(|(index, character)| index + character.len_utf8())
        .last()?;
    trimmed[..end].parse().ok()
}
