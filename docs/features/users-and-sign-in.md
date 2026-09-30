# Users, groups and sign-in

Only the people listed in your configuration can sign in. Each of them signs in with a name and a
password; the portal keeps only a hash of the password, never the password itself. What each
person may do then depends on their **group**.

## Turning it on

Signing in always works. The **Users** module only decides whether people and groups can be
added, changed and removed from the browser. It is off by default: switch it on under
**Management → Modules** (see [Modules](modules.md)), or add `users = true` to `[modules]`.

## Using it

- **The first user** comes with the macOS, deb and rpm packages, which write one called `admin`,
  in the `admin` group, with a random password. To add one by hand, run
  `home-portal password-hash`, type a password, and paste the hash into `users.toml`:

  ```toml
  [[users]]
  name = "admin"
  password_hash = "$argon2id$..."
  group = "admin"
  ```

- **Management → Users** lists everyone with their group, with you marked "you". With the module
  on:
  - **Add user** asks for a name, a group and the password twice (8 characters or more);
  - **Change group** moves someone to another group, or to none;
  - **Change password** sets a new one, and signs that person out everywhere else;
  - **Delete** removes someone after a confirmation.
- **The Groups tab** lists `admin` and your groups with their members. Choosing one shows its
  rights as a table. Members of `admin` can add, edit and delete groups there: the editor is a
  table of areas and actions to tick.
- **Sign out** is in the menu with your name.

## Settings

| Key | File | What it does | Default |
|---|---|---|---|
| `[[users]]` `name` | `users.toml` | The name people sign in with | — |
| `[[users]]` `password_hash` | `users.toml` | The argon2id hash from `home-portal password-hash` | — |
| `[[users]]` `group` | `users.toml` | `admin` or the name of one of your groups | none |
| `[[groups]]` `name` | `users.toml` | A group's name; `admin` is taken | — |
| `[[groups]]` `permissions` | `users.toml` | Rights per area, for example `{ automations = ["read", "execute"] }` | none |
| `users` | `[modules]` in the main file | Lets the browser add, change and remove users and groups | off |
| `public_url` | `[network]` in the main file | When it starts with `https`, the session cookie is sent over HTTPS only | not set |

The action names in the file are `read`, `create`, `update`, `delete` and `execute`. The areas
are `services`, `layout`, `network`, `modules`, `scripts`, `secrets`, `host-permissions`,
`portal`, `proxy`, `dns`, `automations`, `webhooks`, `users`, `workflows` and `notifications`. A
name or action the area does not have stops the portal at start, naming the line.

```toml
[[groups]]
name = "family"
permissions = { services = ["update"], automations = ["read", "execute"], workflows = ["read", "execute"] }
```

## Good to know

- **Only members of `admin`** can do these things:
  - change groups;
  - put someone into `admin`;
  - change the password of a member of `admin`, or delete one.
- **Anyone else with the users rights** may give only a group whose rights are all within their
  own.
- **You cannot delete** yourself, the last user or the last member of `admin`. The last member of
  `admin` cannot leave it either.
- **A group with members cannot be deleted:** move them to another group first.
- **Wrong passwords:** after 5 of them within 15 minutes, sign-ins from that address wait a minute.
- **Behind the [reverse proxy](reverse-proxy.md),** one sign-in covers every service published
  under the proxy's cookie domain, whatever the person's group.
- **Automations, workflows and webhook calls** run as the portal itself. Groups decide only who may
  start or change them from the browser.

## See also

- [Modules](modules.md) · [Reverse proxy](reverse-proxy.md) · [Configuration](../CONFIGURATION.md)
- [Install](../INSTALL.md), for where the first password is written and what to add when upgrading
