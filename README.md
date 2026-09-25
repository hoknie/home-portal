# home-portal

A self-hosted start page for a home network. It shows whether your services are up, how they
have behaved over the last 30 days, and lets you manage them — all from one small binary.

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
- **Three interface languages** — English, Russian and Spanish.

It is one process: a Rust server (axum + tokio) with the web interface (Next.js, static export)
embedded in the binary.

## Install a release

Every tagged version on the releases page carries a package and an archive per platform,
and a `SHA256SUMS` over all of them. Check what you downloaded first:

```sh
sha256sum --check --ignore-missing SHA256SUMS      # shasum -a 256 -c on macOS
```

| File | For |
|---|---|
| `home-portal_<version>.macos.universal.pkg` | macOS 11 or later, Apple silicon and Intel: installs and starts everything |
| `home-portal_<version>.debian.amd64.deb` / `.debian.arm64.deb` | Debian, Ubuntu, Raspberry Pi OS (64-bit) and relatives |
| `home-portal_<version>.el.x86_64.rpm` / `.el.aarch64.rpm` | RHEL, AlmaLinux, Rocky, Fedora and relatives |
| `home-portal_<version>.linux.x86_64.tar.gz` / `.linux.aarch64.tar.gz` | Any Linux, a static binary to place by hand |
| `home-portal_<version>.macos.universal.tar.gz` | macOS, the binary to place by hand |

### macOS

Open the `.pkg`. It installs `/usr/local/bin/home-portal` and sets the portal up for the user
signed in: a configuration in `~/Library/Application Support/home-portal`, the user **admin**
with a random password in `initial-password` beside it, and a LaunchAgent that starts the
portal at every login. The last page of the installer says the same; then open
http://127.0.0.1:8080.

When every service on the local network shows as down, allow **home-portal** in System
Settings → Privacy & Security → Local Network. Another user of the Mac sets the portal up with
`/usr/local/libexec/home-portal/setup`; `sudo /usr/local/libexec/home-portal/uninstall
[--purge]` removes it.

### Debian, Ubuntu, RHEL and relatives

```sh
sudo apt install ./home-portal_<version>.debian.amd64.deb     # or
sudo dnf install ./home-portal_<version>.el.x86_64.rpm
sudo cat /etc/home-portal/initial-password                   # the password of admin
```

The package makes the system user `home-portal`, writes `/etc/home-portal/home-portal.toml`
with the user **admin** and a random password, keeps the data in `/var/lib/home-portal` and
starts the systemd service `home-portal` on http://127.0.0.1:8080. Scripts for automations go
into `/var/lib/home-portal/scripts` (owned by root, 0755). An upgrade keeps the configuration;
removing the package keeps it too (`apt purge` deletes it).

To reach the portal from other machines, set `[network] address` to `0.0.0.0` or the host's
address and restart it: `sudo systemctl restart home-portal`, or on a Mac
`launchctl kickstart -k gui/$(id -u)/lan.home.portal`.

### By hand

```sh
tar -xzf home-portal_<version>.linux.x86_64.tar.gz
cd home-portal_<version>.linux.x86_64
cp config/home-portal.example.toml config/home-portal.toml
./home-portal password-hash                        # add the hash as a [[users]] entry
HOME_PORTAL_CONFIG=config/home-portal.toml ./home-portal
```

Each archive holds the binary, the example configuration, `examples/`, and the systemd or
launchd files for the portal and Caddy.

## Build from source

