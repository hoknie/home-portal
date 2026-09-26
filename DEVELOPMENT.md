# Developing home-portal

home-portal is one process: a Rust server (axum + tokio, a Cargo workspace) that serves a
Next.js static export from a folder shipped beside the binary. For installing and configuring
the portal, see [README.md](README.md).

## Requirements

- Rust 1.98.1, pinned by `rust-toolchain.toml`;
- Node.js 24 and pnpm 11 for the interface;
- [`just`](https://github.com/casey/just) for every task;
- Docker, only to build and check the deb and rpm;
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) (`pip install --requirement
  packaging/requirements.txt`), only for the static Linux binaries: rustls carries C code, and
  zig compiles and links it for either architecture.

## First run

```sh
just web-install                               # the interface's dependencies, as locked
just web                                       # build the interface into web/out
cp config/home-portal.example.toml config/home-portal.toml
cargo run -p home-portal -- password-hash      # type a password, copy the hash
```

Add a user to `config/home-portal.toml`, since the portal does not start without one:

```toml
[[users]]
name = "admin"
password_hash = "$argon2id$..."
```

```sh
just run            # http://127.0.0.1:8080, with config/home-portal.toml and web/out
```

`just run` passes `HOME_PORTAL_CONFIG=config/home-portal.toml` and `HOME_PORTAL_WEB=web/out`;
set either variable to override it. This repository keeps its configuration in `config/` and the
portal's data in `env/` (through `[storage]`). A checkout under a umask of `0002` gives `env/scripts`
mode `0775`, which the portal refuses: `chmod 755 env/scripts env/scripts/*.sh`.

For the interface with live reload, run `just run` in one terminal and `pnpm --dir web dev` in
another. `next dev` proxies `/api` to `HOME_PORTAL_ORIGIN` (default http://127.0.0.1:8080) and
serves the language in `PORTAL_LANGUAGE` (`en`, `ru` or `es`; `en` by default), ignoring the
language switcher's cookie.

A release binary: `just build`, then `target/release/home-portal` with `HOME_PORTAL_WEB=web/out`
(or `web/` copied beside it).

## Layout

```
crates/core/        portal-model · portal-feature (ports, ApiError) · portal-config (the files and
                    their secrets) · portal-widget · portal-web (serves the interface folder)
crates/features/    portal-<name>, one crate per subject; features never depend on each other
                    (portal-dns: the authoritative DNS server, its transports and zones)
bin/home-portal/    the composition root: boot/, features/registry.rs (the only list of features)
web/src/            Feature-Sliced: app → widgets → features → entities → shared
config/             the example configuration and secrets
examples/           a split setup, a service catalogue, launchd and systemd files
packaging/          the archives, deb, rpm and macOS .pkg
openspec/           specifications and changes
docs/designs/       design notes, one per change
```

## Rules the tests enforce

Rust rules live in `bin/home-portal/tests/architecture/`. Web rules live in
`web/src/shared/lib/architecture.test.ts`, eslint and steiger.

- No comments in `.rs`, `.ts` or `.tsx` under `web/src/`. A decision is recorded as a test whose
  name states it.
- `lib.rs` and `mod.rs` hold only `mod` and `use` lines. No `#[allow(...)]`.
- A file has at most 400 lines, a function at most 300, and a folder at most 12 files
  (`mod.rs`, `tests.rs`, `lib.rs` and `main.rs` are not counted).
- A kind of element is a plural folder, even for one element; never `utils/`, `common/` or `misc/`.
- `unsafe` code only in `crates/features/portal-automations/src/clients/process_group.rs`.
- Every `web/src/shared/ui/**/*.tsx` has a test beside it.
- Interface text comes from `web/src/shared/i18n/messages/{en,ru,es}.json`, with one key set in all
  three. There are no string literals in JSX.
- Web imports go down the layers, and only through a slice's `index.ts`.
- Changing a response shape needs `just samples`, or `cargo test` fails, and the matching zod
  schema, or `vitest` fails.
- A widget type served by a Rust provider needs an entry in
  `web/src/features/widget-board/model/registry.ts`.

## Rules for review

- No abbreviations in names (`address`, not `addr`).
- One error type, `portal_feature::ApiError`.
- Wire types stay in `controllers/`, `requests/` and `responses/`.
- Every outbound call has a timeout.
- A route is protected unless its feature puts it in `public_router()`.
- A new dependency comes with numbers: the transitive crate count, the clean build time and the
  binary size.

## Recipes

Every task is a `just` recipe; `just` alone lists them by section. The recipes live in
`env/justice/`, one file per section, and the root `justfile` imports them.

Work is done when `just check` passes. Anything that needs a browser, such as a theme flash, the
phone layout or a live walk-through, is named as unchecked, never assumed.

## Packages

Every package ships the binary with the interface beside it:

| Package | Binary | Interface | Configuration |
|---|---|---|---|
| archive | `home-portal` | `web/` | wherever `HOME_PORTAL_CONFIG` points, `~/.config/home-portal` by default |
| deb, rpm | `/usr/bin/home-portal` | `/usr/share/home-portal/web` | `/etc/home-portal`, data in `/var/lib/home-portal` |
| macOS `.pkg` | `/usr/local/bin/home-portal` | `/usr/local/share/home-portal/web` | `~/.config/home-portal` |

`packaging/package-macos.sh check` and `packaging/package-check.sh deb|rpm` install what was built
into a scratch home or a clean container, start the portal, and check that the interface is served
and that admin signs in.

