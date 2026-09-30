# macOS permissions

On a Mac, the system asks the owner once before a program may reach the local network, read
removable volumes or your folders, or control other applications. The portal and the scripts it
runs need these. Without them a service on your LAN looks down, and a script that ejects a disk
or drives another application waits on a prompt nobody sees until its time limit ends.

The portal therefore asks for them itself and shows the answers.

## What it asks for

| Permission | Why | How it is asked |
|---|---|---|
| Local network | Probing services on your LAN | One multicast packet |
| Removable volumes | Scripts that read or eject external disks | Listing each mounted external volume |
| Documents, Desktop, Downloads | Scripts that use your folders | Listing each folder |
| Automation of System Events (and any applications you add) | Scripts that use `osascript` | One harmless Apple Event |
| Full Disk Access | Scripts that read protected files | Only checked: macOS never asks for it; add the portal by hand |

None of these actions changes anything. Once you have answered, macOS does not ask again.

## Using it

- **At start** the portal asks in the background, without waiting for the answers. Answer the
  prompts on the Mac. The log names anything that is still missing.
- **Management → Permissions** lists every permission with its state (allowed, denied, waiting for
  an answer, not needed here) and says where to allow it in **System Settings → Privacy &
  Security**. **Ask again** repeats the requests. It works only from inside your networks, never
  from the internet.
- **`home-portal permissions`** does the same from a terminal: it asks, waits for the answers and
  prints each permission with what to do. It exits `1` when something is denied. Piped, it prints
  plain `name<tab>state` lines.

## Who the permissions belong to

macOS gives permissions to the program that started the process:
- started from a terminal, the permissions belong to the terminal application (Terminal, iTerm…);
- started by the LaunchAgent, they belong to the `home-portal` binary itself.

Every script the portal runs has the portal's permissions. Run `home-portal permissions` from the
same place the portal runs from.

macOS knows a program by its contents, so a newly built or upgraded binary may be asked again.
After an upgrade, open **Management → Permissions** or run `home-portal permissions` once.

## Settings

All of them live in `[permissions]` in the main file:

| Key | What it does | Default |
|---|---|---|
| `request_at_start` | Ask at every start; when false, only Full Disk Access is checked | `true` |
| `automation` | Applications whose Automation permission is asked for | `["System Events"]` |
| `folders` | Any of `Documents`, `Desktop`, `Downloads` | all three |

```toml
[permissions]
automation = ["System Events", "Finder"]
folders = ["Documents", "Downloads"]
```

## Good to know

- **On Linux** every permission shows as "not needed here", and nothing is asked.
- **Removable volumes** show as "not needed here" until an external disk is mounted. Connect one
  and press **Ask again**.
- **Automation of another application** may start that application for a moment.
- **Seeing this page** needs the right to read Mac permissions, and **Ask again** needs the right
  to change them (see [Users, groups and sign-in](users-and-sign-in.md)).

## See also

- [Scripts](scripts.md) · [Services and their status](services-and-status.md) · [Install](../INSTALL.md)
- The commented `[permissions]` section in [`config/home-portal.example.toml`](../../config/home-portal.example.toml)
