# Network and restart

Where the portal listens — its IP address and port — is set in the configuration or from the
browser. New network settings take effect after a restart, which you can start from the
interface.

## Using it

- **Management → Network** shows the address the portal listens on now, your saved settings, and
  the machine's network interfaces. Change the IP address or port and save.
- When the saved settings differ from what is in use, the page says a restart is required and
  offers **Restart now**.
- **Restart portal** is also in the menu with your name. The page is unavailable for a few
  seconds, then reloads by itself — at the new address, if you changed it. If the portal has not
  answered after a minute, the page says so and lets you try again.

## Settings

| Key | File | What it does | Default |
|---|---|---|---|
| `address` | `[network]` in the main file | The IP address to listen on: `0.0.0.0` for every interface, `127.0.0.1` for this machine only | `127.0.0.1` |
| `port` | `[network]` in the main file | The port to listen on | `8080` |
| `public_url` | `[network]` in the main file | How people open the portal, for example `https://portal.home.lan` | not set |
| `trusted_proxies` | `[network]` in the main file | Proxies whose forwarded client address is believed | none |
| `HOME_PORTAL_ADDRESS` | environment | `ip:port` that overrides `address` and `port` | not set |

## Good to know

- While `HOME_PORTAL_ADDRESS` is set, the page says so, and your saved address only applies once
  the variable is removed.
- A restart first runs the scripts of automations on `portal.stopping`, and stops scripts still
  running after 10 seconds.
- If the address cannot be used (already taken, or not an address), the portal refuses to start
  and names it.
- On macOS, reaching devices on your home network needs the Local Network permission; see the
  note in [`examples/deploy/home-portal.plist`](../../examples/deploy/home-portal.plist).

## See also

- [Reverse proxy](reverse-proxy.md) · [Users, groups and sign-in](users-and-sign-in.md) ·
  [Configuration](../CONFIGURATION.md) · [Install](../INSTALL.md)
