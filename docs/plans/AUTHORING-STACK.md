# Infinite Solutions — The Authoring Stack (E14–E20)

> **Status:** draft 2, 2026-08-29. **E14 and E14.1 landed** (readable text via glyphon;
> hysteresis threaded through `place`). Remaining stages E15–E20 are not started.
> R20 / D41: a stage is `landed` only when its **Verified by** cell names a check.
>
> **Records nothing** (R29). Every refactor below is a *proposal*; every trigger is a
> candidate. Same standing as [`PARALLELISM.md`](../PARALLELISM.md).
>
> **Changed in draft 2:** §2.1 gains the root cause of the fifteen-child ceiling, which
> draft 1 recorded only as a symptom. §5 is new — the alphabet, and the two rules that
> bound it. E18 splits into E18a / E18b. Findings 26 rewritten; 28–29 added. O32
> widened; O35 opened.
>
> Rules: [`../RULES.md`](../RULES.md) · Decisions: [`../DECISIONS.md`](../DECISIONS.md) ·
> Charter: [`../CHARTER.md`](../CHARTER.md) · Ledger: [`../SALVAGE.md`](../SALVAGE.md) ·
> Structural precedent: [`EDITOR-BOOTSTRAP.md`](./EDITOR-BOOTSTRAP.md), which spanned
> nine stages under one thesis.
>
> Written under R28. Raises findings 25–29. Opens O32–O35.

---

## 0 · The thesis, in one sentence

> **Every prior attempt clicked when it got a way to turn authored data into running
> structure. Infinite Solutions built the running structure first and has no way to
> author into it, so everything on the screen had to be typed into Rust by hand — and
> that cost grows with the app, which is why the app is two rectangles.**

Measurable, and the numbers are the argument for everything below.

| Measure | Today | What it should be |
|---|---|---|
| Well-known address constants in `editor/addresses.rs` | **83** | ~6 — the bootstrap ABI D34 describes |
| Lines of `editor/genesis.rs` to seed **19** spaces | **675** | Innovator's `catalogue.rs` is 6.5 KB and produces twelve reusable components |
| Fields on `SpaceRecord` empty for most records | **3** of 10 (`primitive`, `link`, `text`) | 0 |
| Maximum children a space can hold | **15** (`mint.rs`: `next > 0x0F` → `None`) | unbounded |
| Native blocks | **11**, of which ~6 are one-offs (§5.3) | a small, stable alphabet |
| Blocks that are **not** native — i.e. composed (D14.6) | **0** | the normal case |

The causal chain is short and it explains the symptom exactly:

> no authoring vocabulary → every node needs a Rust name and a literal address →
> 83 constants and 675 lines → **the cost of authoring is linear in Rust edits** →
> nothing large is ever authored → the screen stays small.

The four layers are not the problem. On this audit they are sound. The problem is that
the only authoring tool in the system is a text editor pointed at `genesis.rs`.

---

## 1 · What made each prior attempt click

**Innovator clicked on `Spec` → `commit`.** From `pgraph/store.rs`:

> `Spec` is *authoring sugar only* — a transient nested value that `commit` flattens
> into nodes and edges. **Nesting at authoring time does not make the runtime a tree.**

That seam is why `panel(vec![section_header("Wall"), field_row("Height", …)])` could be
written by a person and land as a flat graph. **Infinite Solutions has no equivalent,
and this is the single biggest gap.**

**bion clicked on derived identity.** `soma/id.rs` threads an `IdSeed` through values,
so identity is computed rather than allocated — no entropy, no I/O, `no_std`. A graph
can be built offline and rebuilt byte-identical. Infinite Solutions mints by scanning
the store for the highest sibling and adding one (§2.1).

**biomimicry clicked on the program being a value.** `transduction/spec.rs` is explicit
— *"no closures, no trait objects"* — so a computation is `PartialEq`, serialisable and
content-hashable, which is what makes the genotype, the seven-pass linker, version
pinning and exact equivalence all possible. Infinite Solutions is closest here;
`CompositionRecord` is already data. What is missing is §5's discipline about *what
belongs in it*.

So: **bion's mechanism is E15, Innovator's is E16 and E18, biomimicry's is §5.**

---

## 2 · Why the current code must be refactored, not extended

### 2.1 · Address minting is not fit for authoring — **the hard blocker**

