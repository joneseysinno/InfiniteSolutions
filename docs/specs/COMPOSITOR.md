# infinite-compositor — Layer Specification

> **Status:** draft 1, 2026-08-21. S1 is the specification; S2–S8 landed
> with E5–E9. R20: a status line is written by the change that lands the
> phase.
>
> Layer: **compositor** (D2, D17). Rules: [`../RULES.md`](../RULES.md) · Decisions:
> [`../DECISIONS.md`](../DECISIONS.md) · Charter: [`../CHARTER.md`](../CHARTER.md) ·
> Sibling: [`RUNTIME.md`](./RUNTIME.md)
>
> Satisfies R18 for this layer. The crate may exist once this document does.
> Records D26, D27 (closes O6), D28 (closes O9). Opens O13.

---

## Stage table

| # | Stage | Status | Green check |
|---|---|---|---|
| S1 | This specification | draft 1 | Recorded as D26–D28; reviewed line by line against R31, D11, D13, D14, D19, D21, D22 |
| S2 | Crate skeleton, pure core | landed | `cargo build -p infinite-compositor` with **no features** succeeds and the manifest's `[dependencies]` is empty |
| S3 | Ports and the fakes | landed | The whole test suite passes against fakes; no crate belonging to another layer is named anywhere in the crate |
| S4 | `link` and findings | landed | A corpus of malformed compositions each yields **exactly one** finding, at an address, carrying said / wanted / remedy |
| S5 | Composition closes | landed | The **closure test** (§7.3): wrap a linked composition as a block, nest it, re-link; the plan is identical to the flattened original |
| S6 | Interpreted execution and provenance | landed | A composition runs; provenance recovers the exact declared input set; the store's staleness query returns **exactly** the downstream address set |
| S7 | Backend contract, tier 0, equivalence harness | landed | Tier 0 registers **by passing the harness** over the plan corpus, with no per-backend test code |
| S8 | First real binding, in the facade | landed | The S3–S6 tests pass unchanged against fakes; `tests/behaviour.rs` runs the same `interpret` against real `infinite-db` |

