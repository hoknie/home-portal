# Local DNS

The portal can answer DNS for the names it publishes, so `portal.home` and `jellyfin.home`
resolve without you editing your router's records by hand. Each network gets the proxy's
address in that network: at home the LAN address, over the VPN the VPN address.

## Turning it on

Local DNS is an optional module, off by default, and it needs the
[reverse proxy](reverse-proxy.md). Switch it on under **Management → Modules**, or with
`dns = true` in `[modules]`.

## Using it

1. Open **Management → DNS**. Under the settings, add your zone (for example `home`). Then check
   the table of names and the answer each environment gets. Each environment uses the address
   of this machine in that network, unless you set **The proxy's address per environment**.
2. The server listens on port 53. On Linux that needs a capability: run
   `systemctl edit home-portal` and add `AmbientCapabilities=CAP_NET_BIND_SERVICE`, or pick
   another port.
3. Tell your router to forward the zone to the portal:
   - in dnsmasq or Pi-hole: `server=/home/192.168.1.60`;
   - in a FRITZ!Box, pfSense or OPNsense: a domain override.

   For a VPN, set the portal as the VPN's DNS server.

Answers follow changes to services and the proxy within seconds, without a restart.

## Settings

| Key | File | What it does | Default |
|---|---|---|---|
| `zones` | `dns.toml` | Domains the server answers for, such as `["home"]` | none |
| `port` | `dns.toml` | Plain DNS over UDP and TCP | `53` |
| `addresses` | `dns.toml` | The proxy's address per environment, e.g. `local = "192.168.1.60"` | from this host's interfaces |
| `tls.enabled` | `dns.toml` | DNS over TLS, on port 853 | off |
| `https.enabled` | `dns.toml` | DNS over HTTPS at `https://<portal host>/dns-query` | off |
| `[[dns.records]]` | `dns.toml` | Extra names of your own (`A`, `AAAA`, `CNAME`, `TXT`) | none |

## Good to know

- Only your own environments get answers. Everyone else is refused, and so is every name
  outside your zones and published hosts. Nothing is forwarded to other servers: this is
  not a resolver for the rest of the internet.
- Phones can use DNS over TLS (the host shown on the DNS page, for Android's Private DNS) or
  DNS over HTTPS (`https://<portal host>/dns-query`).
- Android's Private DNS only accepts a certificate the phone trusts. Use a host with ACME
  (Let's Encrypt) there, not Caddy's local authority.
- An environment with no address gets no answers. The DNS page warns about it.

## See also

- [Reverse proxy](reverse-proxy.md): the names this server answers for.
- [Environments](environments.md): how the portal tells your networks apart.
- [`config/home-portal.example.toml`](../../config/home-portal.example.toml): the `[dns]`
  section with every key, commented.
