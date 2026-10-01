# Environments and the public page

The same service is often reached at different addresses: `http://192.168.1.10:8096` at home,
`http://10.8.0.10:8096` over the VPN, `https://media.example.com` from outside. Environments let
the portal work out where a visitor is, from their address, and show each service at the address
that works from there. Anyone who is in none of your networks is in the environment `internet`.

## Using it

- Describe your networks once in the main configuration file; the header then shows the visitor's
  environment.
- In a service's form, the **Addresses** table gives each row its environments and address; an
  environment in no row does not see the service: a service whose rows cover only `local` and
  `vpn` is not listed, linked or counted for anyone else.
- Widgets on the home page can be limited to environments in the same way (see
  [Home page](home-page.md)).
- **View the portal as from another environment**: a visitor inside one of your networks can pick
  another environment in the header, to check what, say, a VPN user sees. From the internet this
  is not possible.

## The public page

Without signing in, `/` shows only what you marked `public`: services and widgets meant for guests,
in the same layout. A public service shows its name, icon, group and the address for the visitor's
environment; its status only with `public_status = true`. Nothing else is revealed — not private
services, other environments' addresses, links, notes, user names or secrets.

## Settings

| Key | File | What it does | Default |
|---|---|---|---|
| `[environments.<name>]` `networks` | main file | IP addresses or ranges of that environment | none: everyone is `internet` |
| `addresses` | `services.toml` | The service's address per environment, falling back to `url` | none |
| `environments` | `services.toml`, `dashboard.toml` | Only show it in these environments | every environment |
| `public` | `services.toml`, `dashboard.toml` | Show it on the page for visitors without a session | `false` |
| `public_status` | `services.toml` | Also show the public service's status | `false` |
| `[network]` `trusted_proxies` | main file | Proxies whose `X-Forwarded-For` is believed, so the real visitor's address counts | none |

```toml
[environments.local]
networks = ["192.168.1.0/24"]

[environments.vpn]
networks = ["10.8.0.0/24"]
```

## Good to know

- The name `internet` is reserved; you never configure it.
- Ranges may not overlap: an address belongs to one environment only.
- Behind a reverse proxy, add it to `trusted_proxies`, or every visitor looks like the proxy.
- The portal checks a service at the address its own machine sees, unless `probe.environment`
  names another.

## See also

- [Services and status](services-and-status.md) · [Home page](home-page.md)
- [Reverse proxy](reverse-proxy.md) — published services get an `https://` address per environment.
- [Local DNS](local-dns.md) — each environment gets the right address for the portal's names.
- The commented [`config/home-portal.example.toml`](../../config/home-portal.example.toml).