**On S2's status line.** The files exist and their green check has been run — `cargo
build -p infinite-compositor` with no features succeeds, `[dependencies]` is empty,
`--features binding` succeeds, both with zero warnings, and `scripts/check-rules.sh`
passes its compositor section. The status line is still written by the change that
lands the phase and by the person landing it (R20), not by the change that wrote the
files: `HISTORY.md` traces every recorded drift to status headers stamped once at
authoring time, before the work was in.

---

## 1 · What this layer is

The runtime owns time (D5). The compositor owns **structure** — and it is the layer
that has no time at all.

| Layer | Its relationship to time |
|---|---|
| **store** | logical time — revisions, HLC. No *now*. |
| **runtime** | *now* — cadence, frame, priority, deadline. |
| **compositor** | **neither.** Linking a composition at revision N is a pure function of the definitions at revision N. |

Membership test for this layer, alongside D5's:

> **If it is true of a program before it runs, it belongs to the compositor. If it is
> only true while it is running, it belongs to the runtime. If it survives a restart,
> it belongs to the store.**

Two laws, carried in the manner of D4's L1 and L2:

> **L3 — The compositor contains no math.** (R31.) It owns the *contract and lifecycle*
> of a computation. Every number is inside a block.
>
> **L4 — The compositor is a function, not a place.** It holds nothing across a call.
> Everything it produces is derived (R12) and passes the discard test by construction,
> because it was computed from the store and nothing else.

The layer's whole job, stated once:

> **Given a set of definitions at a revision, produce a plan that can be executed — or
> a finding that says precisely why it cannot.**

The second half is not the error path. Under D16 (*a child should be able to build a
full stack app*), the finding **is** the primary output about half the time a person is
working, and it is the only part of this layer they will ever read. §6 specifies it
with the same care as the plan.

---

## 2 · The forcing consumer (R19, R26)

**The editor** — the platform's own graph editor (O11). The same consumer as
`RUNTIME.md` §2, deliberately, so the two specifications can be read against each
other.

R19 requires the consumer be *something that breaks if the layer is wrong*. Six
breakages, each with a prior occurrence:

| # | If the compositor is wrong | What the person sees |
|---|---|---|
| C1 | Findings are imprecise | "type error" instead of *"this wire carries a `roster` into a port that wants a `drill`"*. D16 rules out stack traces as the error surface. `biomimicry` M12 already has the good version — unsatisfied imports fail at **link** time with a precise error, before compile is attempted |
| C2 | Composition does not close | The editor cannot be built in the platform, because the editor is a composition of compositions. `Innovator`'s six-level tier ladder (root → workspace → page → pod → component → particle) is hardcoded depth standing in for closure — D20 already names it as a zoom ladder built without being named as one |
| C3 | Two linkers | The editor's own interface graph and the compute graphs it edits need different machinery, and self-hosting dies at the first screen. O6 |
| C4 | Link is not a pure function of a definition set | The editor cannot answer *"if I drop this wire here, does it link?"* while the wire is still **pending** (D8) — so validation can only happen after commit, and the child finds out what was wrong after doing it |
| C5 | A drawn cycle hangs instead of reporting | *"a spinning app with no explanation is the worst failure available to someone learning"* — D21's stated reason for rejecting runtime fixed-point iteration |
| C6 | Compilation is not observationally identical | You test the interpreted graph and ship a different program. D19's equivalence law exists for exactly this |

**C4 is the one that is new here**, and it constrains the whole layer: `link` must be
cheap enough to run on a speculative definition set, at interaction rate, without
touching the store's write queue (D24). That is why §3's `Definitions` port reads a
*set*, not the store, and why the plan is derived rather than stored.

**Scope consequence of choosing the editor alone.** The editor exercises no solve.
D21's iterative regions are therefore specified here as **structure only** — a region
is a block from outside, and a cycle outside a marked region is a finding — while
convergence, damping, and stopping tests are out of scope. `RUNTIME.md` §2 made the
identical cut for the identical reason; the trigger to extend both is the same, the
first named consumer with a solve in it (the crane mat). Recorded so the gap is
deliberate rather than discovered.

---

## 3 · The seam: the compositor depends on no other layer (D26)

**The compositor names no crate belonging to another layer.** It declares the ports it
needs as traits; the platform facade supplies the implementations.

*Why.* D3 established that a computation must not depend on the runtime, or the two
become mutual hostages and neither is testable alone. D23 ran the same argument for the
runtime. It runs a third time here, and the consequence is sharper: a compositor that
names `infinite-db` cannot be link-tested without a database, and **link is the thing
that must run at interaction rate on speculative input** (C4). A layer whose central
function cannot be exercised in a unit test is a layer whose central function is
untested.

### 3.1 The ports

Five, and no more without a decision record. Plain functional nouns (R15).

| Port | The compositor asks it for | Notes |
|---|---|---|
| `Definitions` | the definition at an address — a block's declared ports, and its body | Reads a **set**, resolved against a revision. Speculative sets are ordinary input, which is C4 |
| `Blocks` | resolve a native block key to its declared signature and an invocable primitive | String-keyed registry (R4). The compositor knows a block's *shape*, never what it computes (L3) |
| `Values` | read an input at an address; write an output at an address | Payload and tag both opaque (D13) |
| `Provenance` | record what was computed from what, at which revision | D11. Half of this exists in the store and has never been driven (§11) |
| `Backends` | resolve a compiled-form key to a registered backend | §10. String-keyed for the same reason as `Blocks` |

**There is no `Clock`.** Stated positively so it stays true: *the compositor has no
`now`*. If this layer ever needs one, R10 has been violated and the item belongs in the
runtime.

### 3.2 `Addr`, and what this layer needs of it

The pure core depends on nothing (R3), so — exactly as in `RUNTIME.md` §3.2 — it
cannot use the store's key type.

> **`Addr` is an opaque, totally-ordered byte key.**

The compositor needs strictly **less** of it than the runtime does:

| Property | Runtime | Compositor | Why |
|---|---|---|---|
| Equality | yes | **yes** | a wire names two ports; two blocks are the same block or they are not |
| Total order | yes | **yes** | plan ordering must be deterministic, or D19's equivalence test is statistical rather than exact |
| Prefix truncation = level | yes | **no** | that is priority-by-distance-from-focus, a runtime concern (B6). Admitting it here would give the compositor a reason to care where a block is on screen |
| Permanence | relied on | relied on | the store's invariant; neither layer verifies it |

Recording the *absence* of prefix truncation is the point. R27: generality is a defect
unless a named consumer requires it, and no consumer of this layer requires the
compositor to know a block's depth.

**Cost, recorded rather than solved.** `infinite-runtime` and `infinite-compositor`
each define their own `Addr`, because each core depends on nothing. The facade converts
between them. Today that is a newtype unwrap. **O13** records the trigger for when it
stops being one.

---

## 4 · What the compositor may hold (L4)

Nothing across a call. Stated as three checkable claims:

1. **The core owns no mutable state.** Its types are values, passed and returned;
   there is no place for anything to accumulate. *Checked by:* no `static`, no
   interior mutability (`Cell`, `RefCell`, `Mutex`, `OnceCell`, an atomic), and no
   `&mut self` method that retains across calls.

   *An earlier wording of this check — "no struct field that is not a call argument" —
   was written into draft 1 and does not survive contact with the crate: `Plan` and
   `Composition` obviously have fields. R23's discipline applies to checks as much as
   to claims; a check that fires on correct code is worse than no check, because it
   gets disabled.*
2. **The binding holds two registries** — `Blocks` and `Backends` — populated at
   startup. These are symbol tables, not state: nothing is authored into them while a
   program runs. *Checked by:* registration is not reachable from `interpret`.
3. **The plan is a derived artifact, and it is registered with the runtime.**

Point 3 is the load-bearing one and it is worth stating loudly.

> **D25 already built the machine.** The runtime knows artifact *lifecycle*, never
> artifact *content*: an artifact is registered under a string key with the address
> ranges it derives from, a rebuild function, and a validity watermark. A `Plan` is
> exactly that shape. The compositor registers `link` as the rebuild function; the
> runtime owns when it runs and whether it is stale.

Two things follow, both free:

- **The compositor needs no cache invalidation machinery of its own.** R12's discard
  harness — one generic test, no per-artifact code — covers plans automatically, on
  the day it is written for `RenderList`.
- **This is D25's split for the third time**: presenter owns the function and runtime
  owns the schedule; compositor owns the function and runtime owns the schedule. Three
  instances is a pattern rather than a coincidence, and it means the runtime's artifact
  registry is the real integration surface of the platform.

The three state categories (D8), instantiated:

| Category | In the compositor | Discardable |
|---|---|---|
| **Stored** | nothing (L4) | — |
| **Derived** | the plan; every compiled artifact | yes, by definition |
| **Pending** | nothing — pending is the runtime's (D8, D24). The compositor *links against* a pending set handed to it, and never holds one | — |

---

## 5 · The model

D9 deliberately did **not** carry forward the recalled six-primitive set
(Node, Port, Edge, Value, Block, Instance), on the grounds that O5 (what is a `Value`)
and O6 (one linker or two) were open, and *"asserting the primitive set before those
are settled would be deciding them by implication."*

D13 closed O5. D27 below closes O6. The set can land — and it lands **re-derived from
the corpus**, which is D9's standard, not recalled.

### 5.1 The compositor adds three words to D20's five

D20's vocabulary is `space`, `node`, `graph`, `hyperedge`, `zoom`. This layer adds:

| Word | Meaning | Not a new noun because |
|---|---|---|
| **block** | a space with declared ports | it *is* a space (D20), seen as a unit of composition |
| **port** | a named, directed, tagged attachment point on a block | genuinely new; it is what D14 says the substrate owes an app, item 1 |
| **plan** | the linked form of a composition: a deterministic order of steps with sources and sinks resolved | derived, never authored (L4) |

And three words that are already decided elsewhere and are only *used* here:

- **tag** — opaque; the platform's only operation on one is **match** (D13).
- **value** — opaque payload plus a tag (D13).
- **wire** — a hyperedge (D20) seen at a port. Every wire is drawn (D22).
- **composition** — a graph (D20) seen as a block. This is D14.6.

### 5.2 `Instance` was redundant, and here is the evidence

The recalled set had a sixth word for a block used twice. It is not needed.

Under D20, every space has a permanent address, and a node is a space seen from one
level out. A block used twice in a composition is therefore **two spaces**, each with
its own address, each of whose body *names* another space.

> **Use is delegation.** A block used in a composition is a space whose body is a
> reference.

This is `bion`'s chain grammar — `.node().delegate().connect().seal()` — which D6
explicitly salvaged from `hypernode` as runtime material and which turns out to be
compositor material. `delegate` is the word for exactly this and it has already been
implemented once.

*Consequence:* there is no instantiation step, no template/instance distinction, and no
second identity space. F-2 (a map keyed by id standing in for an edge, ~30 instances in
one codebase) had `Innovator`'s template/instance split as one of its feeders.

---

## 6 · Findings: the error surface (D16)

A `Finding` is what the person reads. It is specified before the plan because it is
read more often.

### 6.1 Shape

```
Finding {
    site:    Addr,          // where. The editor zooms here (D20)
    kind:    &'static str,  // registry key, not an enum — see 6.3
    said:    String,        // what the composition says
    wanted:  String,        // what would have satisfied the linker
    remedy:  String,        // what to do next
}
```

**Three constraints, each checkable:**

1. **Every finding has an address.** So *"go to the error"* is not an editor feature —
   it is a zoom (D20). Nothing in the layer may produce a finding without a site.
2. **Every finding has a remedy.** *"A child should be able to build a full stack app"*
   (D16) means the message says what to do, not only what is wrong. A finding kind
   registered with no remedy sentence is a defect, and that is a test, not a review
   note.
3. **One cause yields one finding.** A single unsatisfied import must not produce a
   cascade. *Checked by:* the S4 corpus — each malformed composition yields exactly one.

### 6.2 The linker's own kinds

Six, and this list is the editor's error surface, so it must be complete:

| Key | Raised when | Forced by |
|---|---|---|
| `unsatisfied-import` | a required input port has no wire | `biomimicry` M12, verbatim |
| `tag-mismatch` | a wire connects tag A to a port wanting tag B | D13 — match is the platform's only operation on a tag |
| `arity` | a port is bound more times than it admits | hyperedges are n-ary (direction read §3.2), so this is per-port policy, not a global rule |
| `cycle` | a wire closes a loop outside a region marked iterative | D21 — *judged, not refused*: the edge may exist, derivation runs as far as it can |
| `unresolved-block` | a body names a native key with no registration, or an address with no definition | R4 — the cost of a string-keyed registry, paid at link time rather than at run time |
| `not-pure` | a composition marked compilable reads something not among its declared inputs | D19 — compilability and staleness are one discipline |

### 6.3 Why `kind` is a string and not an enum

R16 would permit an enum here with a decision record, since the linker is one piece of
code and its own kinds are a closed set. It is still a string, on the two-domain test
(R32): a physics facade wants to say *"this boundary is unconstrained"*, and Coach
Assistant wants to say *"this drill declares twelve players and the roster has ten"*.
Both are findings, both want the editor's existing rendering, and neither is the
linker's.

So the **finding channel is platform and the finding kinds are open**. The compositor
owns the shape and the three constraints; anyone may register a kind. F-1 avoided
without spending a decision record on it.

---

## 7 · Linking

### 7.1 The function

```rust
pub fn link(defs: &DefinitionSet, root: Addr) -> Outcome<Plan>
```

Pure. No I/O, no clock, no store. `DefinitionSet` is whatever the `Definitions` port
resolved — stored, pending, or a mix, which is C4.

`Outcome` carries **both** a plan and findings, never one or the other: D21 requires
that a drawn cycle be judged rather than refused, and *"derivation runs as far as it
can and no further."* A composition with one bad wire still runs the other ninety.

### 7.2 What linking does

1. Resolve each body — a native key through `Blocks`, an address through `defs`
   (§5.2, delegation).
2. Match tags across every wire (D13). Validate arity. Emit findings.
3. Order the steps. Cycle detection here, not at run time (C5).
4. Derive the composition's own signature (§7.3).

That is the whole list, and its shortness is the specification. Anything else that
wants to happen at link time is either math (L3) or scheduling (R10).

### 7.3 Composition closes, and how it is tested

D14.6 is the load-bearing obligation — *without it composition stops after one flat
layer and "primitives into blocks into great things" does not happen.*

The mechanism is one rule:

> **The signature of a composition is its unbound ports.** An input port with no wire
> inside becomes an input of the whole; an output port with no wire inside becomes an
> output.

There is therefore no "top level". The editor is a composition, the app is a
composition, and the difference between them is which one you have zoomed to.

**The closure test** — this layer's equivalent of the discard test, and the S5 green
check:

> Link composition **C**. Wrap the result as a block **B**. Build a composition **C′**
> that contains only **B**, wired straight through. Link **C′**. The plan must be
> identical to the plan for **C**.

If closure is broken, this fails, and it fails mechanically rather than as a judgment
about whether nesting "feels right".

---

## 8 · Iterative regions (D21), structurally

A region is a space marked iterative, carrying a maximum iteration count and a stopping
test as **visible properties** — D21: *"the loop is drawn on the canvas, not hidden in
a black box."*

Three facts are this layer's, and they are all it specifies:

1. **From outside, a region is a block.** Declared inputs and outputs; one step in the
   enclosing plan. D14.6 holds unchanged.
2. **A cycle outside a region is a `cycle` finding**, not a refusal (§6.2).
3. **A region is still a pure function of its declared inputs**, so D19 holds and a
   region is compilable.

Convergence, damping, and non-convergence-as-a-finding are **out of scope by §2** —
they need a consumer with a solve in it. Nesting is permitted and already precedented
(`biomimicry`'s two nested scheduler loops at different cadences).

---

## 9 · Execution: the plan

### 9.1 The split

> **The compositor decides what runs and in what order. The runtime decides when and
> how much.**

Prior art, already built: `bion`'s pure library emits an `ExecutionPlan` and an
external runtime owns the clock and the threads (direction read §4). This is that,
and it is D25's core/binding split across a layer boundary for the **third** time.

### 9.2 What a plan is

A deterministic sequence of steps. Each step names a block, its resolved input sources,
and its output sinks. A region is a single step carrying its own inner plan.

**Determinism is required, not preferred.** D19's equivalence law is exact rather than
statistical only because execution is deterministic, and `biomimicry` already resolved
manufactured determinism. A plan whose order depends on iteration order of a hash map
makes the compile story unverifiable.

### 9.3 Interpreted execution

Walk the plan. For each step: read inputs through `Values`, invoke through `Blocks`,
write outputs through `Values`, record through `Provenance`.

Note what is absent: no math (L3), no scheduling (R10), no allocation policy, no
transport. Every number is inside a block; every *when* is the runtime's.

---

## 10 · Compilation (D19), and O9

### 10.1 O9 is the wrong shape of question

O9 asks: *native code per target, or WASM, or an IR that JITs?* That is a which-one
question, and R16 forbids answering it as one — a closed enum standing for an open set
is F-1, five prior occurrences.

The set is open, and this system's own roadmap already contains two more members
neither `bion` nor `biomimicry` considered:

- a **GPU kernel** — the presenter is wgpu (D15); a fused numeric composition compiled
  to WGSL is a short walk, not an exotic one;
- a **pushdown into the store** — a composition of pure reads over an address range is
  a range scan, and `infinite-db` already has the index.

Answering O9 with a name would be F-1 in the one place where the cost of being wrong
is rebuilding the toolchain.

### 10.2 The answer (D28)

> **The compiled form is a registered backend under a string key. The compositor owns
> the contract a backend must satisfy, and never the backend.**

D25's shape, one layer over: *the runtime knows artifact lifecycle, never artifact
content* becomes *the compositor knows compiled-form lifecycle, never the form*.

**The contract.** A backend supplies four things:

| | | |
|---|---|---|
| 1 | `accepts(&Plan) -> bool` | not every backend takes every plan, and saying so is the backend's job |
| 2 | `compile(&Plan) -> Artifact` | bytes plus an invocation handle |
| 3 | `invoke` | **the same signature as interpreted invocation** — this is what makes equivalence testable rather than aspirational |
| 4 | a **cost declaration** | what it costs to produce (compile time) and to cross into (call overhead). D19 says compilation is chosen *"on the runtime's evidence rather than the author's guess"*, and the runtime cannot choose without this |

### 10.3 The registration gate

**A backend is not registered because someone wrote it. A backend is registered by
passing the equivalence harness.**

For every plan in a maintained corpus: run interpreted, run compiled, compare outputs
bit-for-bit and provenance edge-for-edge. D19 says *the interpreted execution is the
specification*; this is the sentence made executable, and it means the platform cannot
grow a subtly-wrong backend. Adding WASM in 2027 costs a registration, not a redesign.

And R12 is free again, for the same reason as §4: a compiled artifact is derived state
(D19), registered with the runtime (D25), so the generic discard harness drops and
rebuilds it **without knowing what it is**. Compilation needs no invalidation machinery
of its own. That is twice this specification gets something substantial from D25, which
is the strongest evidence available that D25 was the right decision.

### 10.4 Three tiers, in the order they should be built

| Tier | Form | Removes | Costs |
|---|---|---|---|
| **0** | **resolved plan — no compiler** | lookup and dispatch: sources resolved to slots, invocations to function pointers, order fixed | nothing. No toolchain, no codegen, no dependency |
| **1** | **native** — generate Rust for the fused composition, build it with the toolchain block authors already have (D16) | per-edge value boxing; lets the optimizer see across block boundaries | a toolchain at author time; a per-target artifact |
| **2** | **portable** — WASM | nothing extra | a call-boundary cost SES may refuse |

**Tier 0 is the out-of-the-box move and it should be built first.** The first compiled
form requires no compiler. It is the classic large constant-factor win before any real
codegen, it adds no dependency to a crate whose S2 green check is an empty
`[dependencies]`, and its equivalence is the easiest of the three to argue — it runs
the same code in the same order with the lookups hoisted. It is also the honest first
test of the harness, because a backend that *should* be equivalent failing the harness
means the harness is wrong.

Tier 2 is what D18's portals want — one artifact that runs on every target, so a
compiled block can cross a portal because it is native to neither side. It is listed
third because paying its call cost must be a *choice*, which is the entire argument of
§10.2.

**Which tier is used is a placement decision** (D16's word for the same move), made by
the runtime from the cost declarations plus its own measurements. It is not a property
of the block and not the author's guess.

### 10.5 What it costs

Three things, stated so none is discovered:

1. **A plan corpus must be maintained.** The harness is only as good as its corpus, and
   an unexercised plan shape is an unverified one. The corpus is a deliverable, not a
   test fixture.
2. **`accepts` makes refusal reachable.** A plan can be compilable in principle and
   refused by every registered backend. The author must be told so in a finding — not
   left with a composition that silently stayed interpreted.
3. **The cost declaration is self-reported**, so a lying backend misroutes. Mitigated
   structurally: **the runtime measures, and may demote a backend whose measured cost
   contradicts its declaration.** Demotion is a runtime concern; the declaration being
   data rather than a promise is what makes demotion possible.

---

## 11 · Provenance and the staleness contract (D11)

Half of this exists in the store and has never been driven: `infinitedb_core/
computation.rs`, `provenance.rs`, hyperedge payload codec V4 = `computation`,
`check_hyperedge_freshness`, `query_stale_downstream`, and the `engine/derivation/` bus.
The store already knows an edge can carry a computation and that changing an input makes
downstream stale. **The compositor is what invokes one and populates that provenance.**

Every executed step records: outputs, the exact input set, block identity, revision.
The derivation DAG *is* the trace (charter).

**One declaration, three payoffs** — the charter says two, and it is three:

| Payoff | Consumer |
|---|---|
| staleness | the store computes the downstream set (D11) |
| compilability | a composition is compilable iff it is a pure function of its declared inputs (D19) |
| audit | a stamped result is reproducible from its provenance (charter) |

This is why `not-pure` (§6.2) is a link-time finding rather than a compile-time one:
the same declaration that would have made it compilable is the one the store needs
whether or not anyone ever compiles it.

*S6 green check:* an input change at revision N yields **exactly** the downstream
address set — no more, no fewer. Identical in form to `RUNTIME.md`'s S6, deliberately;
if the two disagree, one of the layers is wrong about what a dependency is.

---

## 12 · How every rule is checked, in this layer

Every check below lives in `scripts/check-rules.sh`, which the runtime layer already
established as the place where this project's rules stop being preferences.

| Rule | Check | Lives in |
|---|---|---|
| R3 — pure core depends on nothing | `cargo build -p infinite-compositor` with no features; `[dependencies]` empty | CI |
| R31 / L3 — no math | manifest grep: no numerical or domain dependency; source grep: no `f32`/`f64` | CI |
| R10 / D26 — no `now` | source grep: no `std::time`, `Instant`, `SystemTime`. There is no `Clock` port and there must be no clock | CI |
| L2's analogue — no storage | source grep: no `std::fs`, no file handle | CI |
| L4 — owns no mutable state | no `static`, no interior mutability, no retaining `&mut self` method (§4.1) | CI |
| D26 — no other layer is named | source grep: no `infinite_(db\|runtime\|presenter\|physics\|ux)` | CI |
| R4 / R16 — registries, not enums | the core's enum **count** is pinned; a new one fails the check until a decision record raises the number | CI |
| R12 — artifacts pass the discard test | the runtime's generic harness (D25); this layer contributes **no** per-artifact test code | runtime test suite |
| D13 — tags are matched, never interpreted | source grep: no `Display` impl for `Tag`, no `parse` in the layer | CI |
| D14.6 — composition closes | the closure test (§7.3) | test suite |
| D16 — the error surface | every finding carries a site and a non-empty remedy; one cause → one finding | test suite |
| D19 — equivalence | the harness is the backend registration procedure (§10.3) | test suite |
| D21 — cycles are judged, not refused | a composition with a cycle returns both a plan and a `cycle` finding | test suite |
| D22 — wiring is explicit | no code path creates a wire that was not in the definition set | review |
| F-8 — no `mod.rs` | file listing | CI |

**Every grep check strips comment lines first.** Found the hard way while verifying the
S2 skeleton: the documentation that *cites* a rule trips the grep that *enforces* it —
`addr.rs` names `infinite-runtime` in a doc comment explaining why it must not depend
on it, and `value.rs` names `f64` while explaining why there are none. A check with
false positives is a check that gets switched off, which is F-7's mechanism applied to
CI instead of to a cache. **The runtime's existing checks in `scripts/check-rules.sh`
grep raw source and have the same latent problem**; recorded as finding 4 below.

**Pinning the enum count, rather than grepping for enums.** R16 says a new enum in a
core crate requires a decision record. A grep that merely reports enums says nothing;
a check that fails when the count changes turns R16 into something that has to be
answered before the build goes green. The number is currently **one** (§14, finding 5).

*Considered and set aside: making the core `no_std`.* It would convert three of the
greps above — `std::time`, `std::fs`, `std::thread` — into compiler errors, which is
strictly better enforcement, and `bion`'s soma is precedent. Set aside because
`infinite-runtime` is already landed on `std` and one house style is worth more than
one layer's extra rigour. It stays available as a **both-layers** change: it would
strengthen R9's *"no file handle"* check as much as anything here, and it is a two-line
diff per crate.

---

## 13 · Crate layout

One crate, `crates/infinite-compositor`. Core/binding split (D7) is a **module and
feature** boundary, following `bion`'s proven shape and `RUNTIME.md` §9 — inverted so
the strict build is the default.

- **default features: none.** The core builds alone, `[dependencies]` empty.
- **`binding`**, off by default, adds the ports, the registries, and execution.

Three conventions are taken from `infinite-runtime` rather than re-decided, because
R17's failure mode is two houses in one repository:

- **Module files declare `mod` privately and re-export.** `pub mod` in a module file
  would give every type two paths, which is one name for two things read backwards.
- **`edition`, `rust-version`, `license`, `publish` are inherited** from
  `[workspace.package]`.
- **`autotests = false` with explicit `[[test]]` targets**, so `tests/fakes.rs` can be
  a shared helper without becoming a test target — the conventional Rust answer is
  `tests/common/mod.rs`, and F-8 forbids it.

```
crates/infinite-compositor/
  Cargo.toml
  README.md              → points at this document; does not restate it (R17, R21)
  src/
    lib.rs
    core.rs              module file: docs, mod declarations, re-exports only
    core/
      addr.rs            Addr — opaque ordered key (§3.2)
      tag.rs             Tag — opaque; match is the only operation (D13)
      value.rs           Value — opaque payload + Tag (D13)
      port.rs            Port — name, direction, tag (D14.1)
      signature.rs       Signature — a block's declared ports
      block.rs           Block — a space with a signature and a body
      wire.rs            Wire — a hyperedge seen at a port (D22)
      composition.rs     Composition — blocks and wires
      definition_set.rs  DefinitionSet — what link is handed (§3.1, C4)
      finding.rs         Finding — site, kind, said, wanted, remedy (§6)
      outcome.rs         Outcome<T> — a plan and findings, never one or the other
      order.rs           order — deterministic step order; cycle detection (§7.2)
      link.rs            link — the function (§7.1)
      signature_of.rs    signature_of — unbound ports become the signature (§7.3)
      region.rs          Region — iterative, structurally (§8)
      plan.rs            Plan, Step (§9.2)
    binding.rs           module file
    binding/
      ports.rs           module file
      ports/
        definitions.rs  blocks.rs  values.rs  provenance.rs  backends.rs
      backend.rs         Backend — the four-part contract (§10.2)
      registry.rs        the two registries (§4.2)
      interpret.rs       interpret — walk the plan (§9.3)
      artifact.rs        KEY + encode — the plan as a D25 artifact
  tests/
    findings.rs          the S4 corpus
    closure.rs           the closure test (§7.3)
    interpret.rs         the S6 walk and provenance check
    equivalence.rs       the S7 harness; also the backend registration procedure
    fakes.rs             the only implementations of the ports this layer ever names