`editor/mint.rs::next_child` scans `SCREEN_ROOT_KEY..SCREEN_END_KEY`, filters to rows
at the child bit-depth sharing the parent's prefix, takes the maximum child nibble, and
adds one. Four consequences:

1. **O(screen) per mint.**
2. **Racy** — two sessions pick the same nibble. O28 named this; E13.4 shipped without
   answering it.
3. **Fifteen children maximum, per space, forever.** A list of twenty rows is not
   expressible. **This alone stops E20.**
4. **Addresses are recycled after deletion** (finding 25), and D49's undo restores by
   address.

**The root cause, which draft 1 missed.** The ceiling is not the store's and not really
the key width either.

*It is not the store.* `infinitedb_index::curve_address` is `CurveAddress(u128)` —
`assert!(used <= 128, "dims * bits_per_dim must be <= 128")` — and `SpaceConfig`
carries `bits_per_dim: u32` with a `with_bits_per_dim` builder. The store offers 128
bits and lets a space choose how to spend them. The editor spends 32.

*It is not the width.* Widening to 128 buys **depth** 31 instead of 7 and moves breadth
not at all, because breadth is set by the per-level slice.

*It is the uniform nibble, and that exists for one reason.* `facade::significant_bits`
must answer *how deep is this address* from the bytes alone:

> Four bits per level, and no level's nibble is zero… So the significant length is four
> times the position of the last non-zero nibble.

That trick works only if every level is exactly one nibble and none is zero. **The
fifteen-child ceiling is the price of making depth inferable from a fixed-width key,
and it buys nothing else.**

The irony is sharp: D45 added a `bits` field to `Addr` precisely so an address would
*carry* its significant length rather than have it guessed — that was the fix for
finding 19 — and then `presenter_addr` populates that carried field by calling
`significant_bits(bytes)`, i.e. by guessing it from a fixed-width key.

**D45 rejected the uncapped option for a reason it removed itself.** Its alternative
(a) was variable-length byte addresses, rejected as *"…with no change to the descend
rule. A nesting level would cost eight bits… entering a space would take 256×
magnification."* That objection is entirely about the **old** descend rule. D45's other
half replaced it: descent is now apparent size against `View::opening_extent`. So
**(a) + the new descend rule** has no zoom problem and no cap in either direction — and
that pairing was never evaluated, because the rejection was written against (a) without
(2) while (2) was being adopted in the same decision.

This matters beyond a bug. `CHARTER.md`'s mechanic is *we subdivide spaces*; the
store's doctrine is **permanence by divisibility**, and `infinitedb-spatial-layer.md`
§2 costs a path as *Σ dᵢℓᵢ bits* — per-level ℓᵢ, expected to vary. A fixed four-bit
level says a space may be divided exactly fifteen ways and never again. That is not
subdivision; it is enumeration with a small enum, which is F-1's shape in the address
scheme.

### 2.2 · There is no authoring vocabulary, so every node costs a constant

`addresses.rs` is 12.7 KB and holds 83 `pub const … _KEY`. D34 calls this file the
bootstrap ABI — the addresses findable in an empty store. About six qualify. The other
~77 are app content — `BEHAVIOUR_SELECT_GATE_KEY`, `BEHAVIOUR_PLACE_SET_ORIGIN_KEY`,
`APP_INCREMENT_KEY` — nodes in an authored graph given literal addresses because there
is no way to author a graph without naming every node in Rust.

D34 is not wrong; it has been **overloaded**. One mechanism carries two jobs —
*findable in an empty store* and *I need a name for this node* — and the second is
unbounded (finding 27).

### 2.3 · `SpaceRecord` adopted the alternative D46 rejected

D46 rejected a parallel list in these words:

> F-1 wearing a different hat: **a field per primitive closes an open set just as firmly
> as a variant per primitive**, and a third primitive would need a third field.

`SpaceRecord` then grew `primitive`, `link`, and `text`. All nineteen genesis literals
spell out `primitive: String::new(), link: None, text: String::new()`. The presenter got
this right (`Batch::primitive` is a `Box<str>`); the record layer did not follow.

### 2.4 · The salvaged hysteresis is dead (finding 20)

`core/detail.rs` is faithful and `tests/hysteresis.rs` passes. Both call sites in
`core/place.rs` — lines 140 and 232 — pass `previous: None`, so the dead band never
applies. This is the defect `PRESENTER.md` §13 finding 7 condemned in `hyper-ui`.

