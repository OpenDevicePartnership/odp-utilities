# AGENTS.md

Guidance for coding agents (GitHub Copilot, Claude, Cursor, etc.) working in
the `openDevicePartnership/odp-utilities` repository.

This file is generic enough that any agent should be able to consume it. It
complements `README.md` and `CONTRIBUTING.md` — read those first for product
context and contribution policy.

## Repository overview

`odp-utilities` is a Cargo **workspace** that hosts a small collection of
standalone Rust crates for embedded systems development. Each crate is
publishable independently and may be consumed by other Open Device Partnership
projects.

- Language: Rust (edition 2024 at the workspace level, individual crates may
  opt into older editions — `bit-register` and `debug-non-default` use 2021).
- MSRV: **1.85.0** (`rust-version` in `Cargo.toml`, also enforced by CI on
  `x86_64-unknown-linux-gnu` and `aarch64-unknown-none`).
- License: MIT.
- Default branch: `main`.
- CI: GitHub Actions, single workflow at `.github/workflows/check.yml`.

### Workspace layout

```
.
├── Cargo.toml              # workspace manifest, shared lints, shared deps
├── Cargo.lock              # checked in (workspace, not a single library)
├── deny.toml               # cargo-deny config (licenses, advisories, sources)
├── rust-toolchain.toml     # pins rustfmt + clippy components
├── crates/
│   ├── bit-register/       # no_std macro crate for hardware register bitfields
│   └── debug-non-default/  # proc-macro crate: Debug impl that hides defaults
└── .github/workflows/check.yml
```

When adding a new crate, place it under `crates/<name>/`. The workspace
`members = ["crates/*"]` glob will pick it up automatically.

## Workspace-wide conventions

These are enforced by `[workspace.lints]` in the root `Cargo.toml` and apply
to every member crate (each crate sets `[lints] workspace = true`):

Clippy (all **forbid**, i.e. cannot be downgraded per-crate):
- `clippy::suspicious`
- `clippy::correctness`
- `clippy::perf`
- `clippy::style`

Rust lints:
- `missing_docs = "warn"` — every public item should have a doc comment.
- `unsafe_code = "forbid"` — **no `unsafe` blocks are permitted anywhere in
  the workspace.** If you believe you need `unsafe`, stop and raise it with a
  maintainer first; do not silently introduce an `#[allow(unsafe_code)]`.
- `unexpected_cfgs = "forbid"`
- `unused_qualifications = "forbid"`

