# `home-portal` Architecture

## 0. How to read this document

These are the **rules** of the codebase: how it is built and what must not be broken. They apply to
all new code and to every file you touch. Behaviour — fields, defaults, routes, screens — is
specified in `openspec/specs/<capability>/spec.md`, not here. A gap between the tree and a rule is
debt (§9); a deliberate exception is listed in §10, and any other violation is a defect.

---

## 1. Core principles

1. **A workspace with explicit boundaries.** Crate boundaries are dependency boundaries, enforced by
   the compiler.
2. **Dependencies flow one way:** `bin → features → core`. A feature never knows another feature;
   `core` knows no feature.
3. **A feature declares, the root asks.** Each feature describes itself through the `Feature` port.
   The binary keeps one list of features and no tables that repeat what features declare.
4. **Quality is a gate.** Formatting, lint, tests and the architecture guards run as `just check`.
   A rule a machine can check is checked by one (§9).

---

## 2. Workspace topology

```
crates/core/           LAYER 1 — vocabulary and ports; knows no feature
  portal-model/        service, status and environment vocabulary; pure
  portal-feature/      the ports (Feature, Gate, EventSink, WidgetProvider, Channel, …), ApiError,
                       modules and rights
  portal-config/       the configuration files: merge, secrets, revision, atomic writes
  portal-widget/       the widget registry, its cache and its data endpoint
  portal-web/          serves the built interface found at start-up
crates/features/       LAYER 2 — one crate per subject: auth, services, network, dashboard, proxy,
                       dns, automations, widgets, scripts, notification, modules, …
crates/notification/   LAYER 2 — one crate per notification channel; core crates only
bin/home-portal/       COMPOSITION ROOT — boot/, features/, adapters/, middlewares/, cli/
web/                   the interface: a Next.js static export (§12)
```

- **The folder is the layer.** A `path` dependency may point down, never sideways.
- **A subject is a crate.** Its clients, probes, handlers and tests live in
  `crates/features/portal-<subject>/`, and `Feature::name()` is `<subject>`.
- **Siblings meet through ports.** A feature that needs another subsystem declares a trait in its
  `ports/`; the root implements it in `bin/home-portal/src/adapters/`.
- **A crate is a subsystem of our own.** A wrapper around one library, or a crate of only `pub use`,
  must not exist. A second implementation of a port gets its own crate only when it pulls in a heavy
  dependency.
- **`portal-model` is pure:** no io, no async, no network.

---

## 3. Module and file organization

### 3.1. Module roots contain declarations only

`lib.rs` and `mod.rs` hold only `mod` and `use` lines; `main.rs` only calls `home_portal::start()`.
**Checked:** `tests/architecture/roots.rs`.

### 3.2. One logical element per file

- One trait, or one struct or enum with its `impl` blocks, per file. Free functions go to `helpers/`.
- One controller (the handlers of one resource) per file; one request or response shape per file.
- Magic literals become named `const`s beside what they mean.
- A closed vocabulary is an enum; when it crosses a wire it has an `Unknown` variant.
- A method takes at most 4 arguments, a free function at most 3; group more into a struct.

### 3.3. A kind of element is a folder

Each kind lives in a folder named after it **in the plural**, even for one element. A kind folder
holds only its kind; past 12 files it is split by subdomain. A domain folder is singular (`workflow/`)
and may hold kinds; a kind never holds another kind.

| Folder | Contains | Does not contain |
|---|---|---|
| `ports/` | traits towards other subsystems | implementations |
| `features/` | the crate's `impl Feature` | handlers, logic |
| `controllers/` | axum handlers: extract, call a use case, answer | decisions, storage |
| `requests/`, `responses/` | wire shapes | domain types |
| `usecases/` | one operation of an entry point, one struct with `run` | wire types, other use cases |
| `services/` | logic over the crate's own domain | wire types, axum |
| `clients/` | transport to one external service, with timeouts | what the answer means |
| `probes/` | one status check per file | storage, handlers |
| `parsers/` | bytes of a foreign format → records or a named refusal | io |
| `repositories/` | storage access | domain decisions |
| `loops/` | what one tick of a background pass does | who starts it |
| `types/` | domain values and their invariants | foreign wire formats |
| `helpers/` | pure functions | state |
| `adapters/` | a port implementation, in the root only | anything that implements no trait |
| `boot/` | one assembly step per file, in the root | domain logic |

