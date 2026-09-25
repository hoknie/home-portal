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

Every tagged version on the releases page carries one archive per platform and a
`SHA256SUMS` over them:

| Archive | For |
|---|---|
| `home-portal_<version>.linux.x86_64.tar.gz` | Linux on x86_64, a static binary (musl) |
| `home-portal_<version>.linux.aarch64.tar.gz` | Linux on arm64 (Raspberry Pi 4/5 on a 64-bit system, ARM servers), static |
| `home-portal_<version>.macos.universal.tar.gz` | macOS 11 or later, Apple silicon and Intel |

```sh
sha256sum --check --ignore-missing SHA256SUMS      # shasum -a 256 -c on macOS
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

```sh
just check              # the whole gate: fmt, clippy -D warnings, tests, web lint, typecheck, tests, build
just run                # the portal on :8080
pnpm --dir web dev      # the interface with live reload, proxying /api to the running portal
just samples            # rewrite the API samples after changing a response shape
just package-macos      # a universal macOS archive in dist/release
just package            # a static Linux archive; TARGET=aarch64-unknown-linux-musl for arm64
```

CI (`.github/workflows/release.yml`) runs the web gate, then fmt, clippy and the Rust tests on
Linux and again on macOS, and builds the three archives on every push and pull request. A tag
`vX.Y.Z` that matches the version in `Cargo.toml` publishes them as a release with
`SHA256SUMS`. The Linux build cross-compiles with
[cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) (`pip install ziglang
cargo-zigbuild`), because rustls and the embedded interface carry C code.

## License

[Apache License 2.0](LICENSE).
