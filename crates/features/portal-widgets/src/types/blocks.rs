use portal_widget::Align;
use serde::Deserialize;
use serde_json::Value;

use super::{RawAction, Tone, ToneFields};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Block {
    Stat(StatBlock),
    Text(TextBlock),
    Markdown(MarkdownBlock),
    List(ListBlock),
    Table(TableBlock),
    KeyValues(KeyValuesBlock),
    Progress(ProgressBlock),
    Badge(BadgeBlock),
    Button(ButtonBlock),
    Divider(DividerBlock),
    Row(GroupBlock),
    Column(GroupBlock),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatBlock {
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub tone: Option<super::Tone>,
    #[serde(default)]
    pub thresholds: Option<super::Thresholds>,
    #[serde(default)]
    pub tones: Option<std::collections::BTreeMap<String, super::Tone>>,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[schemars(extend("x-open" = "normal"))]
#[serde(rename_all = "kebab-case")]
pub enum TextSize {
    Small,
    #[default]
    Normal,
    Large,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[schemars(extend("x-open" = "normal"))]
#[serde(rename_all = "kebab-case")]
pub enum Weight {
    #[default]
    Normal,
    Strong,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextBlock {
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    pub text: String,
    #[serde(default)]
    pub size: TextSize,
    #[serde(default)]
    pub weight: Weight,
    #[serde(default)]
    pub muted: bool,
    #[serde(default)]
    pub tone: Option<super::Tone>,
    #[serde(default)]
    pub thresholds: Option<super::Thresholds>,
    #[serde(default)]
    pub tones: Option<std::collections::BTreeMap<String, super::Tone>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkdownBlock {
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListBlock {
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    pub items: String,
    pub text: String,
    #[serde(default)]
    pub secondary: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub empty: Option<String>,
    #[serde(default)]
    pub tone: Option<super::Tone>,
    #[serde(default)]
    pub thresholds: Option<super::Thresholds>,
    #[serde(default)]
    pub tones: Option<std::collections::BTreeMap<String, super::Tone>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableColumn {
    pub header: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TableBlock {
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    pub items: String,
    pub columns: Vec<TableColumn>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub empty: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyValuesBlock {
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    pub pairs: Vec<KeyValue>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressBlock {
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    #[serde(default)]
    pub label: Option<String>,
    pub value: String,
    #[serde(default)]
    pub maximum: Option<Value>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub tone: Option<super::Tone>,
    #[serde(default)]
    pub thresholds: Option<super::Thresholds>,
    #[serde(default)]
    pub tones: Option<std::collections::BTreeMap<String, super::Tone>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BadgeBlock {
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    pub text: String,
    #[serde(default)]
    pub tone: Option<super::Tone>,
    #[serde(default)]
    pub thresholds: Option<super::Thresholds>,
    #[serde(default)]
    pub tones: Option<std::collections::BTreeMap<String, super::Tone>>,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[schemars(extend("x-open" = "secondary"))]
#[serde(rename_all = "kebab-case")]
pub enum ButtonStyle {
    Primary,
    #[default]
    Secondary,
    Ghost,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ButtonBlock {
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    pub action: RawAction,
    #[serde(default)]
    pub confirm: Option<String>,
    #[serde(default)]
    pub style: ButtonStyle,
    #[serde(default)]
    pub tone: Option<Tone>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DividerBlock {}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[schemars(extend("x-open" = "normal"))]
#[serde(rename_all = "kebab-case")]
pub enum Gap {
    Small,
    #[default]
    Normal,
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, serde::Serialize, schemars::JsonSchema,
)]
#[schemars(extend("x-open" = "stretch"))]
#[serde(rename_all = "kebab-case")]
pub enum RowAlign {
    Start,
    Center,
    End,
    #[default]
    Stretch,
}

impl RowAlign {
    pub const NAMES: &'static str = "start, center, end, stretch";
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupBlock {
    pub blocks: Vec<Block>,
    #[serde(default)]
    pub gap: Gap,
    #[serde(default)]
    pub align: Option<Align>,
    #[serde(default)]
    pub valign: Option<Align>,
    #[serde(default)]
    pub row_align: RowAlign,
    #[serde(default)]
    pub widths: Option<Vec<u8>>,
}

impl Block {
    pub fn tone_fields(&self) -> Option<ToneFields> {
        let fields =
            |tone: &Option<super::Tone>,
             thresholds: &Option<super::Thresholds>,
             tones: &Option<std::collections::BTreeMap<String, super::Tone>>| {
                ToneFields {
                    tone: *tone,
                    thresholds: thresholds.clone(),
                    tones: tones.clone(),
                }
            };
        match self {
            Block::Stat(block) => Some(fields(&block.tone, &block.thresholds, &block.tones)),
            Block::Text(block) => Some(fields(&block.tone, &block.thresholds, &block.tones)),
            Block::List(block) => Some(fields(&block.tone, &block.thresholds, &block.tones)),
            Block::Progress(block) => Some(fields(&block.tone, &block.thresholds, &block.tones)),
            Block::Badge(block) => Some(fields(&block.tone, &block.thresholds, &block.tones)),
            _ => None,
        }
    }

    pub fn valign(&self) -> Option<Align> {
        match self {
            Block::Stat(block) => block.valign,
            Block::Text(block) => block.valign,
            Block::Markdown(block) => block.valign,
            Block::List(block) => block.valign,
            Block::Table(block) => block.valign,
            Block::KeyValues(block) => block.valign,
            Block::Progress(block) => block.valign,
            Block::Badge(block) => block.valign,
            Block::Button(block) => block.valign,
            Block::Column(group) => group.valign,
            Block::Divider(_) | Block::Row(_) => None,
        }
    }

    pub fn align(&self) -> Option<Align> {
        match self {
            Block::Stat(block) => block.align,
            Block::Text(block) => block.align,
            Block::Markdown(block) => block.align,
            Block::List(block) => block.align,
            Block::Table(block) => block.align,
            Block::KeyValues(block) => block.align,
            Block::Progress(block) => block.align,
            Block::Badge(block) => block.align,
            Block::Button(block) => block.align,
            Block::Column(group) => group.align,
            Block::Divider(_) | Block::Row(_) => None,
        }
    }

    pub fn children(&self) -> &[Block] {
        match self {
            Block::Row(group) | Block::Column(group) => &group.blocks,
            _ => &[],
        }
    }

    pub fn count(blocks: &[Block]) -> usize {
        blocks
            .iter()
            .map(|block| 1 + Block::count(block.children()))
            .sum()
    }
}
