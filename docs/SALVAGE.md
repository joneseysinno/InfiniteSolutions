# Infinite Solutions — Salvage Ledger

> **Status:** draft 1, 2026-08-29. **Records nothing.** Every trigger below is a
> *candidate* written by an assistant, and R29 says a proposal is corrected, not
> merged. Nothing here is locked until a change lands it and stamps it (R20, R22).
> Same standing as [`PARALLELISM.md`](./PARALLELISM.md).
>
> Rules: [`RULES.md`](./RULES.md) · Decisions: [`DECISIONS.md`](./DECISIONS.md) ·
> Charter: [`CHARTER.md`](./CHARTER.md) · Status: [`STATUS.md`](./STATUS.md)
>
> Prior attempts audited on disk at `D:\Rust\Innovator`, `D:\Rust\bion`,
> `D:\Rust\biomimicry`. Opens O29–O31. Raises findings 20–24.

---

## 0 · Why this document exists

**Every architectural deferral in this project carries a trigger. No capability
deferral does.**

O1 names a measurement. O13 names the condition that reopens it, and when that
condition fired (D45) the re-arming was written down rather than assumed. O14, O26,
O27, O28 all name what makes them live again. That discipline is the reason this
project can defer something without losing it.

The capability discarded from three prior attempts got the opposite treatment. D6
records what `hypernode` lost — *"it is split, and half of it dies"*. D9 records the
primitive set deliberately not carried forward. `PRESENTER.md` §13 finding 7 lists
what was taken from `hyper-ui` and what was not, by name. All three are honest,
reasoned, and correct as decisions. **None of them names a condition under which the
discarded thing comes back.** So the architecture's deferrals are tracked and the
product's are not, and thirteen stages later the running editor draws rectangles
while `D:\Rust\Innovator` — 505 KB of Rust, 121 files — draws pages, pods, dividers,
scrolling viewports and shaped text.

That asymmetry is the finding. This document is the correction: one row per
capability, in the same format the project already uses, with a trigger.

**It is not an argument to port anything.** R27 still holds — generality without a
named consumer is a defect, and most of what follows should stay out until something
needs it. The point is that *"not yet, because X"* and *"gone, and nobody noticed"*
look identical from inside a stage table, and only the first one is a decision.

---

## 1 · How to read a row

| Column | Means |
|---|---|
| **Capability** | What the prior attempt could do |
| **Where** | The file, so the row can be checked rather than believed |
| **Disposition** | `taken` · `re-derived` · `superseded` · `not taken` |
| **Trigger** | For `not taken`: what makes it live. Candidate only |

**`re-derived`** is worth its own word and is the most common honest outcome: the
prior attempt and this one arrived at the same answer independently, and the record
credits neither. That is not waste — converging twice from different directions is
the strongest evidence a design is right — but it is worth knowing it happened,
because the second derivation cost time the first had already paid.

---

## 2 · Innovator — taken

`D:\Rust\Innovator`, 121 source files, 505 KB. The only prior attempt with a real
user interface.

| Capability | Where | Into | How faithful |
|---|---|---|---|
| `Extent { min, ideal, weight }` | `crates/hyper-ui/src/container/extent.rs` | `infinite-presenter/src/core/extent.rs` | **Verified.** Field-for-field, `f32`→`f64` per D29, and the file cites its source in its own doc comment. The one drift is semantic and is finding 21 below |
| Asymmetric hysteresis — promote at `bound + SLOP`, demote at `previous_bound − SLOP` | `crates/hyper-ui/src/layout/viewport.rs`, `SizeClass::from_width_hysteretic` | `infinite-presenter/src/core/detail.rs` | **Algorithm verified**, correctly generalized from width to log-zoom, all state in the arguments. **Not reachable in the running system** — finding 20 |
| Depth-first paint order | `crates/hyper-ui/src/pgraph/draw.rs` | `infinite-presenter/src/core/place.rs` | Taken structurally; falls out of address order (D30) rather than being copied |
| Per-keystroke edit vs. commit (`FieldEditing` / `FieldCommit`) | `src/workspace/app_shell/` | D24's pending set and commit boundary | Generalized from fields to any gesture. Cited in D24 |
| Tier ladder root → workspace → page → pod → component → particle | `src/workspace/graph_containers.rs`, `derive_tiers` | D20's zoom ladder | Recognized as a zoom ladder built without being named one. Cited in D20 |
| Capability / role / session | `src/auth/` | O10, noted and deferred | Not built. O10 keeps three doors open for it |

