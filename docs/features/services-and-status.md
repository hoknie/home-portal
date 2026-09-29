# Services and their status

A service is anything on your network you want to open and keep an eye on: a media server, the
router, a NAS. The portal checks each one in the background and tells you whether it is up, slow,
down, or could not be checked at all — and keeps 30 days of history so you can see how it behaved.

## Using it

- **Management → Services** lists every service. **Add service** opens a form with the name, the
  address, a group, an icon and how to check it; editing works the same way.
- Every service is checked on its own schedule. The state is one of **up**, **degraded** (answered,
  but slowly), **down**, **unreadable** (the check itself could not run, for example the name did
  not resolve) or **unknown** (not checked yet, or checking is off).
- Click a service card on the home page to open its **service page**: status and since when, its
  address in every environment, the probe settings, uptime for 24 hours, 7 and 30 days, a latency
  chart, a timeline of outages, your links and notes. **Check now** runs a check right away.
- When a check fails, the page explains why in plain words (connection refused, timeout, name not
  resolved, TLS problem, local network access denied…) and what to do about it.

## Settings

Services live in `services.toml` beside the main configuration file.

| Key | What it does | Default |
|---|---|---|
| `id`, `name`, `url` | Identifier, name shown, and the address to open and check | required |
| `group`, `icon`, `description` | How the card is grouped and drawn | none |
| `probe.kind` | `http` (a web request), `tcp` (open a port) or `icmp` (ping) | `http` |
| `probe.every_seconds`, `probe.timeout_seconds` | How often to check, and how long to wait | 30, 5 |
| `probe.degraded_after_milliseconds` | Slower than this counts as degraded | 1500 |
| `links`, `notes` | Extra links and markdown notes on the service page | none |

```toml
[[services]]
id = "nas"
name = "NAS"
url = "ssh://nas.home.lan"
icon = "server"
probe = { kind = "tcp", port = 22 }
```

## Good to know

- **Icons** can be a Lucide name (`film`), `auto` (the portal finds the service's own icon),
  `catalog:<name>` from the [dashboard-icons](https://github.com/homarr-labs/dashboard-icons)
  catalogue, `url:…` or `file:…`. Fetched icons are cached, so your browser never talks to the
  service just to draw its icon.
- Check a service from a terminal with `home-portal probe <id>` (or a URL, with `--kind tcp|icmp`):
  it runs the same check and prints the same explanation.
- On macOS, every local service showing **down** at once usually means the portal was not allowed
  to use the local network; `home-portal probe` confirms it.
- A failing check backs off, up to one check every 5 minutes, and returns to normal after the
  first success.

## See also

- [Environments](environments.md) — a different address from home, over VPN and from outside.
- [Home page](home-page.md) — where the service cards appear.
- [Notifications](notifications.md) — a message when a service goes down.
- [Reverse proxy](reverse-proxy.md) — publish a service under its own name.
- Ready-made entries for Jellyfin, Plex, Home Assistant, Pi-hole, Proxmox and more:
  [`examples/services/`](../../examples/services/).
