# Home page

The home page at `/` is the one screen your household opens. You arrange it from sections and
widgets: service cards, a status summary, the host's health, the weather and upcoming events.
Everyone sees the same layout, filtered by where they are and whether they are signed in.

## Using it

- Open **Management → Layout** to arrange the page. **Add section** and **Add widget** create new
  parts; drag widgets to reorder them or move them between sections, and drag the handle on a
  widget's right edge to resize it.
- Each widget can be limited to some environments and marked **public** so that visitors without
  an account see it too.
- **Preview** shows the page as a chosen environment sees it, with or without a session, before
  you save. Nothing is written until you press **Save layout**.
- With a session, a service card opens the service's page; a separate button opens the service
  itself.

## Widgets

| Type | Shows |
|---|---|
| `status-summary` | How many services are up, degraded, down… |
| `services` | Service cards, optionally only some groups (`settings.groups`) |
| `host-metrics` | Processor, memory, disks and uptime of the machine running the portal |
| `weather` | The forecast from Open-Meteo (no account or key needed) for a latitude and longitude |
| `calendar` | Upcoming events from any `.ics` calendar link |

## Settings

The layout lives in `dashboard.toml` beside the main configuration file.

| Key | What it does | Default |
|---|---|---|
| `[[dashboard.sections]]` `id`, `title` | A section, drawn top to bottom | one untitled section |
| `[[dashboard.widgets]]` `type` | The widget type from the table above | required |
| `id` | Needed by `host-metrics`, `weather` and `calendar` | none |
| `section`, `size` | Where it goes, and its width: `quarter`, `third`, `half`, `two-thirds` or `full` | first section, `full` |
| `environments`, `public` | Who may see it | everyone signed in |
| `settings` | Options of the type, such as `latitude`/`longitude` or `url` | depends on type |

## Good to know

- Without a `[dashboard]` section the page shows a status summary and then all services.
- Widgets fill a section's rows left to right in file order; on a phone each takes the full width.
- A widget whose source fails keeps showing its last good data, marked as stale.
- The layout editor keeps the comments you wrote in `dashboard.toml`.

## See also

- [Services and status](services-and-status.md) — what the service cards show.
- [Environments](environments.md) — who sees which widget, and the public page.
- A sectioned layout using every widget type and size:
  [`examples/split/dashboard.toml`](../../examples/split/dashboard.toml).
