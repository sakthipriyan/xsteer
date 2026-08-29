# Xsteer — Agent Context & Guidelines

Read [`docs/DESIGN.md`](docs/DESIGN.md) first. It holds the domain model and the
planner algorithm; this file holds the working rules.

## Architecture in one paragraph

Statements are parsed by the published `Xfina` crate and normalized by `core/ingest`
into a `Vault` — accounts, policies, cards, inflows, transactions, tagging rules. The
planner reads a `Vault` and emits a `Plan`: an ordered, dated list of transfers, card
payments and investments. `wasm/` exposes both parsing and planning as a single module.
`web/` decrypts the vault, hands it to WASM, and renders what comes back.

## Hard rules

1. **No financial logic in JavaScript.** If it computes, decides, or rounds a number the
   user acts on, it belongs in `core/`. Vue formats and renders; that is all. This is
   what makes the engine testable and the numbers auditable.
2. **`Xfina` is a published dependency, never a path dependency.** Needing an
   unreleased parser change means cutting an `Xfina` release first. Keep the version
   requirement in the workspace `Cargo.toml` and in `wasm::xfina_version()` in step.
3. **Money is `Money`, never `f64`.** It wraps `Decimal`. Floats never touch an amount.
4. **The planner is deterministic.** Ties break on `AccountId`. The same vault must
   always yield the same plan — snapshot tests depend on it.
5. **Never silently produce an infeasible plan.** If obligations exceed cash, or a floor
   must be breached, emit the matching `Warning`. Under-funding without a warning is the
   worst bug this project can ship.
6. **Re-import must be idempotent.** Transaction identity is content-hashed
   (`ledger::txn_id`); importing an overlapping statement twice must not duplicate a
   row or discard a hand correction.
7. **Manual overrides beat rules, always.** Re-running the tagging rules must never
   clobber a user's correction.

## Conventions

- **Institution names in full**, matching Xfina: `"HDFC Bank"`, `"ICICI Bank"`,
  `"State Bank of India"`, `"Bank of Baroda"`. Never `"HDFC"`.
- **Dates**: `NaiveDate` for statement periods, due dates, transaction dates. Indian
  statements are IST — parse with `Asia/Kolkata` before any UTC conversion.
- **Serde**: `#[serde(rename_all = "camelCase")]` on everything crossing into JS.
  Enums carrying data use `tag = "type"`. UI bindings must match the serialized path,
  not the Rust field name.
- **Transactions are emitted chronologically ascending.**

## Build

```bash
cargo test                                     # engine tests
cargo check                                    # whole workspace
cd wasm && wasm-pack build --target web && cp -r pkg/* ../web/src/wasm/
cd web && npm run dev
```

Vite caches WASM aggressively. If the UI does not change after a rebuild, wipe
`web/node_modules/.vite` and hard-refresh.

## Releasing

One branch, previewed then promoted. Never tag by hand — `cargo xtask release` derives
the tag from `[workspace.package] version`, which is what keeps `Cargo.toml` and the tag
in step.

```bash
cargo xtask beta                     # deploy the current branch to beta.xsteer.in
cargo xtask prepare-release <major|minor|patch>
gh pr create && gh pr merge --squash
git checkout main && git pull
cargo xtask release --wait           # gates on beta, tags, deploys production
```

Squash merging is fine. The commit that lands on `main` is a new one no branch preview
covered, but pushing to `main` deploys beta again — and that post-merge run is what
`release` gates on, covering exactly the artifact production will serve. `--wait` blocks
until it finishes rather than making you poll. Runbook:
[`docs/DEPLOY.md`](docs/DEPLOY.md).

## Testing

Engine logic gets Rust tests. Planner work gets **snapshot tests**: a fixture vault in,
an expected `Plan` JSON out. Never assert a plan by eyeballing the UI — the plan is
data, and it is checked as data.

Real statements live outside this repo in `../xfina-test-data/`. Never commit a real
statement, a real account number, or a real balance to this repository.
