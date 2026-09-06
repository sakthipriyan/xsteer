# Xsteer — Design

> `Xfina` answers **"what happened."** `Xsteer` answers **"what should I do this month."**

Xsteer ingests parsed statements, maintains a private ledger in the browser, holds a
policy per account, and emits an ordered, dated **to-do list** of money movements.

```
Salary  ──▶  Expenses  ──▶  Credit card payment  ──▶  Investable surplus  ──▶  Splits
```

---

## 1. Layers

| Layer | Responsibility | Lives in |
|---|---|---|
| **Core Engines** | `Xfina` (parsing files) and `Xfingine` (categorization) | Rust |
| **Controller** | `Xsteer` orchestrates the engines, computes identity, dedup | Rust (WASM) |
| **Ledger** | unified transactions, multi-dimensional tagging, overrides | Rust (WASM) |
| **Registry** | accounts (by persona), policies, cards, inflows, target allocation | Rust (WASM) |
| **Planner** | balances + obligations + policies → ordered plan (to-do list) | Rust (WASM) |
| **Vault** | encrypted IndexedDB persistence, export/import | JS (WebCrypto) + Rust (Argon2id) |
| **UI** | render plan, edit policies, tick off steps | Vue 3 |

Vue holds **no financial logic**. It decrypts the vault, hands state to the WASM controller, renders
what comes back, and re-encrypts. The UI supports progressive complexity (from simple bill tracking to full planning), but the core outputs an absolute, deterministic to-do list computed in Rust.

---

## 2. Account identity

`Xfina` parses one file at a time and has no notion of "the same account across
statements." Xsteer derives a stable identity:

```rust
AccountKey { institution, account_type, masked_number }  →  blake3 → AccountId
```

Masked numbers differ in format between statements from the same bank
(`XXXXXX1234` vs `****1234`), so the key normalizes to **the trailing digits only**
plus institution and type. Collisions across two accounts at one bank sharing the last
four digits are possible; the UI surfaces a merge/split control and the user's decision
is persisted as an identity override that wins over the derived key.

---

## 3. Domain model

### Account

```rust
struct Account {
    id: AccountId,
    owner: PersonaId,         // Groups accounts by family member (e.g. self, spouse)
    institution: String,      // "HDFC Bank" — full names, per Xfina convention
    kind: AccountKind,        // Savings | Current | CreditCard | Brokerage | MutualFund
    masked_number: String,
    display_name: String,     // user-supplied: "Salary", "Travel"
    policy: Option<Policy>,   // deposit accounts only
}
```

### Policy — the purpose of an account

```rust
struct Policy {
    role: Role,               // Salary | Spend | Medical | Travel | Investment | Buffer | Custom
    floor: Money,             // never draw below this (min balance + cash buffer)
    target: Option<Money>,    // desired steady-state balance; top up toward it
    sweep: Sweep,             // excess above target → Nothing | To(AccountId) | ToInvestable
    obligations: Vec<Obligation>,
}
```

### Obligation — a claim against an account

```rust
enum Obligation {
    CreditCardDue  { card: AccountId, split_id: Option<String> }, // pay this card (or a partial category slice)
    FixedExpense   { name: String, amount: Money, day: u8 },      // rent, EMI, mandate
    PlannedExpense { name: String, amount: Money, due: Date },    // one-off
    ManualBill     { name: String, amount: Money, due: Date, card: Option<AccountId> }, // e.g. yearly mobile recharge on CC
    Reserve        { name: String, amount: Money },               // earmark, never spend
}
```

`CreditCardDue` is deliberately a *reference*, not an amount — the amount comes from
the card's latest statement, so re-importing a statement updates the plan with no edit.

### Credit card

```rust
struct CardState {
    id: AccountId,
    statement: Option<Statement>,   // { period, total_due, min_due, due_date }
    unbilled: Money,                // spends after statement close
    paid_from: AccountId,
    cycle: Cycle,                   // statement day + grace days → project next due date
}
```

