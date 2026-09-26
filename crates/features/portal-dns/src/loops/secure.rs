use std::net::SocketAddr;
use std::time::Instant;

use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use super::binding::Binding;
use super::streams::serve_streams;
use super::supervisor::DnsRuntime;
use crate::services::{locate, server_config};
use crate::types::{CertificatePair, TransportState};

pub const TLS_OFF: &str = "DNS over TLS is off";

type Key = (SocketAddr, CertificatePair);

#[derive(Default)]
pub struct Secure {
    bound: Option<Binding<Key>>,
    failed: Option<(Key, Instant)>,
}

impl Secure {
    pub async fn reconcile(&mut self, runtime: &DnsRuntime) {
        let settings = runtime.library.settings();
        if !(settings.enabled && settings.tls.enabled) {
            self.bound = None;
            runtime
                .library
                .update_state(|state| state.tls = TransportState::off(TLS_OFF));
            return;
        }
        let address = SocketAddr::new(settings.address, settings.tls.port);
        let caddy = runtime.configuration.storage(portal_config::Storage::Caddy);
        let pair = match locate(&settings, &caddy) {
            Ok(pair) => pair,
            Err(reason) => {
                self.bound = None;
                runtime
                    .library
                    .update_state(|state| state.tls = TransportState::off(&reason));
                return;
            }
        };
        let key = (address, pair);
        if self.bound.as_ref().is_some_and(|bound| bound.key == key) {
            return;
        }
        if self
            .failed
            .as_ref()
            .is_some_and(|(failed, at)| *failed == key && at.elapsed() < runtime.cadence.retry)
        {
            return;
        }
        self.bound = None;
        let started = match server_config(&key.1) {
            Err(reason) => Err(reason),
            Ok(config) => TcpListener::bind(address)
                .await
                .map(|listener| (listener, TlsAcceptor::from(config)))
                .map_err(|error| format!("{error} ({address})")),
        };
        match started {
            Ok((listener, acceptor)) => {
                let task = tokio::spawn(serve_streams(
                    listener,
                    runtime.library.clone(),
                    Some(acceptor),
                    runtime.cadence.idle,
                ));
                self.bound = Some(Binding {
                    key,
                    tasks: vec![task],
                });
                self.failed = None;
                tracing::info!(%address, "DNS over TLS is listening");
                runtime.library.update_state(|state| {
                    state.tls = TransportState {
                        listening: true,
                        address: Some(address),
                        reason: None,
                    };
                });
            }
            Err(reason) => {
                tracing::warn!(%address, reason, "DNS over TLS cannot listen");
                self.failed = Some((key, Instant::now()));
                runtime.library.update_state(|state| {
                    state.tls = TransportState {
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
