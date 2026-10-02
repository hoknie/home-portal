# Home page

The home page at `/` is the one screen your household opens. You arrange it from sections and
widgets: service cards, a status summary, the host's health, the weather, upcoming events, and
widgets of your own built over the data of a workflow or a script. Everyone sees the same layout,
filtered by where they are and whether they are signed in.

## Using it

- **Management → Layout → Widget library** holds every widget the page can show. **Add widget**
  opens a gallery of the types and of ready custom widgets. A built-in widget's **settings** open
  in a window with a live preview (**General**, **Look**, **Access**); **Done** saves it at once.
- A **custom widget** opens in the **widget builder** (`/admin/layout/widgets/edit/?id=<id>`):
  the palette of blocks on the left, the widget itself in the middle at the size you choose, and
  the outline and the selected block's settings on the right. Drag blocks from the palette onto the
  widget or the outline, drop them before, after or into rows and columns (a new row or column
  starts with two empty slots), or press a block in the palette to add it after the selected one.
  On a selected block, Alt with the arrows moves it up, down, out of its group or into the next
  group, Ctrl+D copies it and Delete removes it. Fields that take a template suggest the data's
  paths with their type and a sample, `item` paths inside a list or a table, and the filters; the
  data tree inserts a path where the cursor was. **Data**, **Look** and **Access** are tabs at the
  top; **Save** writes the widget, and a widget opened from the layout returns to it with your
  unsaved arrangement. **Duplicate** copies it as `<id>-2`; a widget can
  be deleted only when no section shows it.
- **Management → Layout** arranges the page from the library. **Add section** creates a section;
  **Add widget** picks a widget from the library, and one widget may sit in several places.
  Drag the handle on a widget's left edge to put it in any free place of any section, leaving
  gaps where you want them; a widget it lands on moves down. Drag its right edge to change its
  width (1 to 12 columns), its bottom edge to change its height (rows of 80 px, or as tall as its
  content), or its corner for both. On the handles the arrow keys do the same, and Page Up and
  Page Down move a widget to the previous or next section. The gear on a widget opens its
  library settings. **Undo** and **Redo** (Ctrl+Z, Shift+Ctrl+Z, Cmd on a Mac) step through every
  change; nothing is written until you press **Save layout**.
- Each widget can be limited to some environments and marked **public** so that visitors without
  an account see it too.
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
| `custom` | Your own blocks over the data of a workflow or a script (see below) |

## Settings

The layout lives in `dashboard.toml` beside the main configuration file: the library defines the
widgets, and the placements put them on the page.

```toml
[[dashboard.library]]
id = "riga"
type = "weather"
public = true
settings = { latitude = 56.95, longitude = 24.11 }

[[dashboard.widgets]]
widget = "riga"
section = "now"
column = 9
row = 1
width = 4
```

| Key | What it does | Default |
|---|---|---|
| `[[dashboard.sections]]` `id`, `title` | A section, drawn top to bottom | one untitled section |
| section `appearance` | `title` (`shown`, `hidden`) and `surface` (`none`, `card`) | shown, none |
| `[[dashboard.library]]` `id`, `type` | A widget: its id and its type from the table above | required |
| `title`, `settings` | Its title, and options of the type such as `latitude`/`longitude` or `url` | depends on type |
| `appearance` | `surface` (`card`, `plain`, `tinted`, `outline`), `accent` (`neutral`, `blue`, `green`, `amber`, `red`, `violet`, `pink`, `teal`), `title` (`shown`, `hidden`), `padding` (`normal`, `compact`, `none`), `align` (`start`, `center`) | a card |
| `environments`, `public` | Who may see it | everyone signed in |
| `[[dashboard.widgets]]` `widget` | The library widget this place shows | required |
| `section` | Which section it belongs to | first section |
| `column`, `row` | Where it starts: column 1 to 12 and row 1 to 200, both or neither | the next free place |
| `width` | Columns of a 12-column row, 1 to 12 (`size` — `quarter`, `third`, `half`, `two-thirds`, `full` — still works) | 12 |
| `height` | `auto`, or 1 to 8 rows of 80 px; content taller than that scrolls | `auto` |