### Inflow

```rust
struct Inflow { into: AccountId, amount: Money, on: Date, recurrence: Recurrence }
```

Salary is an `Inflow` with `Recurrence::MonthlyOn(day)`. Detected from the ledger
(recurring credit, same narration, same account) and confirmed by the user.

### Ledger

```rust
struct Txn {
    id: TxnId,           // dedup key, see below
    account: AccountId,
    date: Date,
    amount: Money,
    direction: Debit | Credit,
    narration: String,
    balance: Option<Money>,
    category: Option<CategoryId>,
    tags: Vec<TagId>,
}
```

**Dedup key** = `hash(account, date, amount, direction, normalized_narration, running_balance)`.
Running balance is included because banks legitimately emit two identical
same-day same-amount transactions; the balance disambiguates them. Where a statement
omits running balance (some credit cards), the key falls back to
`(account, date, amount, narration, ordinal_within_day)`.

Re-importing an overlapping statement is therefore idempotent.

### Tagging & Categorization (Xfingine)

Ordered rule list, first match wins:

```rust
struct Rule { matcher: Matcher, category: CategoryId, tags: Vec<TagId> }
enum Matcher { Narration(Regex), AmountBetween(Money, Money), Counterparty(String), CardName(String), All(Vec<Matcher>) }
```

Tags enable **multi-dimensional tracking**:
- **Nature**: Want vs. Need
- **Frequency**: Monthly vs. Yearly vs. One-off
- **Merchant**: Derived or matched from counterparty strings.

This historic tagged data fuels baseline fixed expense predictions for upcoming months.
Manual per-transaction overrides live in a separate table keyed by `TxnId` and always
beat rules — so re-running rules after editing them never clobbers hand corrections.

### Execution Rails & Routing

Accounts, Credit Cards, and Policies configure **Execution Rails** (e.g., `Samsung Wallet`, `HDFC NetBanking`, `Cred`, `Money2World`, `RTGS`). 
- When generating the plan, Xsteer resolves the specific payment route. 
- For example, an international investment might specify an intermediate routing requirement: if the rail is `ICICI + Cred`, the planner generates a two-step sequence: an internal RTGS transfer to ICICI, followed by the actual international investment via Cred.

---

## 4. The plan

```rust
struct Plan {
    as_of: Date,
    horizon: DateRange,
    opening: Vec<AccountBalance>,
    steps: Vec<PlanStep>,
    projected: Vec<AccountBalance>,   // balances after every step executes
    investable: Money,
    warnings: Vec<Warning>,
}

struct PlanStep {
    seq: u32,
    due_by: Date,
    kind: StepKind,
    status: Pending | Done | Skipped,
}

enum StepKind {
    Transfer    { from: AccountId, to: AccountId, amount: Money, reason: String, rail: Option<String> },
    CardPayment { from: AccountId, card: AccountId, amount: Money, due_date: Date, rail: Option<String> },
    Investment  { from: AccountId, asset: AssetId, amount: Money, rail: Option<String> },
    Manual      { text: String },     // "get an FX quote", "raise an NEFT limit"
}
```

Rendered, the UI groups the requested output into actionable "Login Sessions" so the user can execute the plan linearly with zero thinking. It also provides **traceability**—clicking any number reveals the exact math used to compute it (e.g., *"Medical card split is ₹5000, current balance ₹10000, target ₹10000 → move ₹5000"*):

```
**Session 1: SBI (Parent)**
  [ ] by 05 Sep   SBI → HDFC                  ₹10,000   via Samsung Pay

**Session 2: HDFC (Self)**
  [ ] by 08 Sep   HDFC → HDFC Infinia         ₹24,310   via HDFC NetBanking
  [ ] by 15 Sep   HDFC → ICICI                ₹30,000   via RTGS (Investment Routing)

**Session 3: ICICI & FX Execution**
  [ ] by 15 Sep   Action                      Use BHIM/Cred to book FX Retail deal
  [ ] by 15 Sep   Action                      Login into ICICI, settle the deal
  [ ] by 16 Sep   Action                      Verify deposit notification
  [ ] by 16 Sep   ICICI → Nasdaq 100          ₹30,000   Buy shares in IBKR
  [ ] by 17 Sep   Action                      Update Payroll team with Form 122
  [ ] by 30 Sep   Action                      Verify Payroll team updated TDS sheet
```