### 2.1 · Re-derived from Innovator without the record noticing

These are cases where this project reached Innovator's answer independently. Worth
recording because each one is evidence the answer is right, and because a future
session that reads only `DECISIONS.md` will think it was invented here.

| This project | Innovator already had it |
|---|---|
| **D46** — a primitive is an opaque `Box<str>`, not an enum, because block authors publish new ones | `pgraph/data.rs` holds `ParticleData` as an enum **and says why that is not the contract**: `particle_kind` is written to the graph as an open `PropValue::Text` prop, and *"that is the surface the rest of the system queries, serialises, and extends."* Same answer, same reasoning, three months earlier |
| **D14.1** — port declaration: name, direction, tag | `ParticleData::default_valence() -> (&[&str], &[&str])` — emit and absorb lists per particle kind, with `VALENCE_EMIT_PROP` / `VALENCE_ABSORB_PROP` on the node |
| **D30 L5** — refer to the store's address, never mint identity | `hypernode::binding` — containment is a `Binding` **edge** carrying an `order` prop, not a child vector and not a map. F-2 avoided, in the codebase D6 cites as having ~30 instances of F-2 elsewhere |
| **D27** — one linker; the compute/interface distinction is a tag convention | `binding.rs` deliberately excludes role-tagged `Binding` edges from traversal, so one edge type carries both containment and relation, distinguished by a prop |

**The strongest item Innovator has that this project has not re-derived** is in
`pgraph/store.rs`:

> `Spec` is *authoring sugar only* — a transient nested value that `commit` flattens
> into nodes and edges. **Nesting at authoring time does not make the runtime a tree.**

That sentence solves a problem E13.4 is going to hit: a palette drop is authored as a
nested thing and must land as flat addressed records. Innovator built the seam and
named it. Nothing in `DECISIONS.md` corresponds.

---

## 3 · Innovator — not taken

| Capability | Where | Candidate trigger |
|---|---|---|
| **Shaped text** — `glyphon` + `cosmic-text`, content-keyed buffer cache, `measure(text, size, weight) -> Vec2` | `crates/hyper-ui/src/text/` (13 files) | **Fired.** E13.0 shipped a 6-glyph 5×7 bitmap with a hollow-box fallback and no digits — finding 23. The trigger is any label a person is expected to read |
| **Page tree** — split, merge, leaf rects, hidden/shown leaves, template ids | `crates/hyper-ui/src/page/` (16 files, incl. `layout_tests.rs`) | A second view open at once. D20 says tabs *are* held-open spaces, so this is the test of that claim |
| **Pods** — collapse, min height, nav icon, divider drag, icon rail, shell | `crates/hyper-ui/src/pod/` (13 files, incl. 7 KB of layout tests) | Same trigger as the page tree. D20 explicitly says pod-collapse is zoom; nothing has exercised that |
| **Seams** — draggable dividers with cursor icons and rebuild | `crates/hyper-ui/src/seam/` (11 files) | A person resizing anything |
| **Component catalogue** — 12 named components (`field_row`, `list_row`, `status_badge`, `filter_bar`, `table`, `scroll_host`, `panel`…) over 9 particle kinds | `src/components/catalogue.rs` | **E13's actual target.** §6 below |
| **Definition / instance via `instance_of` edge**, with `ComponentKind::ALL` seeded as `component_def` nodes a future editor edits | `src/components/definitions.rs` | Superseded in principle by D27 (*use is delegation*), but D27's delegation has never been used for appearance. Trigger: the second block that wants the same authored look |
| **Two-pass measure/arrange over graph edges** | `crates/hyper-ui/src/pgraph/layout.rs`, 15 KB | Anything whose size depends on its children. `arrange` here is the 1-D distribute step only |
| **Visibility cascade + budget + `ResolveReport`** | `crates/hyper-ui/src/layout/resolve/` (5 files, 9.6 KB of tests) | A space that must shrink rather than clip. `Extent::min` currently has no consumer — finding 21 |
| **Focus and scrolling viewport** | `container/focus.rs`, `particles/field/` | Keyboard input. E13.3 writes properties and has no focus model |
| **Force-directed graph layout + spatial index** | `src/pages/graph_view/force.rs`, `spatial.rs` | More nodes than can be hand-placed. E13.4 mints addresses and places by pointer only |
| **PDF report generation** | `src/documents/report_pdf.rs`, `write_pdf.rs` | An app that must emit a document |
| **Domain analysis** — retaining wall, PM, engineering library, templates | `src/analysis/` (~120 KB) | SES. Deferred by D12; listed so it is not re-derived by accident |

