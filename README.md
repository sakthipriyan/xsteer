# Xsteer

**Xsteer** turns your financial statements into a month's money to-do list.

[`Xfina`](https://github.com/sakthipriyan/xfina) answers *"what happened."*
Xsteer answers *"what should I do this month."*

```
Salary  ──▶  Expenses  ──▶  Credit card payment  ──▶  Investable surplus  ──▶  Splits
```

Upload your bank, credit card, mutual fund and IBKR statements. Give each account a
purpose and a policy — a cash buffer, the cards it pays, the fixed expenses it carries.
Xsteer reconciles balances against dues and emits an ordered, dated plan:

```
1. by 05 Sep   Account 1 → Account 2        ₹10,000   fund card due
2. by 08 Sep   Account 2 → HDFC card        ₹24,310   statement due 10 Sep
3. by 15 Sep   Account 2 → Nifty 50          ₹5,000   underweight 2.1%
4. by 15 Sep   Account 2 → Gold              ₹5,000   underweight 1.4%
```

Everything runs in your browser. Nothing is uploaded anywhere.

---

## Design

| Piece | Choice |
|---|---|
| **Parsing** | the published `Xfina` crate — no vendored parsers, no path deps |
| **Engine** | Rust/WASM. Ledger, policies, planner and allocator all live in `core/` |
| **UI** | Vue 3 + Vite + Tailwind, following the Xfina web app. Renders only — no financial logic |
| **Storage** | encrypted chunks in IndexedDB under a device-held key; a portable `.xsteer` export is the durable copy |
| **Shipping** | one WASM module exposing both parsing and planning, so the parse schema cannot drift |

The full domain model, planner algorithm and phase plan are in
[`docs/DESIGN.md`](docs/DESIGN.md).

> **Browser storage is disposable — the export is your durable copy.** Clearing site
> data wipes the vault, so keep a current `.xsteer` export. The passphrase guarding that
> export is never persisted and never leaves the device, which also means a forgotten one
> costs you the backup. There is no reset link, because there is nobody on the other end
> to reset it.

---

## Layout

```text
xsteer/
├── web/         # Vue 3 + Vite marketing site today, the app later
├── core/        # xsteer-core — domain model and planning engine (all the logic)
│   ├── model/     accounts, policies, cards, ledger, plan
│   ├── ingest/    Xfina output → normalized entities, identity, dedup   [phase 1]
│   ├── tagging/   rule engine + manual overrides                        [phase 3]
│   ├── planner/   the cashflow solver                                   [phase 4]
│   └── allocator/ drift-minimizing splits                               [phase 5]
├── wasm/        # xsteer-wasm — bindings: Xfina parsing + the engine
└── docs/        # DESIGN.md, DEPLOY.md
```

## Website

[xsteer.in](https://xsteer.in) — a static Vite + Tailwind site hosted on an assets-only
Cloudflare Worker, deployed from GitHub Actions. Every push to `main` goes live on
[beta.xsteer.in](https://beta.xsteer.in); production moves only on a `v*.*.*` tag.
Setup and runbook: [`docs/DEPLOY.md`](docs/DEPLOY.md).

```bash
npm --prefix web install
npm run dev
```

## Status

Scaffold. The domain model and account identity are implemented and tested; ingest,
tagging, planner and allocator are specified in `docs/DESIGN.md` and not yet built.

## Build

```bash
cargo test                                    # engine
cd wasm && wasm-pack build --target web       # browser module
```

## License

Apache 2.0
