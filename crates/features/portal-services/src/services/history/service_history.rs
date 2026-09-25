use std::collections::VecDeque;

use portal_model::{ProbeOutcome, ServiceState};
use serde::{Deserialize, Serialize};

use crate::types::{
    HistoryLine, HistoryRange, HistoryView, HourBucket, LatencyPoint, Sample, Transition, Uptime,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServiceHistory {
    pub samples: VecDeque<Sample>,
    pub buckets: VecDeque<HourBucket>,
    pub transitions: VecDeque<Transition>,
    #[serde(skip)]
    pub dirty: bool,
    #[serde(skip)]
    pub unwritten: Vec<HistoryLine>,
    #[serde(skip)]
    pub touched_hours: Vec<i64>,
    #[serde(skip)]
    pub rewrite: bool,
}

impl ServiceHistory {
    pub const SAMPLE_SECONDS: i64 = 86_400;
    pub const KEEP_SECONDS: i64 = 30 * 86_400;
    pub const MAXIMUM_COVERED_SECONDS: i64 = 360;
    pub const THINNING_SECONDS: i64 = 5;
    pub const MAXIMUM_SAMPLES: usize = 17_280;

    pub fn record(&mut self, at: i64, outcome: &ProbeOutcome) {
        let previous = self.samples.back().cloned();
        let covered = previous.as_ref().map_or(0, |last| {
            (at - last.at).clamp(0, Self::MAXIMUM_COVERED_SECONDS)
        });
        let sample = Sample {
            at,
            state: outcome.state,
            latency: outcome
                .latency_milliseconds
                .map(|latency| u32::try_from(latency).unwrap_or(u32::MAX)),
            diagnosis: outcome.diagnosis,
            covered: u32::try_from(covered).unwrap_or(u32::MAX),
        };
        let from = previous
            .as_ref()
            .map_or(ServiceState::Unknown, |last| last.state);
        if from != sample.state {
            let transition = Transition {
                at,
                from,
                to: sample.state,
                error: outcome.error.clone(),
            };
            self.unwritten
                .push(HistoryLine::Transition(transition.clone()));
            self.transitions.push_back(transition);
        }
        self.absorb(&sample);
        match self.samples.back_mut() {
            Some(last) if at - last.at < Self::THINNING_SECONDS && last.state == sample.state => {
                let covered = last.covered.saturating_add(sample.covered);
                *last = Sample { covered, ..sample };
                self.unwritten.push(HistoryLine::Replace(last.clone()));
            }
            _ => {
                self.unwritten.push(HistoryLine::Sample(sample.clone()));
                self.samples.push_back(sample);
            }
        }
        while self.samples.len() > Self::MAXIMUM_SAMPLES {
            self.samples.pop_front();
        }
        self.prune(at);
        self.dirty = true;
    }

    pub fn prune(&mut self, now: i64) {
        let samples_after = now - Self::SAMPLE_SECONDS;
        while self
            .samples
            .front()
            .is_some_and(|sample| sample.at <= samples_after)
        {
            self.samples.pop_front();
        }
        let kept_after = now - Self::KEEP_SECONDS;
        while self
            .buckets
            .front()
            .is_some_and(|bucket| bucket.hour + HourBucket::SECONDS <= kept_after)
        {
            self.buckets.pop_front();
        }
        while self
            .transitions
            .front()
            .is_some_and(|transition| transition.at < kept_after)
        {
            self.transitions.pop_front();
        }
    }

    pub fn view(&self, range: HistoryRange, now: i64) -> HistoryView {
        let from = now - range.seconds();
        let points = match range {
            HistoryRange::Day => self
                .samples
                .iter()
                .filter(|sample| sample.at > from)
                .map(|sample| LatencyPoint {
                    at: sample.at,
                    state: sample.state,
                    average: sample.latency,
                    minimum: sample.latency,
                    maximum: sample.latency,
                })
                .collect(),
            HistoryRange::Week | HistoryRange::Month => self
                .buckets
                .iter()
                .filter(|bucket| bucket.hour + HourBucket::SECONDS > from)
                .map(|bucket| LatencyPoint {
                    at: bucket.hour,
                    state: bucket.worst(),
                    average: bucket.average(),
                    minimum: bucket.minimum,
                    maximum: bucket.maximum,
                })
                .collect(),
        };
        HistoryView {
            range,
            from,
            to: now,
            uptime: HistoryRange::ALL
                .into_iter()
                .map(|range| self.uptime(range, now))
                .collect(),
            points,
            transitions: self
                .transitions
                .iter()
                .filter(|transition| transition.at >= from)
                .cloned()
                .collect(),
        }
    }

    pub fn uptime(&self, range: HistoryRange, now: i64) -> Uptime {
        let from = now - range.seconds();
        let (covered, answered) = match range {
            HistoryRange::Day => self.samples.iter().filter(|sample| sample.at > from).fold(
                (0u64, 0u64),
                |(covered, answered), sample| {
                    (
                        covered + u64::from(sample.covered),
                        answered + sample.answered_seconds(),
                    )
                },
            ),
            HistoryRange::Week | HistoryRange::Month => self
                .buckets
                .iter()
                .filter(|bucket| bucket.hour + HourBucket::SECONDS > from)
                .fold((0u64, 0u64), |(covered, answered), bucket| {
                    (
                        covered + bucket.covered_seconds,
                        answered + bucket.answered_seconds,
                    )
                }),
        };
        Uptime {
            range,
            ratio: (covered > 0).then(|| answered as f64 / covered as f64),
            covered_seconds: covered,
        }
    }

    pub fn take_lines(&mut self) -> Vec<HistoryLine> {
        let mut lines = std::mem::take(&mut self.unwritten);
        for hour in std::mem::take(&mut self.touched_hours) {
            if let Some(bucket) = self.buckets.iter().find(|bucket| bucket.hour == hour) {
                lines.push(HistoryLine::Hour(bucket.clone()));
            }
        }
        lines
    }

    pub fn lines(&self) -> Vec<HistoryLine> {
        self.buckets
            .iter()
            .cloned()
            .map(HistoryLine::Hour)
            .chain(
                self.transitions
                    .iter()
                    .cloned()
                    .map(HistoryLine::Transition),
            )
            .chain(self.samples.iter().cloned().map(HistoryLine::Sample))
            .collect()
    }

    pub fn from_lines(lines: impl IntoIterator<Item = HistoryLine>, now: i64) -> ServiceHistory {
        let mut history = ServiceHistory::default();
        for line in lines {
            match line {
                HistoryLine::Sample(sample) => {
                    if history
                        .samples
                        .back()
                        .is_none_or(|last| last.at < sample.at)
                    {
                        history.samples.push_back(sample);
                    }
                }
                HistoryLine::Replace(sample) => match history.samples.back_mut() {
                    Some(last) => *last = sample,
                    None => history.samples.push_back(sample),
                },
                HistoryLine::Transition(transition) => {
                    let known = history.transitions.iter().any(|existing| {
                        existing.at == transition.at && existing.to == transition.to
                    });
                    if !known {
                        history.transitions.push_back(transition);
                    }
                }
                HistoryLine::Hour(bucket) => match history
                    .buckets
                    .iter_mut()
                    .find(|existing| existing.hour == bucket.hour)
                {
                    Some(existing) => *existing = bucket,
                    None => history.buckets.push_back(bucket),
                },
            }
        }
        history
            .buckets
            .make_contiguous()
            .sort_by_key(|bucket| bucket.hour);
        history
            .transitions
            .make_contiguous()
            .sort_by_key(|transition| transition.at);
        history.prune(now);
        history
    }

    fn absorb(&mut self, sample: &Sample) {
        let hour = HourBucket::hour_of(sample.at);
        if !self.touched_hours.contains(&hour) {
            self.touched_hours.push(hour);
        }
        if self.buckets.back().is_none_or(|bucket| bucket.hour != hour) {
            self.buckets.push_back(HourBucket::starting(hour));
        }
        if let Some(bucket) = self.buckets.back_mut() {
            bucket.absorb(sample);
        }
    }
}
