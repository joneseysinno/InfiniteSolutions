# infinite-runtime — Layer Specification

> **Status:** draft 1, 2026-08-21. S3 and S4 landed (E2); S5–S7 landed (E6).
>
> Layer: **runtime** (D2, D17). Rules: [`../RULES.md`](../RULES.md) · Decisions:
> [`../DECISIONS.md`](../DECISIONS.md) · Charter: [`../CHARTER.md`](../CHARTER.md)
>
> Satisfies R18 for this layer. The crate may exist once this document does.

---

## Stage table

| # | Stage | Status | Green check |
|---|---|---|---|
| S1 | This specification | draft 1 | Recorded as D23 + D24; reviewed against R8–R14 line by line |
| S2 | Crate skeleton, pure core | not started | `cargo build -p infinite-runtime` with **no features** succeeds and the manifest's `[dependencies]` is empty |
| S3 | Ports and the fake store | landed | The whole test suite passes with only `FakeStore`; `infinite-db` appears nowhere in the crate |
| S4 | Pending set + journal | landed | Saturation test (§7.4) — write queue full, input driven at 60 Hz for 10 s, no dropped keystroke, no tick over budget |
| S5 | Artifact registry + discard harness | landed | Every registered artifact is dropped and rebuilt bit-identically by a generic test, with no per-artifact code |
| S6 | Staleness frontier | landed | An input change at rev N yields **exactly** the downstream address set — no more, no fewer |
| S7 | First real binding, in the facade | landed | The S3–S6 tests pass unchanged against real `infinite-db` |

---

## 1 · What this layer is

**The runtime owns time.** (D5, R10.)

The store has revisions and logical clocks but no **now** — no cadence, no frame, no
priority, no deadline. Everything that only means something while something is
running belongs here.

Membership test, applied per item:

> If it survives a restart it belongs to the store. If it only means anything while
> something is running it belongs to the runtime.

Two prohibitions, from D4, carried as law:

> **L1 — The runtime owns no thread pool.** It is driven; it does not drive. Cadence
> in, work out.
>
> **L2 — The runtime owns no storage.** Nothing is authored here. Nothing survives a
> restart, except what §6 declares as *pending* and journals through a port.

The layer's whole job, stated once:

> **Given a cadence and a budget, decide what to do next and do as much of it as the
> budget allows — without ever blocking the person who is typing.**

---

## 2 · The forcing consumer (R19, R26)

**The editor** — the platform's own graph editor (O11).

R19 requires the consumer be *something that breaks if the layer is wrong*. Six
concrete breakages, each of which has already happened at least once in `bion`,
`biomimicry`, or `Innovator`:

