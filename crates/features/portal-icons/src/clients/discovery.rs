use serde_json::Value;
use url::Url;

use super::Fetcher;

pub const MANIFEST_RELATIONS: [&str; 1] = ["manifest"];
pub const ICON_RELATIONS: [&str; 3] = ["icon", "shortcut icon", "apple-touch-icon"];
pub const FALLBACK: &str = "/favicon.ico";

pub async fn discover_icon(fetcher: &Fetcher, address: &Url) -> Result<Url, String> {
    let page = fetcher.get(address, Fetcher::PAGE_CEILING_BYTES).await?;
    let html = String::from_utf8_lossy(&page.bytes).to_string();
    if let Some(manifest) =
        link_in(&html, &MANIFEST_RELATIONS).and_then(|href| address.join(&href).ok())
        && let Ok(fetched) = fetcher.get(&manifest, Fetcher::PAGE_CEILING_BYTES).await
        && let Ok(document) = serde_json::from_slice::<Value>(&fetched.bytes)
        && let Some(source) = largest_manifest_icon(&document)
        && let Ok(icon) = manifest.join(&source)
    {
        return Ok(icon);
    }
    if let Some(icon) = link_in(&html, &ICON_RELATIONS).and_then(|href| address.join(&href).ok()) {
        return Ok(icon);
    }
    address.join(FALLBACK).map_err(|error| error.to_string())
}

fn link_in(html: &str, relations: &[&str]) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let mut best: Option<String> = None;
    let mut cursor = 0;
    while let Some(start) = lower[cursor..].find("<link") {
        let start = cursor + start;
        let end = lower[start..]
            .find('>')
            .map(|offset| start + offset)
            .unwrap_or(lower.len());
        let tag = &html[start..end];
        let lower_tag = &lower[start..end];
        if let Some(relation) = attribute(lower_tag, "rel")
            && relations.contains(&relation.trim())
            && let Some(href) = attribute(tag, "href")
        {
            best = Some(href);
            break;
        }
        cursor = end.max(start + 5);
    }
    best
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let end = rest[1..].find(quote)? + 1;
        Some(rest[1..end].to_string())
    } else {
        let end = rest
            .find(|character: char| character.is_whitespace())
            .unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}

fn largest_manifest_icon(document: &Value) -> Option<String> {
    let icons = document.get("icons")?.as_array()?;
    icons
        .iter()
        .filter_map(|icon| {
            let source = icon.get("src")?.as_str()?.to_string();
            let size = icon
                .get("sizes")
                .and_then(Value::as_str)
                .and_then(|sizes| sizes.split_whitespace().next())
                .and_then(|size| size.split('x').next())
                .and_then(|width| width.parse::<u32>().ok())
                .unwrap_or(0);
            Some((size, source))
        })
        .max_by_key(|(size, _)| *size)
        .map(|(_, source)| source)
}