### Planner algorithm — deterministic, in order

1. **Snapshot.** Opening balance per account = latest statement closing balance,
   adjusted by any ledger transactions dated after that close.
2. **Project inflows.** Add every `Inflow` falling inside the horizon.
3. **Collect obligations.** Card dues (from statements), fixed expenses, planned
   expenses, reserves. Sort by due date; a card's due date comes from its statement,
   or is projected from `Cycle` when the statement has not arrived yet.
4. **Fund each obligation** from its designated account. Where the account falls short,
   pull from accounts with excess over floor — largest excess first, and prefer one
   transfer over several (ties broken by `AccountId` so runs are reproducible).
5. **Restore floors and targets.** Top up any account left below `floor`; then toward
   `target` if surplus remains.
6. **Sweep.** Apply each policy's `sweep` rule. What lands in `ToInvestable` is the
   investable surplus.
7. **Allocate.** Feed investable into the drift-minimizing allocator (ported from the
   Family SIP Allocator): buy only underweight assets, never sell, honor asset-group
   caps, and apply TCS impact to international legs. *(Note: LRS headroom is not natively tracked, but compliance workflows are).*
8. **Order steps** by due date, then by dependency — money must arrive in an account
   before a step spends from it. Workflows like international investments are expanded into multi-step sequences (e.g., FX Retail Booking → Buy Shares → File Form 122).

### Warnings

The planner never silently produces an infeasible plan:

- `Shortfall { amount, at_risk: Vec<Obligation> }` — obligations exceed available cash
- `FloorBreach { account, by }` — a floor had to be violated to meet a due date
- `DueDateAtRisk { card, due_date }` — funding cannot land before the due date
- `StaleStatement { account, last_seen }` — planning on data older than a cycle
- `Form122Pending { asset }` — international leg compliance not yet accepted by payroll

---

## 5. Absorbed tools

Each `building-wealth/tools` script is today an island with its own localStorage.
In Xsteer they become views over one model:

| Tool | Becomes |
|---|---|
| Family SIP Allocator | the allocator in planner step 7 (supports perpetual rebalancing) |
| RealValue Portfolio | real-time holdings + XIRR view over CAS/IBKR imports |
| FX Engine | TCS impact, true cost on international investment legs, and Form 122 workflow generation |
| EMI Engine | `Obligation::FixedExpense` generator |
| Emergency Fund | `Policy::floor` on the buffer account |
| IBKR Tax Engine | stays separate — tax reporting, not cashflow |

---

## 6. Storage

**`.xsteer` is the durable interchange format; IndexedDB is disposable.** Nothing
important couples to browser storage — clearing it costs a cache, not the ledger.

```
┌───────────────────────────┐
│       Xsteer WASM         │  Vault + rebuilt indexes — all querying happens here
└─────────────┬─────────────┘
              │ decrypt / load
┌─────────────▼─────────────┐
│         IndexedDB         │  opaque encrypted chunks — disposable cache
└─────────────┬─────────────┘
              │ durable user action
┌─────────────▼─────────────┐
│      .xsteer export       │  encrypted, portable — the durable artifact
└───────────────────────────┘
```

### Keys

Envelope encryption. A random 256-bit AES-GCM **data key** encrypts content and never
changes; independent wrappers unwrap it, so adding an unlock method or changing a
passphrase never re-encrypts the vault.

