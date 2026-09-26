# home-portal

A self-hosted start page for a home network. It shows whether your services are up, how they
have behaved over the last 30 days, and lets you manage them — all from one small program.

- **Status of every service** — HTTP, TCP and ICMP probes, uptime and latency history, a
  diagnosis when something is down, and Telegram notifications.
- **A home page you arrange** — sections and widgets (service tiles, status summary, host
  metrics, weather, calendar) in quarter-to-full widths, edited in the browser.
- **Environments** — the same service reachable at one address at home and another over VPN;
  visitors from the internet see only what is marked `public`.
- **Publishing through Caddy** — the portal downloads, runs and configures Caddy, with
  Let's Encrypt, Caddy's own authority or your certificates, optionally behind the portal's
  sign-in.
- **Automations and webhooks** — run your own scripts on a cron schedule, on portal events
  (a service goes down, someone signs in, the configuration changes…), by hand, or when another
  system calls `POST /webhook/<id>`.
- **One TOML file** — the interface edits it in place and keeps your comments; secrets live in
  a separate file of mode 0600.

Building it yourself or working on the code: see [DEVELOPMENT.md](DEVELOPMENT.md).

## Install

Every release carries a package and an archive per platform, and a `SHA256SUMS` over all of
them. Check what you downloaded first:

```sh
sha256sum --check --ignore-missing SHA256SUMS      # shasum -a 256 -c on macOS
```

| File | For |
|---|---|
| `home-portal_<version>.macos.universal.pkg` | macOS 11 or later, Apple silicon and Intel: installs and starts everything |
| `home-portal_<version>.debian.amd64.deb` / `.debian.arm64.deb` | Debian, Ubuntu, Raspberry Pi OS (64-bit) and relatives |
| `home-portal_<version>.el.x86_64.rpm` / `.el.aarch64.rpm` | RHEL, AlmaLinux, Rocky, Fedora and relatives |
| `home-portal_<version>.linux.x86_64.tar.gz` / `.linux.aarch64.tar.gz` | Any Linux, to place by hand |
| `home-portal_<version>.macos.universal.tar.gz` | macOS, to place by hand |

### macOS

Open the `.pkg`. It installs the portal and sets it up for the user signed in:
- the configuration in `~/.config/home-portal/home-portal.toml`;
- the user **admin**, with a random password in `~/.config/home-portal/initial-password`;
- a LaunchAgent that starts the portal at every login.

Then open http://127.0.0.1:8080. An older installation in
`~/Library/Application Support/home-portal` is moved to `~/.config/home-portal`, and a link is
left at the old place.

Another user of the Mac sets the portal up with `/usr/local/libexec/home-portal/setup`.
`sudo /usr/local/libexec/home-portal/uninstall` removes it; add `--purge` to delete the
configuration and data too.

### Debian, Ubuntu, RHEL and relatives

```sh
sudo apt install ./home-portal_<version>.debian.amd64.deb     # or
sudo dnf install ./home-portal_<version>.el.x86_64.rpm
sudo cat /etc/home-portal/initial-password                   # the password of admin
```

The package runs the portal as the systemd service `home-portal`, under a system user of the
same name, on http://127.0.0.1:8080:
- the configuration is `/etc/home-portal/home-portal.toml`, with the user **admin**;
- the data is kept in `/var/lib/home-portal`;
- scripts for automations go into `/var/lib/home-portal/scripts` (owned by root, 0755).

An upgrade keeps the configuration, and so does removing the package; `apt purge` deletes it.

### From the archive

```sh
tar -xzf home-portal_<version>.linux.x86_64.tar.gz
cd home-portal_<version>.linux.x86_64
mkdir -p ~/.config/home-portal
cp config/home-portal.example.toml ~/.config/home-portal/home-portal.toml
./home-portal password-hash          # type a password, then add the hash as a [[users]] entry
./home-portal
```

```toml
[[users]]
name = "admin"
password_hash = "$argon2id$..."
```

Keep the `web/` folder beside the binary: it is the interface. Without it the portal still
runs, but its pages answer 503. The archive also holds `examples/` and systemd or launchd files
for the portal and Caddy.

## Configuration

Everything is set in one TOML file. Start from
[`config/home-portal.example.toml`](config/home-portal.example.toml), which explains every
section in its comments. [`examples/`](examples/README.md) has a complete setup split over
several files, ready-made entries for common home software (Jellyfin, Plex, Home Assistant,
Pi-hole, Proxmox…), automations with a sample script, and launchd and systemd files.

| Setting | What it does |
|---|---|
| `HOME_PORTAL_CONFIG` | Path to the main file; by default `~/.config/home-portal/home-portal.toml` (or `$XDG_CONFIG_HOME/home-portal/home-portal.toml`) |
| `HOME_PORTAL_ADDRESS=ip:port` | Overrides `[network]` address and port |
| `HOME_PORTAL_WEB` | The interface folder, when it is not beside the binary or in `../share/home-portal/web` |
| `include = [...]` | Further files read after the main one, inside its directory |
| `[secrets]` | Tokens and passwords, in a file of mode 0600 (see [`config/secrets.example.toml`](config/secrets.example.toml)); settings name a key, never the value |
| `configuration.writes_to` | Which file the interface edits |
| `[storage]` | Where the portal keeps what it writes, see below |

### Scripts for automations

Scripts come only from the scripts directory, never from the interface, and run without a
shell. The directory and every script must be `0755` or stricter and owned by the portal's
user or root:

```sh
chmod 755 ~/.config/home-portal/scripts ~/.config/home-portal/scripts/*.sh
```

A script lies directly in the directory or one subfolder down; hidden paths are ignored. Each
run gets the event's fields as arguments (`args = ["--", "{{service.id}}"]`), as `PORTAL_*`
variables and as JSON on standard input. The automations page shows runs as they happen,
with their output, and can stop them.

## License

[Apache License 2.0](LICENSE).