---

## 4 · bion — taken and not taken

`D:\Rust\bion`, 112 source files, 180 KB. Three layers: `soma` (pure alphabet),
`pns` (store contact), `cns` (execution).

**Taken.**

| Capability | Where | Into |
|---|---|---|
| Reaching the store only through a trait, enforced by a CI grep | `src/cns` + its DB-token grep | **D23** cites it by name. It is the pattern behind every port in this project, and behind `check-rules.sh` |
| Headless — the engine owns no thread | `cns` design | **L1** (D4). Cited |
| Chain grammar `.node().delegate().connect().seal()` | `hypernode` (shared lineage) | **D6** salvaged it explicitly; **D27** uses `delegate` as the answer to why `Instance` is not a primitive |
| Core/binding split verified with `--no-default-features` | `soma` | **R3**'s check is written as *"the way `bion` verifies soma"* |
| A closed `Value` enum is the wrong shape | `soma/signal.rs` — `BoolValue`, `IntValue`, `FloatValue`, `SignalText`, `ByteBlob`, `UnitValue` | **D13** cites this as the F-1 to avoid. Taken as a *negative* lesson, which counts |

**Not taken.**

| Capability | Where | Candidate trigger |
|---|---|---|
| **Value-threaded identity** — `IdSeed` / `IdSource`, so identity is derived with no entropy and no I/O, `no_std` | `src/soma/id.rs` (10.8 KB) | **O28.** *"Who mints an address for a new block?"* is exactly the question `IdSeed` answers, and E13.4 answered it with a subtree read that two sessions race on. This is the closest thing in the corpus to a solution |
| **Constructor-validated wiring** — `Polarity::can_connect_to`, `ValidSynapse::new(src, sink) -> Option<Self>`: a wrongly-polarised synapse cannot be constructed | `src/soma/polarity.rs` | D35 made `Direction { In, Out }` closed and validates at link time into a finding. That is the right call for a novice-facing editor (D21: judged, not refused). Trigger: a wiring error that survives into execution |
| **Budgeted deterministic propagation** — `Executor`, `ActionWave`, `Pacemaker`, `RefractorySet`, `with_wave_budget` | `src/cns/execution/executor.rs` (16.5 KB) | `RUNTIME.md` §7.1's `tick(now, budget) -> Outcome` is the same shape re-derived. Trigger: the first workload that does not finish in one tick — which is O12, and `PARALLELISM.md` §11 already argues it is a placement question |
| **Hierarchical routing labels** — `RoutingLabel` with `.` separator and length bound | `src/soma/tag.rs` | D13 makes tags opaque and match-only, deliberately. Trigger: a tag namespace collision between two facades |

