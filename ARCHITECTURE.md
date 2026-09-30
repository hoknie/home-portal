# `home-portal` Architecture

## 0. How to read this document

This document is a **set of rules** for the codebase. The rules apply to all new code and to every
file you modify. Where the tree does not yet comply, the gap is a ⚠️ row in §9: debt to pay down,
never permission to repeat the pattern. Deliberate exceptions are listed in §10 with their
rationale; a violation missing from §10 is a defect.

---

## 1. Core principles

1. **A workspace with explicit boundaries.** The product is a Cargo workspace. Crate boundaries
   are dependency boundaries, and the compiler enforces them.
2. **Dependencies flow one way.** `bin → features → core`. A library never knows the binary; a
   feature never knows another feature; `core` knows no feature.
3. **A feature declares, the root asks.** Each feature crate describes itself through the
   `Feature` port. The binary keeps exactly one list of features and no tables of routes, names or
   screens — a table that duplicates a declaration is where a new feature is silently forgotten.
4. **Quality is enforced by gates.** Formatting, lint, tests and the architecture guards run as
   one command, `just check`. A rule that can be checked by a machine is checked by one (§9).

---

## 2. Workspace topology

```
crates/core/              LAYER 1 — vocabulary and ports; knows no feature
  portal-model/           service identity, status and environment vocabulary; pure
  portal-feature/         Feature, Gate, StatusObserver, EventSink, WidgetProvider, ModulePreparer, Channel
                          and SecretSource ports, PortalEvent, Notification, ApiError, FieldError, Module
                          and ModuleSwitches
  portal-config/          the configuration files: sources, merge, secrets, revision, atomic 0600 write
  portal-widget/          the widget registry, its cache and the widget data endpoint
  portal-web/             serves the built interface from its folder beside the binary, as a fallback
crates/features/          LAYER 2 — one crate per subject the portal shows or acts on
  portal-health/          the portal's own liveness endpoint
  portal-auth/            users, sign-in, sessions, throttling, the session gate
  portal-services/        the service catalogue, HTTP, TCP and ICMP probes, the status board and its history
  portal-network/         the portal's own network settings, host interfaces and environments
  portal-dashboard/       the home page layout as data: sections, sized widgets, and its editing
  portal-metrics/         the portal host's processor, memory and disks, as a widget
  portal-weather/         Open-Meteo, as a widget
  portal-calendar/        an ICS calendar, as a widget
  portal-icons/           service icons: grabbed, fetched from the catalogue, cached on disk
  portal-notification/    the notifications module: rules, per-channel queues, the delivery journal, its API
  portal-secrets/         which secrets are named and whether each one is set
  portal-public/          the portal page shown without a session
  portal-proxy/           publishing services through Caddy: its configuration, forward-auth, TLS
  portal-automations/     the owner's scripts, run on a schedule or on the portal's events
  portal-scripts/         the scripts directory: where a script may lie and whether it can run
  portal-dns/             an authoritative DNS server for the names the proxy publishes
  portal-modules/         the modules page's API: which optional parts are on, and their requirements
crates/notification/      LAYER 2 — one crate per notification channel; depends on core crates only
  portal-telegram/        the Telegram channel: its settings and the Bot API client
bin/home-portal/          COMPOSITION ROOT — boot/, features/registry.rs, adapters/, middlewares/, cli/
web/                      the interface: Next.js static export, built into web/out (§12)
```

**The folder is the layer.** `path = "../../core/portal-feature"` points down and is legitimate;
`path = "../portal-other"` from a feature points sideways and is forbidden. A reviewer sees it in
`Cargo.toml` long before the compiler reports a cycle.

**A subject is a crate.** Everything about one integrated service — its client, its parsers, its
probes, its handlers and its tests — lives in `crates/features/portal-<subject>/`. The crate
`portal-<name>` reports `Feature::name() == "<name>"`.

**A feature does not know its siblings.** When it needs another subsystem, it declares a trait in
its own `ports/`, and the composition root provides the implementation in
`bin/home-portal/src/adapters/`.

**A crate exists for a subsystem of our own.** A wrapper around one external library stays a plain
dependency; a crate whose whole content is `pub use` must not exist. For that reason
`portal-model` was created by the first change that had a vocabulary to put in it.

**`portal-model` is pure.** It has no io, no async, no axum and no network: service status is
decided by functions over data (`ServiceStatus::after`), testable without a live service.

**Heavy alternatives get their own crate.** A second implementation of a port goes in the kind
folder of the same crate; it becomes a separate crate only when it pulls in a heavy dependency
every user of the port would otherwise build. Cargo features are additive across the graph, so a
feature flag cannot keep a dependency out.

---

## 3. Module and file organization

### 3.1. Module roots contain declarations only

Every `lib.rs` and `mod.rs` contains only `mod`/`pub mod` and `use`/`pub use` lines, plus
attributes on them (`#[cfg(test)]`). No `struct`, `enum`, `trait`, `fn`, `impl`, constants or
tests. A root is a map of its module that reads in five seconds. The only exception is the
`mod.rs` of a split-impl folder (§3.4), listed in §10. **Checked:** `tests/architecture/roots.rs`.

`main.rs` is a shim: `#[tokio::main] async fn main() -> ExitCode { home_portal::start().await }`.
Everything that assembles the application lives in the library, because integration tests can only
use a library crate.

### 3.2. One logical element per file

- One trait with nothing else per file; a type it only accepts or returns goes in `types/`.
- One struct or enum with its `impl` blocks and associated constants per file.
- No free functions in a file that holds a struct, trait or impl; they go to `helpers/` or a
  file of their own.
- One controller (the handlers of one resource) per file; one request shape and one response shape
  per file.
- Magic literals become named `const`s next to what they mean — paths, defaults, environment
  variable names, bodies.
- A closed vocabulary is an enum built from named variants and **carries an open door**
  (`Unknown`) when it crosses a wire; an open vocabulary stays a string.
- A method takes at most 4 arguments, a free function at most 3; beyond that, group them in a
  struct.

### 3.3. A kind of element is a folder — always, even for one element

Each kind lives in a folder named after the kind **in the plural**. A single element still gets a
folder: a flat file invites "one more thing", and three edits later it holds types, conversions
and helpers. Kind folders are named with nouns, never verbs. A kind folder contains only its kind
and subfolders of the same kind (`probes/http/` is the same role, deeper; `controllers/helpers/`
is a violation — `helpers/` is a sibling). Past roughly **12 files** a kind folder is grouped by
subdomain into subfolders of the same kind. **Checked:** `tests/architecture/sizes.rs`.

| Folder | Contains | Does NOT contain |
|---|---|---|
| `ports/` | requirement traits towards other subsystems, one per file | implementations, answer shapes (`types/`) |
| `features/` | the `impl Feature` of the crate: name and router | handlers, business logic |
| `controllers/` | axum handlers for one resource per file: extract, call a use case, answer | business decisions, storage, `ConfigStore` |
| `requests/` | wire shapes of incoming bodies and queries | domain types |
| `responses/` | wire shapes of outgoing bodies | domain types |
| `usecases/` | one operation an entry point performs, one struct per file: storage, business refusals, events | wire types, axum, another use case |
| `services/` | application logic over the crate's own domain | wire types, axum |
| `clients/` | the transport to one external service, with its timeouts | decisions about what the answer means |
| `probes/` | one status check of one service per file: call through a client, return a status | storage, handlers |
| `parsers/` | bytes of an external format → records, or a named refusal | io, decisions |
| `repositories/` | storage access, one file per table or file | domain decisions |
| `loops/` | a periodic pass: what one tick does | who starts it (that is `boot/`) |
| `types/` | domain values, their invariants and closed vocabularies | wire formats of foreign protocols |
| `helpers/` | pure functions without state, at crate level | anything with state |
| `adapters/` | an implementation of a port — only in the composition root | anything that implements no trait |
| `boot/` | one assembly step per file, in the binary | domain logic |
| `tests.rs` | unit tests of its kind folder, placed beside it | tests of a neighbouring kind |

**Forbidden folder names:** `utils`, `util`, `common`, `misc`, `structs`, `enums`, `traits`,
`impls`. **Checked:** `tests/architecture/folders.rs`.

**Entry points reach storage only through use cases.** Storage is a crate's `repositories/` and
`ConfigStore` (with its `Snapshot` and document): `ConfigStore` is a repository, even though it
lives in `portal-config`. An entry point is a file under `controllers/`, and under
`bin/home-portal/src/` `adapters/`, `middlewares/` and `cli/`; it calls a use case and never
names `repositories`, `ConfigStore`, `Snapshot`, `.document` or `.configuration`. Only `cli/` may
open the store and hand it to use cases. Use cases, services, loops and a feature's own assembly
(`features/`) may use storage; another crate reaches it only through the public use cases a feature
exports (`CurrentNetwork`, `CurrentEnvironments`, `CurrentProxySettings`, `CheckPublication`,
`ServiceEntries`, `UserNames`, `CurrentInterface`). A use case that reads or changes the
configuration answers `portal_config::Revisioned`, never `Snapshot`; a use case never calls
another, a shared step goes to `services/`. **Checked:** `tests/architecture/layers.rs`.

**The command line is declared once.** `bin/home-portal/src/types/command_line.rs` (`CommandLine`)
and its `command.rs`, `probe_kind_choice.rs` and `proxy_action.rs` describe every command, argument
and help text for `clap`; `boot/start.rs` parses them with `try_parse` and hands each command to its
handler in `cli/`. Help text is given through attributes and constants, never `///`. No command is
`serve`, so service files and the in-place restart pass the same argv as before. Usage errors exit
2, a failed command writes `error: <message>` and exits 1. `cli/palette.rs` is the only palette:
help, errors and reports are styled through it and written through `anstream`, which drops the
styles when the stream is not a terminal or `NO_COLOR` is set. Machine output (`password-hash`,
`proxy render`) is written with plain `println!` and never styled. **Checked:**
`tests/command_line.rs`, `tests/probe.rs`, `tests/proxy.rs`.

A **domain** folder is named after its subject in the singular (`status/`, `service/`); a **kind**
folder after its role in the plural. A domain may contain kinds; a kind never contains another
kind.

### 3.4. File and function size

- A file targets ≤300 lines; **above 400 fails** the gate unless listed in §10.
- A function has a **hard limit of 300 lines**, measured from its signature to its closing brace.
  Split by cohesion: put the boundary where the fewest values cross, and group those values in a
  struct.