Requirements: Rust 1.98.1 (pinned by `rust-toolchain.toml`), Node.js 24 and pnpm 11 for the
interface, and [`just`](https://github.com/casey/just).

```sh
cp config/home-portal.example.toml config/home-portal.toml
cargo run -p home-portal -- password-hash      # type a password, copy the hash
```

Add a user to `config/home-portal.toml` — the portal will not start without one:

```toml
[[users]]
name = "admin"
password_hash = "$argon2id$..."
```

Then build and run:

```sh
just run            # serves http://127.0.0.1:8080 using config/home-portal.toml
```

For a release binary with the interface embedded: `just build`, then
`target/release/home-portal` with `HOME_PORTAL_CONFIG=/path/to/home-portal.toml`.

## Configuration

Everything is set in one TOML file. Start from
[`config/home-portal.example.toml`](config/home-portal.example.toml), which explains every
section in its comments, and look at [`examples/`](examples/README.md) for a complete setup split
over several files, ready-made entries for common home software (Jellyfin, Plex, Home Assistant,
Pi-hole, Proxmox…), automations with a sample script, and launchd and systemd files.

| Setting | What it does |
|---|---|
| `HOME_PORTAL_CONFIG` | Path to the main file; `./home-portal.toml` by default (`just run` uses `config/home-portal.toml`) |
| `HOME_PORTAL_ADDRESS=ip:port` | Overrides `[network]` address and port |
| `include = [...]` | Further files read after the main one, inside its directory |
| `[secrets]` | Tokens and passwords, in a file of mode 0600 (see [`config/secrets.example.toml`](config/secrets.example.toml)); settings name a key, never the value |
| `configuration.writes_to` | Which file the interface edits |
| `[storage]` | Where the portal keeps what it writes, see below |

### Where the portal keeps its data

The portal writes `sessions.json`, `icons/`, `history/`, `automations/` (the run journal) and
`caddy/`, and runs automation scripts from `scripts/`. By default all of them lie beside the
main configuration file. `[storage]` moves them; relative paths are relative to the
configuration file's directory:

```toml
[storage]
directory = "../env"                        # everything goes here...
scripts = "/usr/local/libexec/home-portal"  # ...except what is named on its own
```

The keys are `directory`, `sessions`, `icons`, `history`, `automations`, `scripts` and `caddy`.
They are read at start, so restart the portal after changing them. This repository keeps the
configuration in `config/` and the data in `env/`.

### Scripts for automations

Scripts come only from the scripts directory, never from the interface, and run without a
shell. The directory and every script must be `0755` or stricter and owned by the portal's user
or root; a checkout under a umask of `0002` gives `0775`, which the portal refuses:

```sh
chmod 755 env/scripts env/scripts/*.sh
```

A script lies directly in the directory or one subfolder down; hidden paths are ignored. Each
run gets the event's fields as arguments (`args = ["--", "{{service.id}}"]`), as `PORTAL_*`
variables and as JSON on standard input.

## Command line

```sh
home-portal                          # serve
home-portal password-hash            # hash a password for a [[users]] entry
home-portal probe <id|url> [--kind http|tcp|icmp]   # one probe with its diagnosis
home-portal proxy render             # print the configuration Caddy is given
```

When every LAN service looks down at once on macOS, the cause is usually the Local Network
permission; `home-portal probe` run from the same context as the portal shows it. See
[`examples/deploy/home-portal.plist`](examples/deploy/home-portal.plist).

## Development

Every task is a `just` recipe; `just` alone lists them by section. The recipes live in
`env/justice/`, one file per section, and the root `justfile` imports them.

| Section | Recipe | What it does |
|---|---|---|
| quality | `just check` | The whole gate, in CI's order: fmt, clippy, Rust tests, web lint, typecheck, web tests, web build |
| | `just fmt` / `just fmt-check` | Format the Rust code, or only check it |
| | `just clippy` | Lint the Rust code, warnings are errors |
| | `just test` | Every Rust test in the workspace |
| | `just samples` | Rewrite the API samples after changing a response shape |
| web | `just web` | Build the interface into `web/out`, which the binary embeds |
| | `just web-lint` / `just web-typecheck` / `just web-test` | Lint and layers, typecheck, tests of the interface |
| | `just web-install` | Install the interface's dependencies exactly as locked |
| build | `just build` | The interface, then the release binary that embeds it |
| run | `just run` | The portal on :8080 with `config/home-portal.toml` (`HOME_PORTAL_CONFIG` changes it) |
| | `just dev` | How to run the interface with live reload next to the portal |
| package | `just package-macos` | A universal macOS binary, its archive and the `.pkg`, checked, in `dist/release` |
| | `just package-linux` | Both static Linux archives, x86_64 and aarch64 |
| | `just package` | One static Linux archive; `TARGET=aarch64-unknown-linux-musl` for arm64 |
| | `just package-deb` / `just package-rpm` | The deb (in `debian:12`) or the rpm (in `almalinux:9`) of a built binary, through Docker |
| | `just package-check deb` / `rpm` | Install it on a clean system, run it, sign admin in, remove it |
| release | `just version` | The version in `Cargo.toml`, the one a release tag must name |
| | `just check-tag vX.Y.Z` | Check a tag against that version |
| | `just checksums` | `SHA256SUMS` over the archives in `dist/release` |

For the interface with live reload, run `just run` in one terminal and `pnpm --dir web dev` in
another; it proxies `/api` to the running portal.

CI (`.github/workflows/release.yml`) runs the web gate, then fmt, clippy and the Rust tests on
Linux and again on macOS, and on every push and pull request builds the archives, the deb and
the rpm for both architectures and the macOS `.pkg`, and installs the x86_64 deb and rpm and the
`.pkg` setup to check that admin signs in. A tag
`vX.Y.Z` that matches the version in `Cargo.toml` publishes them as a release with
`SHA256SUMS`; check the tag first with `just check-tag vX.Y.Z`. The Linux build cross-compiles with
[cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) (`pip install --requirement
packaging/requirements.txt`), because rustls and the embedded interface carry C code.

## License

[Apache License 2.0](LICENSE).
