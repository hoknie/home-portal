# Configuration

The portal reads a small main file and keeps everything else in files of their own beside it.
You rarely need to edit them by hand: the pages under **Management** write them for you, and
keep your comments. When you do edit a file, [`config/home-portal.example.toml`](../config/home-portal.example.toml)
explains every setting in its comments, and [`examples/split/`](../examples/split/) is a complete
setup to copy from.

## Where the files are

The main file is `~/.config/home-portal/home-portal.toml` (under `$XDG_CONFIG_HOME` when that is
set). The packages use `/etc/home-portal/home-portal.toml`; `HOME_PORTAL_CONFIG` points anywhere
else.

The main file holds only the short settings: `[interface]`, `[network]`, `[environments.*]`,
`[modules]`, `[storage]` and `[scripts]`. Each other part lives in its own file in the same folder:

| File | What it holds | Read more |
|---|---|---|
| `services.toml` | `[[services]]` | [Services and their status](features/services-and-status.md) |
| `dashboard.toml` | the home page's sections and widgets | [The home page](features/home-page.md) |
| `users.toml` | `[[users]]` | [Users and sign-in](features/users-and-sign-in.md) |
| `proxy.toml` / `dns.toml` | `[proxy]` / `[dns]` | [Reverse proxy](features/reverse-proxy.md), [Local DNS](features/local-dns.md) |
| `automations.toml` / `webhooks.toml` | `[[automations]]` / `[[webhooks]]` | [Automations](features/automations.md), [Webhooks](features/webhooks.md) |
| `workflows/` | one file per workflow, named after its id | [Workflows](features/workflows.md) |
| `notifications.toml` | `[notifications]` | [Notifications](features/notifications.md) |
| `secrets.toml` | `[secrets]`, see below | |

A file that does not exist yet is simply empty; the first thing you save creates it. If you put
a section in the main file anyway, the portal moves it to its own file when it starts, keeps its
comments, and leaves a `.previous` copy. `[files]` moves a part elsewhere inside the folder
(for example `workflows = "flows/"`).

## The short settings

| Setting | What it does | Read more |
|---|---|---|
| `[network]` `address`, `port` | Where the portal listens; `HOME_PORTAL_ADDRESS=ip:port` overrides it | [Network and restart](features/network-and-restart.md) |
| `[environments.<name>]` `networks` | Which networks count as "home", "vpn"…; everyone else is the internet | [Environments](features/environments.md) |
| `[modules]` | Switches the optional parts on and off | [Modules](features/modules.md) |
| `[interface]` `default_language` | `en`, `ru` or `es` when the visitor has not chosen one | |
| `[scripts]` `editing` | Allows editing scripts in the browser; off unless set here | [Scripts](features/scripts.md) |
| `[storage]` | Where the portal keeps what it writes, see below | |

`HOME_PORTAL_WEB` names the interface folder when it is not beside the binary or in
`../share/home-portal/web`.

## Secrets

Tokens and passwords go in `secrets.toml`, as a `[secrets]` table in a file of mode `0600`
(start from [`config/secrets.example.toml`](../config/secrets.example.toml)). A setting names the
key, never the value, and the portal refuses to start when anyone else can read the file.

## Scripts for automations

Automations, webhooks and workflows run scripts only from the scripts folder, never from
anywhere else. The folder and every script in it must be `0755` or stricter and owned by the
portal's user or root:

```sh
chmod 755 ~/.config/home-portal/scripts ~/.config/home-portal/scripts/*.sh
```

How a script receives its arguments and describes them: [Scripts](features/scripts.md).

## What the portal writes

The session list, cached icons, status history, the automations' run journal and Caddy's files
lie beside the main file, next to the `scripts/` folder your automations run from. `[storage]`
moves them: `directory` moves all of them, and a key of its own (`sessions`, `icons`, `history`,
`automations`, `scripts`, `caddy`) moves only that one. The storage and file settings are read
at start, so restart after changing them.