### 2.5 · The font cannot render the application it is in (finding 23)

Six glyphs — `A B C H I i` — and a hollow-rectangle fallback. No digits.

### 2.6 · The counter's composition was written in Rust (finding 24)

`wire.rs:156` → `app::connect` → `classify()` → installs a pre-written
`increment_graph()`. Not a small cheat: the symptom of §2.2 and §5.3 together. There was
no way to *author* a four-block composition, so it was compiled in.

---

## 3 · Stages

| # | Stage | Status | Verified by | Green check — and what must **stop** passing |
|---|---|---|---|---|
| **E14** | **Text you can read** | landed | `tests/readable_text.rs` | A run containing every digit and mixed case renders with **distinct ink per character**: two different characters produce different ink, and no character resolves to the fallback box. *Must stop passing:* the current font, where `1` and `7` are identical |
| **E14.1** | Hysteresis reaches the running path | landed | `crates/infinite-presenter/tests/hysteresis_live.rs` | Place twice across a level boundary through a real `View`; the level does not change twice inside one dead band. *Must stop passing:* `detail(…, None)` at both `place.rs` call sites |
| **E15** | **Derived identity** | not started | — | (a) A space holds **200 children**. (b) Two sessions mint concurrently, no collision. (c) The same authored screen produces byte-identical addresses on two machines. (d) Delete a child, mint again, undo the delete — the restored value lands on its own address. *Must stop passing:* `next_child`'s store scan, and `significant_bits`' inference (§2.1) |
| **E16** | **The authoring vocabulary** | not started | — | `genesis.rs` under 150 lines while seeding **strictly more** spaces than today; E4's discard test unchanged; the committed store carries no containment field, only addresses. *Must stop passing:* 19 hand-written `SpaceRecord` literals |
| **E17** | **The record is open** | not started | — | A fourth shape touches **no** existing record field and **no** existing decoder branch. *Must stop passing:* `link` and `text` as `SpaceRecord` fields |
| **E18a** | **Declare the alphabet** | not started | — | The set is written down with each element's two-domain justification (R32) and registered by string key. Both bounding rules of §5.5 run in `check-rules.sh`. *Must stop passing:* the current native registry, where §5.3's six one-offs each fail rule 1 |
| **E18b** | **Build the components from it** | not started | — | Two different screens share one `field_row` **definition stored in the store**; editing the definition changes both **with no recompile**. And: **at least one block in the system has `kind != "native"`**. *Must stop passing:* today's count of composed blocks, which is zero |
| **E19** | **Focus and keystrokes** | not started | — | Type into a field row; the value commits through the interpreted composition; `Ctrl+Z` restores it. *Must stop passing:* E13.3's property write, which has no focus model |
| **E20** | **O29 — the Innovator screen** | not started | — | `panel` + `section_header` + two `field_row`s + `action_bar`, authored, commit routed by role, **built with zero primitives added to E18a's set**. Then delete the Rust builders and re-seed from the store: it still renders |

**E20 is the deliverable.** This document repeats E13's warning about itself: a plan
that lands E14–E19 and stops has not answered the question.

### 3.1 · Why this order

**E14 first** because every later green check is something a person looks at, and it is
independent — a font behind the `Glyphs` port in the facade, which is already the right
seam (D29 keeps graphics out of the presenter). Innovator proves `glyphon` +
`cosmic-text` works against `wgpu`.

**E15 second** because of the fifteen-child ceiling. It is the only hard blocker: E16,
E18b and E20 all build trees wider than fifteen.

**E16 before E17** because after E16 there are ~19 builder call sites instead of 19 × 10
literal fields, which makes E17 a small diff instead of a scatter.

**E18a before E18b** because declaring a set and then building from it is a test;
building and then naming what you used is a description. §5.5's budget only means
something if the set is fixed first.

**E19 before E20** because `field_row` without focus is decoration.

---

## 4 · The refactors, each with what forced it

