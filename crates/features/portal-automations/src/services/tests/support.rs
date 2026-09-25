use std::sync::Arc;

use portal_feature::{EventName, PortalEvent};
use time::OffsetDateTime;
use toml_edit::DocumentMut;

use crate::services::{AutomationCache, AutomationSink};
use crate::types::{Automation, AutomationsSection, Pending};

pub fn entry(id: &str, when: &str, extra: &str) -> String {
    format!(
        "[[automations]]\nid = \"{id}\"\ntitle = \"T\"\nwhen = {when}\nrun = {{ script = \"a.sh\" }}\n{extra}\n"
    )
}

pub fn automations(text: &str) -> Vec<Automation> {
    let document: DocumentMut = text.parse().unwrap();
    AutomationsSection::read(&document)
        .unwrap()
        .automations
        .iter()
        .map(|raw| Automation::decode(raw).unwrap())
        .collect()
}

pub fn automation(id: &str, when: &str, extra: &str) -> Automation {
    automations(&entry(id, when, extra)).remove(0)
}

pub fn pending(id: &str, run_id: u64, by: Option<&str>) -> Pending {
    Pending {
        run_id,
        automation: automation(id, "{ event = \"portal.started\" }", ""),
        event: PortalEvent::portal(EventName::PortalStarted, "", OffsetDateTime::UNIX_EPOCH),
        by: by.map(str::to_string),
    }
}

pub fn sink(text: &str) -> Arc<AutomationSink> {
    let document: DocumentMut = text.parse().unwrap();
    Arc::new(AutomationSink::new(Arc::new(AutomationCache::of(
        &document,
    ))))
}

pub fn status_change(service: &str, from: &str, to: &str) -> PortalEvent {
    PortalEvent::of(
        EventName::ServiceStatusChanged,
        OffsetDateTime::UNIX_EPOCH,
        &[
            ("service.id", service),
            ("status.from", from),
            ("status.to", to),
        ],
    )
}