---

## 5 · biomimicry — taken and not taken

`D:\Rust\biomimicry`, 4 crates, ~450 KB. The most complete of the three as a
*computation* engine.

**Taken.**

| Capability | Where | Into |
|---|---|---|
| Block manifest: declared imports and exports, unsatisfied imports fail at **link** time with a precise error | `src/blocks/` (M12) | **D14**, **D27**, and `COMPOSITOR.md` §6.2's `unsatisfied-import` finding kind, which is named after it |
| Manufactured determinism — execution is exactly, not statistically, reproducible | `src/causality/determinism.rs` | **D19**'s equivalence law depends on it and cites it |
| `settle(n)` returns a **status**, never blocks | `src/organism/settle.rs` | **D21**: *"not a hang but 'ran 50 iterations, residual 0.003, did not converge'"* |
| Two nested scheduler loops at different cadences | `src/metabolism/scheduler.rs` | **D21**'s nesting precedent, cited |
| DNA / Expression split — derived never writes into definition | throughout | **R5**, one of its four vocabularies |
| Damping is needed to stop oscillation, so runtime fixed-point is the wrong default | `src/homeostasis/damping.rs` | **D21** cites it as the cost of the rejected path, *written out in advance* |

**Not taken.** This is the richest column in the ledger, and the reason is worth
stating: biomimicry solved the *linking* problem far past where this project has
taken it, and D14's one-sentence job description — *make it easy for an app to get
wired up* — is precisely that problem.

| Capability | Where | Candidate trigger |
|---|---|---|
| **The seven-pass link pipeline** — qualify → requires → resolve → bridge → relocate → merge → validate | `src/blocks/link.rs` | `infinite-compositor`'s `link` is four steps (`COMPOSITOR.md` §7.2). Trigger: the first composition that needs a name to mean different things in two blocks |
| **Versioned block pinning** — `Pin { name, version }`, TOML manifest in and out, `canonical_bytes` | `src/blocks/manifest.rs` | **This project has no versioning at all.** Trigger: the second author, or the first block whose signature changes after something depends on it |
| **`OrganismGenotype(u128)`** — a content hash of the whole linked composition | `src/blocks/manifest.rs` | D19's equivalence harness compares outputs; a genotype identifies *the program*. Trigger: a compiled-artifact cache key (D28 tier 1) |
| **Qualified port kinds** — `LocalKind` vs `QualifiedKind`, namespaced per block | `src/blocks/port_spec.rs` | D13 keeps tags opaque and flat. Trigger: two facades both defining `length` |
| **Structural tag compatibility** — `ValueShape::matches` and `::compatible`, not equality | `src/signal/value.rs` | D13 says the platform's only operation on a tag is *match*, meaning equality. Trigger: the first pair of ports that should connect and do not |
| **Optional ports** — `PortSpec::optional` beside `required` | `src/blocks/port_spec.rs` | `infinite-compositor`'s `PortRecord` has `required: bool`, so this one **is** present. Recorded as taken-by-coincidence rather than by citation |
| **Infer-when-unambiguous wiring** | `src/blocks/resolve.rs` | **D22 rejected auto-wiring on ambiguity grounds and did not know this answer existed.** biomimicry infers only when exactly one export matches, and reports otherwise. D22's evidence-collection plan is the trigger, and this is the shape the evidence should be measured against |
| **Version-range dependency checking with fatal cycles** | `src/blocks/requires.rs` | Same trigger as pinning |
| **Seeded ordering that never depends on hash iteration** — splitmix64 PRNG, `OrderKey`, `(CausalStamp, SignalId)` with a stable tie-break, feature-gated | `src/causality/determinism.rs` | `COMPOSITOR.md` §9.2 requires deterministic plan order and `order.rs` tie-breaks by address. Adequate today. Trigger: `PARALLELISM.md` G1 — worker-count invariance, where this module is the prior art |
| **Computation as hashable data** — `TransductionSpec` / `TransductionFnSpec`: `ArithOp`, `CmpOp`, `MapSpec`, `FoldSpec`, explicitly *"no closures, no trait objects"* so a computation is `PartialEq`, serialisable and content-hashable | `src/transduction/spec.rs` (13 KB), `function.rs` (16.4 KB) | The nearest existing thing to D28's compiled-form contract, and to a composition that survives a restart as data. Trigger: D28 tier 1 |
| **Deterministic arithmetic discipline** — integer millis only, `i128` intermediates, round half away from zero, saturating, never wrapping, no floats on the core path | `src/signal/value.rs`, `transduction/arith.rs` | D13 correctly puts units and exactness in the facade. But `PARALLELISM.md` §6.3 names float fan-in as the thing that will break determinism, and this is the corpus's answer. Trigger: the first facade doing arithmetic — physics or Coach Assistant |
| **Attractor trajectory** — `settle` returns a status *and* `trajectory() -> &[u128]`, a fingerprint per cycle | `src/organism/settle.rs`, `src/attractor/` | O12 and the crane mat. A non-convergence finding that shows the trajectory is a better error surface than one that shows a residual |

