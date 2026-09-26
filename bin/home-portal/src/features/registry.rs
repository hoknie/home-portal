use std::sync::Arc;

use portal_auth::AuthFeature;
use portal_automations::AutomationsFeature;
use portal_calendar::CalendarFeature;
use portal_dashboard::DashboardFeature;
use portal_dns::DnsFeature;
use portal_feature::Feature;
use portal_health::HealthFeature;
use portal_icons::IconsFeature;
use portal_metrics::MetricsFeature;
use portal_network::{NetworkFeature, host_environment};
use portal_proxy::{ProxyFeature, ProxyPorts};
use portal_public::PublicFeature;
use portal_secrets::SecretsFeature;
use portal_services::{ServicesFeature, ServicesPorts};
use portal_telegram::TelegramFeature;
use portal_weather::WeatherFeature;
use portal_widget::WidgetRegistry;

use crate::adapters::{
    AutomationDirectory, DnsDirectory, NetworkConnection, ProxyPublishing, ServiceCatalogue,
    ServicePublications, WidgetLayout,
};
use crate::types::{BootError, Registry, Restart, Wiring};

pub fn registered(wiring: &Wiring) -> Result<Registry, BootError> {
    let configuration = wiring.configuration.clone();
    let configuration_for_widgets = wiring.configuration.clone();
    let connection = Arc::new(NetworkConnection {
        configuration: configuration.clone(),
    });
    let automations = Arc::new(AutomationsFeature::new(
        configuration.clone(),
        Arc::new(AutomationDirectory {
            configuration: configuration.clone(),
        }),
    ));
    let events = automations.events();
    let auth = Arc::new(AuthFeature::new(
        configuration.clone(),
        connection.clone(),
        events.clone(),
    ));
    let host = host_environment(
        &portal_network::read_environments(&configuration.read().document).unwrap_or_default(),
    );
    let telegram = TelegramFeature::new(configuration.clone(), portal_telegram::ENDPOINT).map_err(
        |message| BootError::Feature {
            name: TelegramFeature::NAME,
            message,
        },
    )?;
    let icons = Arc::new(
        IconsFeature::new(configuration.clone(), portal_icons::CATALOG).map_err(|message| {
            BootError::Feature {
                name: IconsFeature::NAME,
                message,
            }
        })?,
    );
    let calendar =
        CalendarFeature::new(configuration.clone()).map_err(|message| BootError::Feature {
            name: CalendarFeature::NAME,
            message,
        })?;
    let weather = WeatherFeature::new(WeatherFeature::PORTAL_TIMEZONE).map_err(|message| {
        BootError::Feature {
            name: WeatherFeature::NAME,
            message,
        }
    })?;
    let services = Arc::new(
        ServicesFeature::new(
            configuration.clone(),
            host,
            ServicesPorts {
                observers: vec![telegram.observer(), automations.observer()],
                publishing: Arc::new(ProxyPublishing {
                    configuration: configuration.clone(),
                }),
                events: events.clone(),
            },
        )
        .map_err(|message| BootError::Feature {
            name: ServicesFeature::NAME,
            message,
        })?,
    );
    let mut features: Vec<Arc<dyn Feature>> = vec![
        Arc::new(HealthFeature),
        auth.clone(),
        services.clone(),
        Arc::new(NetworkFeature::new(configuration.clone(), wiring.effective)),
        Arc::new(MetricsFeature::new()),
        Arc::new(weather),
        Arc::new(calendar),
        icons.clone(),
        Arc::new(telegram),
        Arc::new(SecretsFeature::new(configuration.clone())),
        Arc::new(DashboardFeature::new(configuration.clone())),
        automations,
        Arc::new(DnsFeature::new(
            configuration.clone(),
            Arc::new(DnsDirectory {
                configuration: configuration.clone(),
                services: services.clone(),
            }),
        )),
        Arc::new(ProxyFeature::new(
            configuration,
            ProxyPorts {
                services: Arc::new(ServicePublications {
                    services: services.clone(),
                }),
                gate: auth.gate(),
                peers: connection,
            },
            wiring.effective.address,
        )),
    ];
    let providers = features
        .iter()
        .flat_map(|feature| feature.widget_providers())
        .collect();
    let widgets = Arc::new(WidgetRegistry::new(configuration_for_widgets, providers));
    features.push(Arc::new(PublicFeature::new(
        Arc::new(ServiceCatalogue { services, icons }),
        Arc::new(WidgetLayout {
            widgets: widgets.clone(),
        }),
    )));
    Ok(Registry {
        widgets,
        features,
        gate: auth.gate(),
        configuration: wiring.configuration.clone(),
        events,
        restart: Restart::default(),
        interface: crate::boot::located(),
    })
}
