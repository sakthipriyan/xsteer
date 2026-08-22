# Xsteer — Design

> `xfina` answers **"what happened."** `xsteer` answers **"what should I do this month."**

Xsteer ingests parsed statements, maintains a private ledger in the browser, holds a
policy per account, and emits an ordered, dated **to-do list** of money movements.

```
Salary  ──▶  Expenses  ──▶  Credit card payment  ──▶  Investable surplus  ──▶  Splits
```

---

## 1. Layers

| Layer | Responsibility | Lives in |
|---|---|---|
| **Ingest** | files → `xfina` parse → normalized entities, account identity, dedup | Rust |
| **Ledger** | unified transactions, tagging rules, manual overrides | Rust |
| **Registry** | accounts, policies, cards, inflows, target allocation | Rust |
| **Planner** | balances + obligations + policies → ordered plan | Rust |
| **Vault** | encrypted IndexedDB persistence, export/import | JS (WebCrypto) |
| **UI** | render plan, edit policies, tick off steps | Vue 3 |

Vue holds **no financial logic**. It decrypts the vault, hands state to WASM, renders
what comes back, and re-encrypts. Every number the user sees was computed in Rust.

---

## 2. Account identity

`xfina` parses one file at a time and has no notion of "the same account across
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
    institution: String,      // "HDFC Bank" — full names, per xfina convention
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
    CreditCardDue  { card: AccountId },                       // pay this card in full
    FixedExpense   { name: String, amount: Money, day: u8 },   // rent, EMI, mandate
    PlannedExpense { name: String, amount: Money, due: Date }, // one-off
    Reserve        { name: String, amount: Money },            // earmark, never spend
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

### Tagging

Ordered rule list, first match wins:

```rust
struct Rule { matcher: Matcher, category: CategoryId, tags: Vec<TagId> }
enum Matcher { Narration(Regex), AmountBetween(Money, Money), Counterparty(String), All(Vec<Matcher>) }
```

Manual per-transaction overrides live in a separate table keyed by `TxnId` and always
beat rules — so re-running rules after editing them never clobbers hand corrections.

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
    Transfer    { from: AccountId, to: AccountId, amount: Money, reason: String },
    CardPayment { from: AccountId, card: AccountId, amount: Money, due_date: Date },
    Investment  { from: AccountId, asset: AssetId, amount: Money },
    Manual      { text: String },     // "get an FX quote", "raise an NEFT limit"
}
```

Rendered, that is exactly the requested output:

```
1. by 05 Sep   Account 1 → Account 2        ₹10,000   fund card due
2. by 08 Sep   Account 2 → HDFC card        ₹24,310   statement due 10 Sep
3. by 15 Sep   Account 2 → Nifty 50          ₹5,000   underweight 2.1%
4. by 15 Sep   Account 2 → Gold              ₹5,000   underweight 1.4%
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
   caps, and apply LRS/TCS constraints to international legs.
8. **Order steps** by due date, then by dependency — money must arrive in an account
   before a step spends from it.

### Warnings

The planner never silently produces an infeasible plan:

- `Shortfall { amount, at_risk: Vec<Obligation> }` — obligations exceed available cash
- `FloorBreach { account, by }` — a floor had to be violated to meet a due date
- `DueDateAtRisk { card, due_date }` — funding cannot land before the due date
- `StaleStatement { account, last_seen }` — planning on data older than a cycle
- `LrsHeadroom { used, remaining }` — international leg approaching the ₹10L FY limit

---

## 5. Absorbed tools

Each `building-wealth/tools` script is today an island with its own localStorage.
In xsteer they become views over one model:

| Tool | Becomes |
|---|---|
| Family SIP Allocator | the allocator in planner step 7 |
| RealValue Portfolio | holdings + XIRR view over CAS/IBKR imports |
| FX Engine | LRS/TCS constraint on international investment legs |
| EMI Engine | `Obligation::FixedExpense` generator |
| Emergency Fund | `Policy::floor` on the buffer account |
| IBKR Tax Engine | stays separate — tax reporting, not cashflow |

---

## 6. Storage

Vault encrypted at rest in IndexedDB. Key derived from a user passphrase via
PBKDF2-SHA256 (WebCrypto, high iteration count), random per-vault salt, AES-GCM with a
fresh IV per write. The passphrase is never persisted; the derived key lives in memory
for the session only. Export produces the same encrypted blob for backup and
cross-device transfer.

**Consequence to be explicit about with the user: a forgotten passphrase means the data
is unrecoverable.** No reset path exists by design.

---

## 7. Phases

| Phase | Scope |
|---|---|
| **1 — Ingest** | upload bank / card / MF / IBKR files, account identity, dedup, encrypted vault, ledger view |
| **2 — Registry** | accounts, policies, cards, salary detection and setup |
| **3 — Tagging** | rule engine, categories, manual overrides, spend analysis |
| **4 — Planner** | obligations, cashflow solver, the to-do list, execution tracking |
| **5 — Allocate** | target allocation, drift, splits, LRS/TCS |
| **6 — Open** | user-defined queries and views over their own vault |