---

## 6 · The capability gap, stated as a green check

`CHARTER.md` says the editor is the ultimate forcing consumer. R19 requires a
consumer that **breaks if the layer is wrong**. Thirteen stages in, the editor does
not break when a layer is thin — it does less, quietly, and the stage still goes
green. A consumer that cannot break is not forcing; it is permitting.

**Candidate O29 — the forcing consumer should be one Innovator screen.**

> Reproduce `components::catalogue::panel` containing a `section_header`, two
> `field_row`s with labels and units, and an `action_bar` — **as authored data in
> InfiniteSolutions**, with the field commit routed through the interpreted
> composition.

It is one screen, it already exists in working software 30 metres away on the same
disk, and it cannot be satisfied by anything currently shipped:

| It demands | Current state |
|---|---|
| Words on screen | 6 glyphs, no digits (finding 23) |
| A container that sizes to its children | `arrange` is 1-D over a normalised axis; no measure pass |
| A label / value / unit row | no component vocabulary |
| A field that takes keystrokes | no focus model |
| A commit routed by role | E13.3 writes a property; roles do not exist |
| Nesting three deep | D45 makes this possible; nothing exercises it past two |

That is R19 satisfied properly, and it is D41 applied to the project rather than to a
stage: **the test that could fail.**

---

## 7 · Findings

Numbered continuing `EDITOR-BOOTSTRAP.md` §9, which reached 19.

**20 · The salvaged hysteresis is not reachable in the running system.**
`core/detail.rs` implements the asymmetric rule faithfully and
`tests/hysteresis.rs` exercises it. Both call sites in `core/place.rs` — lines 140
and 232 — pass `previous: None`, so `detail` always returns the naive level and the
dead band never applies. Zoom across a level boundary in the running editor and it
will chatter, exactly as the rule exists to prevent.

This is the defect `PRESENTER.md` §13 finding 7 identified in `hyper-ui` and
condemned in its own words: *"`InputClass::hit_slop()` exists, is unit-tested, and is
called by nothing but its own test… a tested dead function is worse than an untested
one, because the test says it works."* The project reproduced the exact defect it had
documented. **D41's stage table cannot catch this**, because the green check for S5
is the sweep, and the sweep passes — it supplies `previous` itself.

*Candidate remedy:* the placement already knows the level it last drew each address
at. Threading it back is small. The check that could fail: place twice across a
boundary with a real camera and assert the level did not change twice inside one
dead band.

