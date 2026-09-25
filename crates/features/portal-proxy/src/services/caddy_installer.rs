use std::fs;
use std::path::Path;
use std::process::Command;

use sha2::{Digest, Sha512};

use crate::clients::Releases;
use crate::types::{CaddyHome, CaddySource, Platform};

pub const ARCHIVE: &str = "caddy.tar.gz";
pub const EXECUTABLE_MODE: u32 = 0o755;

pub async fn install(
    home: &CaddyHome,
    releases: &Releases,
    platform: &Platform,
    source: &CaddySource,
) -> Result<String, String> {
    let release = releases.release(&source.release_url()).await?;
    let version = release.version().to_string();
    let archive_name = platform.archive(&version);
    let checksums_name = Platform::checksums(&version);
    let archive = release
        .asset(&archive_name)
        .ok_or_else(|| format!("Caddy {version} has no {archive_name}"))?;
    let checksums = release
        .asset(&checksums_name)
        .ok_or_else(|| format!("Caddy {version} has no {checksums_name}"))?;
    let listed = releases.fetch(&checksums.browser_download_url).await?;
    let expected = expected_digest(&String::from_utf8_lossy(&listed), &archive_name)
        .ok_or_else(|| format!("{checksums_name} does not list {archive_name}"))?;
    let bytes = releases.fetch(&archive.browser_download_url).await?;
    let actual = hex(&Sha512::digest(&bytes));
    if actual != expected {
        return Err(format!(
            "the SHA-512 of {archive_name} does not match the release's checksum file; nothing was installed"
        ));
    }
    let home = home.clone();
    let origin = archive.browser_download_url.clone();
    tokio::task::spawn_blocking(move || unpack(&home, &bytes, &version, &origin).map(|()| version))
        .await
        .map_err(|error| format!("the installation stopped: {error}"))?
}

fn expected_digest(listed: &str, name: &str) -> Option<String> {
    listed.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let digest = parts.next()?;
        (parts.next()?.trim_start_matches('*') == name).then(|| digest.to_ascii_lowercase())
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unpack(home: &CaddyHome, bytes: &[u8], version: &str, origin: &str) -> Result<(), String> {
    let staging = home.staging();
    let failed = |what: &str, error: &dyn std::fmt::Display| format!("cannot {what}: {error}");
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|error| failed("create the Caddy directory", &error))?;
    let archive = staging.join(ARCHIVE);
    fs::write(&archive, bytes).map_err(|error| failed("save the archive", &error))?;
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&archive)
        .arg("-C")
        .arg(&staging)
        .arg(CaddyHome::BINARY)
        .status()
        .map_err(|error| failed("run tar", &error))?;
    if !status.success() {
        return Err(format!("tar could not unpack the archive ({status})"));
    }
    let unpacked = staging.join(CaddyHome::BINARY);
    make_executable(&unpacked).map_err(|error| failed("make Caddy executable", &error))?;
    fs::rename(&unpacked, home.binary()).map_err(|error| failed("install Caddy", &error))?;
    fs::write(home.version_file(), version)
        .map_err(|error| failed("record the version", &error))?;
    fs::write(home.origin_file(), origin)
        .map_err(|error| failed("record where Caddy came from", &error))?;
    let _ = fs::remove_dir_all(&staging);
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(EXECUTABLE_MODE))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}
