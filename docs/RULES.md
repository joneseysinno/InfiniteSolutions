# Infinite Solutions — Rules

> **Status:** draft 1, 2026-08-20. Supersedes `Innovator/docs/RULES.md`, which is
> **abandoned — lost, not recoverable from the record** (recorded as O7 in
> `decisions.md`). Nothing in the old document is carried forward except by being
> re-derived here.
>
> This file is short on purpose. It is meant to be read at the start of a working
> session, not consulted when something goes wrong.

**How to use it.** Every rule states the constraint, what it is doing there
(always evidence from `infinite-db`, `bion`, `biomimicry`, or `Innovator` — no
rule is here on taste alone), and how a violation is detected. A rule that
cannot be checked is a preference and belongs in prose, not in this file.

**How to change it.** Rules are amended by a numbered entry in `decisions.md`
stating the rule, the change, and what forced it. Rules are never silently
edited and never deleted — a retired rule keeps its number and gets a terminal
status, the same convention `HISTORY.md` prescribes for plans.

---

## 1 · Layers

**R1 — Infinite Solutions is a platform, not an application.**
Four layers — **store, compositor, runtime, presenter** — behind one facade. A fifth layer requires a decision record before it gets a directory.
*Why:* SES, Coach Assistant and the structural work are all planned consumers, and none of them *is* Infinite Solutions. Three previous attempts each grew an unnamed extra layer in place — `bion` and `biomimicry` each grew a store, `Innovator` grew a runtime (`hypernode` + `app_shell`).
*Checked by:* the crate list. Anything not mapping to one of the four is a finding.

**R2 — Dependency direction is fixed and one-way.**
`layer → platform facade → domain facade → app`. No layer depends on a facade; no facade depends on an app; no app depends on a layer directly.
*Why:* `analysis-substrate-plan.md` §0.11 and F10 — a computation must be drivable from hand-built inputs with no graph, or it stops being testable against a textbook.
*Checked by:* the manifests. A dependency edge against the arrow is a violation.

**R3 — Every layer has a pure core that depends on nothing, and a binding that knows the graph.**
Only the binding knows a graph exists.
*Why:* `bion`'s soma purity, the `mycelium` rule, and `hyper-ui`'s 57 layout tests, which must run with no runtime present.
*Checked by:* building each pure core in isolation, the way `bion` verifies soma with `--no-default-features`.

**R4 — Layers reach each other through string-keyed registries, never compile-time names.**
`BoundaryRegistry`, `SolverRegistry`, `CheckRegistry` — one shape, keyed from a graph prop.
*Why:* it is what lets R2 hold while the runtime still dispatches to a solver it cannot name.
*Checked by:* a cross-layer call by concrete type is a violation.

---

## 2 · The one law

**R5 — Derived state never writes back into the definition it derives from.**
*Why:* this is the single most-repeated invariant in the corpus, written four times in four vocabularies: `infinite-db`'s two-layer doctrine (address vs embedding), `bion`'s Retrovirus Prohibition, `biomimicry`'s DNA/Expression split, and `Innovator`'s F11.
*Checked by:* a test asserting no node in a definition space carries a derived prop. It runs after every stage, not once.

**R6 — Deleting derived state is one delete and leaves no orphan.**
*Why:* the operational form of R5. If deletion is a scatter across definition spaces, R5 was already violated somewhere.
*Checked by:* delete the analysis space; assert model spaces are byte-identical and nothing dangles.

**R7 — The store is the only writable model.**
No mutable in-memory graph. Application code never writes to a node structure outside the store.
*Why:* D6. Every prior attempt built a projection and every one accreted fields the store did not have — `Innovator` reached roughly thirty `HashMap<ParticleId, …>` bridge maps standing in for edges.
*Checked by:* R11.

---

## 3 · Runtime

**R8 — The runtime owns no thread pool.**
It is driven; it does not drive. Cadence in, work out.
*Why:* `infinite-db`'s `D-BLOCKING-CONTRACT` and `bion`'s headless argument. The crate's own name invites the opposite, which is why this is written down.
*Checked by:* no executor or async-runtime dependency in the manifest.

**R9 — The runtime owns no storage.**
Nothing is authored there. Nothing survives a restart.
*Why:* R1 and R7. It is the sentence that keeps the runtime from becoming a fourth store.
*Checked by:* no persistence dependency; no file handle.