| # | If the runtime is wrong | What the person sees |
|---|---|---|
| B1 | The input path touches the store's write queue | Typing stalls when the queue fills (O3) |
| B2 | Invalidation is coarse | One keystroke redraws the page and drops frames (D6's stated cost) |
| B3 | Pending state has no home | A drag in progress is lost on crash — or worse, written into the definition space mid-drag (R5) |
| B4 | Nothing is enumerable | An "unsaved" indicator is unimplementable, because nothing can list what is pending (R13) |
| B5 | Watch is a correctness dependency | An MCP server writes a definition and the canvas is silently wrong (D6's second rejection ground) |
| B6 | Priority is flat | Zoom rebuilds distant spaces before the one under the cursor and misses the frame |

**Scope consequence of choosing the editor alone.** The editor exercises no settling
loop. D21's iterative regions — maximum iteration count, stopping test, non-convergence
as a finding — are therefore **specified only to the extent that §5's drive interface
must not foreclose them**, and are otherwise out of scope for this document. The
trigger to extend this spec is the first named consumer with a solve in it (the crane
mat). Recorded here so the gap is deliberate rather than discovered.

---

## 3 · The seam: the runtime depends on no other layer (D23)

**The runtime names no crate belonging to another layer.** It declares the *ports* it
needs as traits; the platform facade supplies the implementations.

*Why.* D3 established that a computation must not depend on the runtime, or the two
become mutual hostages and neither is testable alone. The same argument runs in the
other direction and has already been built once: `bion`'s CNS reaches the store only
through `&dyn PnsReader`, enforced by a CI grep of `src/cns` for DB tokens — the
enforcement pattern R11 already cites. A runtime that names `infinite-db` cannot be
tested without a database, which means the editor's latency behaviour cannot be tested
at all, which is B1 through B6.

*Cost.* One trait call per store touch, and a fake store to maintain. Both are
cheap; the fake is also the only way S4's saturation test can exist, because a real
store's queue cannot be filled on demand.

### 3.1 The ports

Five, and no more without a decision record. Named as plain functional nouns (R15).

| Port | The runtime asks it for | Notes |
|---|---|---|
| `StoreRead` | records in an address range, at a revision | Records pass through; they are never retained (§4) |
| `StoreWrite` | submit a commit | **Non-blocking and fallible.** Returns `Accepted` or `Full`. Never waits. This is where O3 is closed |
| `StaleFeed` | "these addresses went stale at revision N" | `infinite-db` already has the machinery: derivation bus, watermarks, `check_hyperedge_freshness`, `query_stale_downstream` |
| `Clock` | a monotonic instant | The runtime owns *now* but must not own a thread (L1), so *now* is handed in |
| `Journal` | append a pending record; replay on start | The store's session WAL. The runtime calls it; it does not implement it (L2). The facade wires `append` through `insert_with_session` |

`StaleFeed` is what makes B5 impossible: an external writer's change arrives as
staleness, so a missed notification costs responsiveness and never correctness. This is
D6's second rejection ground, made structural.

### 3.2 `Addr` — the only thing the core knows about identity

The pure core depends on nothing (R3), so it cannot use the store's key type.

> **`Addr` is an opaque, totally-ordered byte key. The runtime compares it, takes
> ranges of it, and truncates it. It never interprets it.**

Three properties, and the runtime needs exactly these three:

1. **Total order**, and that order is spatial locality — this is what makes a range
   scan of a subtree affordable, which is O2's placement policy stated as a
   requirement the runtime depends on.
2. **Prefix truncation is level.** Level ℓ is the key truncated to ℓ·D bits — already
   written three times in the corpus (`infinitedb-spatial-layer.md`, the analysis
   substrate plan §0.5, D15). The runtime uses it for one thing only: **priority by
   distance from focus** (B6), computed from shared prefix length. No other use is
   admitted without a consumer requiring it (R27).
3. **Permanence.** An address, once issued, stays valid under refinement. The runtime
   relies on this and does not verify it; it is the store's invariant.

This is deliberately not a generic parameter. A type parameter for the address would
thread through every type in the layer to buy an abstraction with exactly one
instantiation — R27.

---

## 4 · What the runtime may hold (R11, closing a D9 soft spot)

R11 says the runtime holds addresses, never records, and D9 flags its silence on
*transient versus retained*. Stated:

> **A record may pass through the runtime within a single `tick`. It may not be
> retained across ticks, except inside a declared derived artifact (§5.2).**

*Checked by:* a lint over the layer's struct fields — no field whose type is or
contains a record. Locals and arguments are unrestricted. This is the field-level
form of `bion`'s CI grep, and it is mechanical, which is the entire point of R11.

The three state categories (D8), instantiated for this layer:

| Category | In the runtime | Discardable |
|---|---|---|
| **Stored** | nothing (L2) | — |
| **Derived** | the frontier, plus every registered artifact | yes, by definition |
| **Pending** | the pending set | **no** — dropping it loses user work |

---

## 5 · Derived state

### 5.1 The frontier

`Frontier` — the set of addresses known stale and not yet recomputed, ordered by
priority.

*Rebuild rule:* `StaleFeed::stale_since(last_durable_watermark)`.
*Discard test:* drop it, re-query from the watermark, obtain an identical set.

Priority is a total order over frontier entries, computed from (a) shared prefix
length with the focused address, and (b) arrival revision. Nothing else. B6 is the
consumer that requires (a); (b) exists only to make the order total and therefore the
schedule deterministic, which D19's equivalence law needs.

### 5.2 Artifacts are registered, not enumerated (R4, R16)

**The runtime does not know what any derived artifact is.** It knows only the
lifecycle. An artifact is registered under a string key with:

- the address ranges it derives from,
- a rebuild function,
- a validity watermark.

`RenderList` is the first instance, registered by the presenter's binding — D5 places
it in the runtime as a declared artifact, and D15 places visibility policy in the
presenter. Both hold: the presenter owns the *function*, the runtime owns *when it
runs and whether it is stale*. That is R3's core/binding split applied across a layer
boundary rather than inside one.

*Why a registry and not an enum.* F-1 has occurred five times. More usefully: a
registry makes R12 **free**. The runtime can drop and rebuild any registered artifact
and compare bytes without knowing what it is, so the discard test is one generic test
harness that every artifact anyone ever registers is subject to automatically. An enum
would require the harness to be re-derived per variant, which is how a cache stops
being checked.

*Green check for S5:* the harness exists and no artifact has per-artifact test code.

---

## 6 · Pending state (D8, R13)

The only non-discardable thing the runtime holds, so it carries the extra obligations
explicitly.

**Shape.** An entry is `{ origin: Addr, payload: opaque, seq: u64, boundary }`. The
payload is opaque and its tag is the app's (D13) — the runtime never parses, validates,
converts, or renders it.

**Operations.** `open`, `amend`, `commit`, `abandon`, `list`, `settle`. `Innovator`
already has the shape as `FieldEditing` per keystroke and `FieldCommit` on commit; this
generalizes it beyond fields to any uncommitted gesture, a drag included. `settle` is
the sixth because of §7.2: an entry stays pending *after* its commit boundary until the
store accepts it, so something has to mark it gone.

**Bounded.** An explicit capacity. On overflow the policy is **commit the oldest**,
never drop. Dropping is not available — the category is defined by not being
discardable, and a bound that discards would be a bound on correctness.

**Enumerable.** `list` returns everything pending, always. This is B4, and it is also
how a person is told what is unsaved, which under D16's child constraint is not
optional.

**Journaled.** Every `amend` appends to `Journal`. A crash loses at most the
unflushed tail. Replay on start restores the pending set before the first tick.

---

## 7 · The drive interface, and O3 (D24)

### 7.1 Being driven

```rust
pub fn tick(&mut self, now: Instant, budget: Budget) -> Outcome
```

Cadence in, work out (L1). `tick` never blocks, never sleeps, never spawns. `Budget`
is a work quantum or a deadline. `Outcome` reports what was done and **whether work
remains**, so the caller decides whether to tick again.

*Checked by:* no executor or async-runtime dependency in the manifest (R8), and no
`std::thread`, `sleep`, or blocking call anywhere in the layer.

*Note on O12, which this document does not close.* Because `tick` is budgeted and
already reports "more remains", a long computation yielding between iterations is the
interface's default shape rather than a new concern. O12 stays open; the drive
interface is written so that closing it later adds no new scheduler concept.

*Note on O1, which this document does not close.* Whether a read-only hot working set
sits between store and layout is a measurement — a warm prefix scan of ~1000 nodes
against frame budget. Under §5.2 it would be one more registered artifact and needs no
structural change either way. The measurement is a runtime obligation and belongs in
S6's test bed.

### 7.2 A keystroke is not a write

The mechanism that closes O3, in four sentences:

1. A keystroke is an `amend` to a pending entry. It touches the pending set (in
   memory, bounded) and the journal (append-only, sequential). It does not touch the
   write queue.
2. The store is written only at a **commit boundary**.
3. Commits are **coalesced per address**: at most one in flight per address, and a
   newer pending value supersedes an unsent one rather than queueing behind it.
4. `StoreWrite::submit` returns `Full` rather than blocking. On `Full` the commit stays
   in the pending set and is retried on a later tick.

Therefore the store's write queue is never on the input path, and backpressure
degrades *durability latency* — how soon a committed value is safe — rather than
*input latency*. That is the correct thing to degrade.

### 7.3 What it costs

Pending state becomes load-bearing: it now holds committed-but-unsent values as well as
uncommitted edits, so §6's bound and enumeration obligations are doing real work rather
than being paperwork. A crash under sustained backpressure loses the journal's unflushed
tail. Both are stated so neither is discovered.

### 7.4 Green check

Saturate `FakeStore`'s write queue. Drive input at 60 Hz for 10 seconds. Assert: every
keystroke appears in the pending set within one tick; no tick exceeds its budget; the
pending set stays within its bound; and after the queue drains, the store's final value
per address equals the last input value.

*Verification method (R23):* this is an interaction claim, so it is stated as an
automated test against a fake with a saturable queue — not measured by hand and not
inferred from the design.

---

## 8 · How every runtime rule is checked, in this layer

| Rule | Check | Lives in |
|---|---|---|
| R8 — no thread pool | manifest grep: no executor, no async runtime; source grep: no `thread`, `sleep`, blocking call | CI |
| R9 — no storage | manifest grep: no persistence crate; source grep: no file handle | CI |
| R11 — addresses, never records | field-type lint over the layer's structs (§4) | CI |
| R12 — artifacts pass the discard test | one generic harness over the registry (§5.2) | test suite |
| R13 — pending bounded and enumerable | `list` returns everything; capacity test; overflow commits rather than drops | test suite |
| R14 — backpressure off the input path | §7.4 saturation test | test suite |
| R3 — pure core depends on nothing | `cargo build -p infinite-runtime` with no features; `[dependencies]` empty | CI |
| R4 / R16 — registries, not enums | no closed enum in the core standing for an open set | review + decision record |
| F-8 — no `mod.rs` | file listing | CI |

---

## 9 · Crate layout

One crate, `crates/infinite-runtime`. The core/binding split (D7) is a **module and
feature** boundary rather than a crate boundary, following `bion`'s proven shape — R3's
check explicitly cites `bion` verifying soma with `--no-default-features`. Inverted so
that the strict build is the default:

- **default features: none.** The core builds alone, with an empty `[dependencies]`.
- **`binding`**, off by default, adds the ports, the registry, and the driver — the
  part that knows a graph exists.

```
crates/infinite-runtime/
  Cargo.toml
  README.md              → points at this document; does not restate it (R17, R21)
  src/
    lib.rs
    core.rs              module file: docs, mod declarations, re-exports only
    core/
      addr.rs            Addr — opaque ordered key (§3.2)
      budget.rs          Budget
      outcome.rs         Outcome
      priority.rs        priority — shared-prefix distance from focus
      frontier.rs        Frontier
      pending.rs         PendingSet
      coalesce.rs        coalesce — one in-flight commit per address (§7.2)
    binding.rs           module file
    binding/
      ports.rs           module file
      ports/
        store_read.rs  store_write.rs  stale_feed.rs  clock.rs  journal.rs
      registry.rs        ArtifactRegistry (§5.2)
      driver.rs          tick (§7.1)
  tests/
    discard.rs           the generic R12 harness
    saturation.rs        §7.4
    fake_store.rs        the only store this layer ever names
```

`module.rs` plus a directory of leaf files, no `mod.rs` (F-8). **Reading of the
one-public-function-per-file convention, stated so it is not silently reinterpreted:**
it applies to free functions; a type with an inherent impl is one file. `Frontier` and
its methods are one file; `priority` and `coalesce` are files of their own.

---

## 10 · Findings in the existing scaffold

Recorded here rather than acted on, because each needs a decision or a rename that is
not this document's to make:

1. **`crates/infinite-ux` is an empty directory with no spec** — an R18 finding as it
   stands, and the name is stale: D17 renamed the layer to **presenter**. It should
   become `crates/infinite-presenter` when the presenter layer gets its spec.
2. **Root `src/main.rs` exists (0 bytes)**, implying the workspace root is a binary
   crate. R1 says Infinite Solutions is a platform, not an application, and D12 defers
   every app. The root should be a virtual workspace; `src/main.rs` is a leftover of
   the abandoned three-layer scaffold.
3. **`crates/infinite-db` (vendored) and `crates/infinite-physics` are gone** since the
   2026-08-20 direction read. Consistent with D1 (the store is the published crate) and
   D10 (physics is a facade, not a platform layer) — noted so the removal is on the
   record rather than inferred later.

---

## 11 · Open, carried forward

| # | Item | Trigger |
|---|---|---|
| O1 | Hot working set | Measure a warm prefix scan of ~1000 nodes against frame budget, in S6's test bed |
| O12 | May an iterative region yield | The first named consumer with a solve in it; §7.1 keeps the door open at no cost |
| — | Settling loops (D21) | Out of scope by §2; extend this spec when the crane mat is named |
| O10 | Ownership and capability | Not this layer's, but §3.1's ports are the place a check would be inserted; do not build them so that it cannot be |