| # | Refactor | Forced by | Cost, stated |
|---|---|---|---|
| **R-A** | `addresses.rs` splits: bootstrap ABI (~6, D34 keeps its meaning) vs. derived content addresses | §2.2 — 83 constants, ~77 content | The ABI shrinks and every non-ABI literal changes. No migration machinery (D34's cost, a fourth time) |
| **R-B** | Minting becomes derivation, and depth stops being inferred from a fixed-width key | §2.1, all four consequences plus the root cause | The largest change here. Re-opens D45's key layout and needs its own decision record (O32) |
| **R-C** | `genesis.rs` rewritten as builders over E16's vocabulary | §2.2 — 675 lines for 19 spaces | One large mechanical diff; the discard test is the safety net |
| **R-D** | `SpaceRecord`'s `link` and `text` move out — per-shape payload addressed under the space (O26 option **b**) | §2.3 — D46's rejected alternative was adopted | One more indirection and one more thing that can dangle, as O26 predicted. O26 closes |
| **R-E** | `app::connect` / `classify` / `increment_graph` **deleted**; the counter becomes authored | §2.6, finding 24 | `tests/counter.rs` rewritten to author by pointer. It should get harder to pass |
| **R-F** | `Glyphs` gets a real font behind the port | §2.5, finding 23 | A facade dependency; `check-rules.sh`'s presenter manifest grep must keep passing — that list is the presenter's, not the facade's |
| **R-G** | `detail`'s `previous` threaded from the prior placement | §2.4, finding 20 | Small; the placement already knows the level it last drew each address at |
| **R-H** | The six one-off native blocks become registered pure-function kinds or compositions | §5.3 | Their addresses and constants disappear, which is most of R-A's win |

### 4.1 · Refactors considered and rejected — for now

| Rejected | Why not now | Trigger |
|---|---|---|
| **Split `facade::Store`** — `open.rs` 16.5 KB, `present.rs` 14.8 KB, ~25 methods | Not blocking anything here, and R30 makes the facade the compatibility surface | A second app besides the editor |
| **Structural tag compatibility** (biomimicry's `ValueShape`) | D13 says the only operation on a tag is *match*; equality is a defensible reading | The first pair of ports that should connect and does not |
| **Version pinning + `OrganismGenotype`** | Real gap, no consumer — one author, one composition | A second author, or a cache key for D28 tier 1 |
| **Constructor-validated wiring** (bion's `ValidSynapse`) | D35 chose link-time findings; D21 says a bad edge is judged, not refused. Right for a novice editor | A wiring error that survives into execution |
| **Promote `Addr` to its own crate** (O13) | Trigger re-armed by D45, unchanged | A second layer needing the significant length |
| **The parallel scheduler** | `PARALLELISM.md` §16: do not build it. Only door 2 is time-sensitive | The crane mat |

---

## 5 · The alphabet, and how to know it is closed

### 5.1 · The structural point, not the metaphor

DNA authors everything from a very small set. **Take the structure; leave the naming.**
R15 forbids metaphor names in the core and F-4 counts two codebases lost to it — bion
and biomimicry each ended up maintaining a `VOCABULARY.md` whose only job was policing a
metaphor's border. No `base`, no `codon`, no `gene`. The candidate alphabet for this
project is [`../vocabulary.md`](../vocabulary.md).

Two structural properties are worth more than the number four:

**There are two alphabets, not one.** Four bases encode; twenty amino acids function; a
fixed table maps between them, and one machine reads any codon — the variety lives in
the *data*, not in the machine. §5.4 is that split applied here.

**The same alphabet encodes the thing and its regulation.** A gene and its promoter are
one substrate. That is D27 verbatim — *one linker; the editor is an interface graph that
edits compute graphs* — and the charter's thesis that frontend, backend and persistence
are one substrate. So the admission criterion for an element is not *is the set small*
but **does one alphabet serve both what is built and what builds it.**

### 5.2 · A small population in an open registry is not a closed enum

R16 forbids a closed set standing for an open one. It does not forbid a *small
population*. The target is a small population, an open registry, and a rule that holds
it there — the number is not the design, §5.5 is.

Innovator stated this exactly, in `pgraph/data.rs`: `ParticleData` is an enum for
convenience, but `particle_kind` is written to the graph as an open text prop, and
*"that is the surface the rest of the system queries, serialises, and extends."*

### 5.3 · The registry already exists and has become a junk drawer

Eleven native blocks: `read`, `amend`, `commit`, `gate`, `probe_at`, `offset`,
`displace`, `set_origin`, `encode_selection`, `encode_wire`, `increment_text`. About
five are primitives; six were written for exactly one feature.

`increment_text` is read → parse → add one → format → write. It costs a literal
address, a port declaration and a constant in `addresses.rs` **because there was no way
to chain three pure steps.** Every such block is one row of §2.2's 83.

### 5.4 · Three orthogonal sets — conflating them is what filled the drawer

**Shape** — how a space draws; the existing `primitive` key: `area`, `text`, `link`.
Possibly all, because a button is an area that `accepts` and a panel is a space with
children, exactly as E13 §2 already requires.

**Arrangement** — **not a primitive.** A property of the parent (across / down /
absolute) over the `Extent`s already on the record. `arrange()` exists and is called
with a normalised axis; nothing authors an axis. Innovator made `Stack` a kind; a
primitive that draws nothing is a smell. Recorded as **O35** rather than decided.

**Behavior** — what a block computes: `read`, `write`, `map`, `fold`, `gate`, `probe`.

The load-bearing move is between the last two and the drawer:

> **Effects are blocks. Pure functions are data.**

A pure function needs no address, no ports and no provenance edge, so it must not be a
block. `map` is one block kind dispatching on a registered function key — one machine
reading any codon. `increment_text` stops being a node and becomes a table entry.
biomimicry proved the shape: `ArithOp`, `CmpOp`, `MapSpec`, `FoldSpec` are `PartialEq`,
serialisable, hashable data with *"no closures, no trait objects"*, and
`TransductionFn::call` is the single interpreter.

`fold` is also where `PARALLELISM.md` §6.3's **declared combination order** lives — the
requirement that a non-commutative fan-in name its order or raise a finding. Under this
split it has a home instead of a retrofit.

*The obvious objection:* does a registry of map kinds just relocate the junk drawer?
No, and the difference is checkable. A map kind is a pure function of a value —
testable alone, no ports, no store, no address, no constant in `addresses.rs`. A block
is a node with ports and effects. The drawer is expensive because each entry costs
structure; a table entry costs a row. §5.5 rule 1 applies to both regardless.

### 5.5 · How to know the set is right

"Twenty feels right" is not checkable. Two rules bound the set from both sides:

> **Rule 1 — a primitive used by fewer than two components is a one-off.**
> Applied today it flags `increment_text`, `encode_selection`, `encode_wire` and
> `set_origin` immediately.
>
> **Rule 2 — a component that requires a new primitive means the alphabet was wrong**,
> and the primitive it needed names precisely what was missing.

Then a budget rather than a feeling: **declare the set at E18a, build the Innovator
screen and the counter from it, count primitives added.** Two consumers with no shared
purpose is R32 applied to the alphabet instead of to the platform. If the count rises,
that is a fact rather than a judgment — D41-shaped.

### 5.6 · The check that says chaining is real

D14.6 — *composition closes; a wired set of blocks is itself a block* — is locked, has a
closure test (`COMPOSITOR.md` §7.3, `tests/closure.rs`), and **has been used zero
times.** `record.rs:58` supports it — *"Native key bytes, or the delegated address"* —
and `kind: "native"` is the only kind that appears anywhere in `src/` (finding 29).

So the alphabet question and the chaining question are one question: **a small alphabet
is only possible if composition closes in practice.** Otherwise every capability must be
native, and eleven blocks for two rectangles is the run rate.

E18b's check is therefore stated two ways deliberately — *edit a stored definition and
both screens change*, and *at least one block has `kind != "native"`*. They are the same
fact, and the second is a number that is currently zero.

---

## 6 · How this plan could fail the same way E13 did

E13 followed every rule and produced six glyphs and a hardcoded graph. The mechanism:
**R27 measures whether a capability is justified by a consumer, and the consumer was
allowed to be whatever the current stage needed.** Four guards.

**Guard 1 — the consumer is E20, named up front, and it does not move.** Every stage
below is justified by E20, not by its own test. If a check can be met without advancing
E20, the check is wrong.

**Guard 2 — every green check names what must stop passing.** That column is not
decoration. D41 requires a check that can fail *for the reason the claim would be
false*; naming the currently-passing thing that must break is the only way to know it
has that property before writing it. E13.0's check was true, verified, and compatible
with an unreadable font because nothing had to stop passing.

**Guard 3 — E18 is where a widget toolkit gets in.** E13 §2 predicted it and the grep
could not see it (finding 22 — anchored, so `increment_text.rs` passes). The structural
guard is stronger than a name check: **a component is a stored definition that
delegation points at (D27), not a Rust function.** E18b's check is not satisfiable by a
builder.

**Guard 4 — the alphabet is declared before it is used, and counted after.** §5.5. A
set named after the fact describes whatever was built.

---

## 7 · Findings

Continuing `SALVAGE.md`, which reached 24.

**25 · `mint::next_child` recycles addresses, and undo restores by address.**
Minting is `max(child nibble) + 1`, so deleting the highest child frees its address for
reuse, and D49's `CommitEntry` restores `previous` **at an address**. An undo crossing a
delete-then-create can write the old node's value onto the new node. *Not traced end to
end* — the deletion path was not read — so this is a candidate, and E15's check (d)
settles it either way.

**26 · The fifteen-child ceiling is the cost of inferring depth from a fixed-width
key.** *(Rewritten in draft 2; draft 1 recorded only the symptom.)* Breadth is capped at
15 and depth at 7 by one cause: four bits per level with zero reserved. The store is not
the limit — `CurveAddress` is a `u128` and `SpaceConfig::bits_per_dim` is configurable —
and widening the key moves depth only. The uniform nibble exists so
`facade::significant_bits` can recover depth by finding the last non-zero nibble.
D45 gave `Addr` a carried `bits` field to stop depth being guessed, then populated it by
guessing. D45's alternative (a) had no cap and was rejected *"with no change to the
descend rule"* — while D45's own second half replaced that rule, so (a) + the new rule
was never evaluated. See §2.1.

**27 · D34 is carrying two jobs.** *Findable in an empty store* is bounded and correct.
*I need a name for this node* is unbounded, and took the file to 83 constants.

**28 · The native block registry is a junk drawer.** Eleven blocks, ~5 primitives and
~6 one-offs, each added for a single feature because pure steps cannot be chained.
§5.3.

**29 · `composition closes` is locked, tested, and used zero times.** `kind: "native"`
is the only block kind appearing in `src/`. D14.6 is the platform's stated reason
composition scales, `tests/closure.rs` proves the linker supports it, and the editor has
never once delegated. This is the most-cited unused decision in the project.

---

## 8 · Open

| # | Item | Trigger |
|---|---|---|
| **O32** | **What encodes an address's depth?** *(widened in draft 2 — draft 1 asked only what derives an address.)* Candidates: variable-length keys where length is structural (D45(a) + the new descend rule); per-level bit allocation, which is the store's own *Σ dᵢℓᵢ* model; a carried length that is never re-inferred. The choice decides breadth, depth, whether two machines agree, and whether `max+1` recycling can exist at all. bion's `IdSeed` answers the *derivation* half | E15. Needs its own decision record; this is R-B |
| **O33** | **Is a component a definition or a delegation?** Innovator used an `instance_of` edge to a `component_def` node. D27 says use is delegation and `Instance` is not a primitive. They may be one thing said twice, or D27 may not cover appearance | E18a |
| **O34** | **Does the authoring vocabulary live in the editor or the facade?** A vocabulary that mints addresses and encodes records touches both. R2 forbids the facade naming the editor; if it is platform, it is the first thing above the four layers that is not the editor | E16 |
| **O35** | **Is arrangement a shape kind or a property of the parent?** §5.4. Innovator made `Stack` a particle kind; the argument against is that a primitive which draws nothing is a smell, and the record already carries the extents. Deciding it wrong is cheap to reverse before E18b and expensive after | E18a |
| O26 | Where a text run's string lives | **Answered by R-D as option (b)**, ahead of its stated trigger, because §2.3's argument is structural rather than about run length |
| O28 | Who mints an address | Subsumed by O32 |
| O10 | Ownership and capability | E15 creates addresses; *who owns this space* wants answering at creation, not backfill. `Innovator/src/auth` is still unread |

---

## 9 · What this plan is not

It is not a port of `hyper-ui`. `PRESENTER.md` §13 finding 7's refusals stand: the two
unrelated layout systems, `SceneNode`, the linear scan named `InMemorySpatial`, the
monotonically-growing `Overrides` maps, the unnamed magic constants. None of that comes
over.

What comes over is the **one mechanism from each attempt that made it work**: bion's
derived identity (E15), Innovator's authoring-sugar-to-flat-graph seam (E16) and its
component vocabulary (E18b), and biomimicry's discipline that a computation is data
rather than a function (§5.4).

The four layers are not the thing being fixed. Every refactor above is above them or
beside them. What is being fixed is that there has never been a way to put anything
into them, and — separately, and just as badly — no way to build a second thing out of
the first.
