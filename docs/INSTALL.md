# Install

Every release carries a package and an archive per platform, and a `SHA256SUMS` over all of
them. Check what you downloaded first:

```sh
sha256sum --check --ignore-missing SHA256SUMS      # shasum -a 256 -c on macOS
```

| File | For |
|---|---|
| `home-portal_<version>.macos.universal.pkg` | macOS 11 or later, Apple silicon and Intel: installs and starts everything |
| `home-portal_<version>.debian.amd64.deb` / `.debian.arm64.deb` | Debian, Ubuntu, Raspberry Pi OS (64-bit) and relatives |
| `home-portal_<version>.el.x86_64.rpm` / `.el.aarch64.rpm` | RHEL, AlmaLinux, Rocky, Fedora and relatives |
| `home-portal_<version>.linux.x86_64.tar.gz` / `.linux.aarch64.tar.gz` | Any Linux, to place by hand |
| `home-portal_<version>.macos.universal.tar.gz` | macOS, to place by hand |

## macOS

Open the `.pkg`. It installs the portal and sets it up for the user signed in:
- the configuration in `~/.config/home-portal/home-portal.toml`;
- the user **admin**, in the built-in `admin` group, with a random password in `~/.config/home-portal/initial-password`;
- a LaunchAgent that starts the portal at every login.

Then open http://127.0.0.1:8080. At its first start the portal asks macOS for the permissions it and your
scripts need: the local network, removable volumes, your Documents, Desktop and Downloads folders,
and control of System Events. Answer the prompts on the Mac, or check them later under
**Management → Permissions** (see [macOS permissions](features/host-permissions.md)). An older installation in
`~/Library/Application Support/home-portal` is moved to `~/.config/home-portal`, and a link is
left at the old place.

Another user of the Mac sets the portal up with `/usr/local/libexec/home-portal/setup`.
`sudo /usr/local/libexec/home-portal/uninstall` removes it; add `--purge` to delete the
configuration and data too.

## Debian, Ubuntu, RHEL and relatives

```sh
sudo apt install ./home-portal_<version>.debian.amd64.deb     # or
sudo dnf install ./home-portal_<version>.el.x86_64.rpm
sudo cat /etc/home-portal/initial-password                   # the password of admin
```

The package runs the portal as the systemd service `home-portal`, under a system user of the
same name, on http://127.0.0.1:8080:
- the configuration is `/etc/home-portal/home-portal.toml`, with the user **admin** in the `admin` group;
- the data is kept in `/var/lib/home-portal`;
- scripts for automations go into `/var/lib/home-portal/scripts` (owned by root, 0755).

An upgrade keeps the configuration, and so does removing the package; `apt purge` deletes it.

## From the archive

```sh
tar -xzf home-portal_<version>.linux.x86_64.tar.gz
cd home-portal_<version>.linux.x86_64
mkdir -p ~/.config/home-portal
cp config/home-portal.example.toml ~/.config/home-portal/home-portal.toml
./home-portal password-hash          # type a password, then add the hash as a [[users]] entry
./home-portal
```

```toml
[[users]]
name = "admin"
password_hash = "$argon2id$..."
group = "admin"
```

At least one user must be in the `admin` group, or the portal refuses to start and says so.

Keep the `web/` folder beside the binary: it is the interface. Without it the portal still
runs, but its pages answer 503. The archive also holds `examples/` and systemd or launchd files
for the portal and Caddy.

## Next

See [Configuration](CONFIGURATION.md) for how the files fit together, and [the features](features/README.md)
for what each part of the portal does and how to switch it on.