- A struct with more than 7 fields, or one that mixes concerns, becomes a composition of
  sub-structs.
- A large inherent `impl` is split across concern files as **child modules** of a `<type>/`
  folder, whose `mod.rs` holds the struct and a thin, delegating trait impl — the one exception to
  §3.1, listed in §10 when used.

**Checked:** `tests/architecture/sizes.rs`.

### 3.5. Re-export facade

Consumers write `portal_feature::ApiError`; the path `portal_feature::types::...` is private
detail. Moving a file inside a crate must not touch its neighbours.

---

## 4. The `Feature` port and the composition root

```rust
pub trait Feature: Send + Sync {
    fn name(&self) -> &'static str;
    fn router(&self) -> Router;
    fn public_router(&self) -> Router { Router::new() }
    fn validator(&self) -> Option<Validator> { None }
    fn loops(&self) -> Vec<Loop> { Vec::new() }
    fn widget_providers(&self) -> Vec<Arc<dyn WidgetProvider>> { Vec::new() }
    fn stop(&self) {}
}
```

- A feature returns its routers **with its own state already applied**; the root sees
  `Router<()>` only and holds no per-feature state type.
- **Access is decided by which router a route is in.** The root wraps every `router()` in the
  session layer and merges every `public_router()` outside it, so a route is protected unless its
  feature puts it in `public_router()` on purpose. No guard parses router source text.
- `validator()` returns a plain function over the configuration document. The root collects them
  and hands them to `ConfigStore::adopt`; a function pointer, not the feature, so the store holds
  no reference back to the features that hold the store. Checks that need state — the widget
  providers' settings checks — are closures handed to `ConfigStore::adopt_checks`; they run at start-up
  and on every write and hand edit, exactly like validators.
- `loops()` returns the background passes the feature needs; `boot/loops.rs` spawns them all.
- `stop()` runs once after a graceful shutdown (`boot/serve.rs`); a feature that keeps state on disk
  flushes it there, as `portal-services` does with its history.
- `widget_providers()` returns what the feature can draw on the home page. The root collects them
  into the `WidgetRegistry` of `portal-widget`, which validates every configured widget at start-up
  and serves their data. A provider declares its type (`"weather"`), checks its settings and
  fetches its reading; it knows nothing about routes, sessions or environments.
- Shared handles (the `ConfigStore`, the effective address) are passed into a feature's constructor
  by the root. A feature that needs another subsystem declares a port (`portal_auth::Connection`)
  and the root adapts it (`adapters/network_connection.rs`).
- `bin/home-portal/src/features/registry.rs` holds `registered()`, the **only** list of features,
  and returns the session `Gate` beside it. The guard `tests/architecture/registry.rs` fails when a
  crate under `crates/features/` is not registered, when a registered feature has no crate, or when
  two features share a name.
- `boot/` has one file per step, and `boot/run.rs` holds **only the order of steps**: logging,
  configuration, address, registry, validation, router, listener, loops, the start event, serve.
  Any failure at start-up is reported with what failed and exits non-zero — a portal that listens
  nowhere must never start silently.
- **The root alone restarts the process.** `POST /api/portal/restart` lives in
  `controllers/restart.rs`, inside the session layer, and sets the `Restart` handle kept in
  `Registry::restart`. `boot/shutdown.rs` waits for SIGINT, SIGTERM or that handle; the stop steps
  are the same for all three. After them `boot/start.rs` replaces the process with the same
  executable, arguments and environment (`CommandExt::exec`, same pid), so launchd, systemd and a
  hand-started portal all come back with the configuration read again. No feature can restart the
  portal: the handle reaches only this controller.
- **The interface is a folder found once at start-up** (`boot/interface.rs`): `HOME_PORTAL_WEB`,
  then `web/` beside the canonical executable (the release archive), then
  `../share/home-portal/web` (`/usr/share/home-portal/web` in deb and rpm,
  `/usr/local/share/home-portal/web` in the `.pkg`). `portal-web`'s `Directory` serves it through the
  `AssetSource` port and refuses paths that leave it. Without it the portal still runs, and pages
  answer 503 naming the places searched.
- **Events go to one sink.** A feature that publishes events takes the `EventSink` port in its
  constructor (`AuthFeature::new`, `ServicesPorts`). The root builds `portal-automations` first,
  hands its sink out and keeps it in `Registry::events`. `boot/lifecycle.rs` publishes
  `portal.started` once the loops run, and `portal.stopping` after a graceful shutdown, then settles
  for at most 10 seconds before any feature's `stop()`.
