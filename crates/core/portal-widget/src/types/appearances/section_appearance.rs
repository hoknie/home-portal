use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::tokens::{SectionSurface, TitleVisibility, names_of};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionAppearance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<TitleVisibility>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface: Option<SectionSurface>,
    #[serde(flatten, default, skip_serializing)]
    pub unknown: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResolvedSectionAppearance {
    pub title: TitleVisibility,
    pub surface: SectionSurface,
}

impl SectionAppearance {
    pub fn resolved(&self) -> ResolvedSectionAppearance {
        ResolvedSectionAppearance {
            title: self.title.unwrap_or(TitleVisibility::Shown),
            surface: self.surface.unwrap_or(SectionSurface::NoSurface),
        }
    }

    pub fn of(resolved: ResolvedSectionAppearance) -> SectionAppearance {
        SectionAppearance {
            title: (resolved.title != TitleVisibility::Shown).then_some(resolved.title),
            surface: (resolved.surface != SectionSurface::NoSurface).then_some(resolved.surface),
            unknown: BTreeMap::new(),
        }
    }

    pub fn is_default(&self) -> bool {
        *self == SectionAppearance::default()
    }

    pub fn problems(&self) -> Vec<(String, String)> {
        let mut problems = Vec::new();
        if self.title.is_some_and(|title| !title.known()) {
            problems.push((
                "title".to_string(),
                format!("must be one of {}", names_of(TitleVisibility::NAMES)),
            ));
        }
        if self.surface.is_some_and(|surface| !surface.known()) {
            problems.push((
                "surface".to_string(),
                format!("must be one of {}", names_of(SectionSurface::NAMES)),
            ));
        }
        for key in self.unknown.keys() {
            problems.push((
                key.clone(),
                "is not a setting of a section's appearance, which takes title and surface"
                    .to_string(),
            ));
        }
        problems
    }
}