**R10 — Membership test: if it survives a restart it belongs to the store; if it only means anything while something is running it belongs to the runtime.**
Positive form: **the runtime owns time.** The store has revisions and logical clocks but no *now* — no cadence, no frame, no priority, no deadline.
*Why:* everything genuinely runtime-shaped in `bion`'s CNS, `biomimicry`'s metabolism, and `Innovator`'s `app_shell` is about *when*.
*Checked by:* applied per-item at design time; disputes go to `decisions.md`.

**R11 — Outside a declared derived artifact, the runtime holds addresses, never records.**
A `NodeId`, a key range, a revision, a watermark: yes. A node's props: no.
*Why:* it makes R7 mechanically detectable instead of aspirational.
*Checked by:* a lint over the runtime's types, in the manner of `bion`'s CI grep of `src/cns` for DB tokens.

**R12 — Every derived artifact is named, carries a written rebuild rule, and passes the discard test.**
Discard test: drop it at any instant, rebuild from the store, get a bit-identical result.
*Why:* an unnamed cache is R7's rejected projection arriving through the side door.
*Checked by:* the artifact list. An artifact not on it is a violation; an artifact on it that fails the discard test is a bug.

**R13 — Pending state is bounded, enumerable, and has an explicit commit boundary.**
Pending is the third state category — not stored, not derivable, and *not* discardable (half-typed values, a drag in progress). It is the only non-discardable thing the runtime holds.
*Why:* without naming it, uncommitted edits get parked in a derived artifact, that artifact stops passing R12, and R7 is violated silently.
*Checked by:* you can always list everything pending. It may be journaled to the session WAL so a crash does not lose a half-finished calculation.

**R14 — Store backpressure never reaches the input path.**
*Why:* the write queue blocks when full. If a keystroke is a write, a full queue stalls typing.
*Checked by:* a test that saturates the queue and asserts input still responds.

---

## 4 · Naming

**R15 — No metaphor names in the core.**
No biology, no physics, no anatomy. Domain crates may use domain words; the core may not.
*Why:* `bion` and `biomimicry` each ended up maintaining a `VOCABULARY.md` whose only job was policing a metaphor's border. A metaphor is a good intuition pump and a bad type system — when a concept arrives that it lacks (a weld, a page tab), you either strain the name or break the metaphor.
*Checked by:* review. The prefix `infinite-` plus a plain functional noun is the pattern.

**R16 — A closed enum is a defect wherever the set is open.**
Kind is a prop; dispatch is a registry.
*Why:* named as the same mistake five times in the plans — `kind_id`, `WorkspaceInstance`, `IoKind`, `AppSignal`, and the `ParticleKind` that was proposed and rejected.
*Checked by:* a new enum in a core crate requires a decision record justifying that the set is genuinely closed.

**R17 — One name, one thing. A name is never reused for a second structure.**
*Why:* three different data structures were called `PageTree`, and two surviving plans reason about "`PageTree`" as though it named one stable thing.
*Checked by:* a name that is removed is retired, not recycled.

---

## 5 · Working

**R18 — Theory before code. A layer gets a specification document before it gets a crate.**
*Why:* the only plan in the corpus that completed all its stages is the one that specified the consumer before the substrate existed, deliberately inverting the usual order.
*Checked by:* an empty directory with no spec is a finding.

**R19 — No layer is built without a forcing consumer named in its spec.**
The consumer is something that breaks if the layer is wrong.
*Why:* `engineering-substrate-plan.md`: *"Designing the substrate against the one consumer that already fits it is precisely how the current layer was produced; this inverts the order deliberately."*
*Checked by:* the spec names it, or the spec is incomplete.

**R20 — A phase's status is updated in the change that lands the phase.**
Not at authoring time, and not in a later audit.
*Why:* `HISTORY.md` traces every recorded drift to status headers stamped once at authoring time and never revisited.
*Checked by:* a status line written before the work is a prediction, and predictions in this codebase have not held.

**R21 — A document is never deleted. It gets a terminal status and stays.**
`superseded by X`, `abandoned — reason`, `complete`.
*Why:* six plans were deleted in one commit with no terminal status, costing the provenance of ten commits and two lessons still cited by surviving documents.
*Checked by:* the document set only grows.

