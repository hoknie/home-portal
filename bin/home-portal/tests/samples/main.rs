#[path = "../support/mod.rs"]
mod support;

mod automations;
mod catalogue;
mod dns;
mod history;
mod proxy;
mod public;
mod widgets;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use axum::response::IntoResponse;
use home_portal::registered;
use http_body_util::BodyExt;
use portal_auth::SessionResponse;
use portal_calendar::{CalendarEvent, Repeats};
use portal_dashboard::{DashboardResponse, SectionView, WidgetView};
use portal_feature::{ApiError, FieldError};
use portal_icons::IconState;
use portal_metrics::{DiskReading, HostReading, Usage};
use portal_model::{Diagnosis, Environment, ProbeOutcome, ServiceState, ServiceStatus};
use portal_network::{
    EffectiveAddress, EnvironmentResponse, InterfaceResponse, NetworkResponse, NetworkSettings,
};
use portal_public::{PortalResponse, PublicSection, PublicService, PublicWidget};
use portal_secrets::{SecretResponse, SecretsResponse};
use portal_services::{
    HistoryRange, HistoryResponse, ProbeKind, ServiceEntry, ServiceHistory, ServiceResponse,
    ServicesResponse,
};
use portal_weather::{
    CONDITIONS, CurrentWeather, DailyWeather, UNKNOWN_CONDITION, Units, WeatherReading,
};
use portal_widget::{WidgetData, WidgetInstance, WidgetSize};
use serde_json::{Value, json};
use time::macros::datetime;

const SAMPLES: &str = "web/src/shared/api/samples";
const WRITE_VARIABLE: &str = "HOME_PORTAL_SAMPLES";

fn sample_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(SAMPLES)
        .join(format!("{name}.json"))
}

fn check(name: &str, value: Value) {
    let text = format!("{}\n", serde_json::to_string_pretty(&value).unwrap());
    let path = sample_path(name);
    if env::var(WRITE_VARIABLE).as_deref() == Ok("write") {
        fs::write(&path, &text).unwrap();
        return;
    }
    let on_disk = fs::read_to_string(&path).unwrap_or_default();
    assert_eq!(
        on_disk,
        text,
        "{} is out of date with the serializers; run `just samples`",
        path.display()
    );
}

fn entry(id: &str, name: &str, url: &str) -> ServiceEntry {
    ServiceEntry::new(id, name, url)
}
