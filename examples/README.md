# Examples

How to build, run and configure the portal as a whole: [the README](../README.md).

Every file here is loaded by `cargo test` (`bin/home-portal/tests/examples.rs`) through every
validator of the portal, so an example that stops working fails the build.

| Path | What it shows |
|---|---|
| `split/` | A complete setup in five files: the main file with the network, environments, the `[proxy]` section, Telegram and `include`; `services.toml`, with services published in every TLS mode and one behind the portal's sign-in; `widgets.toml` with a sectioned layout using every widget type and every size; `automations.toml` with a nightly schedule, a restart when a service goes down, an audit of sign-ins from outside and a script on start-up; `secrets.example.toml` |
| `split/scripts/echo-event.sh` | A POSIX `sh` script that prints the arguments and the `PORTAL_*` variables it received and the event from standard input — a starting point for your own |
| `services/media.toml` | Jellyfin, Plex, qBittorrent, Transmission, Immich |
| `services/home.toml` | Home Assistant, Nextcloud, Grafana |
| `services/network.toml` | A router, Pi-hole, AdGuard Home, a printer and an SSH host probed over `tcp`, a switch probed over `icmp` |
| `services/servers.toml` | Proxmox VE (published with `upstream_verify = false` for its self-signed certificate), Synology DSM |
| `deploy/home-portal.plist` | A launchd agent for macOS, with the Local Network permission it needs |
| `deploy/home-portal.service` | A systemd unit for Linux, with the setting ICMP probes need |
| `deploy/caddy.plist` | A launchd agent for Caddy, the reverse proxy the portal manages |
| `deploy/caddy.service` | A systemd unit for Caddy, allowed to bind ports 80 and 443 |

## Where to start

1. Copy `split/` somewhere of your own, copy `split/secrets.example.toml` to `secrets.toml`,
   run `chmod 600 secrets.toml` and fill in the secrets you use.
2. Add a user: run `home-portal password-hash`, paste the hash into a `[[users]]` entry in
   `home-portal.toml`.
3. Replace the services in `services.toml` with yours; copy entries from `services/`.
4. Start the portal: `HOME_PORTAL_CONFIG=/path/to/home-portal.toml home-portal`.
5. Arrange the home page in the interface under **Management → Layout**, or edit
   `widgets.toml` by hand; both keep your comments.

## Layout in two minutes

- `[[dashboard.sections]]` lists the sections of the home page, top to bottom, each with an
  `id` and an optional `title`.
- Each `[[dashboard.widgets]]` names its `section` (the first one when left out) and a `size`:
  `quarter`, `third`, `half`, `two-thirds` or `full` (the default), a share of a 12-column row.
  Widgets of a section fill rows left to right in file order.
- `environments = ["local"]` shows a widget only there; `public = true` also shows it to
  visitors who are not signed in.

## Publishing through Caddy

The portal can publish services under their own names through [Caddy](https://caddyserver.com)
(2.8 or later). Caddy carries the traffic; the portal writes its whole configuration.

1. Under **Management → Proxy** press **Download the latest version**, then **Start**: the portal downloads
   the latest release, checks its SHA-512, keeps it in `caddy/` beside the configuration (or where `[storage]` says) and keeps
   it running (`proxy.managed = true`), across restarts of the portal too. Or start Caddy yourself
   from `deploy/caddy.plist` or `deploy/caddy.service`; it needs no Caddyfile.
2. Add the `[proxy]` section from `split/home-portal.toml`: `enabled = true`, `portal_host`,
   and `cookie_domain` when services sit behind the portal's sign-in. Add `127.0.0.1` to
   `network.trusted_proxies`, and let the portal listen on `127.0.0.1` only.
3. Give a service a `proxy` table, by hand or under **Publication** in the service form:
   - `host`: the name it is published under;
   - `environments`: where that name is shown (default `["internet"]`);
   - `auth`: from where the portal's sign-in comes first;
   - `tls`: a certificate mode other than the one in `[proxy.tls]`.
4. Point the names at this machine. For Let's Encrypt, forward ports 80 and 443 to it.
5. **Management → Proxy** shows whether Caddy took the configuration, and offers the root
   certificate of Caddy's own authority for hosts in mode `internal`.
   `home-portal proxy render` prints the configuration Caddy is given; `caddy validate`
   checks it.

Anything Caddy should serve besides the portal's services is added the same way: as a
service with a `proxy` table and `probe = { enabled = false }`. A site added to Caddy by
hand is replaced within a minute.

## When a service looks down

Run `home-portal probe <service id>` from the same account and the same terminal (or launchd
agent) the portal runs under. It prints what the probe saw, a diagnosis and what to do. On
macOS the usual cause of every LAN service being down at once is the Local Network permission;
see `deploy/home-portal.plist`.

## Automations

An automation runs a script of yours on this host when something happens: on a cron schedule,
when a service goes down or comes back, when someone signs in, when a service or the
configuration changes, when the portal starts or stops, when a webhook is called — or only when
you press **Run now**. Build one under
**Management → Automations**, or copy an entry from `split/automations.toml`.

1. Create `scripts/` beside the main configuration file (or where `[storage]` puts it) and
   put executable scripts in it:
   `chmod 755 scripts scripts/*.sh`. The portal refuses a script that group or others can
   write, or that sits in such a directory; under a umask of `0002` a copy or a checkout gives
   `0775`, so check the modes.
2. Arguments come from the event: `args = ["--", "{{service.id}}"]`. Each item is one argument,
   and no shell is involved, so a value is never split or run. Put `--` before placeholders so a
   value that starts with `-` is not taken for an option, and quote `"$1"` in the script.
3. The script also receives every field as a `PORTAL_*` variable and the whole event as JSON on
   standard input; `split/scripts/echo-event.sh` prints all three.
4. A script lies directly in `scripts/` or in one of its subfolders, never deeper, and hidden
   files and folders are ignored. The interface explains every script it cannot run, with the
   command that fixes it.
5. **Webhooks** (**Management → Webhooks**) give other systems an address `POST /webhook/<id>`:
   with a token the portal generates and shows once, required variables from the JSON body or
   the query string, and either a script of their own or the event `webhook.received` for
   automations. `split/automations.toml` has one of each.
6. Scripts run as the portal's user and inherit its permissions — see the notes in
   `deploy/home-portal.plist` and `deploy/home-portal.service`.
