# Automations

An automation runs one of your scripts, or a [workflow](workflows.md), when something happens:
on a schedule, when a service goes down, when someone signs in, or when you press a button.

## Turning it on

Automations are on unless you switch them off under **Management → Modules** (or with
`automations = false` in `[modules]`). Webhooks and workflows need them.

## Using it

1. Put a script in the scripts folder (see [Scripts](scripts.md)).
2. Open **Management → Automations** and press **Add automation**.
3. Choose **when** it runs: an event (for example "a service changes state") and, if you like,
   filters such as which services or which states.
4. Choose **what** it does: a script with its arguments, or a workflow with its inputs.
5. Save. **Run now** tries it at once with sample values.

Every run lands in the **Run journal** (**Management → Run journal**, `/admin/runs`): when it
started, what started it (with a link to it), how it ended, and what the script printed. A run that
is still going can be followed and stopped from there. The journal shows 50 runs a page; **Older**
and **Newer** move through the rest, and only the newest page refreshes itself. The automations
table names what each automation does in its **Action** column, linked to the script or workflow.

## Settings

Automations live in `automations.toml` beside the main configuration file.

| Key | What it does | Default |
|---|---|---|
| `when` | The event and its filters, e.g. `{ event = "schedule", cron = "30 3 * * *" }` | — |
| `run` | The script, its `args` and `timeout_seconds` | timeout 60 s |
| `workflow` / `inputs` | Run a workflow instead of a script | — |
| `cooldown_seconds` | Minimum pause between two runs of this automation | 0 |
| `enabled` | Switch one automation off without deleting it | `true` |
| `[automation_settings] timezone` | Time zone for schedules | the host's |

```toml
[[automations]]
id = "restart-jellyfin"
title = "Restart Jellyfin when it goes down"
when = { event = "service.status-changed", services = ["jellyfin"], to = ["down"] }
run = { script = "restart.sh", args = ["--", "{{service.id}}"] }
```

## Good to know

- `{{service.id}}` and other event fields in `args` are filled in for each run; the builder lists
  the fields of the chosen event.
- At most 4 scripts run at once, one automation never runs twice at the same time, and each one
  starts at most 60 times an hour, so a script that triggers itself cannot run away.
- The journal keeps the last 200 runs.

## See also

- [Scripts](scripts.md): where scripts live and what they receive
- [Workflows](workflows.md): several steps with conditions instead of a single script
- [Webhooks](webhooks.md) and [Notifications](notifications.md)
- The full example: [`examples/split/automations.toml`](../../examples/split/automations.toml)