| Wrapper | Role | Friction |
|---|---|---|
| **Device key** | opening on a browser already used before | none |
| **Passphrase** | portable — required on every export | typing |
| **Recovery key** | portable — required on every export | emergencies only |
| **WebAuthn PRF** | portable convenience, deferred to M3 | a biometric touch |

The device key is a `CryptoKey` with `extractable: false` held in IndexedDB, so **there is
no passphrase prompt to open the app**. The passphrase guards what leaves the machine.

**Every `.xsteer` carries both a passphrase and a recovery-key wrapper**, so a backup is
always recoverable without the original browser or device. PRF is convenience only — a
passkey is bound to one ecosystem, and a PRF-only backup would die with the account
holding it.

Passphrase wrapping uses Argon2id (64 MiB, t=3, p=1) in Rust/WASM on a worker; the
parameters travel in the export header, so they can be retuned without orphaning old
backups. Content encryption stays in WebCrypto.

### What "encrypted at rest" means here

> Financial data is encrypted at rest, and the raw data key is not exportable through the
> Web Crypto API.

That is the whole claim. `extractable: false` stops the key bytes leaving through
WebCrypto; it does **not** stop JavaScript running in this origin from using the key, and
it does not survive a compromised browser. Two consequences: a strict CSP is a first-class
control here rather than hygiene, and XSS is a total compromise regardless of what is
encrypted on disk.

### Chunks

Each chunk is a self-contained encrypted record:

```
format_version │ vault_id │ chunk_id │ generation │ nonce │ ciphertext │ tag
```

AAD binds `{vault_id, chunk_id, schema_version, generation}`, which buys two properties:
`ledger/2026-08` cannot be relocated into `ledger/2025-08`, and because `generation` is
monotonic, a stale chunk cannot silently replace a newer one.

| Chunk | Size | Churn |
|---|---|---|
| `registry` | KBs | accounts, policies, cards — frequent |
| `ledger/{YYYY-MM}` | ~100s of KB | append-mostly |
| `overrides` | small | manual tags, identity overrides |

Editing one transaction rewrites one period, not the decade. All dirty chunks are written
in a single `readwrite` transaction, so a partial write cannot leave chunks at
inconsistent generations.

### Querying

Storage is opaque; querying happens over the decrypted `Vault` in memory. The persisted
model is minimal and versioned; indexes are runtime-only, rebuilt on load in O(n)
(~10–50 ms at 100k transactions), which keeps the format small and migrations tractable.

| Persisted (serde) | Runtime (rebuilt, never serialized) |
|---|---|
| `accounts: Vec<Account>` | `by_account: HashMap<AccountId, Vec<TxnIdx>>` |
| `transactions: Vec<Transaction>` | `by_date: Vec<TxnIdx>` — sorted, binary-searchable |
| `overrides: Vec<Override>` | `by_category: HashMap<CategoryId, Vec<TxnIdx>>` |

At this scale — ~5 MB typical, ~30 MB for a decade across eight accounts — that beats an
embedded database outright: a binary search answers in microseconds what SQLite would pay
VFS round-trips and page decryption for. Open time is dominated by deserialization, not
crypto (~5 ms to decrypt 5 MB, ~50 ms to parse it), so if opening ever gets slow the fix
is a binary serde format, not a storage engine.

### Durability

`.xsteer` is self-describing — the header carries the format version and KDF parameters,
so a backup taken today still opens after the schema has moved on. Filenames carry the
date and a short content hash, making a downloads folder a legible version history.

Export belongs at the **end of ingest**, as the closing step of the workflow, not in a
banner the user can dismiss. The app tracks the last export against a vault mutation
counter and says plainly when the backup has fallen behind.

**The consequence to be explicit about with the user: browser storage is disposable, so an
out-of-date export is what actually loses data.** A forgotten passphrase costs that
backup — not the live vault — and there is no reset path, because there is nobody on the
other end to reset it.

### Milestones