```

`module.rs` plus a directory of leaf files, no `mod.rs` (F-8). One public function per
file for **free** functions; a type with an inherent impl is one file — the same
reading `RUNTIME.md` §9 records, restated here only because it is the rule most likely
to be silently reinterpreted.

---

## 14 · Findings in the existing scaffold

`RUNTIME.md` §10 recorded three findings. **All three are now resolved on disk**, and
saying so here is R20's habit applied to a finding rather than to a stage: a finding
list that is never re-read becomes a document describing a repository that no longer
exists.

1. ~~`crates/infinite-ux` is an empty directory with no spec.~~ The directory is gone.
   When the presenter layer gets its spec it arrives as `crates/infinite-presenter`
   (D17), and `scripts/check-rules.sh` already enforces that every crate directory has
   a matching `docs/specs/<LAYER>.md`.
2. ~~Root `src/main.rs` exists.~~ Gone; the root is a virtual manifest. **The root
   `Cargo.toml`'s comment still says it exists** — a stale note about a resolved
   finding, corrected in the change that adds this crate to `members`.
3. ~~`crates/infinite-db` (vendored) and `crates/infinite-physics` are gone.~~ Recorded
   and still true.

New with this document:

4. **The runtime's checks in `scripts/check-rules.sh` grep raw source**, so a doc
   comment citing a rule can trip the check enforcing it (§12). The compositor's
   section strips comments; the runtime's does not yet. Not urgent — no current
   comment collides — which is exactly why it is worth writing down now.
5. **`infinite-runtime` and `infinite-compositor` each define an `Addr`.** Correct under
   R3 and D23, and recorded as O13 rather than solved. Note they are not the same type
   even in shape: the runtime's carries `shared_prefix_bits` and `in_range`; the
   compositor's deliberately carries neither (§3.2).

Raised by writing the S2 skeleton, and left for decision rather than decided (R29 — a
proposal that adds an enum is corrected, not merged):

6. ~~**`Direction { In, Out }` is an enum in a core crate.**~~ Closed by **D35**:
   the set is genuinely closed. A third variant would be a different concept. The
   enum count stays pinned at one.
7. **`Body` is deliberately *not* an enum**, for the mirror-image reason: D18 added
   portals and D21 added iterative regions on the same day the model was drawn — two
   new body kinds in two days, which is as much evidence as an open set ever provides
   in advance. It is a string key plus an address, and every space having a permanent
   address (D20) is what makes one address field enough. If that turns out to be
   wrong, it wants a decision record too.

---

## 15 · Open, carried forward

| # | Item | Trigger |
|---|---|---|
| **O13** | Two `Addr` types, one per layer core | When the facade's conversion is more than a newtype unwrap, promote `Addr` to a zero-dependency crate. Not a layer, so R1 is not engaged; still needs a decision record |
| O11 | Is the editor self-hosted | **Closed by D36.** Yes. |
| O12 | May an iterative region yield between iterations | The first consumer with a solve. §9.2's plan makes a region one step with an inner plan, which is the shape a yielding region needs — no new concept required |
| — | Settling loops (D21) | Out of scope by §2; extend when the crane mat is named. Same trigger as `RUNTIME.md` |
| O10 | Ownership and capability | Not this layer's, but `link` is where a *"may this composition use that block"* check would go. Do not build `Blocks` so that it cannot be inserted |
