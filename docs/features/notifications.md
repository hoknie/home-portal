# Notifications

The portal can message you when a service goes down and when it comes back. Workflows can send
their own messages through the same channels. Today the channel is Telegram.

## Turning it on

The notifications module is on by default (**Management → Modules**). Nothing is sent until a
channel is set up.

## Using it

1. Create a Telegram bot with [@BotFather](https://t.me/BotFather) and get its token and your
   chat id.
2. Put the token in the secrets file (see [Configuration](../CONFIGURATION.md)).
3. Open **Management → Notifications**, fill in the Telegram channel and switch it on.
4. Choose which changes are announced, then send a test message from the same page.

The page also shows the last delivery, the last error and how many messages are waiting.

## Settings

Notifications live in `notifications.toml`.

| Key | What it does | Default |
|---|---|---|
| `[notifications] states` | Which states are announced when a service reaches them | `["down", "unreadable"]` |
| `[notifications] recovered` | Also announce when a service is back up | `true` |
| `[notifications.telegram] enabled` | Switch the Telegram channel on | `false` |
| `[notifications.telegram] secret` | The name of the bot token in the secrets file, never the token itself | — |
| `[notifications.telegram] chat_id` | Where the messages go | — |
| `notify` (on a service) | `false` keeps one service silent | `true` |

## Good to know

- A slow or broken channel never slows down probing or the pages: messages wait in a queue and are
  retried a few times.
- Right after the portal starts, a service's first state is only announced if it is a failure.
- In a [workflow](workflows.md), a `notify` step sends any text you like to one channel or all.

## See also

- [Services and status](services-and-status.md)
- [Workflows](workflows.md)
- The example: [`examples/split/notifications.toml`](../../examples/split/notifications.toml)
