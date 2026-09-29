# Webhooks

A webhook is an address, `/webhook/<id>`, that another system calls to tell the portal something
happened: a CI job finished, a camera saw motion, a home automation hub fired. The portal then runs
a script or a [workflow](workflows.md), or passes the call on to your [automations](automations.md).

## Turning it on

Webhooks are on unless you switch them off under **Management → Modules**. They need the
automations module.

## Using it

1. Open **Management → Webhooks** and press **Add webhook**.
2. Give it a title and, if the caller sends data, the **Required variables** (for example `branch`).
3. Choose **What to do on a call**:
   - run its own script or workflow, or
   - publish an event that automations pick up (`webhook.received`).
4. Keep **Require a token** on. The portal shows the token **once**: copy it into the caller.
5. The webhook's page shows its address, an example call, and its last runs.

A call looks like this:

```sh
curl -X POST -H 'Authorization: Bearer <token>' -d '{"branch":"main"}' https://portal.example.com/webhook/<id>
```

## Settings

Webhooks live in `webhooks.toml`. Create them in the interface: it makes the id and the token.

| Key | What it does | Default |
|---|---|---|
| `variables` | Names the call must carry, in its JSON body or query string | none |
| `action` | `script` (its own `run` or `workflow`) or `event` (for automations) | — |
| `run` / `workflow` | What to run, with `{{webhook.<variable>}}` in its arguments or inputs | — |
| `token_sha256` | Only the token's hash is stored; replace the token from the interface | — |
| `enabled` | Switch it off without deleting it | `true` |

## Good to know

- The address is public by design, so keep the token. Without one, anyone who knows the address can
  call it.
- The portal answers `202` when it accepts a call, `401` for a wrong token and `422` when a required
  variable is missing.
- A webhook accepts at most 60 calls a minute and bodies up to 64 KiB.

## See also

- [Automations](automations.md): react to `webhook.received`
- [Workflows](workflows.md) and [Scripts](scripts.md)
- The example: [`examples/split/webhooks.toml`](../../examples/split/webhooks.toml)
