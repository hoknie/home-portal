# Reverse proxy

Publish your services under names of their own, such as `https://media.home.example.com`,
with real certificates, through [Caddy](https://caddyserver.com). The portal writes Caddy's
whole configuration for you, and can ask people to sign in to the portal before they reach a
service.

## Turning it on

The proxy is an optional module, off by default. Switch it on under **Management → Modules**,
or with `proxy = true` in the `[modules]` section. It needs a portal address (`portal_host`)
before it does anything.

Caddy itself can come from two places:
- **Let the portal run it.** On **Management → Proxy**, the **Caddy** block downloads Caddy
  (the archive is checked against its release checksum), then starts and stops it.
- **Run it yourself** with the launchd or systemd file in `examples/deploy/`. Add nothing to
  Caddy's own configuration by hand: the portal replaces it.

## Using it

1. On **Management → Proxy**, fill in **Portal address** and, if some services need
   sign-in, **Sign-in domain**. Pick the **Default certificate**.
2. On **Management → Services**, edit a service. Under **Addresses**, add a row with the
   environments it is for, `https://<published name>` and **Through the proxy** checked; choose
   **Portal sign-in** if visitors from there must sign in first. Caddy forwards that name to the
   service's main address, the first row. **Proxy settings** holds a certificate for that name
   alone.
3. The **Published addresses** table on the proxy page lists every host. The badge tells you
   whether Caddy is reachable and holds the current configuration, and **Apply now** reloads it.

Changes take effect at once, without restarting the portal.

## Settings

| Key | File | What it does | Default |
|---|---|---|---|
| `portal_host` | `proxy.toml` | The name the portal itself is published under | required |
| `cookie_domain` | `proxy.toml` | One sign-in covers every host under this domain | none |
| `tls.mode` | `proxy.toml` | Certificates: `acme` (Let's Encrypt), `internal` (Caddy's own authority) or `files` | `acme` |
| `https_port` / `http_port` | `proxy.toml` | Where Caddy listens | `443` / `80` |
| `proxy.host` | `services.toml` | A service's published address | not published |
| `proxy.auth` | `services.toml` | Environments that must sign in first, e.g. `["internet"]` | `[]` |

```toml
[[services]]
id = "media"
url = "http://192.168.1.10:8096"
proxy = { host = "media.home.example.com", auth = ["internet"] }
```

## Good to know

- Let's Encrypt (`acme`) checks your domain from outside: forward ports 80 and 443 to Caddy.
- For names that exist only at home, use `internal`. Then download the **Root certificate**
  from the proxy page and install it on your devices as trusted, so browsers stop warning.
- Use a subdomain of your own for `cookie_domain`, not your whole domain: every host under it
  receives the sign-in cookie.
- `home-portal proxy render` prints the configuration Caddy is given.

## See also

- [Users, groups and sign-in](users-and-sign-in.md): who can sign in.
- [Local DNS](local-dns.md): make the published names resolve at home.
- [Services and status](services-and-status.md): adding the services you publish.
- [`examples/split/proxy.toml`](../../examples/split/proxy.toml) and
  [`examples/split/services.toml`](../../examples/split/services.toml): a full setup with every
  certificate mode.
- [`examples/deploy/`](../../examples/deploy/): launchd and systemd files for Caddy.