**R22 — A decision is recorded when it lands, with its rejected alternative.**
*Why:* a decision without its rejected alternative gets re-litigated, and the reasoning that killed the alternative is exactly what gets lost.
*Checked by:* `decisions.md` entries carry both.

**R23 — A claim about rendering or interaction states its verification method.**
*Why:* `HISTORY.md`'s own closing observation — the graph model is well covered, rendering and pointer interaction are covered by nothing, and that is exactly where the longest-lived drifts survived unnoticed.
*Checked by:* the claim names how it was verified, or it is marked unverified.

---

**R30 — The facade is the compatibility surface.**
Internals may churn freely; the facade may not. It is the only thing a domain facade or an app depends on.
*Why:* it is what lets the runtime keep being refactored after an app exists — the absence of this is why every prior attempt had to restart rather than evolve.
*Checked by:* a facade change requires a decision record; a layer change does not.

**R31 — The compositor contains no math.**
It owns the *contract and lifecycle* of a computation: declaration of inputs and outputs, the registry mechanism, provenance, the staleness contract, execution shape, and explanation. Every numerical method and every domain law lives in a facade above it.
*Why:* the error that first produced `infinite-physics` as a platform crate. Navier-Stokes, R-functions, quadrature, Nitsche, and ACI clauses are all instances of the contract, never part of it.
*Checked by:* a numerical or domain dependency in the compositor is a violation.

**R32 — The two-domain test: a concept is platform only if both planned consumers need it.**
Fluid and solid mechanics on one side, basketball coaching on the other. They share no domain vocabulary, so anything serving both is structural and anything serving one is a facade concern.
*Why:* every previous attempt was designed against a single domain and generalized afterward, which is the drift mechanism itself. Two consumers with zero overlap converts R19 from a judgment call into a procedure.
*Checked by:* before admitting anything to a platform layer, ask whether Coach Assistant needs it. If not, it belongs in a facade.

---

## 6 · Working with an assistant

*This section exists because all three previous attempts drifted during AI-assisted work, not during solo work.*

**R24 — A session opens by reading this file and `decisions.md`.**
An assistant that has not read them is not working on this project yet.

**R25 — Every request names its layer.**
"Build X" without a layer is under-specified and is sent back.

**R26 — Every request names the consumer that breaks if it is wrong.**
This is R19 applied at the level of a single task.

**R27 — Generality is a defect unless a named consumer requires it.**
*Why:* the pattern is self-identified — staying very general with an assistant because the idea is not yet in focus, and then accepting whatever generality comes back.

**R28 — When the idea is fuzzy, the deliverable is a specification, not code.**
Fuzzy plus code is the drift mechanism. Fuzzy plus a written spec is how it comes into focus.

**R29 — A proposal that adds an enum, a second graph, or a metaphor name is corrected, not merged.**
These are the three recurrences with the highest historical rate. They arrive as reasonable-sounding suggestions.

---

## 7 · Forbidden

The specific recurrences, with how many times each has already happened. This table is the fastest way to recognize a mistake while it is still a suggestion.

| # | Forbidden | Times |
|---|---|---|
| F-1 | A closed enum standing for an open set | 5 |
| F-2 | A map keyed by id standing in for an edge | ~30 instances, 1 codebase |
| F-3 | A second in-memory graph beside the store | 3 |
| F-4 | A metaphor as a core type name | 2 codebases |
| F-5 | A plan written after the code it describes | every plan in `Innovator/plans` |
| F-6 | A layer built with no consumer | 3 |
| F-7 | A cache that is written to | the mechanism behind F-2 |
| F-8 | A `mod.rs` | convention held in `bion` and `biomimicry`: `module.rs` plus a leaf file per concern, one public function per file |
| F-9 | Deleting a document instead of giving it a terminal status | 6 documents, 1 commit |

---

## 8 · Conventions

Not rules — house style, recorded so it does not have to be re-decided.

- `module.rs` plus a directory of leaf files; no `mod.rs`. A module file holds only docs, `mod` declarations, and re-exports.
- One public function per file in the layers that carry the convention.
- Decision records are numbered, state the choice, what forced it, and what it costs — the `infinite-db/SEMANTICS.md` format.
- Plans carry a stage table at the top with a per-stage status, and each stage carries a **green check**: the specific observable result that says the stage is done.