`utils`, `common`, `misc`, `structs`, `enums`, `traits`, `impls` are forbidden. **Checked:** `folders.rs`.

**Entry points reach storage only through use cases.** Files under `controllers/` and the root's
`adapters/`, `middlewares/` and `cli/` never name `repositories`, `ConfigStore`, `Snapshot`,
`.document` or `.configuration`. Only `cli/` opens the store. Other crates reach a feature's settings
through its public use cases, and a use case answers `Revisioned`, never `Snapshot`.
**Checked:** `tests/architecture/layers.rs`.

**The command line is declared once** (`bin/home-portal/src/types/command_line.rs`) and styled only
through `cli/palette.rs`.

### 3.4. File and function size

Above 400 lines per file, 300 per function or 12 counted files per folder fails the gate; a struct
of more than 7 fields becomes a composition. **Checked:** `tests/architecture/sizes.rs`.

### 3.5. Re-export facade

Consumers write `portal_feature::ApiError`, never the inner path, so moving a file stays local.

---

## 4. The `Feature` port and the composition root

```rust
pub trait Feature: Send + Sync {
    fn name(&self) -> &'static str;
    fn router(&self) -> Router;
    fn public_router(&self) -> Router { Router::new() }
    fn rules(&self) -> Vec<Rule> { Vec::new() }
    fn validator(&self) -> Option<Validator> { None }
    fn loops(&self) -> Vec<Loop> { Vec::new() }
    fn widget_providers(&self) -> Vec<Arc<dyn WidgetProvider>> { Vec::new() }
    fn stop(&self) {}
}
```

- **Routers come with their state applied**; the root sees only `Router<()>`.
- **A route is protected unless its feature puts it in `public_router()`**, and every protected
  route declares its right in `rules()` (§6.15).
- **Validators are plain functions** over the configuration document, collected into the store;
  checks that need state are closures handed to `adopt_checks`. Both run at start and on every write.
- **`bin/home-portal/src/features/registry.rs` is the only list of features**, and
  `features/channels.rs` the only list of channels. **Checked:** `tests/architecture/registry.rs`,
  `channels.rs`.
- **`boot/run.rs` holds only the order of start-up steps**, one step per file in `boot/`. A
  configuration or feature failure hands over to `bin/home-portal/src/failures/`, which starts no
  feature; any other failure names what failed and exits non-zero.
- **Only the root restarts the process**: the restart handle reaches one controller and the
  failure recheck, and `boot/start.rs` re-`exec`s the same binary.
- **Events go to one sink.** Emitting features take the `EventSink` port; `portal-automations`
  provides it.
- **Optional parts are modules.** Every crate learns whether its module is on only through
  `ModuleSwitches`; routes stay mounted and check the switch.
- Two features claiming one path fail at assembly, which runs in a test.

---

## 5. The web layer

- **Wire types stay at the edge**: controllers convert requests to domain values and results to
  responses; services never see wire types.
- **One error type**, `portal_feature::ApiError`, maps to statuses. `Invalid` answers field errors
  as JSON; `Internal` logs its diagnostic and never sends it.
- **Server layers** (`bin/home-portal/src/middlewares/`) apply to every request:
  - security headers on every answer;
  - a deadline on `/api/`;
  - mutating `/api/` requests must be JSON and same-origin;
  - the session, then the right a route declares;
  - the visitor's `Environment`, decided once from the address.
- **Unknown `/api/` paths are 404**; the interface is the fallback after them, and a page request
  gets its nearest folder's page.
- **The public half is explicit.** `portal-public` answers from ports with its own response types,
  and never tells a stranger what exists.

---

## 6. Integrations with services

Rules for every integration:

- **A port or a client per external system**; tests use a fake, never a live service.
- **Every outbound call has a timeout**, and every background pass a ceiling on what it reads.
- **Work that waits on the world runs in loops, not handlers.** A handler answers what is known.
- **"Could not ask" is not "down", "never asked" is not "up"**: status is `Unknown`, `Up`,
  `Degraded`, `Down` or `Unreadable`, written only by the probe that last ran.