## Good to know

- Without a `[dashboard]` section the page shows a status summary and then all services; the
  first save writes both into the library.
- An older `[[dashboard.widgets]]` entry with a `type` instead of a `widget` keeps working and
  moves into the library, with its comments, on the next save.
- From 1024 px a widget with a `column` and a `row` sits exactly there; the others fill the free
  places in file order. Below that, widgets go in row and column order: a widget of up to 6
  columns takes half the row, and on a phone each takes the full width.
- A widget whose source fails keeps showing its last good data, marked as stale.
- The layout editor keeps the comments you wrote in `dashboard.toml`.

## Custom widgets

A `custom` widget shows **blocks** over **data** that a workflow or a script gathers.

```toml
[[dashboard.library]]
id = "disks"
type = "custom"
title = "Disks"

[dashboard.library.settings]
source = { workflow = "disk-space" }        # or { script = "disks.sh", args = ["/"], timeout_seconds = 30 }
refresh_seconds = 300                       # 30 to 86 400
blocks = [
  { kind = "progress", label = "Used", value = "{{data.used}}", maximum = "{{data.total}}", thresholds = { warning = 75, danger = 90 } },
  { kind = "text", text = "{{data.free}} GB free", muted = true },
  { kind = "button", label = "Clean up", confirm = "Delete old snapshots?", action = { automation = "clean-snapshots" } },
]
```

- **Data.** A workflow gives its declared `outputs` when it ends, or all its variables when it
  declares none (`data.free`); the editor suggests outputs with their descriptions before a run; a script gives its output,
  read as JSON when it is JSON (`data.clients`) and as text otherwise (`data`). The source runs
  only when someone looks at the widget and the data is older than `refresh_seconds`, once for
  everyone, and at most four sources run at once. A failing source keeps the last data, marked
  stale.
- **Blocks:** `stat`, `text`, `markdown`, `list`, `table`, `key-values`, `progress`, `badge`,
  `button`, `divider`, and `row` and `column` to place them side by side or stacked. A row may
  hold columns and a column rows, up to three groups deep. `widths = [8, 4]` shares a row by
  parts, and a row's `align` (`start`, `center`, `end`, `stretch`) places its blocks vertically. Text fields are templates with the workflow filters
  (`{{data.name | upper}}`); in a list or a table, `item` and `index` name each element.
  `widget.id`, `widget.title` and `fetched_at` are available too.
- **Alignment.** Every block but `divider` and `row` takes `align` (`start`, `center`, `end`);
  a block without one takes its column's, then the widget's `appearance.align`.
- **Colours** come from `tone` (`neutral`, `ok`, `info`, `warning`, `danger`), from `thresholds`
  on a number (`direction = "below"` for values that are bad when low), or from `tones`, a table
  of values and tones (`{ up = "ok", down = "danger" }`).
- **Buttons** run an automation (`fields` from the data), a workflow (`inputs` from the data),
  refresh the widget, or open an `http`/`https` link. `confirm` asks first.
- **Safety.** The portal renders the blocks itself: the page receives the texts, never the data,
  the script, the workflow or a secret, and markdown is shown without HTML or images. Saving a
  source or a button that runs something needs the right to run it (`workflows.execute` or
  `automations.execute`) on top of `layout.update`; pressing a button needs the same right.
  A public custom widget shows its texts to visitors, never its buttons.
- **The journal** records every button press and every source run that failed, as a run of the
  widget; successful refreshes are not recorded.
- In the editor, **Data → Run now** runs the source once and lists its data, and the block
  fields suggest its paths. The gallery has ready custom widgets to start from.

## See also

- [Services and status](services-and-status.md) — what the service cards show.
- [Environments](environments.md) — who sees which widget, and the public page.
- [Workflows](workflows.md) and [Automations](automations.md) — what feeds and what buttons run.
- A sectioned layout using every widget type and size, and a custom widget:
  [`examples/split/dashboard.toml`](../../examples/split/dashboard.toml).
