# Users and sign-in

Only the people listed in your configuration can open the management pages. Each of them signs
in with a name and a password; the portal keeps only a hash of the password, never the password
itself.

## Turning it on

Signing in always works. The **Users** module only decides whether people can be added, removed
and given new passwords from the browser. It is off by default: switch it on under
**Management → Modules** (see [Modules](modules.md)), or add `users = true` to `[modules]`.

## Using it

- **The first user** comes with the macOS, deb and rpm packages, which write one called
  `admin` with a random password. To add one by hand, run `home-portal password-hash`, type a
  password, and paste the hash into `users.toml`:

  ```toml
  [[users]]
  name = "admin"
  password_hash = "$argon2id$..."
  ```

- **Management → Users** lists everyone, with you marked "you". With the module on:
  - **Add user** asks for a name and the password twice (8 characters or more);
  - **Change password** on a row sets a new one, and signs that person out everywhere else;
  - **Delete** removes someone after a confirmation.
- **Sign out** is in the menu with your name.

## Settings

| Key | File | What it does | Default |
|---|---|---|---|
| `[[users]]` `name` | `users.toml` | The name people sign in with | — |
| `[[users]]` `password_hash` | `users.toml` | The argon2id hash from `home-portal password-hash` | — |
| `users` | `[modules]` in the main file | Lets the browser add, change and remove users | off |
| `public_url` | `[network]` in the main file | When it starts with `https`, the session cookie is sent over HTTPS only | not set |

## Good to know

- The portal does not start without at least one user, and you cannot delete yourself or the
  last user from the browser.
- After 5 wrong passwords within 15 minutes, sign-ins from that address wait a minute.
- Behind the [reverse proxy](reverse-proxy.md), one sign-in covers every service published
  under the proxy's cookie domain.

## See also

- [Modules](modules.md) · [Reverse proxy](reverse-proxy.md) · [Configuration](../CONFIGURATION.md)
- [Install](../INSTALL.md), for where the first password is written
