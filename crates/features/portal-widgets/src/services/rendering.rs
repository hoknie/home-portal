use serde_json::Value;

use super::block_checks::tone_fields_of;
use super::tones::{number_in, tone_for};
use portal_widget::Align;

use crate::ports::{TemplateContext, WidgetTemplates};
use crate::responses::{ActionKind, RenderedBlock, RenderedItem, RenderedNested, RenderedPair};
use crate::types::{Block, CustomWidget, ListBlock, TableBlock, WidgetTarget};

pub const LONGEST_TEXT: usize = 1000;
pub const LARGEST_ANSWER: usize = 64 * 1024;
pub const DEFAULT_LIMIT: i64 = 10;
pub const DEFAULT_MAXIMUM: f64 = 100.0;
pub const TOO_MUCH: &str = "too much content";

pub struct WidgetMeta<'a> {
    pub id: &'a str,
    pub title: Option<&'a str>,
    pub fetched_at: Option<String>,
}

pub fn widget_context(
    templates: &dyn WidgetTemplates,
    data: &Value,
    meta: &WidgetMeta,
) -> Box<dyn TemplateContext> {
    templates.open(data, meta.id, meta.title, meta.fetched_at.clone())
}

pub fn render_widget(
    templates: &dyn WidgetTemplates,
    widget: &CustomWidget,
    data: &Value,
    meta: &WidgetMeta,
) -> Result<Vec<RenderedBlock>, String> {
    let mut frame = widget_context(templates, data, meta);
    let blocks = render_blocks(&widget.blocks, widget, frame.as_mut())?;
    let size = serde_json::to_vec(&blocks).map_or(0, |bytes| bytes.len());
    if size > LARGEST_ANSWER {
        return Err(TOO_MUCH.to_string());
    }
    Ok(blocks)
}

fn render_blocks(
    blocks: &[Block],
    widget: &CustomWidget,
    frame: &mut dyn TemplateContext,
) -> Result<Vec<RenderedBlock>, String> {
    let mut buttons = 0;
    let mut rendered = Vec::with_capacity(blocks.len());
    for block in blocks {
        rendered.push(render_block(block, widget, frame, &mut buttons, None)?);
    }
    Ok(rendered)
}

fn text(template: &str, frame: &dyn TemplateContext) -> Result<String, String> {
    frame.text(template).map(cut)
}

fn optional(
    template: &Option<String>,
    frame: &dyn TemplateContext,
) -> Result<Option<String>, String> {
    template
        .as_deref()
        .map(|template| text(template, frame))
        .transpose()
}

pub fn cut(text: String) -> String {
    if text.chars().count() <= LONGEST_TEXT {
        return text;
    }
    let kept: String = text.chars().take(LONGEST_TEXT - 1).collect();
    format!("{kept}…")
}

fn render_block(
    block: &Block,
    widget: &CustomWidget,
    frame: &mut dyn TemplateContext,
    buttons: &mut usize,
    inherited: Option<Align>,
) -> Result<RenderedBlock, String> {
    let tones = tone_fields_of(block);
    Ok(match block {
        Block::Stat(stat) => {
            let value = text(&stat.value, frame)?;
            RenderedBlock::Stat {
                align: stat.align.or(inherited),
                valign: stat.valign,
                label: text(&stat.label, frame)?,
                tone: tone_for(&tones, &value),
                value,
                unit: optional(&stat.unit, frame)?,
                caption: optional(&stat.caption, frame)?,
                icon: stat.icon.clone(),
            }
        }
        Block::Text(block) => {
            let rendered = text(&block.text, frame)?;
            RenderedBlock::Text {
                align: block.align.or(inherited),
                valign: block.valign,
                tone: tone_for(&tones, &rendered),
                text: rendered,
                size: block.size,
                weight: block.weight,
                muted: block.muted,
            }
        }
        Block::Markdown(markdown) => RenderedBlock::Markdown {
            align: markdown.align.or(inherited),
            valign: markdown.valign,
            text: text(&markdown.text, frame)?,
        },
        Block::Badge(badge) => {
            let rendered = text(&badge.text, frame)?;
            RenderedBlock::Badge {
                align: badge.align.or(inherited),
                valign: badge.valign,
                tone: tone_for(&tones, &rendered),
                text: rendered,
            }
        }
        Block::List(list) => render_list(list, frame, &tones, inherited)?,
        Block::Table(table) => render_table(table, frame, inherited)?,
        Block::KeyValues(pairs) => RenderedBlock::KeyValues {
            align: pairs.align.or(inherited),
            valign: pairs.valign,
            pairs: pairs
                .pairs
                .iter()
                .map(|pair| {
                    Ok(RenderedPair {
                        key: text(&pair.key, frame)?,
                        value: text(&pair.value, frame)?,
                    })
                })
                .collect::<Result<_, String>>()?,
        },
        Block::Progress(progress) => {
            let value = number_in(&frame.value(&progress.value)?).unwrap_or(0.0);
            let maximum = match &progress.maximum {
                None => DEFAULT_MAXIMUM,
                Some(Value::String(template)) => {
                    number_in(&frame.value(template)?).unwrap_or(DEFAULT_MAXIMUM)
                }
                Some(other) => number_in(other).unwrap_or(DEFAULT_MAXIMUM),
            };
            RenderedBlock::Progress {
                align: progress.align.or(inherited),
                valign: progress.valign,
                label: optional(&progress.label, frame)?,
                tone: tone_for(&tones, &value.to_string()),
                value,
                maximum: if maximum > 0.0 {
                    maximum
                } else {
                    DEFAULT_MAXIMUM
                },
                caption: optional(&progress.caption, frame)?,
            }
        }
        Block::Button(button) => {
            let action = widget.actions.get(*buttons).cloned();
            *buttons += 1;
            let Some(action) = action else {
                return Err("a button lost its action".to_string());
            };
            let (does, link) = match &action.target {
                WidgetTarget::Automation { .. } => (ActionKind::Automation, None),
                WidgetTarget::Workflow { .. } => (ActionKind::Workflow, None),
                WidgetTarget::Refresh => (ActionKind::Refresh, None),
                WidgetTarget::Link(template) => (
                    ActionKind::Link,
                    Some(text(template, frame)?).filter(|link| web_address(link)),
                ),
            };
            RenderedBlock::Button {
                align: button.align.or(inherited),
                valign: button.valign,
                action: action.id,
                does,
                label: text(&button.label, frame)?,
                icon: button.icon.clone(),
                style: button.style,
                confirm: optional(&button.confirm, frame)?,
                link,
            }
        }
        Block::Divider(_) => RenderedBlock::Divider,
        Block::Row(group) => RenderedBlock::Row {
            blocks: render_group(&group.blocks, widget, frame, buttons, inherited)?,
            gap: group.gap,
            align: group.row_align,
            widths: group.widths.clone(),
        },
        Block::Column(group) => {
            let align = group.align.or(inherited);
            RenderedBlock::Column {
                blocks: render_group(&group.blocks, widget, frame, buttons, align)?,
                gap: group.gap,
                align,
                valign: group.valign,
            }
        }
    })
}

