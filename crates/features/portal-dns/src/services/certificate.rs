use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use portal_model::TlsMode;
use rustls::ServerConfig;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

use crate::types::{CertificatePair, DnsSettings};

pub const CERTIFICATES: [&str; 3] = ["data", "caddy", "certificates"];
pub const CERTIFICATE_SUFFIX: &str = ".crt";
pub const KEY_SUFFIX: &str = ".key";
pub const NO_HOST: &str =
    "there is no host for DNS over TLS; set proxy.portal_host or dns.https.host";
pub const NOT_MANAGED: &str =
    "Caddy is not managed by the portal; set dns.tls.certificate and dns.tls.key";

pub fn locate(settings: &DnsSettings, caddy: &Path) -> Result<CertificatePair, String> {
    if let (Some(certificate), Some(key)) = (&settings.tls.certificate, &settings.tls.key) {
        return Ok(pair(certificate.clone(), key.clone()));
    }
    let host = settings.secure_host().ok_or_else(|| NO_HOST.to_string())?;
    let policy = &settings.proxy.tls;
    match policy.mode {
        TlsMode::Files => match (&policy.certificate, &policy.key) {
            (Some(certificate), Some(key)) => Ok(pair(certificate.into(), key.into())),
            _ => Err(format!("proxy.tls names no certificate files for {host}")),
        },
        _ if !settings.proxy.managed => Err(NOT_MANAGED.to_string()),
        _ => newest_from_caddy(caddy, host)
            .ok_or_else(|| format!("Caddy has no certificate for {host} yet")),
    }
}

pub fn server_config(pair: &CertificatePair) -> Result<Arc<ServerConfig>, String> {
    let certificates: Vec<CertificateDer<'static>> =
        CertificateDer::pem_file_iter(&pair.certificate)
            .and_then(|found| found.collect())
            .map_err(|error| format!("cannot read {}: {error}", pair.certificate.display()))?;
    let key = PrivateKeyDer::from_pem_file(&pair.key)
        .map_err(|error| format!("cannot read {}: {error}", pair.key.display()))?;
    ServerConfig::builder_with_provider(Arc::new(rustls::crypto::aws_lc_rs::default_provider()))
        .with_safe_default_protocol_versions()
        .map_err(|error| error.to_string())?
        .with_no_client_auth()
        .with_single_cert(certificates, key)
        .map(Arc::new)
        .map_err(|error| error.to_string())
}

fn pair(certificate: PathBuf, key: PathBuf) -> CertificatePair {
    let modified = fs::metadata(&certificate)
        .and_then(|metadata| metadata.modified())
        .ok()
        .zip(
            fs::metadata(&key)
                .and_then(|metadata| metadata.modified())
                .ok(),
        );
    CertificatePair {
        certificate,
        key,
        modified,
    }
}

fn newest_from_caddy(caddy: &Path, host: &str) -> Option<CertificatePair> {
    let root = CERTIFICATES
        .iter()
        .fold(caddy.to_path_buf(), |path, part| path.join(part));
    fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .map(|issuer| issuer.path().join(host))
        .map(|folder| {
            pair(
                folder.join(format!("{host}{CERTIFICATE_SUFFIX}")),
                folder.join(format!("{host}{KEY_SUFFIX}")),
            )
        })
        .filter(|found| found.certificate.is_file() && found.key.is_file())
        .max_by_key(|found| found.modified.map(|(certificate, _)| certificate))
}
