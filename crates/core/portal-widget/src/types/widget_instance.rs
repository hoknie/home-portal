use portal_model::Environment;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::{WidgetAppearance, WidgetHeight, WidgetSize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetInstance {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row: Option<i64>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default = "WidgetInstance::empty_settings")]
    pub settings: Value,
    #[serde(default)]
    pub environments: Option<Vec<String>>,
    #[serde(default)]
    pub public: bool,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<WidgetSize>,
    #[serde(default)]
    pub height: WidgetHeight,
    #[serde(default)]
    pub appearance: WidgetAppearance,
}

impl WidgetInstance {
    pub const COLUMNS: u8 = 12;
    pub const WIDTH_RULE: &'static str = "must be a whole number of columns from 1 to 12";
    pub const LAST_ROW: i64 = 200;
    pub const DEFINITION_KEYS: [&'static str; 7] = [
        "type",
        "id",
        "title",
        "settings",
        "environments",
        "public",
        "appearance",
    ];
    pub const PLACEMENT_KEYS: [&'static str; 8] = [
        "widget", "section", "column", "row", "width", "height", "size", "key",
    ];

    pub fn placed(widget: &str, section: &str) -> WidgetInstance {
        WidgetInstance {
            widget: Some(widget.to_string()),
            section: Some(section.to_string()),
            ..WidgetInstance::of("")
        }
    }

    pub fn is_inline(&self) -> bool {
        self.widget.is_none()
    }

    pub fn placed_from(&self, definition: &WidgetInstance) -> WidgetInstance {
        WidgetInstance {
            kind: definition.kind.clone(),
            id: definition.id.clone(),
            title: definition.title.clone(),
            settings: definition.settings.clone(),
            environments: definition.environments.clone(),
            public: definition.public,
            appearance: definition.appearance.clone(),
            ..self.clone()
        }
    }

    pub fn position(&self) -> Option<(u8, u32)> {
        match (self.column, self.row) {
            (Some(column), Some(row)) => {
                Some((u8::try_from(column).ok()?, u32::try_from(row).ok()?))
            }
            _ => None,
        }
    }

    pub fn empty_settings() -> Value {
        Value::Object(Map::new())
    }

    pub fn of(kind: &str) -> WidgetInstance {
        WidgetInstance {
            kind: kind.to_string(),
            widget: None,
            column: None,
            row: None,
            id: None,
            title: None,
            settings: Self::empty_settings(),
            environments: None,
            public: false,
            section: None,
            width: None,
            size: None,
            height: WidgetHeight::Auto,
            appearance: WidgetAppearance::default(),
        }
    }

    pub fn columns(&self) -> u8 {
        match (self.width, self.size) {
            (Some(width), _) => u8::try_from(width)
                .ok()
                .filter(|columns| (1..=Self::COLUMNS).contains(columns))
                .unwrap_or(Self::COLUMNS),
            (None, Some(size)) => size.columns(),
            (None, None) => Self::COLUMNS,
        }
    }

    pub fn layout_problems(&self) -> Vec<(String, String)> {
        let mut problems = Vec::new();
        if self.width.is_some() && self.size.is_some() {
            problems.push((
                "width".to_string(),
                "cannot be given together with size; keep width, which size only abbreviates"
                    .to_string(),
            ));
        }
        if let Some(width) = self.width
            && !(1..=i64::from(Self::COLUMNS)).contains(&width)
        {
            problems.push(("width".to_string(), Self::WIDTH_RULE.to_string()));
        }
        if self.size == Some(WidgetSize::Unknown) {
            problems.push((
                "size".to_string(),
                format!("must be one of {}", WidgetSize::NAMES),
            ));
        }
        match (self.column, self.row) {
            (None, None) => {}
            (Some(column), Some(row)) => {
                if !(1..=i64::from(Self::COLUMNS)).contains(&column) {
                    problems.push((
                        "column".to_string(),
                        "must be a column from 1 to 12".to_string(),
                    ));
                } else if column + i64::from(self.columns()) - 1 > i64::from(Self::COLUMNS) {
                    problems.push((
                        "column".to_string(),
                        format!("with a width of {} the widget would pass the 12 columns; start it at column {} or before", self.columns(), i64::from(Self::COLUMNS) - i64::from(self.columns()) + 1),
                    ));
                }
                if !(1..=Self::LAST_ROW).contains(&row) {
                    problems.push((
                        "row".to_string(),
                        format!("must be a row from 1 to {}", Self::LAST_ROW),
                    ));
                }
            }
            (Some(_), None) | (None, Some(_)) => {
                problems.push((
                    "column".to_string(),
                    "column and row are given together, or neither".to_string(),
                ));
            }
        }
        if self.height == WidgetHeight::Invalid {
            problems.push(("height".to_string(), WidgetHeight::RULE.to_string()));
        }
        problems.extend(
            self.appearance
                .problems()
                .into_iter()
                .map(|(key, message)| (format!("appearance.{key}"), message)),
        );
        problems
    }

    pub fn visible_to(&self, environment: &Environment) -> bool {
        match &self.environments {
            None => true,
            Some(names) => names.iter().any(|name| name == environment.as_str()),
        }
    }

    pub fn public_in(&self, environment: &Environment) -> bool {
        self.public && self.visible_to(environment)
    }
}
