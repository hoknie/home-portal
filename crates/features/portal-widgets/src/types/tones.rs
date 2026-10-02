use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-open" = "neutral"))]
#[serde(rename_all = "kebab-case")]
pub enum Tone {
    #[default]
    Neutral,
    Ok,
    Info,
    Warning,
    Danger,
    Blue,
    Cyan,
    Teal,
    Green,
    Lime,
    Amber,
    Orange,
    Red,
    Pink,
    Violet,
    Indigo,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    #[default]
    Above,
    Below,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    #[serde(default)]
    pub warning: Option<f64>,
    #[serde(default)]
    pub danger: Option<f64>,
    #[serde(default)]
    pub direction: Direction,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ToneRule {
    #[default]
    Fixed,
    Thresholds(Thresholds),
    Map(BTreeMap<String, Tone>),
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ToneFields {
    #[serde(default)]
    pub tone: Option<Tone>,
    #[serde(default)]
    pub thresholds: Option<Thresholds>,
    #[serde(default)]
    pub tones: Option<BTreeMap<String, Tone>>,
}

impl ToneFields {
    pub const ONE_RULE: &'static str = "give only one of tone, thresholds and tones";

    pub fn rule(&self) -> Result<(Tone, ToneRule), &'static str> {
        match (&self.tone, &self.thresholds, &self.tones) {
            (tone, None, None) => Ok((tone.unwrap_or_default(), ToneRule::Fixed)),
            (None, Some(thresholds), None) => {
                Ok((Tone::Neutral, ToneRule::Thresholds(thresholds.clone())))
            }
            (None, None, Some(tones)) => Ok((Tone::Neutral, ToneRule::Map(tones.clone()))),
            _ => Err(Self::ONE_RULE),
        }
    }
}

impl Thresholds {
    pub fn tone_of(&self, value: f64) -> Tone {
        let past = |limit: Option<f64>| {
            limit.is_some_and(|limit| match self.direction {
                Direction::Above => value >= limit,
                Direction::Below => value <= limit,
            })
        };
        if past(self.danger) {
            Tone::Danger
        } else if past(self.warning) {
            Tone::Warning
        } else {
            Tone::Ok
        }
    }
}
