# Scripts

Automations, webhooks and workflows do their real work by running your own scripts: restart a
container, back up a folder, wake a machine. The portal only runs scripts from one folder, and it
checks them before every run.

## Using it

1. Put the script in the scripts folder: `scripts/` beside the main configuration file (on Linux
   packages, `/var/lib/home-portal/scripts`). One level of subfolders is allowed.
2. Make it executable and safe: the folder and the file must not be writable by anyone else,
   for example `chmod 755 scripts scripts/restart.sh`.

3. Pick it in an automation, webhook or workflow step. **Management → Automations** marks a
   script that cannot run and tells you the exact command that fixes it.

## What a script receives

- **Arguments**: whatever you put in `args`, with fields such as `{{service.id}}` filled in.
- **Variables**: every field as `PORTAL_*`, for example `PORTAL_SERVICE_ID`.
- **Standard input**: the event as JSON.

The script runs directly, without a shell, as the portal's user, in the scripts folder.
[`echo-event.sh`](../../examples/split/scripts/echo-event.sh) prints all three and is a good start.

## Describing a script

Comments at the top of a script can say what it does and what it takes. The builders then show
named fields instead of a bare list:

```sh
#!/bin/sh
# @description Restart a container
# @arg service <text> The service id
# @arg --force Skip the health check
```

## Editing scripts in the browser

**Management → Scripts** lets you browse, create, edit, rename and delete scripts. It is off unless
the configuration file says so, because anyone who can sign in could then put code on the host:

| Key | File | What it does | Default |
|---|---|---|---|
| `[scripts] editing` | main file | Allows the Scripts page to change files | `false` |
| `[storage] scripts` | main file | Moves the scripts folder elsewhere | `scripts/` beside the file |

## Good to know

- Hidden files and anything deeper than one subfolder are ignored.
- A script that others can write, or that is owned by someone else, is refused, not run.
- Scripts can only be changed from inside your own networks, never from the internet.

## See also

- [Automations](automations.md), [Webhooks](webhooks.md), [Workflows](workflows.md)
- [Configuration](../CONFIGURATION.md)
