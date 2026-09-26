use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use portal_config::ConfigStore;
use tokio::net::{TcpListener, UdpSocket};

use super::binding::Binding;
use super::datagrams::serve_datagrams;
use super::streams::serve_streams;
use crate::ports::DnsSources;
use crate::services::{Library, build, read_settings};
use crate::types::{Cadence, TransportState};

pub const DISABLED: &str = "the DNS server is off";

pub struct DnsRuntime {
    pub configuration: Arc<ConfigStore>,
    pub sources: Arc<dyn DnsSources>,
    pub library: Arc<Library>,
    pub cadence: Cadence,
}

#[derive(Default)]
struct Plain {
    bound: Option<Binding<SocketAddr>>,
    failed: Option<(SocketAddr, Instant)>,
}

impl DnsRuntime {
    pub async fn run(self: Arc<Self>) {
        let mut plain = Plain::default();
        let mut secure = super::secure::Secure::default();
        let mut interfaces = self.sources.interfaces();
        let mut interfaces_read = Instant::now();
        loop {
            if interfaces_read.elapsed() >= self.cadence.interfaces {
                interfaces = self.sources.interfaces();
                interfaces_read = Instant::now();
            }
            self.refresh(&interfaces);
            self.reconcile_plain(&mut plain).await;
            secure.reconcile(&self).await;
            tokio::time::sleep(self.cadence.tick).await;
        }
    }

    pub fn refresh_now(&self) {
        self.refresh(&self.sources.interfaces());
    }

    fn refresh(&self, interfaces: &[std::net::IpAddr]) {
        let snapshot = self.configuration.read();
        match read_settings(&snapshot.document) {
            Ok(settings) => {
                let book = build(
                    &settings,
                    &self.sources.published(),
                    interfaces,
                    self.sources.environments(),
                );
                self.library.publish(settings, book);
            }
            Err(errors) => {
                let message = errors
                    .iter()
                    .map(|error| format!("{}: {}", error.field, error.message))
                    .collect::<Vec<_>>()
                    .join("; ");
                self.library
                    .update_state(|state| state.last_error = Some(message));
            }
        }
    }

    async fn reconcile_plain(&self, plain: &mut Plain) {
        let settings = self.library.settings();
        let wanted = settings
            .enabled
            .then(|| SocketAddr::new(settings.address, settings.port));
        let Some(address) = wanted else {
            plain.bound = None;
            plain.failed = None;
            self.library
                .update_state(|state| state.plain = TransportState::off(DISABLED));
            return;
        };
        if plain
            .bound
            .as_ref()
            .is_some_and(|bound| bound.key == address)
        {
            return;
        }
        if plain
            .failed
            .is_some_and(|(failed, at)| failed == address && at.elapsed() < self.cadence.retry)
        {
            return;
        }
        plain.bound = None;
        match bind(address).await {
            Ok((socket, listener)) => {
                let tasks = vec![
                    tokio::spawn(serve_datagrams(socket, self.library.clone())),
                    tokio::spawn(serve_streams(
                        listener,
                        self.library.clone(),
                        None,
                        self.cadence.idle,
                    )),
                ];
                plain.bound = Some(Binding {
                    key: address,
                    tasks,
                });
                plain.failed = None;
                tracing::info!(%address, "DNS is listening");
                self.library.update_state(|state| {
                    state.plain = TransportState {
                        listening: true,
                        address: Some(address),
                        reason: None,
                    };
                });
            }
            Err(error) => {
                let reason = format!("{error} ({address})");
                tracing::warn!(%address, %error, "DNS cannot listen");
                plain.failed = Some((address, Instant::now()));
                self.library.update_state(|state| {
                    state.plain = TransportState {
                        listening: false,
                        address: Some(address),
                        reason: Some(reason.clone()),
                    };
                    state.last_error = Some(reason);
                });
            }
        }
    }
}

async fn bind(address: SocketAddr) -> std::io::Result<(UdpSocket, TcpListener)> {
    let socket = UdpSocket::bind(address).await?;
    let listener = TcpListener::bind(address).await?;
    Ok((socket, listener))
}
