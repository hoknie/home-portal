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
  portal-feature/         Feature, Gate, StatusObserver, EventSink and WidgetProvider ports, PortalEvent,
                          ApiError, FieldError
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
  portal-telegram/        Telegram messages when a service changes state
  portal-secrets/         which secrets are named and whether each one is set
  portal-public/          the portal page shown without a session
  portal-proxy/           publishing services through Caddy: its configuration, forward-auth, TLS
  portal-automations/     the owner's scripts, run on a schedule or on the portal's events
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
| `controllers/` | axum handlers for one resource per file: extract, call a service, answer | business decisions, io beyond the service |
| `requests/` | wire shapes of incoming bodies and queries | domain types |
| `responses/` | wire shapes of outgoing bodies | domain types |
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
- Two features claiming one path make axum panic at assembly; assembly runs in a test, so the
  conflict fails `just check`.
- **When roles arrive**, `Feature` gains a method returning its routes as data (method, path,
  required permission), and the gate reads that data.

---

## 5. The web layer

- **Wire types stay at the edge.** A controller converts a request into domain values, calls a
  service and converts the result into a response. Services never see `requests/` or
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
  "contains a dot" misroutes hostnames and addresses in route parameters — and everything else gets
  the entry page. `_next/static/` is cached immutably, HTML with `no-cache`. A binary built without
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

- **One TOML file holds every setting** — `network`, `users`, `services`, `dashboard`,
  `environments`, `notifications` — at `HOME_PORTAL_CONFIG`, by default
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
- **The file may name further files.** `include` lists paths inside the main file's directory, read
  in that order. Lists (`services`, `users`, `dashboard.widgets`) are the concatenation of every
  file's entries; a plain key defined twice is an error naming both files. Nesting is refused, so
  the set of files is visible in one place. The merged document carries where each entry came from,
  and `configuration.writes_to` says which file the interface edits.
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
  (`validator()`) and edits it (`repositories/`). `portal-config` knows no section.
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
  Before every run `services/scripts.rs` resolves it and refuses a file outside the directory, one
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

## 7. Async and cost

- The portal is async (`tokio`, `axum`). vigil's "no async runtime" was a measured decision for a
  daemon on a customer's host and does not carry over; its cost discipline does.
- **A handler never performs work whose cost grows with its input.** Long work is a background
  loop or job; the handler returns what is already known.
- **Everything that waits on a third party has a timeout**, and every background pass has a
  ceiling on how much it reads.
- **Measure first.** A new dependency comes with its transitive crate count, clean build time and
  binary size.

  Most of the second step is `reqwest` with rustls and its aws-lc crypto provider. The last step
  adds 15 crates: `sysinfo`, `ical`, `ipnet`, `if-addrs`, `mime_guess` and what they pull in.

  ICMP probes send the echo themselves over `socket2`, already in the graph, so probe kinds add
  no crate. `surge-ping` was measured and refused: it takes the graph from 206 to 223 crates
  (`pnet_packet` and its macros, `parking_lot`, a second `rand`) for about 60 lines of code.

  Automations add `jiff` with only the system zone and the zoneinfo database (`jiff` and
  `jiff-core`): cron needs IANA zones and daylight saving, which `time` cannot give safely in a
  threaded process. `cron` and `croner` were refused because they bring `chrono` and a zone
  database of their own; the cron parser is ours and tested by tables. `libc` was in the graph
  already. The rest of the last step is the automation pages in three languages.

  The interface counts its cost in gzipped JavaScript a page loads. `react-markdown` renders service
  notes and costs the service page 35 KiB (445 → 481 KiB); `@dnd-kit` drags widgets in the layout
  editor and costs `/admin/layout` about 9 KiB over `/admin/services`. Neither reaches the home page.

  Every interface language is a build of its own (§12.3). The three builds share one `_next/`
  byte for byte, because the messages travel in each page's RSC payload and not in the scripts, so a
  language costs only its HTML: 1.95 MB on disk, 0.8 MB for the two added languages in the binary.
  `just web` runs three `next build`s, about 45 s in all against 15–25 s for one.

- Measured on the same host with the release binary running:

  | What | Measured |
  |---|---|
  | One host-metrics refresh (a full `sysinfo` pass, end to end over HTTP) | 15–26 ms |
  | The same reading from the cache | 0.6–0.7 ms |
  | The icon cache after ten catalogue icons | 279 KiB in `icons/`, one file each |

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
  `TagInput`, `TagFilter`, `Redirect`). Hooks without a look (`useLeaveGuard`, `useElementWidth`) live in
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

