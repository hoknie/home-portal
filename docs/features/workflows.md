# Workflows

A workflow is a chain of steps the portal runs for you: check a service, wait, call an API, run
a script, decide with an `if`, repeat with a `loop`, run branches side by side, send a message.
You draw it on a canvas instead of writing one big script.

## Turning it on

Workflows are off by default. Switch them on under **Management → Modules**. They need the
automations module.

## Using it

- **Management → Workflows** lists them. **Add workflow** starts from a template or from scratch,
  and opens the editor.
- **A workflow's page** (`/admin/workflows/<id>/`) shows it read-only. Its toolbar has **Run**,
  **History**, **Edit** and **Delete**. Click a step to read its settings.
- **The editor** (`…/edit/`): press **+** on an arrow to add a step, click a step to change it
  on the right, and press **Save**. **Cancel** goes back to the workflow's page. The problems
  button lists anything that would stop it from saving.
- **Run** asks for the inputs, starts the workflow and draws the run on the canvas as it goes:
  - finished steps turn green or red;
  - a `wait` counts down its seconds;
  - each step shows the values its settings turned into (switch to **Templates** to see the
    settings as written).
- **History** lists past runs. Pick one to replay it on the canvas, with each step's values and
  log beside it.

Automations and webhooks can run a workflow too, and **Run now** on the list runs one by hand.

## Values in steps

Settings can use values from earlier on, in double braces:
- `{{inputs.service}}` for an input of the workflow;
- `{{steps.ping.status}}` for what an earlier step produced;
- `{{vars.name}}` for a variable set by a `set` step.

Type `{{` in any field to get suggestions.

## Settings

Each workflow is its own file in `workflows/` beside the main configuration file, named after its
id (`workflows/revive.toml`). The editor writes it for you.

| Key | What it does | Default |
|---|---|---|
| `inputs` | What the workflow is given when it starts | none |
| `timeout_seconds` | The longest a whole run may take (1–3600 s) | 300 |
| `enabled` | Switch it off without deleting it | `true` |
| `fail_on_error` (on an `http` or `script` step) | `false` lets the run go on after an error answer or a failing script | `true` |

## Good to know

- By default a script that exits with an error stops the run. In the editor, turn on **Continue
  the run when the script fails** on that step, then check `{{steps.<id>.exit_code}}` in an `if`.
  A script that runs past its own timeout still stops the run.
- A workflow used by an automation, a webhook or another workflow cannot be deleted until they stop
  using it.
- Runs appear in the run journal (`/admin/runs`) as well, with the full trace.

## See also

- [Scripts](scripts.md), [Notifications](notifications.md), [Automations](automations.md)
- Every kind of step, one file each: [`examples/split/workflows/`](../../examples/split/workflows/)