**21 · `Extent::min` lost its consumer in translation.**
In `hyper-ui` the doc reads *"Hard floor. Below this the container is demoted, never
squeezed"* — `min` was the input to the demotion ladder. D31 replaced the ladder with
a level number, correctly. But `core/extent.rs` now reads *"Below this, the thing is
not worth showing at all"*, and nothing acts on that: `arrange` scales `min` down
proportionally when space is short and never demotes or hides. The field is carried,
documented, and inert.

**22 · The widget-block check is satisfied by naming.**
`check-rules.sh` runs
`! ls src/editor/blocks/ | grep -Eq "^(rectangle|label|panel|widget|button|text)\.rs$"`.
Anchored, so `increment_text.rs` passes. E13's own §2 names a widget toolkit as the
most likely failure and the check cannot see the shape it was written for.

**23 · The bootstrap font has six glyphs and no digits.**
`facade/ports/glyphs.rs` defines `A B C H I i` and space; every other character
falls through to `[0x1F,0x11,0x11,0x11,0x11,0x11,0x1F]`, a hollow rectangle. The
counter's total is therefore always drawn as a box. `tests/text.rs` uses `"Hi"` — two
of the six defined glyphs — so the E13.0 check passes and the claim *text reaches the
screen* is true while *a person can read it* is false.

**24 · E13.7's composition was authored in Rust, not by pointer.**
`editor/wire.rs:156` calls `app::connect`, which `classify()`s the two endpoints on
`style == "bump"` / `style == "total"` and, on a match, installs
`increment_graph()` — a four-block, two-wire `CompositionRecord` built in Rust — at
`APP_ROOT_KEY`. The palette holds two purpose-built templates. What a person authors
is two positions and one wire; the graph they are said to have built was pre-written
and triggered by pattern-match.

`increment-text` as a **native block** is legitimate and is exactly D16's two-tier
model — a block author writes Rust, an app author composes. `increment_graph()` is
not: that is the app author's artifact, and it was supplied. E13.7's claim, *"authored
entirely by pointer"*, does not hold for the load-bearing half.

---

## 8 · Open

| # | Item | Trigger |
|---|---|---|
| **O29** | **Should the forcing consumer be an Innovator screen rather than "the editor"?** §6 | Before E14 is planned. It is a change to what R19 means for this project, so it needs a decision record either way |
| **O30** | **Does a capability deferral need a trigger, the way an architectural one does?** §0 | This document is the argument that it does. If accepted, the rule belongs in `RULES.md` beside R21 — *a document is never deleted* — as its capability twin: **a capability is never dropped, it is deferred with a trigger** |
| **O31** | **Is `IdSeed` the answer to O28?** §4 | E13.4 minted addresses by reading the parent's subtree for the next free index, which two sessions race on. `bion/src/soma/id.rs` derives identity by threading a seed through values, with no entropy and no read |
| O26 | Where a text run's string lives | Unchanged. Note finding 23 makes the question moot until there is a font |
| O10 | Ownership and capability | Unchanged. `Innovator/src/auth` is the prior art and is still unread by this project |

---

## 9 · What this document is not

It is not a plan to port `hyper-ui`. `PRESENTER.md` §13 finding 7 gives good reasons
for most of what it refused — two unrelated layout systems in one crate, `SceneNode`,
`InMemorySpatial` (a linear scan named `Spatial`), the `Overrides` maps that grow
monotonically, and every unnamed magic constant in `pgraph/layout.rs`. Those
judgements stand and this ledger does not reopen them.

The claim is narrower and, I think, harder to argue with: **the architecture went
forwards and the product went backwards, and no instrument in this project measures
the second thing.** Every rule asks whether a capability is justified. None asks
whether a person can do anything. R27 plus a consumer that cannot break is a machine
for producing the minimum that passes the test, and findings 20, 23 and 24 are three
independent outputs of that machine in one stage.

The prior attempts are not sunk cost. They are the acceptance suite this project has
never had, sitting on the same disk, in Rust, with tests.
