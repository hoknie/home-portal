use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use portal_widget::Align;

use crate::types::{ButtonStyle, Gap, RowAlign, TextSize, Tone, Weight};

macro_rules! rendered_blocks {
    ($name:ident { $($groups:tt)* }) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
        #[serde(tag = "kind", rename_all = "kebab-case")]
        pub enum $name {
            Stat {
                align: Option<Align>,
                valign: Option<Align>,
                label: String,
                value: String,
                unit: Option<String>,
                caption: Option<String>,
                icon: Option<String>,
                tone: Tone,
            },
            Text {
                align: Option<Align>,
                valign: Option<Align>,
                text: String,
                size: TextSize,
                weight: Weight,
                muted: bool,
                tone: Tone,
            },
            Markdown {
                align: Option<Align>,
                valign: Option<Align>,
                text: String,
            },
            List {
                align: Option<Align>,
                valign: Option<Align>,
                items: Vec<RenderedItem>,
                empty: Option<String>,
                more: usize,
            },
            Table {
                align: Option<Align>,
                valign: Option<Align>,
                headers: Vec<String>,
                rows: Vec<Vec<String>>,
                empty: Option<String>,
                more: usize,
            },
            KeyValues {
                align: Option<Align>,
                valign: Option<Align>,
                pairs: Vec<RenderedPair>,
            },
            Progress {
                align: Option<Align>,
                valign: Option<Align>,
                label: Option<String>,
                value: f64,
                maximum: f64,
                caption: Option<String>,
                tone: Tone,
            },
            Badge {
                align: Option<Align>,
                valign: Option<Align>,
                text: String,
                tone: Tone,
            },
            Button {
                align: Option<Align>,
                valign: Option<Align>,
                action: String,
                does: ActionKind,
                label: String,
                icon: Option<String>,
                style: ButtonStyle,
                tone: Option<Tone>,
                confirm: Option<String>,
                link: Option<String>,
            },
            Divider,
            $($groups)*
        }
    };
}

macro_rules! rendered_groups {
    ($name:ident, $child:ident) => {
        rendered_blocks!($name {
            Row {
                blocks: Vec<$child>,
                gap: Gap,
                align: RowAlign,
                widths: Option<Vec<u8>>,
            },
            Column {
                blocks: Vec<$child>,
                gap: Gap,
                align: Option<Align>,
                valign: Option<Align>,
            },
        });
    };
}

rendered_blocks!(RenderedLeaf {});

rendered_groups!(RenderedGroup, RenderedLeaf);

rendered_groups!(RenderedNested, RenderedGroup);

rendered_groups!(RenderedBlock, RenderedNested);

impl RenderedBlock {
    pub fn lowered<T: serde::de::DeserializeOwned>(self) -> Option<T> {
        serde_json::to_value(self)
            .ok()
            .and_then(|value| serde_json::from_value(value).ok())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RenderedItem {
    pub text: String,
    pub secondary: Option<String>,
    pub tone: Tone,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RenderedPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(extend("x-open" = "refresh"))]
#[serde(rename_all = "kebab-case")]
pub enum ActionKind {
    Automation,
    Workflow,
    Refresh,
    Link,
}