fn render_group(
    blocks: &[Block],
    widget: &CustomWidget,
    frame: &mut dyn TemplateContext,
    buttons: &mut usize,
    inherited: Option<Align>,
) -> Result<Vec<RenderedNested>, String> {
    let mut children = Vec::with_capacity(blocks.len());
    for block in blocks {
        let rendered = render_block(block, widget, frame, buttons, inherited)?;
        children.push(
            rendered
                .lowered()
                .ok_or_else(|| "groups nest deeper than a widget allows".to_string())?,
        );
    }
    Ok(children)
}

pub fn web_address(link: &str) -> bool {
    let lower = link.trim().to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

fn items_of(
    template: &str,
    frame: &dyn TemplateContext,
    limit: Option<i64>,
) -> Result<(Vec<Value>, usize), String> {
    let items = match frame.value(template)? {
        Value::Array(items) => items,
        _ => Vec::new(),
    };
    let limit = usize::try_from(limit.unwrap_or(DEFAULT_LIMIT)).unwrap_or(10);
    let more = items.len().saturating_sub(limit);
    Ok((items.into_iter().take(limit).collect(), more))
}

fn render_list(
    list: &ListBlock,
    frame: &mut dyn TemplateContext,
    tones: &crate::types::ToneFields,
    inherited: Option<Align>,
) -> Result<RenderedBlock, String> {
    let (items, more) = items_of(&list.items, frame, list.limit)?;
    let mut rendered = Vec::with_capacity(items.len());
    for (index, item) in items.into_iter().enumerate() {
        frame.set_item(Some((item, index)));
        let shown = text(&list.text, frame)?;
        rendered.push(RenderedItem {
            tone: tone_for(tones, &shown),
            secondary: optional(&list.secondary, frame)?,
            text: shown,
        });
    }
    frame.set_item(None);
    Ok(RenderedBlock::List {
        align: list.align.or(inherited),
        valign: list.valign,
        items: rendered,
        empty: optional(&list.empty, frame)?,
        more,
    })
}

fn render_table(
    table: &TableBlock,
    frame: &mut dyn TemplateContext,
    inherited: Option<Align>,
) -> Result<RenderedBlock, String> {
    let headers = table
        .columns
        .iter()
        .map(|column| text(&column.header, frame))
        .collect::<Result<Vec<_>, String>>()?;
    let (items, more) = items_of(&table.items, frame, table.limit)?;
    let mut rows = Vec::with_capacity(items.len());
    for (index, item) in items.into_iter().enumerate() {
        frame.set_item(Some((item, index)));
        rows.push(
            table
                .columns
                .iter()
                .map(|column| text(&column.value, frame))
                .collect::<Result<Vec<_>, String>>()?,
        );
    }
    frame.set_item(None);
    Ok(RenderedBlock::Table {
        align: table.align.or(inherited),
        valign: table.valign,
        headers,
        rows,
        empty: optional(&table.empty, frame)?,
        more,
    })
}