- **Optional parts are modules.** The proxy, DNS, automations and webhooks can be switched on and
  off; `Module` in `portal-feature` lists them and what each requires (DNS the proxy, webhooks
  automations). Every crate learns whether its module is on only through
  `ModuleSwitches::resolve`, which reads `[modules]`, then the legacy `proxy.enabled` and
  `dns.enabled`, then the defaults, so no two crates can disagree. `portal-modules` owns the switch
  (`/api/modules`) and the validator that refuses an enabled module whose requirement is off. A
  feature that must adjust the file when its module is switched on implements `ModulePreparer`
  (`portal-proxy`'s `PrepareProxy` trusts loopback); the root hands preparers to `ModulesFeature`.
  Routes stay mounted while a module is off; each checks its switch and the settings stay editable.
- Two features claiming one path make axum panic at assembly; assembly runs in a test, so the
  conflict fails `just check`.
- **Every protected route declares the right it needs.** `Feature::rules()` returns its routes as
  data (method, path, requirement), and one middleware checks them (§6.15).

---

## 5. The web layer

- **Wire types stay at the edge.** A controller converts a request into domain values, calls a
  use case and converts the result into a response. Services never see `requests/` or
  `responses/` types, and domain types are never serialized to the client directly.
- **One error type**: `portal_feature::ApiError`. Variants map to statuses — `BadRequest` 400,
  `Unauthorized` 401, `NotFound` 404, `Conflict` 409, `UnsupportedMediaType` 415, `Invalid` 422,
  `PreconditionRequired` 428, `TooManyRequests` 429 (with `Retry-After`), `BadGateway` 502,
  `ServiceUnavailable` 503, `Internal` 500. Bodies are plain text, except `Invalid`, which is
  `{"errors": [{"field", "message"}]}` so a form can show each error beside its field. `Internal`
  logs its diagnostic and sends only `internal error`. New variants are added when a feature needs
  them, not in advance.
- **Mutating requests are JSON.** `middlewares/json_only.rs` answers 415 to a `POST`, `PUT`,
  `PATCH` or `DELETE` under `/api/` whose body is not `application/json`; together with the
  `SameSite=Strict` session cookie and no CORS, a cross-site form cannot act for a signed-in person.
- **Unknown API paths are 404.** The root routes `/api` and `/api/{*rest}` to a 404 handler;
  feature routes under `/api/` win over it because static routes outrank the catch-all.
- **The interface is a fallback after the `/api` 404** (`portal_web::serve`), so an unknown endpoint
  never receives `index.html` with a 200. It tries the exact file, then `path/index.html`; a request
  is a missing asset (404) only when its last path segment ends in a **known file extension** —
  "contains a dot" misroutes hostnames and addresses in route parameters. Everything else gets the
  page of its nearest folder that has one (`controllers/serve.rs#nearest_page`, so
  `/admin/workflows/revive/history/42/` gets `admin/workflows/index.html`, whose screen reads the rest
  of the path), and the entry page when no folder above it has a page. `_next/static/` is cached immutably, HTML with `no-cache`. A binary built without
  the interface answers 503 saying so.
- **The public half is explicit.** `portal-public` answers `/api/public/portal` from two ports the
  root adapts (`adapters/public_services.rs`, `adapters/public_layout.rs`), with response types of
  its own: a public service carries no probe settings, no per-environment addresses and no revision.
  A widget that is not public in this environment answers 404 exactly as an unknown id, so the
  public half never tells a stranger what exists.
- **Every request carries an `Environment`.** `middlewares/environment.rs` decides it from the
  client address once and inserts it as an extension; handlers read it instead of looking at
  addresses again. The same middleware applies the visitor's choice (§6.3) and inserts a
  `DetectedEnvironment` beside it, so no handler ever reads the cookie.
- `/health` is unauthenticated and contacts nothing external.

---

## 6. Integrations with services

`portal-services` implements these rules for HTTP probes; a new kind of integration follows them
too.

- **A port per external system.** A feature reaches a service through a trait in its `ports/`
  or a client in `clients/`; tests use a fake implementation, never a live service.
- **"Could not ask" is not "down", and "never asked" is not "up".** The status vocabulary in
  `portal-model` is `Unknown`, `Up`, `Degraded`, `Down`, `Unreadable`. An `Option` or a boolean
  spells different facts the same way. When a credential or a setting cannot be read, the last
  good value is kept rather than replaced by an empty one.
- **Status is written only by the probe that last ran**, never by a form or a request; it carries
  when it last succeeded and the last error.
- **Every outbound call has an explicit timeout** — a connect timeout and a total timeout. A call
  without one can wait forever.
- **Probing runs in loops, not in handlers.** A handler reads what the last probe found. A loop has
  a period, a per-pass timeout and backoff after failures, and publishes a change only when the
  status changed.
- **Outbound addresses come from configuration**, never from a request body.
- **Probing is supervised.** `services/supervisor.rs` compares the configuration with the running
  probe tasks every 2 seconds and after each write through the API, restarting a task only when its
  id, URL or probe settings changed. The wait doubles after each failure up to 5 minutes.

### 6.1. The configuration file

- **One main TOML file holds the short settings** — `interface`, `network`, `environments`,
  `modules`, `storage`, `scripts`, `files` — at `HOME_PORTAL_CONFIG`, by default
  `$XDG_CONFIG_HOME/home-portal/home-portal.toml` when that variable is absolute, else
  `~/.config/home-portal/home-portal.toml`; the working directory is never consulted
  (`portal_config::configuration_path`). A missing default file with a `home-portal.toml` in the
  working directory is reported with both paths. `ConfigStore::read` notices a hand edit by the
  files' length and modification time; one thread reloads it while any other reader waits for that
  reload instead of getting the old snapshot, and the reloading thread itself (a validator that
  reads a secret) gets the current one.
  `config/home-portal.example.toml` is the commented starting point; `just run` reads
  `config/home-portal.toml`. `HOME_PORTAL_ADDRESS` overrides
  `network.address` and `network.port`.
- **Every other section has a home beside it.** `types/layouts/` names them (`Section`, `Home`,
  `Layout`): `services.toml`, `dashboard.toml`, `users.toml`, `notifications.toml`, `proxy.toml`,
  `dns.toml`, `automations.toml` (with `automation_settings`), `webhooks.toml`, `secrets.toml`, and
  the folder `workflows/`. `[files]` in the main file moves any of them inside the directory;
  `helpers/layout.rs` resolves the homes once at start, as `[storage]` is, and checks them under
  `files.<key>`. A missing home reads as empty; the first write creates it 0600
  (`services/writing.rs`). New entries go to `ConfigStore::home_of(section)`, edits to the file
  that holds the entry (`Origins`), and nothing writes `[files]`, `include` or `configuration`.
- **One workflow, one file.** `workflows/<id>.toml` holds the workflow's own keys and `[[steps]]`.
  The loader wraps each file into a one-entry `[[workflows]]` document (`helpers/entries.rs`), so
  merging, `Origins`, validation and the workflow repository see one list; writing unwraps it.
  `update` creates a file, `update_moved` renames it when the id changes (new file first, then the
  old one removed with its `.previous`), and `remove` deletes it. The folder's listing is part of
  the stamps and the revision, so a file dropped in by hand is read and makes older revisions
  stale. Errors on `workflows[i]` name the file.
- **An old layout is moved at start.** `services/settling.rs` plans every move of a section found
  outside its home (the main file, an included file, another home) in memory, refuses a collision
  naming both places before writing anything, then writes targets before sources with
  `.previous` and logs each move. Table positions are shifted past the target's
  (`helpers/positions.rs`); the merge shifts every source's positions too, because
  `deserialize_section` prints and re-reads the merged document. `include` is still read after the
  homes and its sections are moved the same way; `configuration.writes_to` is ignored with a
  warning. Lists are the concatenation of every file's entries, homes first; a plain key defined
  twice is an error naming both files.
- **`[storage]` says where the portal keeps what it writes.** `sessions.json`, `icons/`,
  `history/`, `automations/`, `caddy/` and the `scripts/` automations run lie beside the main
  file unless `storage.directory` moves them all or `storage.<name>` moves one; relative paths are
  relative to the main file's directory. `portal-config` resolves them once at start
  (`ConfigStore::storage`), so a change takes effect on restart; an unknown key or an empty path
  stops the start and refuses an edit. Features receive a resolved path and never derive one from
  the configuration's location.
- **Secrets live in `[secrets]`, apart from the settings that name them.** A section names a key
  (`notifications.telegram.secret = "telegram_token"`); the value stays in the file that holds the
  table, which must be mode 0600. `portal-config` strips the table from the merged document and
  keeps each value in a `SecretString` that neither prints nor serializes, so a secret cannot reach
  a response by accident. `/api/secrets` lists the names and whether each one is set.
- **TOML because the interface writes the file a person also edits.** `toml_edit` changes the keys
  it touches and keeps comments, order and formatting everywhere else. When a service is deleted,
  the comment lines separated from it by a blank line stay with the file.
- **Each section belongs to one feature**: the feature parses it (`types/`), validates it
  (`validator()`) and edits it (`repositories/`). `portal-config` knows only where each section
  lives, never what it means.
- **Writes are atomic and guarded.** Responses carry the file's revision (SHA-256) as `ETag`; a
  write needs `If-Match` (428 without it) and is refused with 409 when the file changed since. The
  new content goes to a temporary file with mode 0600, the old content to `<name>.previous`, and a
  rename replaces the file.
- **Hand edits are picked up** on the next read (the store stats the file). A broken edit is logged
  and ignored: reads keep the last valid configuration, writes answer 409 naming the problem.

### 6.2. Authentication

- Users are `[[users]]` entries with an argon2id hash printed by `home-portal password-hash`. The
  portal refuses to start without one, and the example ships without one on purpose — a known
  password in an example becomes a live account the day someone copies it.
- **Users can also be written by the interface** while the `users` module is on: `portal-auth`
  owns `/api/users` (`ListUsers`, `CreateUser`, `ChangePassword`, `DeleteUser`) and writes the same
  `[[users]]` entries through `repositories/users.rs`, back to the file each entry came from and a
  new one after the last. Passwords are hashed off the runtime (`spawn_blocking`) and never stored,
  logged or answered; nobody deletes themselves or the last user. While the module is off the list
  is readable and every write answers 409.
- **A session carries a credential fingerprint**: 16 hex digits of SHA-256 over its user's
  `password_hash`. A hash that changes, from the interface or by hand, ends the other sessions on
  their next request; changing one's own password restamps the session that did it. Sessions from
  before the fingerprint are stamped on their next request.
- A session is a random 256-bit token in an `HttpOnly`, `SameSite=Strict` cookie, sliding 7 days,
  and invalid as soon as its user leaves the configuration. The cookie scope follows the way the
  sign-in came: through Caddy under `proxy.cookie_domain` it is `Secure` with
  `Domain=<cookie_domain>`, so one sign-in covers every published host; directly it is `Secure`
  only when `network.public_url` is https, because a browser drops a `Secure` cookie over plain
  http and a `Domain` that does not match the host.
- **Sessions survive a restart.** `sessions.json` in its storage place (§6.1), mode 0600, holds the
  SHA-256 of each token, never the token. Sign-in and sign-out are written at once; the sliding
  expiry every 5 minutes and on stop.
- A wrong name and a wrong password look the same, and an unknown name is checked against a dummy
  hash so timing does not tell them apart. 5 failures in 15 minutes lock a client address for 60
  seconds; `X-Forwarded-For` is believed only from `network.trusted_proxies`.

### 6.3. Environments

- **An environment is where the visitor is**, decided from their address: `[environments.<name>]`
  lists networks, and everyone outside every range is in the reserved environment `internet`.
  Overlapping ranges are a configuration error naming both.
- A service may carry one address per environment (`addresses`), and `environments` limits who sees
  it at all. `GET /api/services` answers with the address of the visitor's environment, and hides
  what this environment may not see — filtering in the interface would be a list of private
  addresses sent to a stranger.
- **The probe picks its own address**: `probe.environment`, else the environment of the portal's
  own host (resolved at boot from its interface addresses), else `url`. The portal probes what it
  can reach, whoever is asking.
- Widgets carry the same two keys, and `public` adds them to the portal page of §5.
- **A visitor inside may look from elsewhere.** The `portal_environment` cookie names another
  environment; it is honoured only when the detected environment is not `internet` and the name is
  configured (`Environments::effective`), and then it decides everything the environment decides.
  From `internet` it is ignored, so a choice can move a trusted visitor sideways but never widen what
  a stranger sees. Sign-in throttling keeps using the client address.

### 6.4. Widgets

- A widget instance is `type`, an optional `title` and `settings`, plus `id` when a provider serves
  its data. `portal-widget` validates every instance at start-up through its provider, so a bad
  setting stops the portal with the widget's id and the setting named.
- **The registry caches per instance.** A reading is reused while it is younger than the provider's
  refresh period; concurrent readers of a cold instance share one upstream call; a failed refresh
  keeps the last good reading and marks it `stale` with the problem; an instance that never
  succeeded answers 502.
- The interface polls `refresh_seconds` from the answer, so the period lives in one place.
- **The layout is sections and sized widgets in file order.** `[[dashboard.sections]]` lists sections
  (`id`, `title`); a widget names its `section` (the first when left out) and a `size` — `quarter`,
  `third`, `half`, `two-thirds` or `full` — a share of a 12-column row. No coordinates: a person adds a
  widget by appending a table.
- **The editor moves tables, it does not regenerate them.** `PUT /api/dashboard` names each widget by
  `key` (its id, or `#<index>`); `portal-dashboard/src/repositories/layout.rs` rebuilds the arrays from
  the existing tables, so each keeps its comment lines and only changed keys are rewritten. Widgets
  without an id get one. A layout spread over several files answers 409 naming them.

### 6.5. Service icons

- `icon` is `lucide:<name>` (a bare name means this), `auto`, `catalog:<slug>`, `url:<address>` or
  `file:<path>`. `auto` reads the service's own page: the manifest's icons largest first, then
  `<link rel="icon">`, then `/favicon.ico`.
- Everything fetched is cached in the `icons` storage place (§6.1) and reused
  across restarts; an entry is refreshed when its source changes or after 7 days. Only image types
  are accepted, the bytes must match the declared type, and 512 KiB is the ceiling. A failed fetch
  keeps the previous icon and records why.
- **The interface never asks a service for its icon.** `GET /api/icons/{service}` serves the bytes
  with an `ETag`, so the browser of a visitor on the internet never touches a private address.

### 6.6. Probe kinds and diagnosis

- `probe.kind` is `http` (default), `tcp` (a connection to a port, nothing sent) or `icmp` (one echo
  over an unprivileged `SOCK_DGRAM` socket built on `socket2`). `probes/probe.rs` dispatches; the
  loop and `home-portal probe <id|url>` use the same code.
- **A failure carries a diagnosis code** (`portal_model::Diagnosis`: `refused`, `timeout`,
  `host-unreachable`, `name-not-resolved`, `tls`, `not-http`, `http-status`, `icmp-not-permitted`,
  `local-network-denied`, `other`). It is decided where the error is seen
  (`portal-services/src/helpers/failure.rs`); the interface and the command line turn it into advice.
- **macOS Local Network privacy is not an outage.** An instant `EHOSTUNREACH` to an address on a
  directly connected subnet, on macOS, is `unreadable` with `local-network-denied`
  (`helpers/local_network.rs`), because the operating system refused the process, not the service.
- `POST /api/services/{id}/probe` wakes the service's loop through a `Notify` and answers 202; no
  handler waits for a probe. A second request within 5 seconds answers 429.

### 6.7. Status history

- Every probe outcome is recorded under the status board's lock: single outcomes for 24 hours
  (thinned to one per 5 seconds), hourly buckets for 30 days, and state changes for 30 days — about
  80 KiB per service at a 30-second period.
- The history lives in `<id>.ndjson` in the `history` storage place (§6.1), mode 0600: one JSON object
  per line (`sample`, `replace`, `transition`, `hour`, where the last `hour` line wins). Every 5
  minutes and on shutdown only the new lines are appended; at start-up and past 1 MiB the file is
  rewritten atomically in compact form. A line cut short is skipped, a file with no readable line is
  renamed `.broken` and the service starts empty, and a former `<id>.json` is carried over once. A
  new or renamed service, or one whose last write failed, is rewritten whole instead of appended. `GET /api/services/{id}/history?range=24h|7d|30d` answers from memory.
- Uptime counts probed time only: time without probes is not downtime.

### 6.8. Reverse proxy

- **Caddy carries the traffic; the portal writes its configuration.** `portal-proxy` renders
  Caddy's whole JSON configuration from `[proxy]` and the services' `proxy` tables
  (`renderers/caddy.rs`, a pure function tested by exact fixtures, the forward-auth block taken
  from `caddy adapt` in `crates/features/portal-proxy/fixtures/caddy/`) and loads it through the
  admin API on loopback or a unix socket. A proxy inside the portal was rejected: every restart
  of the portal would cut every service off.
- **The portal owns the whole document.** It loads at start, within a second of a revision change
  and on `POST /api/proxy/apply`, and every 60 seconds compares Caddy's document with its own and
  loads again when they differ. A Caddy that is down never stops the portal; the failure is state
  (`GET /api/proxy`), not an error at boot. Caddy runs with `--resume`, so routes keep working
  while the portal is down.
- **Forward-auth trusts only the proxy and only the detected environment.**
  `GET /api/proxy/authorize` is public, answers 404 to a peer outside `trusted_proxies`, decides
  from `DetectedEnvironment` (never the environment cookie) and builds its redirect from
  `proxy.portal_host`, never from a request header. The return after sign-in goes through
  `GET /api/proxy/continue`, which follows only a published host.
- **The portal can run Caddy itself** (`proxy.managed`). The binary is downloaded by a background
  job from the release that `[proxy.caddy]` names: `source`, a GitHub-compatible releases API
  (GitHub by default, or a Gitea or Forgejo mirror), and `version`, `latest` or an exact one.
  `PUT /api/proxy/caddy` writes the two keys, and `GET /api/proxy` shows the release URL and the
  archive before anything is fetched. The archive is checked against the release's SHA-512 checksum
  file, unpacked with the system `tar` and installed into the `caddy` storage place (§6.1), with
  the archive URL recorded in `caddy/ORIGIN`. A plain archive URL was rejected: it has no checksum
  file to check against. It runs as
  `caddy run --resume` in a process group of its own, so a restart of the portal leaves it serving;
  the sync loop starts it again when its admin API stops answering, at most every 30 seconds. Stop
  is Caddy's own `POST /stop`: the portal never signals a process.
- **The feature reads other subsystems through ports.** Published services with their upstream
  (`adapters/service_publications.rs`), trusted peers (`adapters/network_connection.rs`) and the
  core `Gate`. `portal-services` asks whether publishing is on through its own `Publishing` port
  (`adapters/proxy_publishing.rs`), because the shown address of a published environment is
  `https://<host>`.

### 6.9. Automations

- **An automation is the owner's script, run on a schedule or on a portal event.** An
  `[[automations]]` entry names a trigger (`when`: an event from the closed catalogue of
  `portal_feature::EventName` and that event's filters) and a script with its arguments (`run`);
  `[automation_settings] timezone` sets the zone cron counts in. `portal-automations` parses,
  validates and edits the section like any other, keeping comments.
- **Emitters only emit.** `portal-auth`, `portal-services` (its status board through a
  `StatusObserver`, its catalogue writes directly) and `boot/lifecycle.rs` call `EventSink::emit`
  and return. `emit` matches against the decoded list the revision watch keeps
  (`services/cache.rs`) and never reads the configuration file, so no handler or probe waits on it.
  Admission — already running, one run already waiting, the cooldown, 60 starts an hour — is
  decided before the queue, which holds at most 100 runs and drops the oldest.
- **No shell, and only `scripts/`.** A script is a path inside the `scripts` storage place (§6.1).
  Before every run the `ScriptLibrary` port resolves it: `bin/home-portal/src/adapters/script_library.rs`
  calls `portal-scripts` (§6.13), which refuses a file outside the directory, one
  that is not an executable regular file, one owned by neither the portal's user nor root, and one
  that group or others can write — the file or any directory down to it. The interface lists the
  scripts and never writes one. Each argument is one argv item with `{{field}}` placeholders; the
  same fields arrive as `PORTAL_*` variables and as JSON on standard input, in an otherwise clean
  environment. The examples put `--` before placeholders. `status.error` can carry an upstream's
  address into argv and the journal, both of which only a signed-in person sees.
- **Runs are bounded.** At most 4 run at once. A timeout (60 s by default, 3600 s at most) and the
  exit of the script's own process both kill its whole process group, so nothing it started in the
  background outlives it. Output is read to the end and only its last 64 KiB is kept, cut at the
  start of a UTF-8 character. On shutdown `portal.stopping` runs and the portal waits for it at
  most 10 s.
- **Runs that have not finished live in `services/dispatch/active_runs.rs`**, keyed by run id, from
  admission until the journal records them; the journal holds finished runs only. Each entry owns
  the live output tails, which `clients/runner.rs` fills, and a `watch` channel for stop. A stop
  takes a queued run out of the queue, or makes the runner send SIGTERM to the group and SIGKILL
  5 s later; the run is recorded as `stopped`. A `watch` keeps the value, so a stop that arrives
  before the spawn is still seen. `GET /api/automations/runs` lists active runs before the journal.
- **The one `unsafe` in the workspace** is
  `crates/features/portal-automations/src/clients/process_group.rs`: `killpg` and the user and
  groups of the process. `tests/architecture/unsafe_code.rs` fails on `unsafe` anywhere else. The
  rule of §6.8 that the portal never signals a process is about Caddy; a script's process group
  belongs to the portal.
- **The journal is kept as NDJSON**: the last 200 runs, consecutive skips of one automation (and
  one webhook) merged with a count. New and merged records are appended to
  `automations/runs.ndjson` by a writer loop within a second and on stop, never on the publishing
  path; the last line of a run id wins, and the file is rewritten past 1 MiB or twice its compacted size,
  and whole after a failed write. Run ids continue past the highest id in the file, broken lines
  included. Every run is also
  logged at `info`, without its output.
- **Scripts lie in `scripts/` or one subfolder down**, never deeper and never in a hidden path. A
  refusal carries a code and the path it concerns, so the interface explains it with the command that
  fixes it. The event `manual` is never published: such an automation runs only from "Run now".
- **Webhooks are an entrance to the same machinery.** A `[[webhooks]]` entry is the public
  `POST /webhook/{id}` (in `public_router()`, outside `/api/`, body capped at 64 KiB). Its token is
  generated by the portal, answered once and kept only as `token_sha256`, compared in constant time.
  Its declared variables come from the JSON body, then the query, and become `webhook.<name>` fields.
  Its action publishes `webhook.received` through the same sink, or queues its own run, which the
  dispatcher treats like an automation's. An unknown and a disabled id answer 404 alike; 60 calls a
  minute are allowed per webhook.
- **Tags are shared.** Automations and webhooks carry `tags`; the catalogue lists every tag in use
  so both forms suggest the same set, and both tables filter by them. The journal narrows by
  automation, webhook (its own runs and the runs its event started) and a text in field values or
  arguments, and refreshes itself every 5 seconds.

---

### 6.10. Local DNS

- **`crates/features/portal-dns` is an authoritative server for the portal's own names only**:
  `proxy.portal_host`, each service's `proxy.host` and the `[dns] zones`. Other names are REFUSED,
  and nothing is forwarded, recursed or cached, so it can never be an open resolver. The router
  forwards the zones to it.
- **One port, `DnsSources`**, adapted by `bin/home-portal/src/adapters/dns_directory.rs`, gives it
  the environments, the published hosts (with the service's own `environments`, not
  `proxy.environments`) and the host's interface addresses. It reads `proxy` and `environments` as
  raw views, like `portal-proxy` reads `network`.
- **A pure `services/zone_book.rs::build`** turns settings, hosts and interfaces into an immutable
  `ZoneBook`: name → records per environment, the proxy's address per environment (from
  `[dns.addresses]`, else the interface inside that environment's networks), and an SOA serial that
  changes exactly when the answers do. `loops/supervisor.rs` rebuilds it every second and swaps
  it in; `services/answer.rs` answers from it without locks beyond cloning an `Arc`.
- **Every transport shares `services/handler.rs`**: UDP (answers capped at 1232 bytes, TC beyond),
  TCP and DoT (64 connections, 10 s idle), and DoH at the public `/dns-query`, which takes the
  visitor's address from `ClientAddress`. The questioner's environment decides the answer; `internet`
  is always REFUSED. Sockets rebind within 2 s of a settings change, and a port that cannot be bound
  is reported in `GET /api/dns`, never a boot failure.
- **DoT** uses `dns.tls.certificate`/`key`, else the certificate of the DoT host's TLS policy: its
  files, or the newest one a managed Caddy keeps under `caddy/data/caddy/certificates/`. It is read
  again when it changes. `portal-proxy` renders a `/dns-query` route when DoH runs on a host other
  than `portal_host`.
- The wire format is `hickory-proto` (numbers in §7); `hickory-server` was rejected, because its
  zone-file authorities do not fit per-environment answers.

### 6.11. Workflows

- **A workflow is a tree of steps that an automation, a webhook or "Run now" starts.** A
  `[[workflows]]` entry has `inputs` and `steps`; `if`, `loop` and `parallel` steps hold nested lists
  (`then`/`else`, `body`, `branches`). The engine lives in `portal-automations`
  (`types/workflow/`, `services/workflow/{checking,evaluating,running}/`), because a workflow run is
  an automation run: the same queue, gatekeeper (cooldown, 60 starts an hour), 4 parallel runs,
  journal and stop. An automation or a script webhook names `run` or `workflow` with `inputs`,
  never both. A manual run uses the gatekeeper key `workflow:<id>`, which no automation id can take.
- **One catalogue** (`types/stepping/kinds.rs`) describes every step kind: its group, its fields with
  their types, defaults and bounds, and its result fields. Validation reads it, and
  `GET /api/workflows/catalogue` serves it to the editor's forms; a test ties it to `StepKind`.
- **Exclusive groups.** A kind may declare groups of fields of which a step holds exactly one
  (`exclusive`: a loop's `repeat`/`for_each`/`while`, a set's `value`/`json`/`list`/`object`). The
  decoders keep their own "exactly one" rule; the editor draws each group as one choice
  (`widgets/workflow-editor/ui/inspector/data/exclusive-choice.tsx`), shows only the chosen field,
  and gives a new step the first field of each group (`entities/workflow/model/exclusive.ts`).
- **Every kind is audited.** `bin/home-portal/tests/samples/workflows.rs` holds one minimal valid
  step per kind, fails when a kind is missing, validates them and writes `step-defaults.json`; the
  web test `model/checks/step-defaults.test.ts` builds each kind from the palette, fills the same
  fields and requires no client problem, the same keys and one field per exclusive group.
- **Validation is structural and static.** Every error names its full path
  (`workflows[1].steps[2].then[0].url`). `checking/scope.rs` walks the tree in order and refuses a
  template that names an input the workflow does not declare, a variable no earlier step sets, a step
  that does not come earlier, or `loop.*` outside a loop. `checking/calls.rs` refuses unknown
  workflows, undeclared inputs, cycles and calls deeper than 4. Automations and webhooks are checked
  against the section whether the module is on or off.
- **A run is bounded.** `running/budget.rs` holds the deadline (`timeout_seconds`, 300 by default,
  3600 at most), 1000 executed steps, 100 iterations per loop, 8 levels of nesting and the run's stop
  `watch`; every wait, HTTP request and probe races the stop and the deadline. `parallel` runs 2–4
  branches with `join_all` and merges their results in branch order. Values are capped at 64 KiB.
- **Leaving a loop is a flow, not an ending.** `Flow` (`types/progress/flow.rs`) is `Continue`,
  `Break`, `NextPass` or `End(Ending)`. `runner.rs` returns any flow but `Continue` at once, an `if`
  passes it up, and the loop driver (`running/flow_steps.rs`) consumes `Break` (setting `left_early`)
  and `NextPass`; `stop` still ends the whole run. `checking/flow_decoding.rs#check_loop_exits`
  refuses `break` and `continue` outside a loop body or through `parallel`, so they never reach a
  join or the root, where the runner treats them as `Continue` anyway. `Place.loop_label` names the
  loop in their trace detail.
- **Templates** `{{event.*}}`, `{{inputs.*}}`, `{{vars.*}}`, `{{steps.<id>.<path>}}`, `{{loop.*}}`
  and `{{secrets.<key>}}` are rendered by `evaluating/values.rs`. A secret is sent as it is, but every
  rendered secret is replaced by `***` in the trace and the journal.
- **Portal values.** `{{portal.services}}` (a list), `{{portal.services.<id>.<field>}}`,
  `{{portal.network.{address,port,url}}}`, `{{portal.modules.<name>.is_enabled}}` and
  `{{portal.environments}}` read the portal's own state. `PortalActions::state()` gives a
  `PortalState` (`types/portal/`), the crate adds the module switches, and the runner refreshes
  `Frame.portal` before every step whose templates name `portal`, so a step after a `probe` sees
  the new state and other steps cost nothing. `checking/scope.rs` refuses at load an unknown
  service (when the document has `[[services]]`), module or field. `GET /api/workflows/portal`
  (`ReadPortalValues`) answers the same value, which the editor offers with examples.
- **Actions leave the crate through ports.** `http` uses one rustls `reqwest` client
  (`clients/http.rs`: 5 redirects, a per-request timeout of 1–60 s, 10 MiB read, 64 KiB kept);
  `script` reuses the process-group runner and the `scripts/` rules of §6.9; `notify`, `probe`
  and `status` call the `PortalActions` port, which `bin/home-portal/src/adapters/portal_actions.rs`
  implements with the public use cases `SendNotification` (§6.12), `ProbeService` and
  `CurrentStatus`. The old `telegram` kind is read as `notify` to the `telegram` channel, and the
  repository writes it back as `notify`. The adapter
  is filled after those features are built, so it answers "the portal is still starting" before.
- **Filters and the transform step.** A template name may carry filters, `{{name | f(a) | g}}`,
  from one closed catalogue (`types/transforming/filters.rs`: text, numbers, lists, objects, JSON
  and `default`, each with the types it takes and gives). `evaluating/chains.rs` parses the chain,
  `evaluating/filters.rs` applies it, and `checking/filter_checks.rs` refuses at load an unknown
  filter, a wrong argument and a type mismatch that is certain from the name. The `transform` step
  (`running/transform_step.rs`, at most 20 operations) applies filters and the list operations
  `filter`, `map`, `sort_by`, `group_by` and `count_by`, where `{{item}}` and `{{index}}` are in
  scope; the trace keeps the value after each operation. `GET /api/workflows/catalogue` serves
  `filters` and `operations`.
- **One evaluator, mirrored.** `entities/workflow/model/transforming/` repeats the evaluator in
  TypeScript for the editor's previews and `|` completion by type. The Rust test
  `bin/home-portal/tests/samples/transforms.rs` writes `transforms.json` through the
  `TransformValue` use case, and the web test runs the same cases, so the two cannot drift.
- **Values in and out.**
  - Inputs are plain names or tables with a `type` (`text`, `number`, `boolean`, `list`,
    `object`), a `default` and a `description` (`types/inputs/`).
  - `evaluating/inputs.rs` binds them when a run or a call starts: it reads a text as its type and
    refuses the run naming the input. Callers pass templates or literals (`InputValue`).
  - A `set` step builds a `list` or an `object` from templates, each keeping the type it renders.
  - A `script` step adds `env` variables (names checked; `PORTAL_*` and the passed-through ones are
    reserved) and replaces its stdin with the JSON of `stdin`.
  - An `automation` step queues an automation through the `AutomationStarter` port
    (`services/dispatch/starter.rs`), with its `fields` overriding the manual event.
    - A run it queues carries the chain of automations that led to it (`Pending.origin`). An
      automation in its own chain, or a chain longer than 4, is refused.
    - Chained runs go to a second queue that `dispatch_children_forever` runs without a dispatcher
      permit, so parents that wait on them never deadlock.
    - `wait = true` polls the journal for the outcome within the step's budget.
- **Filters over lists and `each`.** The element filters (`FilterDescription.element`) map over a
  list. `Operation::Each` runs a nested chain on every element, nests at most 3 deep, and names a
  failure by its path, such as `operation 2.1`.
- **The answer's shape.** An `http` entry keeps, beside the body cut at 4 KiB, a `shape`
  (`helpers/shape.rs`):
  - it is valid JSON of at most 16 KiB;
  - lists keep 3 elements and texts keep 200 characters;
  - nesting is cut at 8 levels.

  The editor reads keys, types and previews from it, and from the run it last showed.
- **Variables in typed settings.** A filter argument may be a name without quotes
  (`get(loop.item.id)`, `FilterCall.names`), resolved from the frame when the template renders
  and checked by scope at load. In a `transform` step, `args` elements and `sort_by`/`group_by`/`count_by`
  keys that hold `{{` are rendered per operation. `set.object` and `http.headers` keys may be
  templates (`render_keys`: empty or duplicate keys and invalid header names fail the step). The
  number fields `repeat`, `max_iterations`, `seconds` and the step `timeout_seconds` are
  `NumberSetting`: a literal checked at load or a template resolved by `render_number` within the
  same bounds. The catalogue marks them `templated`, and table fields whose keys take templates
  `template_keys`. Pickers, choices, variable names, declared-name keys and the workflow timeout
  stay literal.
- **Step logs.** Every trace entry keeps `values`, the templates the step itself rendered and what
  each gave, and `log`, lines of what it did. `Frame.rendered` collects the values: `run_step`
  swaps in a fresh collector per step, so nested steps keep their own. `StepReport.log` carries
  the lines each step runner writes, such as a condition's sides and the branch, an HTTP status and
  time, or the value after each operation. Both are masked for secrets and bounded
  (`types/logging/`: 20 values and 20 lines of 300 characters per entry, 256 KiB per run, the rest
  counted). A `for_each` pass puts its item on its entries (`item`), so the timeline groups passes.
  The `log` kind writes a message with a level (`info`, `warning`, `error`) and never ends the
  run. Collection sits behind `StepLogging`, always on for now, so a later setting can switch it
  off. When `probe` or `status` gets an unknown id, the runner reads `PortalActions::state()` and
  adds the id of a service with that name and the known ids. A run records `steps_version`, a
  12-character SHA-256 prefix of the steps' JSON, which the workflow answer also carries.
- **The trace rides on the run.** `ActiveRun` holds a live `Trace` the runner appends to; `RunRecord`
  and `StoredRun` gain optional `workflow` and `trace` (200 entries, the rest counted), so old
  journal lines still load. The run journal and the workflow pages draw it as a timeline.
- **Script output.** A `script` entry keeps `Streams` (`types/progress/streams.rs`): standard output
  and standard error apart, each a `Tail` with its true byte count, and the command (the script and
  its rendered arguments), all masked for secrets in `runner.rs`. `Trace::finish` keeps each
  stream's last 16 KiB within 128 KiB per run; past that an entry keeps 1 KiB of each and sets
  `budget_reached`. `output` stays for `http` bodies and for old records, which load unchanged. A
  failed or timed-out script's detail adds the last non-empty line of standard error, else of
  standard output (`helpers/terminal_line.rs#last_line`, the Rust twin of `terminalText`), taken
  from the full tail before the budget. On the web, `TraceEntryView` shows "Details" on script
  entries, in the journal and in the editor's run panel alike; it opens
  `entities/automation/ui/trace/script-log-dialog.tsx`: the command quoted by
  `shared/lib/shell-quote` (also the automation builder's preview), outcome, exit code, duration, and
  both streams through `RunOutput`, or an old entry's single output.
- **Live runs without push.** A running run is polled every second (`ACTIVE_RUNS_MILLISECONDS`);
  nothing is pushed. `Frame.entry` names the trace entry of the running step, and the steps that wait
  (`wait`, `http`, `script`, `probe`, `notify`, a waiting `automation`) call
  `WorkflowRunner::publish` before waiting. It copies the values collected so far
  (`rendered::snapshot`), masked, onto the running entry through `Trace::progress`, and for a `wait`
  it also records `wait_seconds`. `Trace::finish` still replaces them with the full, bounded log.
  `TraceEntryResponse::of(entry, now)` answers a running entry's duration as the time since it
  started. The web ticks between answers (`widgets/workflow-editor/model/live-time.ts`): elapsed time
  is the answer's duration plus the time since `dataUpdatedAt`, so the two clocks never meet. A
  WebSocket or SSE channel was rejected: it would add a second authenticated path through restarts
  and Caddy, for under a second of gain.
- **Scripts that may fail.** `script` has `fail_on_error` (default `true`), as `http` does. With
  `false`, a non-zero exit is a success that keeps `exit_code`, the streams and the detail with the
  last line, and logs "tolerated". A timeout still fails. A boolean field set back to its catalogue
  default leaves the file (`step-form.tsx`); `script.fail_on_error` is shown inverted as "Continue the
  run when the script fails" (`INVERTED_FIELDS`), off by default. The appended error line is cut to
  120 characters; the trace timeline shows a step's detail on one line and its values and log lines
  on at most two, each whole on hover.
- **The interface rewrites `steps` as a whole.** `repositories/workflows.rs` sets the entry's own keys
  keeping their decor, as for automations, and replaces `steps` with freshly built arrays of tables
  (`[[workflows.steps.then]]`, …); `branches` stay inline. Comments inside `steps` are lost, comments
  above and on the entry's keys stay. A workflow in use cannot be deleted (409 naming its users).
- **Web:** `entities/workflow` holds the recursive zod schema, the pure tree functions (`at`,
  `insert`, `remove`, `move`, `duplicate`) on paths equal to the server's error paths, and:
  - `model/flow/`: `flowOf` builds the diagram (start, one node per step, joins, loop frames, end,
    arrows and the "+" slots) from the tree, and `layoutOf` places it deterministically, so nothing
    about positions is stored;
  - `model/flow/ends.ts`: how a list ends. `closingOf` gives `succeeded` or `failed` for a `stop`,
    and the loop exits for `break` and `continue`; `closes` also covers an `if`/`parallel` all of whose
    branches end; `closedAt` and `unreachableSteps` find the steps that never run; `needsLoop` and
    `insideLoop` say where `break` and `continue` may go (a loop body, through `if`, never through
    `parallel`). `build.ts` puts a `marker` node (`<path>:end`, `ui/nodes/marker-node.tsx`) under
    every ending step with no arrow to the join, drops the join of a fully ending `if`/`parallel`,
    emits the shared end node only when a path reaches it, and gives never-run steps `unreachable`
    (dimmed) and `dashed` arrows, which `checks/problems.ts` warns about. Each branch's closing arrow
    carries `rail`, a level `GAP_Y / 2` above its join: `ui/canvas/flow-edge.tsx#railPath` draws it
    down its own column to that level and only then across, so a short branch never crosses a long
    one, and its "+" (`slotSpot` in the same file) sits right under the branch's last step. The room lives in `model/flow/sizes.ts`: `GAP_Y = 96`,
    `GAP_X = 80`, `MARKER` and `MARKER_GAP`, and `FRAME_PADDING = 32`; the 32 px "+" and the loop
    return's `LOOP_CLEARANCE` are in `flow-edge.tsx`.
    `widgets/workflow-editor/model/checks/placing.ts` (`slotContext`, `placeable`) filters the
    palette, "Move to…" and dragging, and the palette offers the quick ends ("End run", "Skip", and
    inside a loop "Leave loop" and "Next pass");
  - `model/suggestions/`: `suggestionsAt` mirrors `checking/scope.rs` for the `{{` completion, with
    JSON keys from the last run, and `checkTemplate` gives the same reasons as the server;
  - `model/templates.ts` and `ui/templates-gallery.tsx`: the starter templates.

  `widgets/workflow-editor` draws the diagram with `@xyflow/react` (loaded by `next/dynamic` on the
  workflow pages only). It has two screens that share `ui/canvas-area.tsx` and the column on the
  right of the canvas (`ui/panels/side-column.tsx`):
  - `ui/workflow-page.tsx` with `ui/workflow-viewer.tsx`: the workflow's page, read-only.
    `ViewToolbar` offers "Run", "History", "Edit" and "Delete" (`entities/workflow/ui/delete-workflow-button.tsx`). The
    column holds the run list (`RunsPanel`, `GET /api/automations/runs?workflow=<id>`) on `history/`,
    or the run panel of `history/<run>/`, whose run is drawn on the canvas as a live path (highlighted
    flowing arrows, order numbers, dimmed unreached nodes). Both offer "Close" back to `<id>/`, which
    has no column and draws no run. Choosing a node opens `ui/inspector/step-card.tsx`, the step read
    back from the catalogue with no inputs, in that column; on a run's address it sits folded as
    "Settings" in the run panel. Running nodes carry `ui/nodes/live-badge.tsx` (a countdown ring on a
    `wait`, elapsed time otherwise). Nodes that ran show their templates' values as chips
    (`model/values-on-nodes.ts#withValues`, marking whole-field matches inside the summary), with the
    "Values | Templates" switch (`ui/canvas/values-switch.tsx`);
  - `ui/workflow-editor-screen.tsx` with `ui/workflow-editor.tsx`: the edit and new pages.
    `EditToolbar` offers undo, redo, "Problems", "Cancel" and "Save". The column holds the
    inspector (a bottom sheet on a phone) or the problems list.

  Both load through `model/use-workflow-data.ts`. `useEditorState` takes `readOnly` and the run to
  show from its screen; it has no mode of its own. `shared/ui/template-input` is the field with
  highlighting and completion, also used by the automation builder. `widgets/workflows` is the list,
  and the trace timeline sits in `entities/automation` beside the run details.

  **Addresses.** `app/admin/workflows/workflows-route.tsx` hosts every workflow address. It is the
  page of both `/admin/workflows/` and `/admin/workflows/new/`, and the portal serves it for
  `/admin/workflows/<id>/…` as their nearest page (§ on serving above). After mount it reads the
  path with `shared/lib/navigation#useAddress` and picks, through
  `entities/workflow/model/address.ts#workflowAddressOf`, one of:
  - the list;
  - the editor (`new`, `<id>/edit/`);
  - the workflow's page (`<id>/`, `<id>/history/`, `<id>/history/<run>/`).

  It also keeps the run last shown for the editor's suggestions. Moves between these addresses go
  through `pushAddress` (native `history.pushState`) and `shared/ui/address-link`, never through
  `router.push`, which would fetch a route payload the export does not have. `useLeaveGuard`
  registers with `useAddress`, so an unsaved edit asks before any such move, "Back" included.
  `/admin/workflows/edit/?id=` is a redirect to `<id>/edit/`, which is why `new` and `edit` are
  reserved workflow ids. `pnpm dev` gets the same fallback from a `rewrites().fallback` entry.

### 6.12. Notifications

- **A module with pluggable channels.** `crates/features/portal-notification` is the feature
  `notification` behind the module switch `notifications` (on by default, needs nothing). It owns
  the rules, one bounded `Outbox` (100 messages, the oldest dropped and counted) and one sender loop
  per channel, and the `DeliveryBook` of each channel's last delivery and last error.
- **A channel only delivers.** A channel is a crate under `crates/notification/` that implements the
  `Channel` port of `portal-feature`: its name, the problems of its table, its readiness (ready, off,
  or the missing key), its settings as JSON without secret values, applying new settings to its table,
  and `deliver`. Secrets come through the `SecretSource` port, which `ConfigStore` implements.
  `bin/home-portal/src/features/channels.rs` is the only channel list; `tests/architecture/channels.rs`
  compares it with the folder and refuses a channel that depends on a feature.
- **The rules** are `[notifications]` `states` and `recovered`; each key falls back to the legacy
  `[notifications.telegram]` and then to the default. Saving the rules writes them to
  `[notifications]` and removes the legacy keys in the same write, keeping comments. A channel table
  (`[notifications.<channel>]`) is written only by `ChangeChannel`.
- **Delivery.** The `StatusObserver` pushes each announced change to every ready channel's outbox
  while the module is on. `SendNotification` delivers at once, to one channel or to every ready one,
  and is what the workflow `notify` step calls; it fails while the module is off or when no channel
  is ready. Telegram keeps its 10 s timeout and three attempts.
- **API:** `GET/PUT /api/notifications` (the rules, with `If-Match`),
  `PUT /api/notifications/channels/{name}` (422 by field, 404 for an unknown channel) and
  `POST /api/notifications/test` (409 while off or not ready). No answer carries a secret value.
- **Web:** `entities/notification` holds the schemas and queries; the page
  `/admin/notifications` lives in `widgets/modules` (`ui/notifications/`), because `widgets/` is at
  steiger's slice limit. Channel settings are drawn from their JSON: a switch for a boolean, the
  secret picker for `secret`, a text field otherwise.

### 6.13. Scripts

- **The scripts directory is its own feature, `portal-scripts`.** It owns where a script may lie
  and whether it can run (`services/directory.rs`: links, owner, modes, every folder down to the
  file) and serves them as the use cases `ResolveScript` and `ListScripts`. Automations, webhooks and
  workflows only run scripts: `portal-automations` reaches the directory through its
  `ScriptLibrary` port, adapted by `adapters/script_library.rs`, which maps each problem code onto
  the journal's `RefusalCode` one to one. Spawning a process stays in `portal-automations`.
- **One path rule.** `portal_model::ScriptPath` is the pure shape rule (relative, no `..`, no hidden
  part, at most one folder deep) that both crates check with, on load and before every run.
- **Who the portal is.** The runnability checks need the portal's user and groups, which only
  `libc` answers, and `unsafe` lives in one file (§6.9). `portal-scripts` asks its `ProcessIdentity`
  port, and `adapters/process_identity.rs` answers from
  `portal_automations::{effective_user, effective_groups}`.
- **What a script declares.** `portal_model::ScriptHeader::parse` reads `@description` and `@arg`
  lines from the comments at the top of a file (after `#!`, at most 64 lines and 8 KiB), never by
  running it. `services/headers.rs` caches it by path, mtime and size, and reads it only for files
  that lie inside the directory, so a link out never shows another file's first lines. The
  automations listing carries it through the port; the interface maps it onto `args` itself
  (`entities/script/model/arguments.ts`), so the server never interprets it and hand-written `args`
  keep working. `samples/script-headers.json` holds the cases both parsers must agree on.
- **Editing is switched in the file only.** `[scripts] editing` (`services/settings.rs`, validated as
  the feature's own section) is not a module and no endpoint writes it. The routes of `/api/scripts`
  are always registered and every handler asks `ScriptEditing` first, answering 404 while it is off,
  so a change applies on the next read without a restart. Writes also need the address-detected
  environment (`DetectedEnvironment`, never the cookie's choice) not to be `internet`, else 403
  (`ApiError::Forbidden`), and are logged with the user and the path, never the content.
- **Writes are whole.** `services/writer.rs` holds one lock per directory, refuses any path part that
  is a symbolic link, writes a hidden `.<name>.tmp-…` with `create_new` and mode 0700, syncs it,
  renames it over the target and syncs the folder, so a run never sees half a file and a crash
  leaves only a hidden file, which the feature sweeps after an hour. Revisions are
  `portal_config::Revision` (SHA-256 of the bytes), with 428 without `If-Match` and 409 when stale.
  Texts are at most 256 KiB of UTF-8 without NUL; the directory holds at most 500 files; folders
  are one level deep, created 0700 and removed only when empty.
- **Web:** `entities/script` holds the schemas and queries of `/api/scripts`, the TypeScript mirror of
  the header parser (`model/header.ts`, checked against `samples/script-headers.json`), the mapping
  onto `args` and the argument form (`ui/declared-arguments.tsx`), which the automation builder's
  `RunCard` (also used by the webhook form) and the workflow inspector
  (`ui/inspector/script-arguments-field.tsx`) share. The page `/admin/scripts` lives in
  `widgets/automations` (`ui/scripts/`), because `features/` and `widgets/` are at steiger's slice
  limit; its text area is `shared/ui/code-area`, which paints a small in-house highlight
  (`highlight.ts`) under a transparent `<textarea>`, so no editor library is needed. The navigation adds "Scripts" after the modules
  from `editing` in the automations listing (`widgets/app-shell/model/navigation.ts#sectionLinks`).

### 6.14. Host permissions

- **The feature `portal-permissions` asks macOS for its privacy permissions.** It is always on and is not a module. The permissions are:
  - `local-network`;
  - `removable-volumes`;
  - `folder:<name>` for each folder in `[permissions] folders`;
  - `automation:<application>` for each application in `[permissions] automation`;
  - `full-disk-access`.
- **There is no silent check.** For every permission except Full Disk Access, macOS shows its prompt on the first access the owner has not decided. So asking and checking are the same act: one harmless action per permission, one file per kind in `clients/`, with `std` only and no `unsafe`:
  - an mDNS query to `224.0.0.251:5353`;
  - `read_dir` of each removable volume under `/Volumes`, filtered with `diskutil info -plist`;
  - `read_dir` of each named folder;
  - `osascript` with `tell application (item 1 of argv) to get name`.

  Full Disk Access has no prompt. It is only read, by opening `TCC.db`, and its advice says to add the process by hand.
- **A blocked read stays pending.** A file read waits in the kernel until the owner answers. So `services/asking.rs` runs every check in `spawn_blocking`, waits up to the limit (`types/limits.rs`: 60 s for prompts, 10 s for the network, 5 s for `diskutil`), records `pending`, and leaves the task running to record the answer later.
  - At most one check per permission is in flight (`PermissionBoard::begin`).
  - One request runs at a time. `RequestPermissions` holds a `try_lock`, and a second request is `409`.
  - States live in memory only and are learned again at each start.
- **At start** the request runs through `Feature::loops()`, which `boot/run.rs` spawns right after the listener is bound, so start-up never waits for an answer. `[permissions] request_at_start = false` leaves only Full Disk Access checked. The process tests and `tests/modules.rs` set it, so `cargo test` on a Mac shows no prompt.
- **Who owns a permission.** macOS gives a process the permissions of its *responsible* process:
  - from a terminal, the terminal application (`TERM_PROGRAM`);
  - under launchd, the `home-portal` binary.

  Every script the portal runs inherits them. `Owner` names that process in the log, in `home-portal permissions` and on `/admin/permissions`. macOS keys an unsigned binary by its code hash, so a rebuilt binary loses its grants: run launchd from an installed copy, and run `home-portal permissions` after each upgrade.
- **Entry points:**
  - `GET /api/permissions` gives the states with an `advice` and a `pane` code, which the interface turns into text;
  - `POST /api/permissions/request` answers 202, or 403 from `internet`;
  - `home-portal permissions` (`cli/permissions.rs`) asks, waits and prints `<code>\t<state>` lines when piped, and exits 1 when anything is denied.
- **Web:** `entities/permission` holds the schema, the queries (a 2 s poll while anything is `pending`) and the state badge. The page lives in `widgets/modules/ui/permissions/`, because `features/` and `widgets/` are at steiger's slice limit. The menu lists it after the modules.

### 6.15. Access control

- **Groups hold rights.** `[[groups]]` live in the users' home beside `[[users]]` (`Section::Users` owns both tables). Each group has a `name` and `permissions = { <area> = [<actions>] }`. A user names at most one group in `group`.
  - `admin` is built in: it is never declared, it holds every right, and at least one user must be in it, or the start fails with a message naming `group = "admin"`.
  - A user without a group holds no rights.
- **The matrix is one list.** `portal_feature::Area` gives each area the actions it accepts (`Area::actions`):
  - modules: `proxy`, `dns`, `notifications`, `automations`, `webhooks`, `users`, `workflows`, with `execute` for the three that run things;
  - functions outside modules: `services`, `layout`, `network`, `modules`, `scripts`, `secrets`, `host-permissions`, `portal`, each only with the actions that mean something for it.

  `GET /api/groups` answers the same matrix, so the interface draws the editor without a list of its own. `Rights::update` never implies `read`.
- **Rights are resolved per request.** `SessionGate::admit` reads `[[users]]` and `[[groups]]` on every request and builds `Principal { name, group, rights }` (`portal-auth/src/services/groups.rs`), so a change of group or rights applies at once, without signing in again. Sessions store no rights.
- **Routes declare what they need.** `Feature::rules()` lists `Rule { method, path, requirement }`, where the requirement is:
  - `Signed`, for what the home page and the service page need;
  - `AnyOf(&[Right])`;
  - `Admin`, for writing groups.

  `boot/router.rs` collects every rule, plus those of the widgets router and `POST /api/portal/restart`, into a `RuleBook`. `middlewares/require_right.rs` runs inside `require_session`: it finds the rule by `MatchedPath` and method, answers 403 `needs <area>.<action>`, and fails closed for a route without a rule. Handlers stay as they were.
- **Checks that depend on the target live in use cases** (`services/granting.rs`):
  - one's own password needs no right;
  - only `admin` touches a member of `admin` or gives `admin`;
  - anyone else may give only a group whose rights are within their own;
  - the last member of `admin` cannot leave it or be deleted.
- **Tests keep the table complete:**
  - `tests/architecture/rules.rs` reads every `.route(…)` in `router()`, `widgets_router()` and `assemble()` with `syn`, and compares both directions with the `RuleBook`;
  - `tests/rights.rs` sends one request per rule as a user without a group, and every `GET` as `admin`.
- **Web:**
  - `entities/session` carries `group`, `admin` and `rights`, with `useCan()`, `mayOpen()`, `<Allowed>` and `<RequireRight>`;
  - `AreaGate` in `app/admin/layout.tsx` maps each admin path to its area and shows `shared/ui/no-access` without the right, before any query of the page runs;
  - action buttons are guarded inside their `features/` slices, and forms without `update` become a disabled `fieldset` without a submit;
  - the navigation filters links with `mayOpen`;
  - `testQueryClient()` starts with an `admin` session, and tests of other people set theirs.
- **Webhooks can be run from the interface.** `POST /api/webhooks/{id}/run` needs `webhooks.execute` and runs exactly what a received call runs, without the token, through `services/webhook_reception.rs`, which the public receiver shares. The journal records who started it.

## 7. Async and cost

- The portal is async (`tokio`, `axum`). vigil's "no async runtime" was a measured decision for a
  daemon on a customer's host and does not carry over; its cost discipline does.
- **A handler never performs work whose cost grows with its input.** Long work is a background
  loop or job; the handler returns what is already known.
- **Everything that waits on a third party has a timeout**, and every background pass has a
  ceiling on how much it reads.
- **Measure first.** A new dependency comes with its transitive crate count, clean build time and
  binary size.

---

## 8. Conventions

- **Language.** English in code, log lines, API bodies, `ARCHITECTURE.md` and commit messages.
  Text shown by the interface comes from message dictionaries (§12): English, Russian and Spanish,
  one file each, on one key set.
- **No abbreviations in names.** `address`, not `addr`; `configuration`, not `cfg`; `request`, not
  `req`. Names forced by an external API are the only exception.
- **No comments in `.rs` files** — no `//`, `///`, `//!` or `/* */`. The reason for a decision
  goes into a test whose name is the sentence you wanted to write
  (`an_internal_error_never_shows_its_diagnostic_to_the_client`). Prose for a reader of the code
  goes into this document or a design under `docs/designs/`. **Checked:**
  `tests/architecture/comments.rs`.
- **Build files carry no comments either** (`Cargo.toml`, `justfile`), except the one line `just`
  prints as a recipe's description.
- **No `#[allow(...)]`.** Fix the cause of the warning. **Checked:**
  `tests/architecture/allows.rs`.
- **Shipped text never mentions internal process** — no task numbers, phases or document names in
  code, logs or responses.
- **Documents are dated**: `docs/designs/YYYY-MM-DD-DESIGN-slug.md`, named on the day written.

---

## 9. Quality gates

| Aspect | Decision | Status |
|---|---|---|
| Edition / resolver | `2024` / `3` via `[workspace.package]` | ✅ |
| Toolchain | `rust-toolchain.toml`, channel `1.98.1` with rustfmt and clippy | ✅ |
| The gate | `just check` = fmt-check → clippy → test → web lint → web typecheck → web test → web build | ✅ |
| Style | `cargo fmt --all --check` | ✅ |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ |
| Tests | `cargo test --workspace`; no live service, no network | ✅ |
| Comments in `.rs` | none — `tests/architecture/comments.rs` | ✅ zero |
| `#[allow(...)]` | forbidden — `tests/architecture/allows.rs` | ✅ zero |
| `unsafe` | only in the process-group file of `portal-automations` — `tests/architecture/unsafe_code.rs` | ✅ one file |
| Module roots | declarations only — `tests/architecture/roots.rs` | ✅ |
| File / function / folder size | 400 / 300 lines, 12 files — `tests/architecture/sizes.rs` | ✅ |
| Folder names | no catch-alls — `tests/architecture/folders.rs` | ✅ |
| Feature registry | matches `crates/features/` — `tests/architecture/registry.rs` | ✅ |
| Channel registry | matches `crates/notification/`, core dependencies only — `tests/architecture/channels.rs` | ✅ |
| Entry points and storage | only through use cases — `tests/architecture/layers.rs` | ✅ |
| `main.rs` is a shim, the root is `boot/` | `bin/home-portal` | ✅ |
| API samples match the frontend schemas | `bin/home-portal/tests/samples/` + entity schema tests; `just samples` rewrites | ✅ |
| Widget kinds the interface can draw | `web/src/features/widget-board/model/registry.test.ts` against `widget-kinds.json` | ✅ |
| The example configuration and its secrets file | `bin/home-portal/tests/configuration.rs` | ✅ |
| Every file under `examples/` loads, and names only known fields | `bin/home-portal/tests/examples.rs` | ✅ |
| Both language copies of this document | equal headings, every path named exists — `tests/architecture/documents.rs` | ✅ |
| Frontend layers | `steiger` with the FSD plugin, `fsd/insignificant-slice` off (§12) | ✅ |
| Frontend source rules | no comments, 400/300 lines, a test beside every `shared/ui` file — `web/src/shared/lib/architecture.test.ts` | ✅ |
| Interface text through dictionaries | `react/jsx-no-literals` + typed message keys | ✅ |
| Node / pnpm | `web/.nvmrc` (24), `packageManager` pnpm 11, `--frozen-lockfile` | ✅ |
| No abbreviations | enforced in review | ✅ |
| Continuous integration | implemented | ✅️ |

---

## 10. Deliberate exceptions

Each exception is one line in the block below: a rule (`size` or `root`) and a path relative to
the workspace root. The guard tests read this block, so an exception without a line here does not
exist. Each line needs a row in the table with its rationale. An empty list is better than an
unrecorded violation.

```exceptions
```

| Where | What | Why |
|---|---|---|
| `crates/core/portal-widget` | a controller in `core/`, contrary to §3.3.1 | The widget data endpoint is the same for every widget type, and the cache, the shared refresh and the environment filter behind it are the plumbing every widget feature links. Putting the route in a feature would make that feature a hub every other widget feature depends on, which §2 forbids; putting it in the composition root would move domain code into `bin/` and make it untestable without the whole binary. The crate knows no widget type: the types are declared by providers, exactly as `portal-web` serves an interface it does not build. |
| `crates/core/portal-config` | io in `core/`: it reads, stats and writes the configuration file | Every feature reads and writes its own section of one file, and all of them must agree on the revision, the stale-write check, the atomic rename and the 0600 mode. One owner in the layer every feature links is the only place that agreement can live; a copy per feature would drift on the first edit. The crate knows no section: parsing and validating a section stay in the feature that owns it. |

---

## 11. Recipes

**Add a feature.** Create `crates/features/portal-<name>/` with `Cargo.toml` (`edition.workspace`,
`version.workspace`, `portal-feature.workspace = true`) and add it to the workspace members and
`[workspace.dependencies]`. Inside: `features/<name>.rs` with a struct and `impl Feature`
(`name() == "<name>"`), handlers in `controllers/`, and a `lib.rs` that re-exports the feature
struct only. Add it to `bin/home-portal/src/features/registry.rs`. `just check` fails until both
the crate and the registry line exist.

**Add a route to a feature.** Add the handler to the controller file of its resource (a new
resource is a new file in `controllers/`), the wire shapes to `requests/` and `responses/`, the
logic to `services/`, and the route to the feature's `router()`. Test it through the router with
`tower::ServiceExt::oneshot`, without a socket. Errors are `ApiError`.

**Add an integration with a service.** Create the feature crate as above. Put the transport in
`clients/` with explicit timeouts, the status check in `probes/`, the periodic pass in `loops/`
started by the root, and the status vocabulary in `portal-model` (creating the crate if this is
the first). Test "the service answered", "the service answered garbage" and "the service could not
be reached" as three different results, against a fake.

**Add a dashboard widget.** Create its component in `web/src/features/widget-board/ui/kinds/`, add a
line to `model/registry.ts` with its settings schema, its title key and — when the portal serves its
data — the schema of that data, and add its messages to every dictionary. The server passes any
widget type through unchanged; it needs a Rust change only when the widget needs new data.

**Add a widget the portal fetches data for.** Create the feature crate as above, put the transport
in `clients/` with explicit timeouts and a `WidgetProvider` in `providers/` that declares its type,
checks its settings and returns a reading with a refresh period. Return it from
`widget_providers()`; `portal-widget` does the caching, the staleness and the route. Add the sample
to `bin/home-portal/tests/samples/`, run `just samples`, and add the entry to the interface
registry — the registry test fails until it is there.

**Split an overgrown file.** Identify cohesive groups of methods, move them into concern files as
child modules of a `<type>/` folder, keep the trait impl thin, and leave the crate's public facade
unchanged.

---

## 12. Frontend (`web/`)

Next.js (App Router) with `output: "export"`, pnpm, TypeScript strict, Tailwind CSS v4 and
shadcn/ui primitives on Radix. The design and its rejected alternatives are in
`docs/designs/2026-09-22-DESIGN-web-interface.md`; widgets, environments, icons and the public
portal are in `docs/designs/2026-09-22-DESIGN-widgets-and-environments.md`; the single home page,
the management area, service pages, the layout editor and probe diagnosis are in
`docs/designs/2026-09-23-DESIGN-portal-usable.md`.

### 12.1. Layers

- Feature-Sliced layers under `web/src/`: `app` (Next routes only: a page mounts one screen) →
  `widgets` (screens) → `features` (user actions) → `entities` (schemas, API calls, query hooks) →
  `shared` (kit, API client, i18n, config). Imports go down only, through a slice's `index.ts`, never
  sideways; `steiger` enforces it. `fsd/insignificant-slice` is off: a user action such as the theme
  switch or the sign-in form stays its own slice even while one screen uses it, because it is the
  unit the dashboard editor and future screens reuse.
- **Routes.** `/` is one home page for every visitor (`widgets/home`) and `/service/?id=` a service's
  page (`widgets/service-page`); both sit in the `app/(site)` group under one header
  (`widgets/site-header`). Management lives under `/admin/` (`services`, `layout`, `network`,
  `proxy`, `automations`) inside
  `AppShell`; the old `/services/` and `/settings/network/` redirect there (`app/(legacy)`). A
  service is added and edited on pages of its own, `/admin/services/new/` and
  `/admin/services/edit/?id=` (`widgets/service-editor` around `features/service-form`). An automation
  is built the same way on `/admin/automations/new/` and `/admin/automations/edit/?id=`
  (`widgets/automation-editor` around `features/automation-builder`).
- The widget registry is a slice of its own, `features/widget-board`: widgets may not import each
  other, and the home page, the service page and the layout editor's preview all draw with its
  `BoardGrid`. The layout editor is split the same way: `features/layout-editor` holds the draft, the
  dragging and the saving and knows no widget type; `widgets/layout-editor` hands it the registry's
  types, settings forms and preview. The service card is a view of a service, so it
  lives in `entities/service`, and the status vocabulary both halves parse lives in `shared/api`.
- Static export has no middleware, rewrites or dynamic segments: sign-in redirects happen in the
  browser after a 401, a record is chosen by query parameter or in a sheet, and the dev server alone
  proxies `/api` to `HOME_PORTAL_ORIGIN` (`next.config.ts` chooses by phase).

### 12.2. The kit

- `shared/ui/primitives/` holds the shadcn/Radix primitives as owned code; `shared/ui/<component>/`
  holds the application kit (`PageHeader`, `SectionCard`, `KvList`, `StatusBadge`, `EmptyState`,
  `DataTable`, `FormField`, `ConfirmDialog`, `ErrorNotice`, `LatencyChart`, `UptimeStrip`, `TimeAxis`,
  `TagInput`, `TagFilter`, `Redirect`, `JsonView`, `TemplateInput`). Hooks without a look (`useLeaveGuard`, `useElementWidth`) live in
  `shared/lib/`.
  Screens compose them and do not restyle
  them: a variation is a prop, never a class passed in.
- Colours, radii and the status palette are CSS variables in `app/globals.css`, in both themes —
  the hook for customizing the look later.
- The look is glass: a fixed backdrop (`--backdrop`, `--backdrop-glow-*`) painted on `body::before`,
  and translucent surfaces from two utilities defined in `globals.css`. `glass-panel` is for what sits
  on the page; `glass-overlay`, more opaque, is for dialogs, sheets, menus and toasts. A component
  gets blur only through these utilities, never its own `backdrop-blur`; the overlay dimmers of
  dialogs and sheets are the one exception. An element inside a glass surface uses the
  `bg-glass-tint` tint and does not blur again. The fallbacks for no `backdrop-filter`, reduced
  transparency and more contrast make the surfaces opaque. `shared/lib/contrast` tests that text on
  glass keeps AA contrast over every backdrop colour in both themes.
- The management menu collapses on a wide screen into a rail of icons with tooltips
  (`widgets/app-shell`). The choice is kept in `localStorage` under `home-portal.menu-collapsed`
  and read by the lazy state initializer. The shell renders only after the session loads, so the
  menu never jumps.

### 12.3. Data, forms and text

- HTTP only through `shared/api/request`: it sends `If-Match`, reads `ETag`, redirects to sign-in on
  401 and turns 409, 422 and 429 into typed errors. Every response is parsed with a zod schema;
  status values the interface does not know become `unknown`.
- Status refreshes every 10 seconds while the tab is visible, pauses while hidden, and never replaces
  data on screen with a skeleton.
- Forms use react-hook-form with zod schemas that mirror the server's rules; server field errors are
  shown beside their fields, and a 409 keeps what the person typed.
- Every visible text is a message key (`react/jsx-no-literals`, typed keys through next-intl);
  a test keeps all dictionaries on one key set and every message on its original's placeholders.
- **Languages.** `web/scripts/build-languages.mjs` runs `next build` once per dictionary with
  `PORTAL_LANGUAGE` set, puts each build's pages under `web/out/<language>/` and merges the
  `_next/` trees, failing on any path whose bytes differ. The portal decides a page request's
  language in `bin/home-portal/src/middlewares/language.rs` (the `portal_language` cookie, then
  `Accept-Language`, then `[interface] default_language`, English by default) and `portal-web`
  answers from that language's tree at the same address, with `Content-Language` and
  `Vary: Cookie, Accept-Language`. `features/language-switch` writes the cookie and reloads. A
  `[locale]` segment was rejected because it puts the language in every address; the reasons and
  the measurements are in `docs/designs/2026-09-25-DESIGN-interface-languages.md`. `next dev`
  serves the language of `PORTAL_LANGUAGE` and ignores the cookie.

