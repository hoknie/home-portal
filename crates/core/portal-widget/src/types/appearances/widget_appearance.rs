use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::tokens::{Accent, Align, Padding, Surface, TitleVisibility, names_of};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidgetAppearance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<Surface>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<Accent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<TitleVisibility>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<Padding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align: Option<Align>,
    #[serde(flatten, default, skip_serializing)]
    pub unknown: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResolvedAppearance {
    pub surface: Surface,
    pub accent: Accent,
    pub title: TitleVisibility,
    pub padding: Padding,
    pub align: Align,
}

impl WidgetAppearance {
    pub fn resolved(&self) -> ResolvedAppearance {
        ResolvedAppearance {
            surface: self.surface.unwrap_or(Surface::Card),
            accent: self.accent.unwrap_or(Accent::Neutral),
            title: self.title.unwrap_or(TitleVisibility::Shown),
            padding: self.padding.unwrap_or(Padding::Normal),
            align: self.align.unwrap_or(Align::Start),
        }
    }

    pub fn of(resolved: ResolvedAppearance) -> WidgetAppearance {
        WidgetAppearance {
            surface: (resolved.surface != Surface::Card).then_some(resolved.surface),
            accent: (resolved.accent != Accent::Neutral).then_some(resolved.accent),
            title: (resolved.title != TitleVisibility::Shown).then_some(resolved.title),
            padding: (resolved.padding != Padding::Normal).then_some(resolved.padding),
            align: (resolved.align != Align::Start).then_some(resolved.align),
            unknown: BTreeMap::new(),
        }
    }

    pub fn is_default(&self) -> bool {
        *self == WidgetAppearance::default()
    }

    pub fn problems(&self) -> Vec<(String, String)> {
        let mut problems = Vec::new();
        let mut check = |key: &str, known: Option<bool>, names: &str| {
            if known == Some(false) {
                problems.push((
                    key.to_string(),
                    format!("must be one of {}", names_of(names)),
                ));
            }
        };
        check("surface", self.surface.map(Surface::known), Surface::NAMES);
        check("accent", self.accent.map(Accent::known), Accent::NAMES);
        check(
            "title",
            self.title.map(TitleVisibility::known),
            TitleVisibility::NAMES,
        );
        check("padding", self.padding.map(Padding::known), Padding::NAMES);
        check("align", self.align.map(Align::known), Align::NAMES);
        for key in self.unknown.keys() {
            problems.push((
                key.clone(),
                "is not a setting of appearance, which takes surface, accent, title, padding and align".to_string(),
            ));
        }
        problems
    }
}
