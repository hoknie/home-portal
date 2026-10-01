# Modules

The optional parts of the portal are modules you can switch on and off. A module that is off
keeps its settings and its page, so you can prepare it first and switch it on when it is ready.

## Using it

Open **Management → Modules**. Each card shows a module, what it is for, and an on/off switch.
A switch takes effect within a few seconds, without restarting the portal, and the menu shows
only the modules that are on. **Configure** on a card opens the module's page.

A switch is locked while another module depends on it: the card says which module to switch
first. If a module's settings are incomplete (for example, the proxy without a portal host), the
switch stays off and the card names what is missing.

The page of a module that is off still opens, with a notice at the top. Its forms keep working,
but actions that need the module running, such as **Run now** on automations, are disabled.

## Settings

All switches live in `[modules]` in the main configuration file:

| Module | What it does | Default | Needs |
|---|---|---|---|
| `proxy` | Publishes services under their own names through Caddy | off | — |
| `dns` | Answers DNS for the names the proxy publishes | off | `proxy` |
| `automations` | Runs your scripts on a schedule or when something happens | on | — |
| `webhooks` | Lets other systems start automations or workflows | on | `automations` |
| `workflows` | Chains of steps started by automations, webhooks or by hand | off | `automations` |
| `users` | Adds, removes and changes users from the browser | off | — |
| `notifications` | Sends announcements, for example to Telegram | on | — |

## Good to know

- Turning a module on while the module it needs is off is refused, both on the page and when
  the portal reads its configuration.
- Switching the proxy on also lets the portal trust Caddy running on the same machine.
- Seeing which modules are on needs the right to read modules, and switching them needs the
  right to change them (see [Users, groups and sign-in](users-and-sign-in.md)). Someone without
  the first still finds, in the menu, every module they may read.

## See also

- [Reverse proxy](reverse-proxy.md) · [Local DNS](local-dns.md) · [Automations](automations.md) ·
  [Webhooks](webhooks.md) · [Workflows](workflows.md) · [Users, groups and sign-in](users-and-sign-in.md) ·
  [Notifications](notifications.md)
- The commented `[modules]` section in [`config/home-portal.example.toml`](../../config/home-portal.example.toml)