Other house rules:
- Keep `[workspace.dependencies]` as the single source of truth for shared
  dependency versions (`syn`, `quote`, `proc-macro2`, `num-traits`, plus
  path entries for the workspace's own crates). Reference them from member
  crates with `dep.workspace = true`.
- Edition: prefer 2024 for new crates unless there is a specific reason to
  pick an older edition.
- Features must be **additive** — `cargo hack --feature-powerset check` runs
  in CI and will fail if enabling a feature breaks another combination.

## Build, lint, and test commands

All commands run from the repository root unless noted.

### Verified locally

| Purpose         | Command                                  |
|-----------------|------------------------------------------|
| Build workspace | `cargo build`                            |
| Format check    | `cargo fmt --check`                      |
| Apply formatting| `cargo fmt`                              |
| Lint            | `cargo clippy`                           |
| Run tests       | `cargo test`                             |
| Build docs      | `cargo doc --no-deps --all-features`     |

`cargo clippy` (no extra flags) mirrors what the CI `clippy` job runs via
`giraffate/clippy-action`. Prefer this form when validating changes — the
workspace lint table treats `suspicious`/`correctness`/`perf`/`style` as
**forbid**, so any violation will fail the job.

Per-crate variants:

```bash
cargo build -p bit-register
cargo test  -p debug-non-default
```

### Also run in CI (install on demand)

These require extra tooling and are not part of the verified-local set
above; install them only when you specifically need to reproduce a CI
failure locally.

| Purpose                  | Command                                 |
|--------------------------|-----------------------------------------|
| Feature-powerset check   | `cargo hack --feature-powerset check`   |
| Feature-powerset tests   | `cargo hack --feature-powerset test`    |
| Supply-chain / licenses  | `cargo deny --all-features check`       |
| MSRV check (linux)       | `cargo +1.85 check --target x86_64-unknown-linux-gnu` |
| MSRV check (bare-metal)  | `cargo +1.85 check --target aarch64-unknown-none`     |
| Nightly doc build        | `RUSTDOCFLAGS=--cfg docsrs cargo +nightly doc --no-deps --all-features` |

Install helpers:

```bash
cargo install cargo-hack
cargo install cargo-deny
rustup toolchain install 1.85 --profile minimal
rustup target add aarch64-unknown-none
```

### Minimum agent check-in checklist

Before pushing a commit, run at minimum:

```bash
cargo fmt --check
cargo clippy
cargo test
cargo doc --no-deps --all-features
```

If you touched feature flags, conditional compilation, or dependencies in
any `Cargo.toml`, also run `cargo hack --feature-powerset check` and
`cargo deny check`.

## Crate-specific notes

### `crates/bit-register`

- `no_std` macro crate that generates type-safe accessors for hardware
  register bitfields.
- Uses `proptest` and `rstest` for testing; regressions are stored in
  `crates/bit-register/proptest-regressions/`. **Commit any new regression
  files** that proptest produces — they are how we lock down minimised
  counter-examples.
- The crate's `src/lib.rs` is large (~40 KB) and macro-heavy. Read it with a
  ranged view rather than loading it whole. The companion `src/traits.rs`
  holds the supporting trait definitions.
- Endianness matters for register layouts — see the existing `dymk/bit-
  register-endianness` branch for the current direction of travel.

### `crates/debug-non-default`

- `proc-macro = true` crate that derives a `Debug` impl which omits fields
  equal to their `Default` value.
- All integration tests live in `crates/debug-non-default/tests/`. When you
  add a new shape of input (e.g. enums, generics), add a matching test file
  there instead of inflating an existing one.

## Contribution and commit hygiene

`CONTRIBUTING.md` is the source of truth; the highlights agents must follow:

- **Each commit must build successfully without warnings.** This rules out
  "fix lints in a follow-up commit" patterns — fix lints in the commit that
  introduced them.
- **No squash on merge.** Maintainers preserve commit history, so split work
  into logically coherent commits and squash typo / formatting fixups into
  the commit that introduced them before pushing.
- Use meaningful commit messages following the
  ["A Note About Git Commit Messages"](http://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html)
  conventions (imperative subject ≤ 50 chars, blank line, wrapped body).
- Open PRs as **draft** first and let CI go green before requesting review.
- For regressions, use `git bisect` to identify the first offending commit
  and mention it in the report.

### Agent attribution

When an agent (Copilot, Claude, etc.) materially assists with a commit, add
a trailer naming the assisting model, for example:

```
Assisted-by: GitHub Copilot:claude-opus-4.7
```

Place it in the trailer block alongside any `Co-authored-by` lines.

### Author identity

Do **not** modify the global `user.name` / `user.email` git config when
working on this repo on shared machines. Set the author per-commit instead:

```bash
git -c user.name="Your Name" -c user.email="you@example.com" commit ...
```

## Things to avoid

- Introducing `unsafe` code (forbidden workspace-wide).
- Downgrading or `#[allow(...)]`-ing the workspace lint groups
  (`suspicious` / `correctness` / `perf` / `style`) without an explicit
  maintainer sign-off captured in the PR description.
- Adding non-additive Cargo features.
- Pinning a dependency version directly in a member crate when the same
  dependency already exists in `[workspace.dependencies]`.
- Pulling in a dependency under a license that is not in
  `deny.toml`'s `[licenses] allow` list (currently MIT, Apache-2.0,
  Unicode-3.0, BSD-3-Clause).
- Adding a new git source: `deny.toml` only allows the `embassy-rs` GitHub
  org. Anything else must be added with a justification.
- Squashing or force-pushing to `main` or to a shared feature branch.

## Quick reference

- Workspace manifest: `Cargo.toml`
- Shared lints: `[workspace.lints]` in `Cargo.toml`
- CI workflow: `.github/workflows/check.yml`
- Supply-chain config: `deny.toml`
- Contribution policy: `CONTRIBUTING.md`
- Code of conduct: `CODE_OF_CONDUCT.md`
- Security policy: `SECURITY.md`
- Code owners: `CODEOWNERS`

## Model selection & cost discipline

Premium models (Opus, GPT-5 family, "high"/"xhigh" reasoning variants)
cost an order of magnitude more than standard models (Sonnet, Haiku,
mini). Most steps in a typical task do not need premium reasoning,
and over-using premium models wastes credits without improving
outcomes. The rules below apply to *all* model selection: your own
session, sub-agents launched via the `task` tool, and parallel work
launched via `/fleet`.

### Default posture

- **Default to the cheapest model that can do the job.** Reach for a
  premium model only when one of the escalation triggers below is hit.
- **Plan with premium, execute with cheap.** Spend at most one or two
  premium turns on design / planning, then downshift to a cheaper
  model for mechanical execution of the plan.
- **Never bump the model "just in case."** If you cannot articulate
  *why* a cheaper model would fail, use the cheaper model.

### Escalation triggers (use a premium model)

Reach for a premium model when *any* of these are true:

- Cross-module refactor, architectural design, or API design from
  scratch.
- Subtle correctness reasoning: concurrency, lifetimes, `unsafe`,
  FFI ABI, cryptography, safety-critical control paths.
- Debugging a failure that survived one prior cheap-model attempt.
- Reviewing code on a safety-, security-, or money-critical path.
- The diff cannot be predicted in advance — i.e. there is genuine
  creative or design work to do, not just typing.

### De-escalation triggers (use a cheap model)

Use the cheapest available model when *any* of these are true:

- Searching, reading, summarising files or docs.
- Single-file mechanical edits: rename, format, lint fix, dependency
  bump, boilerplate, scaffolding from a known template.
- Generating tests for code that already works.
- Running builds, tests, linters, or other commands where the model
  only needs to report success/failure.
- Routine commits, PR descriptions, changelog entries.
- The diff is essentially predictable before generation.

### Sub-agent routing (the `task` tool)

When delegating with the `task` tool, set `model:` explicitly. Do not
let sub-agents inherit a premium default for cheap work.

| Sub-agent type    | Default model             | Override to                                     |
|-------------------|---------------------------|-------------------------------------------------|
| `explore`         | cheap                     | keep cheap (`claude-haiku-4.5` or `gpt-5-mini`) |
| `task` (run cmd)  | cheap                     | keep cheap                                      |
| `research`        | cheap for breadth         | premium only for the final synthesis            |
| `general-purpose` | match task                | cheap for mechanical work; premium for design   |
| `rubber-duck`     | premium                   | keep premium — this is where reasoning pays off |
| `code-review`     | premium on critical paths | cheap on cosmetic / mechanical diffs            |

### `/fleet` (parallel sub-agents) rules

- Fleet mode multiplies cost by the fleet width. Apply the rules
  above *per worker*, not in aggregate.
- Split a fleet job along complexity lines: route the cheap,
  parallelisable workers (file edits, test runs, doc updates) to a
  cheap model; reserve premium models for the small number of
  workers that need real reasoning.
- If every worker in a fleet would need a premium model, the work is
  probably not a good fit for fleet mode — reconsider the
  decomposition before paying N× premium.

### Session hygiene

- Keep sessions short and focused. Long premium sessions are the
  single largest source of waste because every turn re-processes the
  full history.
- Use `/compact` when the conversation grows long, and `/new` for
  unrelated work.
- Prefer `/ask` for one-off side questions so they don't extend the
  main session.

### When in doubt

Ask: *"If a cheaper model produced the wrong answer here, would I
catch it in seconds (compiler, tests, my own review) or in
weeks (production incident)?"* If the former, use the cheap model
and let the feedback loop do its job.