| | Scope |
|---|---|
| **M1 — Persistence** | encrypted chunks, device-key unlock, atomic saves, generation counters |
| **M2 — Portable durability** | `.xsteer`, passphrase and recovery-key wrappers, import/export, the save step |
| **M3 — Unlock UX** | WebAuthn PRF, password-manager integration, home-screen guidance |

M1 and M2 together are the storage model; PRF is deliberately outside them, since nothing
in the security or durability argument depends on it.

---

## 7. Phases

| Phase | Scope |
|---|---|
| **1 — Ingest** | upload bank / card / MF / IBKR files, account identity, dedup, encrypted vault (M1–M2, §6), ledger view |
| **2 — Registry** | accounts, policies, cards, salary detection and setup |
| **3 — Tagging** | rule engine, categories, manual overrides, spend analysis |
| **4 — Planner** | obligations, cashflow solver, the to-do list, execution tracking |
| **5 — Allocate** | target allocation, drift, splits, LRS/TCS |
| **6 — Open** | user-defined queries and views over their own vault |

## 8. End-to-End Vision & Progressive Complexity

Xsteer is designed to serve as a complete financial cockpit requiring **zero day-to-day thinking**—everything is delegated to policy. 
However, the engine supports progressive complexity:
- **Level 1 (Basic)**: Bank account and manual bill management only.
- **Level 2 (Tracking)**: Simple portfolio and XIRR tracking via CAS/IBKR imports.
- **Level 3 (Planning)**: Full utilization of policies, automatic sweeps, and the perpetual rebalancing allocator.

Regardless of the complexity level, the primary goal of Xsteer remains absolute: **to emit a simple, deterministic to-do list of actionable money items for the end user.**

### Interface & Ingest UX

From the user's perspective, the monthly ritual begins with a bulk upload:
1. The user downloads all required statements (SBI, HDFC, ICICI, CAMS, IBKR, Axis) into a single local folder.
2. In the Xsteer UI, the user selects and uploads all these files at once.
3. The engine automatically parses, identifies accounts, and deduplicates. Exception handling is minimal: if a statement lacks an identifier (e.g., ICICI credit card reports omitting the card ID), the UI simply prompts the user to manually map it. The rest are completely self-contained.

Once ingested, the execution UI groups the output by **Login Sessions** (as shown in the planner section), allowing the user to execute the plan linearly. To build trust in this "zero thinking" model, the UI provides full **Traceability**: clicking any computed number reveals exactly how it was derived from the ingested data and policies.

### Real-World Workflow Example

To ensure the domain model handles real-world complexity, Xsteer supports advanced family setups natively:
- **Personas**: Accounts are grouped by owner (e.g., Self, Spouse, Child) to ensure the planner tracks funds and limits accurately per person, never mixing one persona's obligations with another's balances unless explicitly configured.
- **Account Policies**: Users define goals like "Target ₹40,000 post-bills" (`target: 40000`). The planner pulls funds for obligations first, then sweeps from the Salary account to top up the remaining balance to exactly ₹40,000.
- **Shared/Split Credit Cards**: Add-on cards or distinct categories of spends on a single card (e.g., HDFC Infinia) can be split. The `Medical` portion generates a partial `CardPayment` obligation directly funded from a dedicated Medical account, while the `Child` portion is funded from a separate persona's account.
- **Payment Routing**: The planner explicitly resolves execution rails. Rather than just saying "Transfer ₹10,000", it dictates the exact real-world app and method needed: "Transfer via Samsung Wallet" or "Invest via Cred/Money2World". It natively handles multi-hop sequences if an execution rail demands it (e.g., routing funds to ICICI first to utilize its specific FX gateway).
- **[Perpetual Rebalancing Framework](https://github.com/sakthipriyan/building-wealth)**: The absorbed Family SIP Engine doesn't just calculate one-off SIPs. By monitoring real-time holdings and the inflow of investable surplus, the planner perpetually routes new money to underweight assets to maintain the target asset allocation without incurring the tax drag of selling.
