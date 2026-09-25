use std::collections::HashMap;
use std::time::Duration;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use portal_feature::EventName;

use crate::types::{Automation, CronFilter};

#[derive(Default)]
pub struct ScheduleBook {
    evaluated: HashMap<String, Entry>,
}

#[derive(Clone)]
struct Entry {
    expression: String,
    zone: Option<String>,
    evaluated: Timestamp,
    next: Option<Timestamp>,
}

impl ScheduleBook {
    pub const LATE_GRACE: Duration = Duration::from_secs(2);

    pub fn due(
        &mut self,
        automations: &[Automation],
        zone: &TimeZone,
        now: Timestamp,
    ) -> Vec<(String, String, Timestamp)> {
        let floor = now
            .checked_sub(jiff::SignedDuration::try_from(Self::LATE_GRACE).unwrap_or_default())
            .unwrap_or(now);
        let zone_name = zone.iana_name().map(str::to_string);
        let scheduled = Self::scheduled(automations);
        self.evaluated
            .retain(|id, _| scheduled.iter().any(|(automation, _)| automation.id == *id));
        let mut fired = Vec::new();
        for (automation, cron) in scheduled {
            let entry = self
                .evaluated
                .entry(automation.id.clone())
                .or_insert_with(|| Entry {
                    expression: cron.expression.clone(),
                    zone: zone_name.clone(),
                    evaluated: now,
                    next: None,
                });
            if entry.expression != cron.expression || entry.zone != zone_name {
                *entry = Entry {
                    expression: cron.expression.clone(),
                    zone: zone_name.clone(),
                    evaluated: now,
                    next: None,
                };
            }
            if entry.evaluated < floor {
                entry.evaluated = floor;
                entry.next = None;
            }
            if entry.next.is_none() {
                entry.next = cron.schedule.next_after(entry.evaluated, zone);
            }
            if let Some(due) = entry.next
                && due <= now
            {
                entry.evaluated = due;
                entry.next = cron.schedule.next_after(due, zone);
                fired.push((automation.id.clone(), cron.expression.clone(), due));
            }
        }
        fired
    }

    pub fn earliest(&self) -> Option<Timestamp> {
        self.evaluated.values().filter_map(|entry| entry.next).min()
    }

    fn scheduled(automations: &[Automation]) -> Vec<(&Automation, &CronFilter)> {
        automations
            .iter()
            .filter(|automation| {
                automation.enabled && automation.trigger.event == EventName::Schedule
            })
            .filter_map(|automation| {
                automation
                    .trigger
                    .filters
                    .cron
                    .as_ref()
                    .map(|cron| (automation, cron))
            })
            .collect()
    }
}