- **Outbound addresses come from configuration**, never from a request body.

### 6.1. The configuration file

- One main TOML file holds the short settings; every other section has its own file beside it, and
  each workflow a file of its own. `portal-config` knows where a section lives, never what it means.
- Each section belongs to one feature, which parses, validates and edits it with `toml_edit`, so
  comments survive.
- Writes are atomic and need `If-Match`; a stale revision is refused. A broken hand edit is ignored
  and the last good configuration kept.
- Secrets live in `[secrets]` in a 0600 file; settings name a key, and a value never reaches a
  response or a log.
- Details: `openspec/specs/configuration-file/spec.md`.

### 6.2. Authentication

- Sessions are random tokens in an `HttpOnly`, `SameSite=Strict` cookie; only their hashes are
  stored. A changed password hash ends the other sessions.
- Password checks are bounded (throttle per address, a limit on parallel hashing) and never tell a
  wrong name from a wrong password.
- Details: `openspec/specs/authentication/spec.md`, `user-management/spec.md`.

### 6.3. Environments

- An environment is where the visitor is, decided from the address; outside every range is
  `internet`. Filtering by environment happens on the server, never in the interface.
- A visitor inside may look from another environment; from `internet` that choice is ignored.
- Details: `openspec/specs/environments/spec.md`.

### 6.4. Widgets

- A provider declares a type, checks settings and fetches a reading; `portal-widget` caches it and
  runs fetches in the background (a request waits ≤15 s, then gets the last reading, refreshing).
- **Custom widgets render on the server** (`portal-widgets`): the page gets rendered blocks, never a
  source's data; a button runs only the saved action. Templates, source runs and button runs reach
  the automations engine through `portal-widgets/src/ports/`.
- **Defined once, placed anywhere.** `[[dashboard.library]]` defines widgets, `[[dashboard.widgets]]`
  places them; writes move older inline widgets into the library and keep the comments.

### 6.5. Service icons

- The interface never asks a service for its icon: the portal fetches and caches it, and serves it
  inert (no active SVG, a sandbox policy).
- Details: `openspec/specs/service-icons/spec.md`.

### 6.6. Probe kinds and diagnosis

- A probe is `http`, `tcp` or `icmp`; the loop and `home-portal probe` run the same code.
- A failure carries a diagnosis code decided where the error is seen; macOS Local Network denial is
  `unreadable`, not an outage.
- Details: `openspec/specs/service-status/spec.md`.

### 6.7. Status history

- History lives in memory and is appended to one file per service; uptime counts probed time only.
- Details: `openspec/specs/status-history/spec.md`.

### 6.8. Reverse proxy

- Caddy carries the traffic; the portal renders its whole configuration (a pure function) and loads
  it through the admin API. A Caddy that is down never stops the portal.
- Forward-auth trusts only configured proxies and the detected environment. A published service
  never receives the portal's session cookie.
- A managed Caddy is downloaded with a checksum and stopped only through its own API.
- Details: `openspec/specs/reverse-proxy/spec.md`.

### 6.9. Automations and webhooks

- An automation runs the owner's script on a schedule or a portal event. Emitters only emit; the
  queue, admission and the journal belong to `portal-automations`.
- No shell: a script is an argv from the `scripts` folder, run in its own process group with a
  timeout and bounded output.
- The only `unsafe` is `crates/features/portal-automations/src/clients/process_group.rs`.
  **Checked:** `tests/architecture/unsafe_code.rs`.
- A webhook is a public entrance to the same queue, with a hashed token and bounded input.
- Details: `openspec/specs/automations/spec.md`, `webhooks/spec.md`.

### 6.10. Local DNS

- An authoritative server for the portal's own names only: nothing is forwarded or cached, so it is
  never an open resolver.
- A pure zone book is rebuilt from settings, published hosts and interfaces, and answers per
  environment; `internet` is refused.
- Details: `openspec/specs/dns-server/spec.md`.

### 6.11. Workflows

- A workflow is a tree of steps run as an automation run: the same queue, admission, journal and
  stop.
- One catalogue of step kinds (`types/stepping/kinds.rs`) drives validation and the editor's forms.
- Validation is static and names full paths; templates are checked against what is in scope.
- A run is bounded: time, steps, loop passes, nesting, value sizes.
- Actions leave the crate through ports: `http` cannot reach the portal's host or link-local
  addresses, scripts follow §6.9, and portal actions go through `PortalActions`.
- Secrets are masked in every trace and cannot pass through filters.
- Templates and transforms are evaluated only in Rust; the editor asks for previews and keeps
  no evaluator of its own.
- One host page (`web/src/app/admin/workflows/workflows-route.tsx`) serves every workflow address
  and moves between them with `pushAddress`, since the static export has no dynamic segments.
- Details: `openspec/specs/workflows/spec.md`.

### 6.12. Notifications

- The feature owns the rules and one bounded queue per channel; a channel crate only delivers.
- Channels read secrets through `SecretSource` and never answer their values.
- Details: `openspec/specs/notifications/spec.md`.

### 6.13. Scripts

- `portal-scripts` owns where a script may lie and whether it can run; the others ask it through a
  port.
- A script declares its arguments in its header comments, which are parsed, never run.
- Editing is switched on in the file only, never from `internet`, and writes are atomic.
- Details: `openspec/specs/script-management/spec.md`.

### 6.14. Host permissions

- On macOS, asking and checking a permission are the same act, so every check runs off the runtime
  with a limit, and start-up never waits for an answer.
- Details: `openspec/specs/host-permissions/spec.md`.

### 6.15. Access control

- A user names one group; a group holds rights over areas (`portal_feature::Area`); `admin` holds
  all. Rights are resolved per request.
- Every protected route declares its right, and one middleware checks it; a route without a rule
  is refused. **Checked:** `tests/architecture/rules.rs`, `tests/rights.rs`.
- Checks that depend on the target (granting, taking over a stronger user) live in use cases.
- Some rights amount to admin: writing scripts, choosing what automations run, reading secrets.
- Details: `openspec/specs/access-control/spec.md`.

---

## 7. Async and cost

- The portal is async (`tokio`, `axum`).
- **A handler never does work that grows with its input**; long work is a loop or a job.
- **Configuration is read without waiting**: a read never waits on another's reload, and each
  section is parsed once per revision.
- **Measure first.** A new dependency comes with its transitive crate count, clean build time and
  binary size, recorded here.

| Dependency | Crates added | Note |
|---|---|---|
| `hyper`, `hyper-util` (direct) | 0 | already built by `axum` with the same features |
| `toml_edit` `serde` feature | 0 | its crates already built by `toml` |

---

## 8. Conventions

- **English** in code, logs, API bodies and this document. Interface text comes from the `en`,
  `ru` and `es` dictionaries, on one key set.
- **No abbreviations in names**, unless an external API forces them.
- **No comments** in `.rs` files or in `web/src`; a decision becomes a test whose name states it.
  Prose goes here or into `docs/designs/YYYY-MM-DD-DESIGN-slug.md`. **Checked:**
  `tests/architecture/comments.rs`, `web/src/shared/lib/architecture.test.ts`.
- **No `#[allow(...)]`.** **Checked:** `tests/architecture/allows.rs`.
- Shipped text never mentions task numbers, phases or documents.

---

## 9. Quality gates

| Aspect | Decision | Status |
|---|---|---|
| Edition / toolchain | `2024`, `rust-toolchain.toml` | ✅ |
| The gate | `just check`: fmt → clippy `-D warnings` → tests → web lint → typecheck → tests → build | ✅ |
| Tests | no live service, no network | ✅ |
| Rust architecture guards | `bin/home-portal/tests/architecture/`: comments, allows, `unsafe`, roots, sizes, folders, layers, registry, channels, rules | ✅ |
| API samples | `bin/home-portal/tests/samples/` against the web schemas; `just samples` rewrites | ✅ |
| Response schemas | the samples test writes a JSON Schema per typed answer (`schemars`); `web/scripts/generate-schemas.mjs` turns them into `web/src/shared/api/generated/`; both fail when stale | ✅ |
| Test support | `crates/core/portal-testing` (dev only): `FakeHttp` for outbound calls, `written`/`split`/`opened` for configuration directories | ✅ |
| Widget kinds | `web/src/features/widget-board/model/registry.test.ts` | ✅ |
| Example configurations | `bin/home-portal/tests/examples.rs`, `configuration.rs` | ✅ |
| Frontend layers | `steiger` (FSD) | ✅ |
| Frontend source rules | `web/src/shared/lib/architecture.test.ts`: comments, sizes, a test beside every `shared/ui` file, blur only in `kit/overlays/` | ✅ |
| The kit | `web/src/shared/lib/kit-rules/` and ESLint: elements and classes only from `shared/ui/kit/`; `web/src/app/routes.test.tsx`: a skeleton on every page | ✅ |
| Interface text | `react/jsx-no-literals`, typed message keys | ✅ |
| No abbreviations, one error type, timeouts | review | ✅ |

---

## 10. Deliberate exceptions

The guards have no allowlist. These placements are deliberate:

| Where | What | Why |
|---|---|---|
| `crates/core/portal-widget` | a controller in `core/` | the widget endpoint and its cache are the same for every widget type |
| `crates/core/portal-config` | io in `core/` | every feature edits one set of files and all must agree on the revision |

---

## 11. Recipes

- **Add a feature:** a crate in `crates/features/portal-<name>/` with `impl Feature` and a line in
  `bin/home-portal/src/features/registry.rs`; the guard fails until both exist.
- **Add a route:** handler in its resource's controller, wire shapes in `requests/`/`responses/`,
  logic in a use case, a `Rule` in `rules()`; test through the router with `oneshot`.
- **Add an integration:** transport in `clients/` with timeouts, the check in `probes/`, the pass in
  `loops/`; test "answered", "answered garbage" and "unreachable" against a fake.
- **Add a widget:** a component and a line in `web/src/features/widget-board/model/registry.ts`; a
  `WidgetProvider` only when the portal fetches its data, then `just samples`.
- **Split an overgrown file:** child modules of a `<type>/` folder, the public facade unchanged.

---

## 12. Frontend (`web/`)

Next.js App Router with `output: "export"`, TypeScript strict, Tailwind v4 and shadcn/ui on Radix.
Designs: `docs/designs/2026-09-22-DESIGN-web-interface.md` and the later ones in `docs/designs/`.

### 12.1. Layers

- Feature-Sliced: `app` (routes; a page mounts one screen) → `widgets` (screens) → `features` (user
  actions) → `entities` (schemas, API calls, queries) → `shared` (kit, API client, i18n, config).
  Imports go down only, through a slice's `index.ts`; `steiger` enforces it.
- A concept used by several screens lives in one slice below them (the widget registry in
  `features/widget-board`, module categories in `entities/module`).
- Static export: no middleware or dynamic segments. Records are chosen by query parameter, and the
  portal serves a folder's page for deeper paths.

### 12.2. The kit

- **`shared/ui/kit/` is the only source of elements** (`@/shared/ui/kit`), each with a test.
  Outside it there are no raw controls, tables or headings, no Radix, and no class off the scale
  (arbitrary sizes, colours, shadows or radii; glass, surface or blur classes; faded tones).
  `shared/ui/<component>/` composes the kit. **Checked:** `shared/lib/kit-rules/`, ESLint.
- Tokens live in `app/globals.css`, in both themes. Only `kit/overlays/` blurs, one layer at a time.
- Loading goes through `Loaded` and a skeleton preset; content fades in. Motion is `<ViewTransition>`
  and tw-animate inside the kit, 150–250 ms, off under reduced motion. **Checked:** `app/routes.test.tsx`.

### 12.3. Data, forms and text

- HTTP only through `shared/api`: it sends `If-Match`, reads `ETag` and types 401, 409, 422 and 429.
  Every response is parsed with a zod schema.
- An editor sends the revision its values were loaded with, and offers reload or overwrite on 409.
- Every visible text is a message key, in `en`, `ru` and `es` on one key set. Each language is a
  separate build; the portal picks the language per request.
- Per-viewer conveniences (a collapsed menu, a panel width) live in `localStorage`, read safely.
