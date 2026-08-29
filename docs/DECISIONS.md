# Infinite Solutions — Decision Record

*Format follows `infinite-db/SEMANTICS.md`: each decision states the choice, what forced it, and what it costs. A decision is recorded in the session it lands, never stamped retrospectively (`Innovator/plans/HISTORY.md`, Convention 1).*

Status key: **locked** · **open** · **deferred (with trigger)**

---

## D1 — The store is `infinite-db` · **locked** · 2026-08-20

No re-litigation. It is published (0.4.x, MIT), has ~25 integration test files, and carries the only decision record in the corpus. The spatial-layer doctrine (charts, curve addresses, permanence by divisibility, restriction/aggregation) is adopted as theory, not just as an implementation.

---

## D2 — Four layers, not three · **locked** · 2026-08-20

The scaffold named three (`db` / `physics` / `ux`). The corpus contains four. The missing layer is the **runtime**, and it is where all three previous attempts died — `bion` *is* a runtime, `biomimicry`'s organism/metabolism *is* a runtime, `Innovator`'s `app_shell` + `hypernode` *is* a runtime grown in place because nothing else supplied one.

Built three times, never named. Naming it is most of the fix.

---

## D3 — Dependency shape · **superseded by D10** · 2026-08-20

Originally recorded as a four-crate diamond with `infinite-physics` as the compute layer and `infinite-solutions` as the application. **Both halves were wrong** — physics is not a platform concern, and Infinite Solutions is not an application. Retained for the reasoning that survives:

**A computation must not depend on the runtime.** Forced by `analysis-substrate-plan.md` §0.11 (*"`mycelium` depends on nothing in the workspace… does not know that a graph exists"*) and F10 (*"a solver is drivable from a hand-built `Assembly`, no graph"*). A dependency here makes the solver untestable in isolation and the runtime untestable without a solver — mutual hostages. The runtime never names a computation; it dispatches through a string-keyed registry. Carried into D10 unchanged.

---

## D4 — The runtime is named `infinite-runtime`, with two prohibitions · **locked** · 2026-08-20

The name is kept despite two collisions — in Rust "runtime" connotes *async executor*, and `infinite-db/src/infinitedb_server/runtime.rs` already exists. Renaming for connotation is how metaphor-policing starts again (see D5's history). Instead the risk is answered by law:

> **L1 — The runtime owns no thread pool.** It is driven; it does not drive. Cadence in, work out. (Consistent with `infinite-db`'s `D-BLOCKING-CONTRACT` and `bion`'s headless argument.)
>
> **L2 — The runtime owns no storage.** Nothing is authored there. Nothing survives a restart.

Alternatives considered and set aside: `infinite-tempo`, `infinite-motion`. Reopen only if L1 is observed being violated in practice.

---

## D5 — The membership test: the runtime owns time · **locked** · 2026-08-20

> **If it survives a restart, it belongs to the store. If it only means anything while something is running, it belongs to the runtime.**

Positive form: the store has *logical* time (revisions, HLC) but no **now** — no cadence, no frame, no priority, no deadline. Everything genuinely runtime-shaped in `bion`'s CNS, `biomimicry`'s metabolism, and `Innovator`'s `app_shell` is about *when*. Data structures are the store's job; motion is the runtime's.

Worked cases:

| Thing | Where |
|---|---|
| Node props, edges, solution coefficients | store |
| Queue depth, backpressure state | runtime |
| "This edge's inputs changed at rev N" | store — `check_hyperedge_freshness` already persists it |
| "Therefore recompute now, at this priority" | runtime |
| A solve's active set mid-iteration | runtime |
| Converged solution + its tower level | store |
| `RenderList` | runtime, declared derived artifact |
| Which pod is collapsed | store — authored |
| Which pod has focus | store, session-scoped — session WAL exists and is unused |

---

## D6 — No mutable in-memory graph · **locked** · 2026-08-20

**The store is the only writable model.** Application code never writes to an in-memory node graph.

The runtime holds schedules, subscriptions, the staleness frontier, and a small **named** set of derived artifacts. The defining property of a derived artifact is the **discard test**:

> Drop it at any instant, rebuild from the store, get a bit-identical result.

If that is ever false, something was authored into a cache and the decision has been violated.

**Rejected alternative:** a gateway-guarded live projection (`bion`'s shape — a hydrated `Circuit` populated only through `&dyn PnsReader`). It is the fastest to traverse and it is what all three prior attempts converged on independently. It was rejected on two grounds neither prior attempt weighed:

- **P2P.** Sync would have to reconcile a store *and* a projection — two things that can disagree. Under D6, sync is purely a store concern, and `infinite-db` already ships outbox / merkle / delta / conflict-queue / replicate.
- **External writers.** An MCP server writing definitions must *tell* a projection, or the projection is silently wrong — making the watch loop a correctness dependency. Under D6 the next read simply sees it, and watch is only about responsiveness.

**What this kills.** `hypernode`'s storage and Hilbert indexing (redundant with the store) and `biomimicry-substrate`'s `Store`. `hypernode` is not ported — it is split, and half of it dies. Salvaged: the chain grammar `.node().delegate().connect().seal()` and the mailbox architecture, both of which are runtime material.

**What it costs.** Fine-grained invalidation becomes mandatory, not optional — a keystroke must not dirty a page. The machinery exists (`infinite-db`'s derivation bus, watermarks, staleness closure, `query_stale_downstream`) and has never been wired to a UI.

---

## D7 — Every layer has a pure core and a binding · **locked** · 2026-08-20

> Every layer has a pure core that depends on nothing, and a binding that depends on the runtime. **Only the binding knows a graph exists.**

This is `bion`'s soma purity, the `mycelium` rule, and `hyper-ui`'s situation, generalized. It applies to UX as well: `hyper-ui`'s 57 layout tests (two-pass measure/arrange, `SizeClass` demotion ladder, hysteresis, `Extent`) must still run with no runtime present. Whether the split is a crate boundary or a module boundary is per-layer; the rule is not.

---

## D8 — Three state categories, not two · **locked** · 2026-08-20

D5 and D6 together imply a two-way split (stored / derived) that is incomplete. A third category exists and needs its own rules:

| Category | Definition | Home | Discardable? |
|---|---|---|---|
| **Stored** | authored, or derived and kept | store | no |
| **Derived** | computable from stored; passes the discard test | runtime, declared artifacts | **yes, by definition** |
| **Pending** | not yet in the store and not derivable from it — keystrokes in flight, a drag in progress, an uncommitted command | runtime | **no — dropping it loses user work** |

Pending state is the **only** non-discardable thing the runtime holds, so it carries extra obligations:

- it is bounded and enumerable — you can always list everything pending;
- it has an explicit commit boundary into the store (`Innovator` already has the shape: `FieldEditing` per keystroke, `FieldCommit` on commit);
- it may be journaled to the store's **session WAL** so a crash does not lose a half-typed calculation — a facility already built and currently unused.

Without this category, uncommitted edits get quietly parked in a derived artifact, which fails the discard test and reintroduces D6's rejected projection through the back door.

---

## D10 — Infinite Solutions is a platform; facades stack; apps sit on facades · **locked** · 2026-08-20

```
        SES            Coach Assistant        (structural)      ← apps, deferred
         │                    │                     │
   ┌─────┴──────┐             │                     │
   │  physics   │             │                     │
   │  facade    │             │                     │
   │ (laws of   │             │                     │
   │  physics)  │             │                     │
   └─────┬──────┘             │                     │
         └────────────────────┴─────────────────────┘
                              │
              infinite-solutions  ← the platform facade
         ┌─────────┬────────────┴─────────┬─────────┐
        db      runtime               compute      ux
```

The physics facade houses the laws of physics — Navier-Stokes, solid mechanics, and the numerical methods that serve them (`mycelium`'s boundary algebra, quadrature, Nitsche, assembly, solve). It is a *domain* facade built on the platform facade, not a platform layer.

The pattern generalizes and is expected to recur: a facade houses a family of domain laws over the same platform. A codes facade (ACI / AISC / ASCE) and a sport facade are the obvious next instances.

**Consequence for the tower.** The store's ability to subdivide a space and address the result is **platform**. Using that subdivision as an *h*-refinement hierarchy with retained parents, prolong/restrict, and far-field compression is **physics facade**. Those are two different things wearing one name in `analysis-substrate-plan.md` §0.5, and the seam between them needs stating precisely before either is built.

---

## D11 — Compute is a contract, not a capability · **locked** · 2026-08-20

The compute layer owns the declaration (inputs and outputs as addresses), the registry mechanism generic over signature, provenance, the staleness contract, the two execution shapes (DAG pass and settling loop), and explanation. It contains **no math**.

Half of it already exists in the store and has never been driven: `infinitedb_core/computation.rs`, `provenance.rs`, hyperedge payload codec V4 = `computation`, `check_hyperedge_freshness`, `query_stale_downstream`, and the `engine/derivation/` bus. The store already knows an edge can carry a computation and that changing an input makes downstream stale. The compute layer is what *invokes* one and *populates* that provenance.

---

## D12 — Platform first; apps are named but deferred · **locked** · 2026-08-20

No app is designed until the Infinite Solutions facade is workable. Planned consumers, recorded now because R19 requires the layer's spec to name what breaks if it is wrong: **SES** (physics) and **Coach Assistant** (coaching). The structural / AEC work is a third, unscheduled.

**The pair is the asset.** Fluid and solid mechanics and basketball coaching share no domain vocabulary, which converts R19 from a judgment call into a procedure (R32): a concept is platform only if *both* need it. First application of the test, which moved two things:

- **`Step { label, formula, substitution, result }` is not platform.** It is calc-shaped — "formula" and "substitution" mean nothing to a practice plan. Either the platform's explanation primitive is more abstract and the calc-style `Step` is a facade specialization, or explanation is per-facade entirely. **Open.**
- **The block manifest is platform, and more confidently than before.** A practice plan composed of drill blocks and an analysis composed of solver blocks are the same shape: named blocks, declared imports and exports, link-time failure on unsatisfied imports. `biomimicry`'s M12 linker passes the two-domain test cleanly.
- **O5 shifts toward units.** Coaching carries quantities too — minutes, reps, distances, loads. A `Value` model with units and provenance now serves both consumers rather than only the calc package.

---

## D13 — `Value` is opaque; kinds belong to the app · **locked** · 2026-08-20

**`Value` = opaque payload + opaque tag.** The platform's only operation on a tag is **match** — can this output connect to that input. It compares tags; it never interprets one.

Platform may: store a value, move it along a wire, match tags at wire time, detect change for staleness.
Platform may not: parse, validate, render, convert, evaluate, or check units.

**Rejected alternative:** a `ValueKind` registry that knows how to parse, validate, compare and render each kind, with units and exactness as platform concerns. Rejected because it is a type system inside a wiring substrate — parse, validate and render are app jobs, and admitting them makes the platform grow a kind per domain. This closes **O5**: it was never a platform question. Units, exactness (`f64` vs exact rational), and expression evaluation are app or facade concerns.

**Note on `Value` as an enum.** `bion/soma` defines `BoolValue, IntValue, FloatValue, SignalText, ByteBlob, UnitValue` — a closed enum that would need a seventh variant for video, an eighth for mesh. That is F-1 in the one type that touches everything. Four different `Value` models already exist across the repos (`bion`'s enum, `biomimicry`'s integer millis plus records, `PropValue`, and `BTreeMap<String, f64>`); this is the fifth and is intended to be the last.

**Corollary — expression fields need no machinery.** A field holding `2*L/3` is a *stored* text value; the parse and the evaluated number are *derived*, and R5 already governs the rest. An expression field is one text value plus one registered computation.

---

## D14 — The platform's job, stated in one sentence · **locked** · 2026-08-20

> **Make it easy for an app to get wired up and move between its components.**

Two jobs: **get wired up** (ports, wiring, link-time validation) and **move between components** (transport and change propagation). Everything else the platform was accumulating — kinds, units, math, widgets — is decoration on these two and belongs elsewhere.

**This is what three attempts each built independently.** `bion`: components expose input requirements and output secretions, and the engine links compatible interfaces within a shared hyperedge context. `biomimicry` M12: blocks declare imports and exports, a manifest names them, the linker synthesizes bridges and fails unsatisfied imports at link time. `Innovator`: valence — per-particle emit/absorb declarations. Three vocabularies, one mechanism, already implemented twice.

What the substrate owes an app: (1) port declaration — name, direction, tag; (2) wiring; (3) link-time validation with a precise error; (4) transport — Signal / Stream / Wave; (5) change propagation; (6) **composition closes** — a wired set of blocks is itself a block with ports. Six is the load-bearing one: without it composition stops after one flat layer and "primitives into blocks into great things" does not happen.

**Sub-fork resolved by D22.**

---

## D15 — `infinite-ux` is the embedding layer · **locked** · 2026-08-20

Not a widget toolkit. **The hypergraph↔screen crossover** — organizing wgpu against the space model. Every app needs it, and none of it is domain-specific, so it stays a platform layer.

Its doctrine is already written, in `infinitedb-spatial-layer.md` §6: the **embedding layer** is "d-dimensional, epoch-dependent; seats drift, subtrees are rigidly re-transformed, precision is finite; **nothing in this layer carries identity**." Invariant #7 states scene-graph invariance directly. A rendering doctrine was written while designing a database.

> **Law: the presentation layer holds positions, never identity.** Transforms, camera, visibility, draw order, LOD. Every reference to a thing is by address. (Mirrors R11.)

**Owns:** address → screen and pointer → address; visibility and culling (`nodes_in_bbox` is a rendering concern, as `analysis-substrate-plan.md` §0.4 already records); subdivision level for semantic zoom, which falls out of the space tree since level ℓ is the key truncated to ℓ·D bits; wgpu resource organization.

**Does not own**, resolving what was tangled in `hyper-ui`: layout algorithms (measure/arrange, `Extent`, the `SizeClass` ladder, hysteresis) are a **pure core depending on nothing**, per R3 — the 57 tests are box-packing with no graph and no wgpu. Widgets are **not code**: `Innovator` already made them authored `Entity` nodes with `component_kind` props.

**Third occurrence of the same finding.** The space tree is one structure read three ways: addressing and subdivision (store), *h*-refinement and multigrid (physics facade), scene graph and LOD (presentation). Surfaced independently three times from three directions.

---

## D16 — The thesis · **locked** · 2026-08-20

> **Infinite Solutions is a visual programming platform for building and deploying production apps, in which frontend, backend, and persistence are one substrate rather than three.**

The hypergraph space model earns that claim: spaces subdivide and hyperedges connect them, every space has a permanent address, and therefore *"this runs on the server"* is a **placement** decision rather than an architectural one. Most of what makes full-stack hard is the seams between the three, not the three themselves.

**"A child should be able to build a full stack app" is a design constraint, not marketing.** It rules out a text syntax in the core loop, rules out stack traces as the error surface, and requires the four layers to be invisible to the person composing.

**Two tiers of user, and neither should notice the other:** block authors write native primitives in Rust; app authors compose visually and never see Rust.

Full statement: [`CHARTER.md`](./CHARTER.md).

---

## D17 — Layer names: store · compositor · runtime · presenter · **locked** · 2026-08-20 · closes O8

"Compute" contained no computation and was a name that would have quietly attracted math back into the layer. **Compositor** is an agent noun and names the linker. **Presenter** likewise. Both replace earlier working names.

---

## D18 — A platform boundary is a portal · **locked** · 2026-08-20

Multi-platform is not a build-target matrix. It is the spatial model's existing mechanism: *"cross-chart interaction happens only through **portal nodes** — a node is both a vertex in its parent chart and the root of its own; spaces glue along portals"* (`infinitedb-spatial-layer.md` §7 notes).

Desktop and its server are two spaces glued at a portal. Adding a target adds a space and a portal. The author draws an edge; the **runtime** decides whether that edge is a function call, an IPC message, or a network round-trip. `p2p-swarm` plus `infinite-db`'s outbox / merkle / delta / conflict-queue is already the machine-to-machine transport.

No new concept was required — only noticing one already written.

**Concurrent writes across peers are already solved, at the storage layer.** `infinite-db` M5 — `register_arbiter_stream`, `assert_judgment`, `query_judgments_for_subject` — is MVCC conflict resolution: two writers touch the same node, both versions are kept, a branch is made, and an arbiter designates the winner. It answers *which bytes win*, not *what is true*. Recorded because the resemblance to an epistemic arbitration layer is misleading and was mistaken for one during this session's discussion.

---

## D19 — Execution: interpreted always, compiled opt-in · **locked** · 2026-08-20

A graph runs the moment it is drawn; there is no build step between composing and seeing it work. Compilation is an optimization applied to something that already works, chosen per composition, on the **runtime's** evidence rather than the author's guess. Many apps never need it.

> **Equivalence law.** The interpreted execution is the *specification*. A compiled block must be observationally identical to it.

Without this, "test everything first before compile" proves nothing — you would have tested a different program. It is differentially testable (run both, compare), and *exactly* so rather than statistically, because `biomimicry` already resolved manufactured determinism. Determinism is therefore not a nice property; it is what makes the compile story verifiable.

> **Compiled artifacts are derived state.** The graph is the definition. Delete every compiled block, rebuild from the graphs, and the system is identical.

This is R5 for the fifth time, and it puts compilation under R12's discard test. Compilation is formally a cache.

**Compilability and staleness are one discipline.** A composition is compilable only if it is a pure function of its declared inputs — the identical requirement D11 imposes so the store can compute downstream staleness. One rule, two payoffs, nothing new to enforce.

**Reconciles `mycelium`.** Native primitive blocks (quadrature kernels, linear solve) *plus* authored compositions (R-function boundary trees, assembly wiring) that compile. Both, not either — which is how "built by Infinite Solutions" and "nobody hand-wires a quadrature inner loop" are both true.

---

## D20 — Vocabulary: five words · **locked** · 2026-08-20

The core mechanic, in the form it should always be stated:

> **We subdivide spaces and connect them with hyperedges. Zoom into a space and you see more detail; zoom out and it becomes a node in the graph.**

| Word | Meaning |
|---|---|
| **space** | The unit. A coordinate region. Carries its own coordinates and a permanent address. |
| **node** | An object populating a space, at an address within it. May itself host its own space. |
| **graph** | What you see at one level: the nodes populating a space, and the hyperedges among them. |
| **hyperedge** | Connects any number of nodes. Carries values between them. |
| **zoom** | Crosses the node/space seam — enters a node's own space, or leaves it. The primary navigation. |

**"Chart" is retired.** `infinitedb-spatial-layer.md` uses it in the cartographic sense — a local coordinate patch, an atlas of charts. Correct in a database design document; in a visual programming platform it reads as data visualization, which is precisely the wrong picture. Citations of that document keep the original word; nothing else uses it.

**This was already an R17 violation before the rename.** The design document said *chart* while the API said *space* (`SpaceId`, `SpaceConfig`, `register_space`, and `analysis-substrate-plan.md`'s "a part is a space, not a node"). Two names for one thing, in one repository — the mechanism that produced three `PageTree`s. `space` wins because the code already said it.

**Corrected, 2026-08-23.** This section's consequences originally opened with *"node
and space are one thing at two zoom levels, not two things related by containment"*
— a claim this project actually drifted on. It over-applied `infinitedb-spatial-layer.md` §2's
indexing math — *"every node is simultaneously a vertex on its parent chart's Hilbert
curve and the author of its own chart, of which it is the order-0 center"* — into a
vocabulary claim it never made: that one *address* plays two roles in an indexing
scheme does not mean "node" and "space" are one word for one thing. The actual
mechanism the code already implements (`Addr::contains`'s byte-prefix check,
`hosts_space`, `direct_children`) is containment, plainly: **a space contains nodes,
placed at addresses within it, and a node may itself host its own space** — the same
entity can be a node from its parent's side and a space from its own. Zoom is what
lets a person cross that seam, not evidence there was no seam to cross. This locked
decision keeps its number; the correction is recorded here rather than merged
silently, per R29.

**Consequences worth stating.**

*Zoom is the navigation model, not a view feature.* There is no separate "open", "expand", or "drill into" — those are all zoom. `Innovator`'s tier ladder (root → workspace → page → pod → component → particle) is a zoom ladder that was built without being named as one.

*Detail is per space, not per camera.* Zoom sets a default; individual spaces are held open or closed against it, which is how several things stay legible at once and what tabs and pod-collapse actually are. The mechanism already exists: `hyper-ui`'s resolved-default-versus-sticky-override model with `CLASS_SLOP` / `PROMOTE_SLOP` hysteresis, built for responsive layout and generalizing directly to semantic zoom.

---

## D21 — Acyclic by default; cycles opt-in and marked · **locked** · 2026-08-20 · closes O4

Graphs are acyclic. Feedback is allowed only inside an area the author **marks as iterative**, which carries a maximum iteration count and a stopping test as visible properties. The loop is drawn on the canvas, not hidden in a black box — the same bounding as a loop block, with the box made transparent.

**Correcting the framing.** O4 was originally posed as "derivation versus settling," treating them as two scheduler modes for two kinds of data. That was wrong twice over. First, *settling toward truth* — retaining a user's wrong input and converging on what is the case — is an **app pattern**, not a platform layer; the platform owes only retention, provenance and time, and competing claims are modelled as sibling spaces joined by a hyperedge. Second, *numerical convergence* is not the same thing as epistemic convergence; a contact iteration is a derivation that loops internally, not an arbitration. With both confusions removed, the real question was simply **whether graphs may contain cycles**.

**Rejected alternatives.** *Strictly acyclic* — feedback hidden inside an opaque loop block. Rejected because it makes genuine mutual coupling invisible: the crane mat's three-way coupling between bending, settlement and contact patch is the physics, and it should be legible on the canvas. *Cycles allowed with runtime fixed-point iteration* — the shape `bion` and `biomimicry` both drift toward. Rejected because it makes non-termination reachable by wiring two blocks together, and a spinning app with no explanation is the worst failure available to someone learning. `biomimicry`'s homeostasis chapter — damping required to prevent oscillation — is the cost of that path written out in advance.

**Consequences.**

- **An iterative region is a block from outside.** Declared inputs and outputs; the outer scheduler treats it as one unit. D14.6 (composition closes) holds unchanged.
- **Termination is guaranteed; non-convergence is a finding.** Not a hang but *"ran 50 iterations, residual 0.003, did not converge."* `biomimicry`'s `org.settle(32)` already returns a status rather than blocking.
- **Staleness stays acyclic — the second payoff.** Change propagation is itself a DAG walk and needs acyclicity as much as the scheduler does. One authored concept fixes both.
- **A drawn cycle is judged, not refused.** The edge may exist; a finding says *"this closes a loop — mark the region iterative or remove the edge"*; derivation runs as far as it can and no further. The same stance as `Status::NotImplemented` withholding `all_passed` without refusing to run.
- **Nesting is permitted and already precedented.** `biomimicry`'s two nested scheduler loops at different cadences (K Phase 2 cycles per Phase 1) is a loop inside a loop, which under this model is nested spaces behaving normally.
- **Compilation is unaffected.** An iterative region is still a pure function of its declared inputs, so D19 holds.
- **Damping and convergence tests are app or facade concerns.** The platform runs the loop and counts; it does not know why a particular loop converges.

---

## D22 — Wiring is explicit · **locked** · 2026-08-20 · resolves the D14 sub-fork

Every connection is drawn. Nothing connects itself.

**The reason is methodological, not merely conservative.** Explicit wiring is what *generates the evidence* for what automatic wiring should later do. Good auto-connection cannot be designed without knowing which wires people draw repeatedly, and that data only exists once people have drawn them. Shipping the convenience first would mean guessing at it.

**Tags validate; they do not discover.** A tag mismatch is still caught at link time with a precise error. A tag never proposes a connection. D13 is unchanged — it already held that the platform's only operation on a tag is *match*.

**Rejected for now:** `bion`'s tag-matched automatic binding — *"apps expose input requirements and output secretions… the engine dynamically links compatible interfaces."* Its failure mode is ambiguity: two blocks emitting the same tag leave the author unable to say which one connected, and an unrelated change can silently rewire a working app. Neither is acceptable for a novice, and neither can be designed away without usage data.

**The tedium answer is D14.6, not automation.** If composition closes properly, wire count does not scale with app size — the wires inside *Plan a session* are drawn once and the block is reused. A forty-wire canvas is a symptom of blocks composing badly, not of explicit wiring.

**Available later without disturbing this decision:** bundle wiring (many ports to many ports as one gesture — a drawing shortcut, not discovery), and authored manifests, where an expert writes a composition as text and an app author uses the resulting block. Both are ordinary D16 two-tier moves.

**Evidence to collect during the explicit phase,** so the deferral is a measurement rather than a someday:

- which wiring patterns repeat, and how often;
- where a tag would have been ambiguous had matching been automatic;
- wires drawn per composition — whether tedium is real or anticipated.

---

## Open

**O8 — closed by D17.**

**O12 — May an iterative region yield between iterations?** A fifty-step contact solve that must complete inside one scheduler pass will block a frame. If a region may yield and resume, the interface stays live but the scheduler grows a second concern. Partial iteration state is discardable — restart the loop and you get the same answer — so it is *derived* under D8 and needs no new state category. Not urgent until something long is measured.

**O9 — closed by D28.** *What is the compiled form?* Answered by refusing the question's shape: the compiled form is a registered backend under a string key, and the compositor owns the contract rather than the form.

**O10 — Ownership and capability.** **Deferred with intent, not dropped.** Reframed 2026-08-20: the question is not authentication (proving who someone is — a commodity, bolt-on whenever) but **who owns what, and who may do what** — which is structural and platform, not facade.

*The substrate already holds the seed.* `infinitedb-spatial-layer.md` §2: *"Every node is the author of its own space"* — authorship is already first-class. §12, the open budget-authority decision: *"budgets-as-authorship — each space declares its ceilings; **ancestors impose inheritable ceilings**; current lean federated, consistent with the authorship model."* Per-space authorship plus inheritable ancestral ceilings **is** capability-based security; it was written while designing density governors. Same shape as the Windows AppContainer sandboxing wanted for P2P.

*The two halves.* **Owns** = authorship of a space. **Does what** = a right over a space for a verb, and the verbs are enumerable: read, write, subdivide, connect a hyperedge, run a composition.

*Platform, by R32.* A coach owns a roster, a player sees only their own plan, a team shares film; multi-reviewer analysis needs the same. Both named consumers need it, so it cannot be pushed into an app later.

*Three doors to keep open — cheap now, expensive to retrofit:*

1. **Every space records its author at creation.** Retrofitting ownership means backfilling it, and backfill is guesswork.
2. **Portals are the natural checkpoint.** D18 already makes them the boundary; do not build them so that a check cannot be inserted.
3. **Provenance grows an authority field.** Additive while provenance exists, which it does.

Prior art in the corpus: `Innovator/src/auth` — capability / role / session.

**O11 — closed by D36.** *Is the editor self-hosted?* Yes. The three layer
specifications named the editor as their forcing consumer; E7 is the
demonstration (drag a genesis node, restart, it is still there, nothing was
recompiled). D32 recorded the root-package consequence.

**O15 — closed by D33.** *Does the store admit a non-blocking submit?* Yes: `InfiniteDb::try_insert` returns `EngineError::QueueFull` instead of waiting. `enqueue` still blocks; that is the store's internal path, not the facade's.

**O17 — closed by D37.** *Is the style table authored or native?* Authored under the style root when the store has rows; `editor/styles.rs` holds only the bootstrap default.

**O16 — closed by D48.** *Where does the editor's undo live?* On the committed side of the commit boundary, as a new commit that restores the previous value. The pending set is discarded rather than undone, the camera is outside the stream because it never commits, and the stream itself is a registered derived artifact — so no fourth state category. The implementation is E12's; the decision is made.

**O17 — closed by D37.** *Is the style table authored or native?* Authored under the style root when the store has rows; `editor/styles.rs` holds only the bootstrap default.

**O13 — Three `Addr` types, one per layer core — and now a second duplicated type.** `infinite-runtime`, `infinite-compositor` and `infinite-presenter` each define an opaque, totally-ordered byte key, because each pure core depends on nothing (R3, D23, D26, D29). The facade converts. **Deferred with a trigger:** when the conversion is more than a newtype unwrap, promote `Addr` to a zero-dependency crate. That is not a fifth layer, so R1 is not engaged; it still needs a decision record.

**Amended 2026-08-21 by D29**, in two ways that both strengthen the case without moving the trigger.

*The three layers want different amounts of the type, and now two of them agree.* The runtime truncates prefixes for priority-by-distance-from-focus; the compositor deliberately refuses truncation (`docs/specs/COMPOSITOR.md` §3.2); the presenter truncates because **truncation is level**, which is its whole detail model (`docs/specs/PRESENTER.md` §3.2). So it is no longer one instance against one — it is two identical cores against one deliberate abstainer, which is the shape that usually means the shared thing is real.

*`Revision` is the second type to duplicate.* The runtime's and the presenter's are the same twelve lines. Recorded rather than folded in quietly, because O13 was opened about one type and is now about a *set*, and a deferred decision whose scope grows without anyone noticing is how a deferral becomes a permanent condition.

**The trigger fired 2026-08-28, and the decision is still deferred — with a new
trigger, stated (D45).** `presenter_addr` is no longer a newtype unwrap: it computes
the address's significant bit length from the editor's key scheme. That is exactly the
condition this entry named. The promotion is *not* taken, and the reason is R27:
`significant_bits` is one function over one key scheme, the runtime's and the
compositor's addresses do not want it, and moving three types into a shared crate to
share a function only one of them calls is generality without a consumer. **The new
trigger is a second layer needing the significant length.** A deferral whose trigger
fires and is silently re-armed is how a deferral becomes a permanent condition — which
is what the paragraph above says — so the re-arming is written down rather than
assumed.

**O1 — Hot working set: yes or no?** D6 forbids a writable graph but does not settle whether a read-only hot working set sits between store and layout. **Deferred with a trigger:** measure a warm Hilbert prefix scan of ~1000 nodes out of `infinite-db` against frame budget. If it clears, there is no working set and the layer gets smaller. This is a measurement, not an architecture question.

**Sharpened 2026-08-21 by D30, not closed.** The `Placement` is the candidate, and under D25 it is a registered derived artifact either way — so whichever way the measurement goes, the structure is the same and no layer changes shape. What is still owed is only the number. Recorded because "an architecture question that turned out to be a measurement" is worth distinguishing from an open architecture question, and O1 has been sitting in the second category while belonging in the first.

**O14 — The precision floor.** The address layer is exact forever; the embedding layer is not. `infinitedb-spatial-layer.md` §9 names this as a density governor and gives the detector — *"rendering a seat requires Σ dᵢℓᵢ bits of fixed point along the path… minimum embedded segment length approaching 2^(−P)"* — but not the response. Two candidates: **clamp and report**, which `docs/specs/PRESENTER.md` §10 specifies as the first move, or **re-base** the transform stack on the deepest common ancestor and carry on, which is what a renderer of an unbounded space eventually has to do. The second is the presentation-side twin of §12's ratchet-versus-breathing density question and should probably be decided alongside it. **Deferred with a trigger:** the first composition deep enough that the minimum embedded segment length approaches 2^(−P). Until then the presenter clamps, reports the shallowest offending address, and the facade turns it into a finding — so the condition is never silently rendered as an empty screen.

**O2 — Placement policy.** D6's read path is only affordable if a UI subtree is contiguous in key order — which is a *placement* decision, not luck. The spatial-layer doctrine already answers it: every space authors its own coordinates, a child space nests inside the parent's cell, and cross-space interaction goes through portal nodes. That makes a subtree contiguous **by construction**. Needs to be stated as policy and checked.

**O3 — closed by D24.**

**O4 — closed by D21.**

**O5 — closed by D13.** *What is a `Value`?* Opaque payload plus opaque tag; units, exactness and evaluation are app or facade concerns.

**O6 — closed by D27.** *One linker or two?* One. The compute / interface distinction is a tag convention, and the platform cannot observe it.

**O7 — `Innovator/docs/RULES.md`** — **closed 2026-08-20. Abandoned, not reconstructed.** The file does not exist on disk and is not recoverable from the record, in the manner of `HISTORY.md`'s six deleted plans. Superseded by D9. Nothing from it is carried forward except by being re-derived from repository evidence.

---

## D9 — A new ruleset, derived from evidence rather than recalled · **locked** · 2026-08-20

`InfiniteSolutions/docs/RULES.md`, draft 1: 29 rules in six sections (layers, the one law, runtime, naming, working, working-with-an-assistant), plus a Forbidden table and a conventions list.

**Rejected alternative:** reconstructing the lost document from memory of its shape (a closed six-primitive set, a three-way vocabulary translation table, 16 rules, a Forbidden table, a 7-phase merge plan). Rejected because a recalled rule carries no evidence, and a rule without evidence cannot be argued with when it becomes inconvenient. Every rule in the new document cites the repository fact that forced it.

**Deliberately not carried forward:** the closed six-primitive set (Node, Port, Edge, Value, Block, Instance). It is a *model* decision, and O5 (what is a `Value`) and O6 (one linker or two) are still open — asserting the primitive set before those are settled would be deciding them by implication.

**Amended 2026-08-21.** O5 closed by D13 and O6 by D27, so the set may now land. Re-derived from the corpus rather than recalled, it comes out at **five words, not six** — `Instance` is redundant, because use is delegation (D27, `docs/specs/COMPOSITOR.md` §5.2). The withholding was correct and it paid: the recalled set contained a primitive the evidence does not support.

**Known soft spots in draft 1**, flagged for review rather than left to be discovered: R8's strictness about threads (something must still tick the runtime, and a long solve must not block input); R11's silence on transient versus retained record data; and R2's fixity versus where an external writer such as the MCP server lives, which is not one of the four layers.

---

## D23 — The runtime depends on no other layer; it declares ports · **locked** · 2026-08-21

`infinite-runtime` names no crate belonging to another layer. It declares the ports it
needs as traits — `StoreRead`, `StoreWrite`, `StaleFeed`, `Clock`, `Journal` — and the
platform facade supplies the implementations. A sixth port requires a decision record.

**What forced it.** D3 established that a computation must not depend on the runtime,
or the two become mutual hostages and neither is testable alone. The identical argument
runs in the other direction, and it has already been built once: `bion`'s CNS reaches
the store only through `&dyn PnsReader`, enforced by a CI grep of `src/cns` for DB
tokens — the enforcement pattern R11 already cites approvingly. A runtime that names
`infinite-db` cannot be tested without a database, which means the editor's latency
behaviour cannot be tested at all, and latency is the whole reason this layer exists
(R19 consumer, `docs/specs/RUNTIME.md` §2).

The corollary is `Addr`: the pure core depends on nothing (R3), so it cannot use the
store's key type. **`Addr` is an opaque, totally-ordered byte key** — compared, ranged,
and prefix-truncated, never interpreted. Total order is spatial locality (which is O2
stated as a requirement the runtime depends on), and prefix truncation is level, used
for exactly one thing: scheduling priority by distance from focus.

**Rejected alternative:** depend on `infinite-db` for its address and revision types
only, on the grounds that types are not a coupling. Rejected because it is not
checkable — once the dependency edge exists, R9's check degrades from a manifest grep
to a judgment about which items are being used, and every prior attempt shows what
happens to a rule that becomes a judgment. Also rejected: making `Addr` a generic
parameter, which threads a type parameter through every type in the layer to buy an
abstraction with one instantiation (R27).

**What it costs.** One trait call per store touch, and a fake store to maintain. The
fake is not overhead — it is the only way D24's saturation test can exist, because a
real store's write queue cannot be filled on demand.

---

## D24 — A keystroke is not a write · **locked** · 2026-08-21 · closes O3

The input path never touches the store's write queue.

1. A keystroke is an `amend` to a **pending** entry (D8). It touches the pending set —
   in memory, bounded — and the journal, which is append-only and sequential.
2. The store is written only at an explicit **commit boundary**.
3. Commits are **coalesced per address**: at most one in flight per address, and a
   newer pending value supersedes an unsent one rather than queueing behind it.
4. `StoreWrite::submit` returns `Full` rather than blocking. On `Full` the commit stays
   in the pending set and is retried on a later tick.

Backpressure therefore degrades **durability latency** — how soon a committed value is
safe — instead of **input latency**. That is the correct thing to degrade, and stating
which one is being traded is the substance of the decision.

**What forced it.** R14, and `infinite-db`'s write queue blocking when full. The shape
already exists in `Innovator` as `FieldEditing` per keystroke and `FieldCommit` on
commit; this generalizes it from fields to any uncommitted gesture, a drag included.

**Rejected alternatives.** *A larger queue, or an unbounded channel in front of it* —
rejected because a larger queue moves the stall rather than removing it, and an
unbounded channel converts a stall into an OOM, which is a worse failure and a later
one. *Write every keystroke and rely on coalescing inside the store* — rejected because
it makes the input path depend on the store's internal policy, which R2 forbids the
runtime from assuming, and because it would put the check in the wrong layer.

**What it costs.** Pending state becomes load-bearing: it now holds
committed-but-unsent values as well as uncommitted edits, so D8's bound and
enumeration obligations are doing real work rather than being paperwork. Under
sustained backpressure a crash loses the journal's unflushed tail.

**Green check:** saturate the fake store's write queue, drive input at 60 Hz for 10
seconds, and assert that every keystroke reaches the pending set within one tick, no
tick exceeds its budget, the pending set stays within its bound, and after the queue
drains the store's final value per address equals the last input value.

---

## D25 — The runtime knows artifact lifecycle, never artifact content · **locked** · 2026-08-21

A derived artifact is **registered** under a string key with the address ranges it
derives from, a rebuild function, and a validity watermark. The runtime owns *when it
is rebuilt* and *whether it is stale*; it never knows what one is.

`RenderList` is the first instance, registered by the presenter's binding. This
resolves an apparent tension between D5 (which places `RenderList` in the runtime as a
declared artifact) and D15 (which places visibility and culling in the presenter):
**the presenter owns the function, the runtime owns the schedule.** That is R3's
core/binding split applied across a layer boundary rather than inside one.

**What forced it.** R12 requires every derived artifact to pass the discard test. Under
a registry the runtime can drop and rebuild any artifact and compare bytes *without
knowing what it is*, so R12 is enforced by a single generic harness that every artifact
anyone ever registers is automatically subject to.

**Rejected alternative:** a closed enum of artifact kinds. Rejected on F-1 grounds
(five prior occurrences), but more usefully because the discard harness would have to
be re-derived per variant — and a check that must be rewritten per variant is a check
that stops being run, which is the mechanism behind F-7.

**What it costs.** An artifact's rebuild function is dynamically dispatched and its
inputs are declared as data rather than known at compile time, so a mis-declared input
range is a runtime bug rather than a compile error. Mitigated by the discard harness,
which catches exactly that class.

**Third instance, 2026-08-21.** `docs/specs/COMPOSITOR.md` §4 registers the linked `Plan`
under this mechanism, and §10.3 registers compiled artifacts under it too. Presenter,
compositor, compilation: three consumers, none of which needed its own cache
machinery. The artifact registry is the platform's real integration surface.

---

## D26 — The compositor depends on no other layer; it declares five ports · **locked** · 2026-08-21

`infinite-compositor` names no crate belonging to another layer. It declares the ports
it needs as traits — `Definitions`, `Blocks`, `Values`, `Provenance`, `Backends` — and
the platform facade supplies the implementations. A sixth port requires a decision
record. **There is no `Clock`:** the compositor has no *now* (R10, D5).

**What forced it.** D3 established that a computation must not depend on the runtime,
or the two become mutual hostages. D23 ran the same argument for the runtime. The third
run is sharper than either, because of the compositor's forcing consumer.

The editor must answer *"if I drop this wire here, does it link?"* while the wire is
still **pending** (D8) — that is, while it is by definition **not in the store**. A
compositor that reads the store directly cannot answer that question at all. So `link`
must take a *definition set* — stored records, pending edits, or a mix — as an
ordinary argument. The port is not a purity gesture; it is the only shape that answers
the question the layer exists to answer.

Second consequence: `link` runs at interaction rate on speculative input, so it must be
unit-testable without a database. A layer whose central function cannot be exercised in
a unit test is a layer whose central function is untested.

**Rejected alternative:** depend on `infinite-db` for definitions only, on the grounds
that the definitions *are* the store's records. Rejected for D23's reason verbatim —
once the dependency edge exists, the check degrades from a manifest grep to a judgment
about which items are being used — and additionally because it makes the pending case
unimplementable: the store does not hold pending state, and pending state is exactly
what the editor needs linked.

**What it costs.** Five trait calls and a set of fakes to maintain. And `Addr` is now
defined twice — once in `infinite-runtime`'s core, once in `infinite-compositor`'s —
because each pure core depends on nothing (R3). Today the facade's conversion is a
newtype unwrap. Recorded as **O13** rather than solved.

**Green check:** `cargo build -p infinite-compositor` with no features succeeds and the
manifest's `[dependencies]` is empty; the whole test suite passes against fakes; no
crate belonging to another layer is named anywhere in the crate.

---

## D27 — One linker; the compute/interface distinction is a tag convention · **locked** · 2026-08-21 · closes O6

There is **one** linker. `biomimicry`'s block manifest (compute graphs) and
`Innovator`'s component catalogue plus template registry (interface graphs) are one
mechanism, and the platform does not distinguish them.

**What forced it.** Two independent arguments arriving at the same place.

*From D13.* A tag is opaque and the platform's only operation on one is **match**. The
platform therefore **cannot tell** a compute port from an interface port — it has no
operation that would reveal the difference. A distinction the platform cannot observe
is not a platform distinction. It is a tag convention, and tag conventions are the
app's (D13).

*From the forcing consumer.* The editor is an interface graph that edits compute
graphs (O11). With two linkers, the editor cannot be a composition in the platform,
and self-hosting is dead at the first screen — which under D16 means a child cannot
build an app in it either, since the editor is the proof.

The direction read (§6C) had already observed the two are *"the same idea at two
altitudes"* and asked for one linker; this records the reason rather than the
observation.

**Rejected alternative:** two linkers over a shared core — a compute linker and an
interface linker, sharing port declaration, tag matching, arity, ordering and closure.
Rejected because the shared core would be *everything* and the difference would be
nothing the platform can see: two names for one structure, which is R17's failure with
the sign flipped, and the mechanism that produced three `PageTree`s. It also
reintroduces a kind distinction into the platform, which D13 refused on its own
grounds.

**What it costs.** A block's ports carry no hint of what they are for, so the editor
cannot lay out a solver block differently from a view block without an app-level
convention. That convention has to be authored somewhere and the platform will not
help — deliberately, because helping would mean interpreting a tag.

**Consequence for D9.** D9 declined to carry forward the recalled six-primitive set
(Node, Port, Edge, Value, Block, Instance) because O5 and O6 were open and asserting
the set would decide them by implication. D13 closed O5; this closes O6. The set may
now land — re-derived from the corpus rather than recalled, per D9's own standard, and
it comes out at **five words, not six**: `Instance` is redundant, because a block used
twice is two spaces each of whose body *names* another space. Use is delegation, which
is `bion`'s `.node().delegate()` chain grammar that D6 already salvaged.
`docs/specs/COMPOSITOR.md` §5 carries the derivation.

---

## D28 — The compiled form is a registered backend, not a type · **locked** · 2026-08-21 · closes O9

O9 asked: native code per target, or WASM, or an IR that JITs, with the block perhaps
declaring its form. **The question has the wrong shape.**

> **The compiled form is a registered backend under a string key. The compositor owns
> the contract a backend must satisfy, and never the backend.**

This is D25's shape one layer over: *the runtime knows artifact lifecycle, never
artifact content* becomes *the compositor knows compiled-form lifecycle, never the
form*.

**The contract.** A backend supplies four things: `accepts(&Plan) -> bool`;
`compile(&Plan) -> Artifact`; `invoke`, with **the same signature as interpreted
invocation**; and a **cost declaration** — what it costs to produce and what it costs
to cross into. D19 requires compilation be chosen *"on the runtime's evidence rather
than the author's guess"*, and the runtime cannot choose without the fourth item.

**Registration is the gate.** A backend is not registered because someone wrote it; a
backend is registered **by passing the equivalence harness** — for every plan in a
maintained corpus, run interpreted, run compiled, compare outputs bit-for-bit and
provenance edge-for-edge. D19 says the interpreted execution is the specification;
this makes that sentence executable, and it means the platform cannot grow a
subtly-wrong backend. Adding a form in 2027 costs a registration, not a redesign.

**What forced it.** R16 and F-1 — a closed enum standing for an open set, five prior
occurrences. The set is open, and this system's own roadmap already holds two members
neither `bion` nor `biomimicry` considered: a **GPU kernel** (the presenter is wgpu per
D15, so a fused numeric composition compiled to WGSL is a short walk) and a **pushdown
into the store** (a composition of pure reads over an address range is a range scan,
and the index already exists). Naming one form would be F-1 in the place where being
wrong costs a toolchain rebuild.

**Rejected alternatives.**

*Name one form now.* Rejected as above. The strongest version of this argument — pick
WASM, because D18's portals want one artifact that runs on every target — survives as
tier 2 below rather than as the answer, because paying WASM's call-boundary cost must
be a choice SES is allowed to refuse.

*Let each block declare its form*, which is O9's own suggestion. Rejected on two
grounds. It makes the author guess, contradicting D19's evidence rule directly. And a
composition of blocks with three declared forms has no defined form of its own, which
breaks D14.6 — composition would stop closing at exactly the point compilation starts
mattering.

**Three tiers, named in the order they should be built.**

| Tier | Form | Removes | Costs |
|---|---|---|---|
| **0** | resolved plan — **no compiler** | lookup and dispatch: sources resolved to slots, invocations to function pointers, order fixed | nothing: no toolchain, no codegen, no dependency |
| **1** | native — generate Rust for the fused composition and build it with the toolchain block authors already have (D16) | per-edge value boxing; the optimizer sees across block boundaries | a toolchain at author time; a per-target artifact |
| **2** | portable — WASM | — | a call-boundary cost SES may refuse |

**Tier 0 is built first, and the first compiled form requires no compiler.** It adds no
dependency to a crate whose green check is an empty `[dependencies]`, it is the classic
large constant-factor win before any codegen, and its equivalence is the easiest of the
three to argue — same code, same order, lookups hoisted. It is therefore also the
honest first test of the harness: a backend that *should* be equivalent failing the
harness means the harness is wrong.

**Which tier is used is a placement decision** (D16's word for the same move), made by
the runtime from the cost declarations plus its own measurements. It is not a property
of the block.

**What it costs.** Three things.

1. **A plan corpus must be maintained.** The harness is only as good as its corpus; an
   unexercised plan shape is an unverified one. The corpus is a deliverable, not a
   fixture.
2. **`accepts` makes refusal reachable.** A plan can be compilable in principle and
   refused by every registered backend. The author is told so in a finding, rather than
   left with a composition that silently stayed interpreted.
3. **The cost declaration is self-reported**, so a lying backend misroutes. Mitigated
   structurally: the runtime measures and **may demote a backend whose measured cost
   contradicts its declaration**. Demotion is only possible because the declaration is
   data rather than a promise.

**What it gets for free.** A compiled artifact is derived state (D19) registered with
the runtime (D25), so R12's generic discard harness drops and rebuilds it without
knowing what it is. Compilation needs no invalidation machinery of its own. This is the
second substantial thing `docs/specs/COMPOSITOR.md` takes from D25 without adding anything,
which is the strongest available evidence that D25 was right.

---

## D29 — The presenter depends on no other layer, and on no GPU; it declares three ports · **locked** · 2026-08-21

`infinite-presenter` names no crate belonging to another layer **and no graphics
crate**. It declares the ports it needs as traits — `Scene`, `Surface`, `Glyphs` — and
the platform facade supplies the implementations. A fourth port requires a decision
record. **There is no `Clock`** (R10, D5) and **there is no write port** (D30's L6).

**What forced it.** D3 established that a computation must not depend on the runtime,
or the two become mutual hostages and neither is testable alone. D23 ran the argument
for the runtime and D26 for the compositor. The fourth run is the one with the most
evidence behind it, because the alternative was built and shipped: `hyper-ui` names
`wgpu` and `winit` in the same crate as its camera and its layout algorithms, so
exercising the embedding would mean standing up a device and a window.

The consequence is measurable and it is the reason this decision is not a matter of
taste. Of the 21 files of that crate read while writing `docs/specs/PRESENTER.md` —
2,747 lines — there are 24 tests in 6 files, and **`renderer/` has zero and `geom/` has
zero.** The world↔screen transform, its inverse, the culling and every rectangle
operation are untested. And there is a live bug sitting in exactly that gap:
`SceneCamera::screen_to_world` accounts for the surface origin and
`visible_world_rect` does not, so whenever the canvas is not at the window's corner the
region culled is offset from the region drawn. The doc comment on `viewport_origin`
describes that same bug being found and fixed — **in the draw path**. Nothing could
have caught the other half.

`Innovator/plans/HISTORY.md` predicted this in general terms: *the graph model is well
covered, rendering and pointer interaction are covered by nothing, and that is exactly
where the longest-lived drifts survived unnoticed.* This is that sentence with a file
name attached.

**This does not contradict D15**, which gives the presenter *"wgpu resource
organization"*. The presenter owns the **organization** — what is uploaded, in what
order, at what detail, grouped how. The facade owns the **API**. `Surface` is the seam,
and putting it there is precisely what makes the organization testable without a
device. Recorded explicitly because on a fast read it looks like a reversal.

**Rejected alternative:** depend on `wgpu` for its types only — a texture format, a
buffer usage — on the grounds that a type is not a coupling. Rejected for D23's reason
verbatim: once the dependency edge exists the check degrades from a manifest grep to a
judgment about which items are being used, and every prior attempt shows what happens
to a rule that becomes a judgment. Also rejected: a generic `Backend` parameter over
the graphics API, which threads a type parameter through every type in the layer to buy
an abstraction with one instantiation (R27).

**Also decided here: one scalar, and it is `f64`.** `f32` appears nowhere in the crate;
narrowing happens inside the `Surface` implementation, in the facade, once. `hyper-ui`
runs an `f64` world (`WorldRect`, `SceneNode.world_pos`) through a 32-bit camera
(`SceneCamera.center`, both transform methods), narrowing in `fit_to_content` and
widening back in `visible_world_rect` — so an address space whose entire premise is
unbounded refinement is projected through 24 bits of mantissa, twice a frame. This is
the difference between a precision floor that can be *detected* (O14) and one that
shows up as jitter.

**What it costs.** Three trait calls and a set of fakes to maintain, and a third
definition of `Addr` (O13). The fakes are not overhead: `FakeSurface` is the only
reason the agreement test can exist, because a real surface's origin cannot be varied
on demand at test speed.

**Green check:** `cargo build -p infinite-presenter` with no features succeeds and the
manifest's `[dependencies]` is empty; `--features binding` succeeds; both with zero
warnings; the whole test suite passes against fakes; and `scripts/check-rules.sh`
passes its presenter section, including the manifest grep for `wgpu`, `winit`,
`glyphon`, `cosmic-text`, `raw-window-handle`, `softbuffer` and `glam`.

---

## D30 — The presenter mints no identity; the placement is a registered derived artifact · **locked** · 2026-08-21

Two laws, carried in the manner of D4's L1/L2 and `docs/specs/COMPOSITOR.md`'s L3/L4:

> **L5 — The presenter mints no identity.** Every reference to a thing is the store's
> address. No id of its own, no handle, no index standing in for a node, and no map
> keyed by anything but an address.
>
> **L6 — The presenter authors nothing.** It has no write port. Camera, collapse and
> selection are read; hover and a drag in progress are the runtime's pending set (D8).
> Nothing about a thing is ever written where that thing's geometry is computed.

**L5 corrects D15's law rather than restating it.** D15 says *"the presentation layer
holds positions, never identity"*, quoting `infinitedb-spatial-layer.md` §6's *"nothing
in this layer carries identity."* Read strictly, that is what `hyper-ui`'s `SceneNode`
does — eleven lines of geometry with no id and no address — and the result is a layer
that **cannot be hit-tested at all**, and that had to smuggle `selected: bool` into the
geometry record because there was nowhere else to put selection, so selecting a thing
means re-deriving and re-uploading its geometry.

The correct reading is the one D15's own next sentence gives: *every reference to a
thing is by address.* **Minting** identity is forbidden; **referring** to the store's is
mandatory. The distinction is the whole of L5 and it is what makes the layer
answerable.

**What forced L6.** R5 — derived state never writes back into the definition it derives
from — is the corpus's most-repeated invariant, stated in four vocabularies, and the
presenter is the layer it was written for. It is also the layer where it is currently
violated: `hyper-ui`'s `arrange` writes a measured `content_extent` and a clamped
scroll `offset` back into the `ParticleData::Viewport` it is laying out, so running
layout twice is not the same as running it once; and its input router writes
`TriggerState::Hover` into the model on every mouse move, which stores a transient
visual state where the domain data lives. The fix is not vigilance. `place` takes
`&SceneSet` and `probe` takes `&Placement`, there is no `&mut` in either signature, and
there is no write port — so the violation is a compile error rather than a review note.

**The placement is a registered derived artifact (D25), and this is D25's fourth
instance.** The presenter owns the **function**; the runtime owns the **schedule**.
D25 named `RenderList` as its first instance before either was built;
`docs/specs/COMPOSITOR.md` §4 called the pattern *"three instances is a pattern rather
than a coincidence"*. This is the one D25 was written for, and it arrives adding
nothing: no invalidation machinery, no dirty flag, no per-artifact test code. R12's
generic discard harness covers it on the day it is written for anything else.

That is what kills the failure mode. `hyper-ui`'s render list claims in its module doc
that *"it is rebuilt only when the structure or layout changes"* and carries no dirty
flag, no generation and no watermark; the invalidation protocol is a comment on an enum
variant — *"the host must rebuild layout and the render list"*. A host that misses one
gets stale rectangles and clicks that land on the wrong node, silently.

**Registration happens in the facade, not in either crate.** D23 forbids the runtime
naming another layer and D29 forbids the presenter naming the runtime, so neither can
see the other's registry. `binding::artifact` exposes the three parts D25 asks for —
ranges, rebuild function, watermark — and the facade hands them over. **This corrects
`docs/specs/RUNTIME.md` §5.2**, which says the artifact is *"registered by the
presenter's binding"*; it cannot be. Recorded here rather than silently edited there
(R21, R22).

**A consequence for the probe, and it is the argument for the whole spatial model
arriving as a performance result.** Because the placement is self-sufficient — every
thing carries its address and an `accepts` bit resolved at place time — `probe` needs
no port, so it can answer at pointer rate and for geometry that is still pending. And
because a subtree is contiguous in address order and embedded rigidly, **address order
is spatial order**, so the probe *descends* in O(depth) instead of scanning.
`hyper-ui` reverse-scans a flat list on every cursor move, again on press, and again on
release, and looks up by id inside that same positional vector with a second linear
scan. Nobody optimized the descent; it falls out of addresses being permanent and
locality-ordered. The reason it is fast is the reason addresses exist.

**Rejected alternative:** hold a positions-only scene and a separate id↔rect side table
for hit testing — which is `hyper-ui`'s actual shape, `SceneNode` plus `RenderItem`.
Rejected because it is two structures describing one thing, which is R17's failure and
the mechanism that produced three `PageTree`s; because the side table is a map keyed by
id standing in for an edge, which is F-2 (roughly thirty instances in one codebase in
this corpus); and because the two can disagree, which is precisely the "two things that
can disagree" ground on which D6 rejected the live projection.

**Renamed, and the old name retired.** `RenderList` → `Placement`. It answers pointer
queries, which is not rendering; it holds no draw commands, so *list* names the wrong
thing. R17 permits a rename and forbids a *recycle* — `RenderList` is retired, not
reused — and D20 set the precedent when it retired "chart" for "space", with the same
convention: **citations of D5 and D25 keep the original word; nothing else uses it.**
Flagged rather than buried, because a rename proposed by an assistant is exactly the
class of change R29 says to correct rather than merge, and it is one `sed` from being
reversed.

**What it costs.** The placement must carry, per placed thing, a bit (`accepts`) that
is app policy resolved at place time — so a change in that policy invalidates the
placement rather than being answered on the fly. That is the right trade, because the
alternative is P3, but it is a real cost and it is stated so it is not discovered.

**Green check:** `probe` is called with **no port in scope at all** over a corpus of
overlapping siblings, a clipped subtree, a collapsed space, a point on a seam, a point
in a gutter, and a point outside every space; every case answers, and the answers are
the ones the corpus declares.

---

## D31 — Detail is a level, not a mode · **locked** · 2026-08-21

> **A space is drawn at a *level* — a number of significant address bits. Zoom resolves
> a default, the space may override it, and the result is clamped to what the surface
> can resolve. There is no visibility enum.**

**What forced it.** D20 requires that *detail is per space, not per camera* — *"zoom
sets a default; individual spaces are held open or closed against it, which is how
several things stay legible at once and what tabs and pod-collapse actually are."* The
question is what a "level of detail" **is**, and the corpus offers two answers.

`hyper-ui` answers with `enum Visibility { Shown, Collapsed, Hidden }`, with a derived
`Ord` doing real work — *"`Shown < Collapsed < Hidden` is the demotion ladder, so demote
one step is a successor"* — driven by width pressure in a one-dimensional allocator. A
grep of that crate for `lod` returns nothing; there is no zoom-driven detail anywhere in
the corpus.

D20 answers differently, and it already said so (corrected 2026-08-23, see D20): **a
space, collapsed, is drawn as a node in its parent's graph.** So *collapsed* is not a
third state beside *shown* and *hidden*. **Collapse is zoom.** A space drawn at its
own level is a node; a node's own space, drawn one level deeper, is a graph; one below
the visible range is absent. Three variants become one number.

And the number is not new either. Level ℓ is the address truncated to ℓ·*D* bits —
written three times in this corpus already (`infinitedb-spatial-layer.md`, the analysis
substrate plan §0.5, D15) — so **every address already carries its own level**, and
"how much detail does this space get" is a question about that address rather than
about the camera. D20's requirement becomes impossible to violate rather than a policy
to maintain.

**Bits, not levels.** The presenter counts bits and never learns *D*, because charts
need not share a dimension (`infinitedb-spatial-layer.md` §2). A thing never learned is
a thing that cannot be wrong.

**Rejected alternative:** port `Visibility` and its demotion ladder. Rejected on R16 and
F-1 grounds — a closed enum standing for an open set, five prior occurrences — but more
usefully on evidence: `DemotionLadder::demote` returns the current value unchanged when
that value is not in the ladder's steps, so a container carrying a visibility from a
different arrangement is a **silent no-op** rather than a finding, and the three
ladders in that crate differ. A fourth rung under this decision is a different number;
under the enum it is a new variant plus every match arm.

**Salvaged wholesale: the hysteresis rule**, from `hyper-ui/src/layout/viewport.rs` —
the best-designed and best-tested code in that crate, six tests including full
600 → 1100 → 600 sweeps. It is deliberately **asymmetric**: promote at
`naive.lower_bound() + CLASS_SLOP`, demote below `previous.lower_bound() - CLASS_SLOP`,
with `CLASS_SLOP = 32.0` — so the boundary is a 64-unit dead band, not a 32-unit one,
and the *previous* class's bound is used on the way down. Substitute zoom for width and
level for size class and the algorithm is unchanged. This is the third thing taken from
that crate intact, alongside `Extent { min, ideal, weight }` and depth-first paint
order.

**Not salvaged, and worth knowing why:** `InputClass::hit_slop()` — 4 units for a
pointer, 12 for touch — exists in that crate, is unit-tested, and is called by nothing
but its own test, while the pointer path does an exact both-edges-inclusive containment
test with no slop at all. A tested dead function is worse than an untested one, because
the test says it works.

**Consequence: the core carries zero enums**, and `scripts/check-rules.sh` pins the
count at zero. That is a stronger position than the compositor's, which pins at one and
still owes a decision record for `Direction { In, Out }`
(`docs/specs/COMPOSITOR.md` §14, finding 6).

**What it costs.** The clamp in step 3 means a space can ask for detail the surface
cannot deliver, and the person has to be told rather than shown a blur. That is O14,
and until it is decided the presenter clamps and reports the shallowest offending
address as a fact, which the facade turns into a finding with a remedy. What it must
never become is an empty screen: `hyper-ui`'s `cull_nodes_from_infinite_db` maps a
database error to `Vec::new()`, so a failed query and an empty viewport are
indistinguishable and nothing is logged.

**Green check:** sweep zoom up through every boundary and back down; assert each
boundary is crossed exactly once in each direction, no level changes twice inside one
dead band, and the sequence is identical on replay — which D19's equivalence law needs
and which a hysteretic function has only if all of its state is in the view it was
handed.

---

## D32 — The root is a package: portal, facade, and editor · **locked** · 2026-08-21

The workspace root is a package named `infinite-solutions`. It builds a binary. That
binary is the **portal** — the platform's boundary with the operating system (D18) —
plus the platform facade (D10) and the editor that is the forcing consumer named in
all three layer specifications (O11). It is not "an app" in D12's sense; D12 defers
SES, Coach Assistant and the structural work, and this defers all three still.

`src/` is one crate holding three modules:

| Module | Role | May name |
|---|---|---|
| `src/facade/` | the thirteen port implementations, the `Addr`/`Revision` conversions, artifact registration | every layer crate; graphics crates only in `ports/surface.rs` and `ports/glyphs.rs` |
| `src/portal/` | window, device, OS input, tick loop | graphics and windowing crates; **no** layer crate |
| `src/editor/` | the app | neither a layer crate nor a graphics crate |

R2's dependency direction — `layer → platform facade → domain facade → app` — cannot
be enforced by the manifest inside one crate, so a grep in `scripts/check-rules.sh`
enforces it instead.

**What forced it.** The facade must exist for any of the four layers to be exercised
together. The editor is the forcing consumer named in all three layer specs. O11 is
answered yes, immediately (`docs/plans/EDITOR-BOOTSTRAP.md` §0.3), so the editor is
in this package rather than waiting for a later migration. R1's "a fifth layer
requires a decision record before it gets a directory" makes `crates/` the wrong home
for something that is not a layer.

**Also decided here: the vendored store.** `crates/infinite-db` joins the workspace as
a path member. The facade path-depends on that copy, not on the published
`infinite-db` 0.4.x that D1 names, until **the facade needs no store change for two
consecutive stages** — that is the trigger for switching to the published crate.
Forced by bootstrap-plan §9 finding 2: E1's `StoreWrite` may need a `try_submit` the
published crate does not have (O15, D33), and D1 locks the store *as* the store; it
does not forbid changing it. The crate's inner `Cargo.lock` is ignored once it is a
member; its `target/` stays gitignored.

**Rejected alternative:** the facade as a fifth crate `crates/infinite-solutions` with
the editor as a sixth. Rejected because `crates/` is the wrong home for something that
is not a layer (R1), and because two more crates buys manifest-level R2 enforcement at
the cost of two more `Addr` conversions and a second place for the facade's own
vocabulary to live. Also rejected: depending on published `infinite-db` 0.4.x from
day one — a store change needed by E1 would then be a release cycle rather than a
path edit.

**What it costs.** R1's sentence now needs the qualification in the root
`Cargo.toml`: the binary is the portal and the forcing consumer, not "an app" in
D12's sense. R2's strongest enforcement — a manifest that cannot express the wrong
edge — degrades to a grep that can be circumvented by anyone who wants to. The
mitigation is that the grep runs in `check-rules.sh` alongside checks that have all
been verified to fail (`PRESENTER.md` §11's discipline). And the root now builds a
binary, which `RUNTIME.md` §10 finding 2 had asked to go; it went, and this brings it
back deliberately.

**Green check:** `cargo build` succeeds; `bash scripts/check-rules.sh` prints a
facade section and an editor section; every check in the file passes, including R18
for all four crate directories.

---

## D33 — `StoreWrite` is `try_insert`; the queue never waits on the tick path · **locked** · 2026-08-21 · closes O15

The store grew a non-blocking write. `WriteQueueSender::try_enqueue_write` uses
`try_send`. `InfiniteDb::try_insert` maps a full channel to `EngineError::QueueFull`.
The facade's `StoreWrite::submit` maps that to `Submission::Full`. A tick that
cannot commit returns; it does not stall.

`InfiniteDb::pause_write_drain` parks shard I/O before `recv`, so the bounded queue
can actually fill. Without it, D24's saturation test is a race against the drain
thread, which is how a check nobody has seen fail becomes a check nobody trusts
(`PRESENTER.md` §11).

**What forced it.** D24 requires `Accepted` or `Full`, never a wait. The store's
queue used `send` and blocked when full. D1 locks the store *as* the store; it
does not forbid changing it. E1 is the first consumer of that contract against
the real queue.

**Rejected alternatives.** *A bounded try-channel in the facade, drained by the
tick loop* — the plan's second preference. Rejected because it is a second place
where writes are ordered, which is the smell D24 already named, and because the
store is vendored (D32) so the correct change is a path edit rather than a
wrapper. *Blocked, write a finding, stop* — the plan's third preference.
Rejected because the store can grow the call; there was nothing to be blocked
on. Already rejected by D24, and not revived: a larger queue, and an unbounded
channel in front of it.

**What it costs.** The published `infinite-db` 0.4.x crate does not have
`try_insert`. D32's trigger — switch when the facade needs no store change for
two consecutive stages — starts counting from the stage *after* this one. Until
then the workspace member is the source of truth.

---

## D34 — The well-known addresses are the bootstrap ABI · **locked** · 2026-08-21

`src/editor/addresses.rs` is the only file in the repository that may contain a
literal well-known address. Path strings (`/input/…`, `/style/`, `/screen/`)
name the ABI. Screen and style keys are four bytes so a subtree is one range
under the facade's identity mapping — a change to either form is a migration,
and there is no migration machinery.

**What forced it.** Something has to be findable in an empty store before
anything can be drawn. Genesis writes under the screen root; E4 deletes under
the same root; the input path amends well-known pending addresses (D24). If
those literals lived wherever they were first needed, a rename would be a
scatter, and an emptied store would have no agreed place to look.

**Rejected alternative:** discovery by convention — scan for a space with a
marker prop. Rejected because it makes an empty store and a corrupt store
indistinguishable, which is `PRESENTER.md` §13 finding 8 and the last row of
the bootstrap plan's stop list. An empty screen is a finding that names the
root; a missing marker would be the same silence.

**What it costs.** The four-byte keys and the path strings can drift from each
other. The genesis discard test pins the empty-screen finding's site to
`SCREEN_ROOT_KEY`, so a drift is a failed test rather than a black frame.

---

## D35 — `Direction { In, Out }` is closed · **locked** · 2026-08-21

`Direction` stays an enum of two variants in `infinite-compositor`'s core. A
value crosses a port boundary one way. Bidirectional traffic is two ports, not
a third variant. Coach Assistant and SES both need in and out and neither
needs a third — R32 agrees the set is platform, and R16 agrees it is closed.

**What forced it.** E5 is the first stage that reads a port. Spec §14 finding 6
owed a decision record rather than a comment: *"it looks closed"* is the
sentence that preceded all five prior occurrences of F-1.

**Rejected alternatives.** *A string key `"in"` / `"out"`* — that is an open
set pretending to be closed; a typo becomes a silent miss rather than a
compile error. *A `bool` `incoming`* — it loses the name of the concept and
makes `signature.inputs()` a comment on a flag. Both were available; both
were worse.

**What it costs.** A later genuinely new direction (a bidirectional port that
is not two ports) is a decision, not a variant. The crate's enum count stays
pinned at one.

---

## D37 — The style table is authored; the code table is a fallback · **locked** · 2026-08-21 · closes O17

A style key resolves first from store rows under the style root, then from
`editor::styles::bootstrap_default`. The descriptor is data (fill as four
`f64`s). `facade/ports/surface.rs` is where it becomes wgpu. The editor names
no graphics crate.

**What forced it.** E3 is O17's trigger. An authored table falls under E4's
discard test: delete the style space and the screen still renders the
bootstrap default rather than going black — `PRESENTER.md` §13 finding 8,
avoided by construction. A native-only table would have to be migrated the
first time a person edits a colour.

**Rejected alternative:** a native table in `styles.rs` as the source of truth,
with store rows later. Rejected because that is the migration O17 warned is
cheap now and expensive later, and because a colour that lives only in Rust
cannot be discarded and rebuilt from the store.

**What it costs.** E4 authors the rows under the style root; `styles.rs`
remains the fallback so an emptied style table still draws `plain` rather
than going black. A missing font is a separate finding (bootstrap plan §9
finding 10) — this decision does not invent shaping.

---

## D38 — Provenance is recorded through the port and held by the facade · **locked** · 2026-08-21

The compositor's `Provenance` port records, for every executed step, the output
addresses and the exact declared input set. The facade holds that map, keyed by
address, and answers `inputs_of` / `stale_downstream` as the inverse. That is
COMPOSITOR.md S6 and RUNTIME.md S6, identical in form: if the two disagree, one
layer is wrong about what a dependency is.

`infinite-db`'s `query_stale_downstream` is **not** the first wiring. The
editor's space is a 1-D record range (D32, D34) and has no hyperedge space.
Growing one so E6 can call M7 would be a store-schema change the stage does not
force (R27), and it would reset D32's two-consecutive-stages trigger.

**Rejected alternatives.** *Wire M7 hyperedges now* — rejected as above.
*Hold the lineage inside the compositor* — rejected by L4: the compositor is a
function, not a place. *A second in-memory graph in the runtime* — rejected by
F-3 and D6; the runtime already owns the frontier of *addresses*, and that is
all it may hold (R11).

**What it costs.** Staleness is exact for what `interpret` recorded this
process, and is not yet a durable hyperedge index. Drawing a wire as a
hyperedge would be the trigger for switching the query onto
`query_stale_downstream` without changing either layer's S6 test. E8 did
not take that trigger: C4 is answered by `Definitions` over stored ∪
pending (D39), and M7 is still not forced (R27).

---

## D36 — The editor is self-hosted · **locked** · 2026-08-21 · closes O11

The editor is an app on the platform facade (D32). Its screen is authored
spaces (E4). Its behaviour is an authored composition (E5) that runs
interpreted (E6). E7 is the demonstration: drag a node genesis wrote, commit
the change, restart the process, and the node is still where it was left.
Nothing was recompiled.

That is D16's thesis executed rather than asserted. All three layer
specifications named this as their forcing consumer; they do not have to
re-derive §2.

**Rejected alternative:** keep O11 open until a person has done the gesture in
a window and recorded video. Rejected because R23 requires a stated
verification method, not a format: `tests/self_edit.rs` reopens the same
store directory after drop, which is the restart, and asserts no compiled
backend is registered. A video of the same loop is documentation, not a
second check.

**What it costs.** The editor's screen is now data that can be broken by
editing it. There is no undo beyond the store's revisions and branches.
That is not a gap — revisions, rollback by branch merge, and provenance are
what the charter says production comes from — but the editor exposes none of
it yet. That absence is **O16**, and E7 is the trigger that makes it visible.

---

## D39 — An in-flight wire is a pending composition record · **locked** · 2026-08-21

C4 is *"does this wire link?"* while it is still uncommitted. The answer is
`link` over `Definitions`, and `Definitions` is stored ∪ pending (E1). The
in-flight wire is therefore the composition record itself, amended at the
graph root, not a second structure the linker cannot see.

A tag mismatch on that pending record is a finding with a site, a said, a
wanted, and a remedy (D16). Releasing the gesture commits the same record
(D21 — judged, not refused). Going to the finding is a zoom (D20).

**Rejected alternatives.** *A live-wire list in the facade* — rejected by
F-3: that is a second in-memory graph of the same edges the composition
already holds. *Refuse to commit a mismatched wire* — rejected by D21's
stance: the edge may exist and derivation runs as far as it can. *Put the
wire on a store hyperedge so M7 can answer staleness* — rejected for this
stage by R27; C4 does not read M7, and D38 already records the cost of
keeping staleness in the facade.

**What it costs.** The graph being wired is one composition address. A
gesture that draws many edges still amends that one record. Session camera
is held on the store handle, not as a durable space; a restart does not
keep the zoom-to-finding.

---

## D40 — Tier 0 carries the primitive registry · **locked** · 2026-08-21

D28's contract is `compile(&Plan) -> Artifact`. Lookups have to be hoisted
*at compile*, and `invoke` does not take `Blocks`. The backend therefore
holds the primitives it was constructed with. That is the same startup
registry `Blocks` already is, not a second graph.

Tier 0 is the resolved plan: no compiler, no toolchain, no new crate
dependency. It registers by passing `binding::check` — one function, every
plan in the corpus, no per-backend test. The editor's linked plan is in
that corpus; the steps that are a pure function of their inputs (`offset`,
`displace`) are what D19 can compare bit-for-bit. `read` / `amend` /
`commit` are store effects, so they compile but are not the equivalence
sample.

**Rejected alternatives.** *Add `Blocks` to `compile`* — rejected: it would
rewrite D28's contract for the first backend. *Generate Rust (tier 1)* —
rejected as E9's form; the first compiled form is required to need no
compiler, which is the honest test of the harness. *A live-wire compiled
cache in the compositor* — rejected by L4.

**What it costs.** Placement of tier 1 / WASM is still a runtime decision
from cost declarations, and those backends do not exist yet. `accepts`
refusal is reachable for an unknown native key; a plan of only known keys
is never silently stuck on interpret for lack of tier 0.


## D41 — A status line names the test that could fail · **locked** · 2026-08-22

Every stage table in this repository grows a **`Verified by`** column naming the test
function, by name, that fails if the stage's claim is false. A stage may not be marked
`landed` while that cell is empty, or while it names a check that cannot fail for the
reason the claim would be false.

**What forced it.** Nine stages of `docs/plans/EDITOR-BOOTSTRAP.md` were marked
`landed`, including the deliverable, and the application had never drawn a pixel.
`src/facade/ports/surface.rs` ended `let _ = (_format, verts);` and no adapter, device,
pipeline or render pass existed anywhere in the repository. E3's green check read *"one
space on screen"* and was satisfied by `tests/agreement.rs`, which asserts that
`visible(&view)` and `View::embedding()` agree over eighty sample points — pure `f64`
arithmetic, in which the word *surface* means `SurfaceRect`, a struct of three points.
`PRESENTER.md` S8 read *"a real wgpu `Surface`"* for four stages while naming exactly
one `wgpu` type and using none.

R23 already required a claim about rendering or interaction to state its verification
method, and every stage did state one. **The rule was satisfied and the defect
survived**, because nothing required the method to be capable of failing. This decision
is R23's missing second half.

The column is not paperwork. Fill E3's in and the mismatch is legible to anyone reading
the table: the words say *one space on screen*, and the test named beside them is
arithmetic. The audit becomes a thing you can see rather than a thing you have to do —
which matters, because the four green checks that passed here were each reviewed, and
each read as adequate.

**Rejected alternative:** a reviewer checklist, or a convention that rendering claims
get extra scrutiny. Rejected because the defect was not that nobody looked. It is that
looking could not distinguish the two cases, and a checklist does not change that.
`PRESENTER.md` §11 already holds the sibling discipline for `check-rules.sh` — break
it, watch it fire, restore it — and this extends it from the rule checks to the stage
table, which is where it was needed and did not reach.

---

## D42 — The portal owns the device; the facade owns the drawing · **locked** · 2026-08-22

`src/portal/` owns the `wgpu` instance, the adapter, the logical device, the queue and
the swapchain. `src/facade/ports/surface.rs` owns the pipeline, the buffers, the
encoder and the render pass, and borrows the device it was attached with. The adapter
and device are resolved once, in the event loop's `resumed`, never on the tick path.

**What forced it.** D32's table permits `wgpu` in both places, and before E10 the
device lived in the portal while the only thing that needed a device lived in the
facade — with **no path between them**. That is why `Device::instance` had no callers.
The seam had never been drawn because nothing had ever drawn.

The division is D18's: a swapchain is a property of a window, a window is an operating
system object, and the portal is the seam with the operating system. Drawing is not.
Same organization/API split as D29, one level further down.

**`scripts/check-rules.sh` settles it, and that is the argument worth keeping.** The
`f32` check is `find src -name "*.rs" ! -path "src/facade/ports/surface.rs"` — it
covers the portal too. Under this decision the portal holds a window, a device, a queue
and a texture view, and never a float; every vertex, every clear colour and the whole
WGSL shader stay in the one exempt file. A rule check that was written for a different
reason turns out to encode the answer.

**Also decided here: `Surface::attach` takes no geometry.** `SurfaceRect` is the
presenter's, `src/portal/` may not name a layer crate (R2), and the geometry already
has one home in `Store::set_surface` (D43). The renderer is handed it every frame by
`Store::draw_with`.

**Rejected alternatives.** (a) The facade owns every `wgpu` object and the portal hands
it a raw window handle — rejected because the facade would name `raw-window-handle` and
own swapchain resize, which is an OS event arriving on the portal's side; it puts an
OS-shaped resource behind the wrong seam. (b) Both keep what they have and something
ferries between them — two owners for one device, which is F-7's shape, and it fires
the `f32` check the first time the portal touches a colour. (c) An async tick, so the
device could be requested lazily — rejected by R8 and L1: the runtime owns no thread
pool and no executor, and D24's whole argument is that the input path never waits.

---

## D43 — The surface's geometry has one home, and `/input/surface` is not it · **locked** · 2026-08-22

`Store::set_surface` is the only place the drawable rectangle is set. The portal calls
it on `Resized` and on `ScaleFactorChanged`. `portal/input.rs` continues to amend
`/input/surface` in the pending set, and **nothing reads it**; that address is
reserved, not live.

**What forced it.** `set_surface` had five callers and all five were tests, so the
running binary placed every frame against the 800×600 default whatever size the window
was (`EDITOR-BOOTSTRAP.md` §9 finding 12). The bootstrap plan's §3 promised that every
OS event becomes an amend at a well-known address that the composition reads as an
ordinary input, and for the surface that promise was never kept.

**Rejected alternative:** keep the promise — have the tick read `/input/surface` from
the pending set and apply it, so a composition could react to a resize. Rejected for
now because nothing needs it, R27 makes an unrequired capability a defect, and building
both is two write paths for one fact. **The trigger for revisiting is a composition
that needs to read the surface size**; at that point this decision is superseded rather
than amended, and `set_surface` becomes the tick's, not the portal's.

**Also decided here: the scale factor is divided out once.** `winit` reports pointer
positions in device pixels; `SurfaceRect::size` is logical and carries `scale_factor`
separately. `portal/window.rs` divides, and everything above the portal is logical.
The multiplication back to device pixels happens in the shader, in the one file allowed
to hold an `f32`. One narrowing point, one scaling point, both named.

---

## D44 — A style row carries its own name · **locked** · 2026-08-22

`encode_style` takes a name as well as a fill and writes both. The app binds the style
table's address range with `Store::bind_styles`, and `Store::draw_with` resolves each
`Placeable`'s opaque style key against that table and hands the facade's `Surface` a
map from **address** to fill.

**What forced it.** A space record carries a style *key* — the string `"plain"` — and
the style table is addressed by *address*. Nothing joined the two.
`editor::styles::bootstrap_default` had exactly one caller, genesis, encoding the row
it wrote; no draw path ever read a style, so a renderer would have had no colour to use
(`EDITOR-BOOTSTRAP.md` §9 finding 13).

**`Placed` still carries no style, and that stays true.** `PRESENTER.md` is explicit
that a colour has no place in a geometry record, and `hyper-ui`'s `SceneNode` is the
counter-example: no room for selection except `selected: bool` inside the geometry, so
selecting a thing means re-deriving and re-uploading it. The resolution happens in the
facade, which already holds the `SceneSet` the placement was built from.

**The facade must not name the editor** (R2: `layer → platform facade → domain facade →
app`), so the name cannot be resolved by the facade knowing which address `"plain"`
lives at. Putting the name in the record is what makes the table self-describing, and
it has a second payoff: a new style becomes a new record rather than a recompile, which
is the self-hosting claim applied to appearance.

**Rejected alternatives.** (a) Copy `Placeable::style` into `Placed` — contradicts
`PRESENTER.md` §4 explicitly and needs a decision amending a locked layer. (b) Have the
facade's `Surface` hold the store and read a style row per placed thing per frame — a
store read on the draw path, per thing, per frame; F-7's neighbourhood. (c) Have the
editor hand the facade a resolver closure at `bind` — puts a function where a record
should be, and it would not survive E4's discard test as data.

**Cost, stated:** the style record's layout changed and there is no migration
machinery (D34's cost, again). Acceptable only because no data has shipped. The style
table is read as a `Vec<(String, [f64; 4])>` rather than a map, because L5 forbids a
map keyed by anything but an address and `check-rules.sh`'s `maps_keyed_by_addr`
enforces it; the table is a handful of rows and a linear scan is honest where a lookup
structure would be the letter against the spirit.

---

## D45 — An address carries its significant length; apparent size decides descent · **locked** · 2026-08-28 · closes O23

Two changes, one decision, because either alone leaves the claim untrue.

**(1) `infinite_presenter::core::Addr` gains a bit length.** `Addr::with_bits(bytes,
bits)` is the constructor the facade uses; `Addr::new(bytes)` keeps the old meaning
(significant to the whole byte length) for the pure core's own callers.
`prefix_bits()` returns the carried length, `truncate` sets it, and `contains` is
**bit**-prefix containment over it.

**(2) `place_group` descends on apparent size, not on a bit comparison.** A
`hosts_space` child is entered when its extent on the surface reaches
`View::opening_extent` device pixels — a property of the view, defaulting to 256, set
by the caller for the reason `View::margin` is. `detail_override` still holds a space
open or closed against that default, one step per doubling, exactly as `detail` reads
the same field, so D5 and D20's *"detail is per space, not per camera"* survive intact.

**Also decided here: the editor's well-known keys are a hierarchy, one nibble per
level, with no level's nibble zero.** Four bytes wide, most-significant nibble first,
top nibble the region. `facade::significant_bits` recovers a key's depth from its last
non-zero nibble; `a_well_known_key_is_a_hierarchy` in `tests/genesis.rs` is the check.

**What forced it.** `Inner::coord`/`bytes_of` map every address the facade hands the
presenter to exactly four bytes, so `prefix_bits()` came back 32 for the screen root,
the canvas and a node alike; `contains` was satisfiable only by equality; and
`place_group`'s guard, `level > item.at.prefix_bits()`, compared that 32 against a
level clamped by `surface_floor` to roughly 9–12. The guard could never fire, for any
genesis, at any depth — `EDITOR-BOOTSTRAP.md` §9 finding 19 and O23. D20/D31, *"a
space contains nodes, and a node may itself host its own space; zoom reveals it"*, is
the platform's stated thesis and had never once been exercised.

**Verified by** `tests/nesting.rs`. Written before the fix and seen to fail for the
right reason: with the nested fixture seeded and nothing else changed,
`a_closed_space_does_not_show_its_interior` reported five flat siblings at the resting
camera — `[10 00 00 01], [10 00 00 10], [10 00 00 11], [10 00 00 12], [10 00 00 20]` —
and `the_address_of_an_interior_node_says_it_is_interior` failed on *"the canvas
contains node A"*.

**Rejected alternatives.**

*(a) as `E11-NEXT-STEPS.md` §1 wrote it — variable-length byte addresses past the
facade boundary, with no change to the descend rule.* Half right, and the half it
leaves out is the half that matters. A nesting level would cost eight bits, one bit is
one doubling of zoom, so entering a space would take 256× magnification and entering
two would take 65 536×. The claim would be true and unusable, which is a worse
outcome than false and known-false.

*(b) an authored `depth` field on the space record.* The plan called this a future
liability and it is: depth becomes a fact someone must remember to set correctly
rather than a structural property, which is `hyper-ui`'s failure mode, and a property
inspector reading depth from an address (as the specification says it may) would be
reading a different number from the one layout used.

*(c) accept flat addressing and reframe D20 — make entering a space an explicit
navigation action.* Declaring the original architectural bet void, and the bet is the
answer to *"why does visual programming scale"*. Only worth taking if (a) proved
disproportionate, and it did not.

*(d) keep the bit comparison and widen the surface floor.* The floor measures how much
detail the surface can resolve, which is a real quantity; making it lie so that an
unrelated guard fires is F-7's habit of adjusting the measurement to suit the
consumer. Address depth answers *who is inside whom*; apparent size answers *when can
you see in*. They are different questions and one test cannot be both.

**Cost, stated.** Every well-known address changed, and there is no migration
machinery (D34's cost, a third time) — acceptable only because no data has shipped.
`Addr` grew a field, which is a change to a locked layer's core type: R29 says such a
change is corrected rather than merged, so it is raised as this record rather than
folded into E11's work. Nesting depth is capped at seven levels below a region by the
32-bit key, and the way past that is the store's own key width, not another scheme
here. **O13's trigger has fired** — `presenter_addr` is no longer a newtype wrap —
and the promotion of `Addr` to its own crate is deferred with the trigger restated
under O13.

---

## D46 — The presenter authors the grouping; a primitive is an opaque key · **locked** · 2026-08-28 · closes O20

`Placement` grows `batches: Vec<Batch>`, a partition of `placed` into contiguous runs
that share a primitive. `Batch::primitive` is a `Box<str>`. `Placeable` grows
`primitive` (authored, defaulting to `rect`) and `link: Option<(Addr, Addr)>`; `Placed`
grows `span: Option<(Point, Point)>` — the two surface points a link runs between,
because a bounding box cannot say which diagonal a line takes. The facade selects a
pipeline per batch and invents no grouping of its own.

**What forced it.** D15 and D29 both give this layer *"what is uploaded, in what order,
at what detail, **grouped how**"*, and D29 leans on that last phrase to argue the
facade owns only the API. `Placement` was a flat `Vec<Placed>` with one implicit
pipeline and no way to say it (finding 16). With one primitive the gap was invisible
and the split held by luck; E11's wires are the second primitive, and at that moment
either the artifact says how things group or the facade works it out — which is
`hyper-ui`'s failure relocated rather than avoided.

**Verified by** `tests/wires.rs::the_placement_groups_the_wire_apart_from_the_rectangles`
and `::off_the_line_is_still_the_canvas`. The second was seen to fail, with the link
batch routed to the quad pipeline, as `channel 0 was 242, wanted 31` forty pixels off
the line — a bounding box where a line should be.

**A string and not an enum, deliberately.** The set of primitives is open by
construction: a block author publishes a new one. R16 makes a closed enum a defect
wherever the set is open, F-1 counts five prior occurrences, and R29 names an added
enum as the class of proposal to correct rather than merge. R4 already says layers
reach each other through string keys. An unknown key falls through to the area
pipeline rather than drawing nothing, because a key with no pipeline and an empty
screen must not look alike (`PRESENTER.md` §13 finding 8).

**Rejected alternatives.** (a) `Placed { kind: PrimitiveKind }` — the enum above, and
the shape the plan's own O20 entry leaned towards; rejected on R16 rather than on
taste. (b) A second list, `Placement::links`, parallel to `placed` — F-1 wearing a
different hat: a field per primitive closes an open set just as firmly as a variant per
primitive, and a third primitive would need a third field. (c) The facade groups the
flat list itself, and D29 is amended to give it batching as well as API — the split
D29 exists to draw, quietly moved.

**Cost, stated.** `Placed` gained an `Option` that is `None` for every area, which is
a variant living inside a struct and is not free. It buys one shape for culling,
clipping and hit-testing across every primitive — `rect` stays the bounding box and
only the shader reads `span` — which is what keeps `probe` from needing a case per
primitive. A link is placed after the areas in its group, so it can read where its
ends landed; a link whose end is not on screen is not placed at all, which is the
honest answer and not a line to nowhere.

---

## D47 — `frame` is retired; the binding hands back the scene it composed · **locked** · 2026-08-28 · closes O21

`infinite_presenter::binding::frame` is removed. `binding::compose(scene, view, at) ->
(SceneSet, Placement)` replaces it, and `Store::draw_with` and `Store::place_now` are
its callers. Submitting is deliberately not in it: only the caller can resolve a style
key to a fill, and this crate may not name the app.

**What forced it.** `frame` resolved its own `SceneSet`, placed it, submitted, and
dropped the set. D44's fill resolution needs the set the placement was built from and
D46's batching needs it too, so `Store::draw_with` took the three steps itself and
`frame` lost its last caller — finding 17, and R27 makes an uncalled binding function a
defect. Two consumers now want *"a placement and the scene it came from"*, which is the
threshold `E11-NEXT-STEPS.md` §3 named for reviving it properly rather than deleting it.

**The name is retired, not reused** (R17). `frame` named a function that also
submitted; `compose` names one that does not, and reusing the word for the second
shape is precisely the recurrence R17 exists for.

**Rejected alternative:** delete the file and leave the facade duplicating four lines.
Honest, and it would have left the presenter's binding with no path from a `Scene` to a
`Placement` at all — so the next consumer would write the fourth copy.

---

## D48 — Undo is a commit; the pending set is discarded, not undone · **locked** · 2026-08-28 · closes O16

Four parts, and each answers one of the questions `E11-NEXT-STEPS.md` §4 asked.

1. **Undo operates on committed history.** It writes the previous value as a **new
   commit**. It never rewinds a revision.
2. **The pending set is not in the undo stream.** Abandoning a gesture in progress is
   `discard`, a different verb on a different gesture, and R13 already gives the
   pending set the enumerate-and-commit boundary it needs.
3. **The camera is therefore outside undo by construction, not by exception.**
   `pan_by` and `zoom_by` amend `CAMERA_KEY` and nothing ever commits it (D5: the
   camera is session-scoped; the journal replays it, the store never holds it). A
   pan cannot enter a stream it never reaches.
4. **The undo stream is a registered derived artifact** (D25, R12), a pure function of
   the store's commit history above a session watermark. Drop it, rebuild it, get the
   same stream. **No fourth state category** — D8's Stored / Derived / Pending stands.

**What forced it.** O16 has been open since before the editor drew a pixel, and E10.5
sharpened it: the camera became a pending record, exactly like a drag in progress or a
half-drawn wire, and none of it had an undo story. Every new authored-state kind —
positions, then camera, now wires, eventually text — makes retrofitting more expensive,
and the plan's instruction was to decide the shape before E13 adds more.

**Why the commit boundary is the right seam.** It is the only line in the system that
already means *"the person finished saying it"*. Everything below it is a gesture and
has one verb (discard); everything above it is a fact and has another (undo). The
alternative — a policy per record kind, which the plan floated — is that same
distinction, restated as a table someone has to maintain, and the table would be
wrong the first time a new kind is added and nobody updates it.

**Rejected alternatives.**

*(a) Undo discards the last pending amend.* Cheapest, and useless: a drag commits on
mouse-up, so by the time a person wants it back there is nothing pending. It also
gives one keystroke two meanings depending on invisible state.

*(b) Undo rewinds to the previous revision.* Makes provenance lie. The charter's
production properties — audit from computation provenance, observability from the
derivation DAG — all assume revisions are append-only, and R12's discard test rebuilds
*from* history. An undo that edits history is an undo that edits the audit trail.

*(c) The undo stack is an authored record.* It would survive a restart, and R10's
membership test says a thing that only means anything while something is running
belongs to the runtime. An undo stack from last Tuesday is not something anyone wants
to press Ctrl-Z into.

*(d) Undo is a branch, per the charter's "rollback | branch merge".* Branches are the
right mechanism for a *deliberate* alternative line of work and the wrong one for a
keystroke: a branch per undo is a branch per typo.

**Cost, stated.** Undoing a delete needs the tombstoned value, which is readable at the
previous revision — no new storage, one extra read. Two sessions editing concurrently
get one stream each, because the watermark is per session; cross-session undo is out of
scope until multi-user is, and `STATUS.md` already lists that under Not Yet. **This
record is the decision and not the implementation**: E12's stage plan is owed, and
under R20 no status line for it is written until the change that lands it says so.

---

## D49 — A text run lives on the space record · **locked** · 2026-08-28 · opens O26

**The string is a field beside `style` and `primitive`, not a record the space points
at.** O26's option (a), taken until a run is long enough that re-reading it per frame
is measurable.

**What forced it.** E13.0 needs somewhere to store the run before E13.3 writes one
through the composition. A separate addressed record would work but adds indirection
and a dangling edge for every label; a panel is already a space, and a label is already
a primitive on that space (§2.1).

**Cost, stated.** Every space record carries an empty string for almost all rows. The
layout grows at the end of `encode_space`, so genesis written before E13.0 still
decodes with `text` defaulting to empty.

**Verified by.** `tests/text.rs`.

---

## D50 — Selection is one authored address · **locked** · 2026-08-28 · opens O27

**The selection record holds a single store key, not a set.** O27's marquee question
is deferred: one address is enough for an inspector, and retrofitting a set is cheap
only if the record is a set from the start — which R27 says not to build until
something selects two things.

**What forced it.** E13.1 needed a committed fact that survives restart and is written
through the behaviour composition on pointer release, without putting `selected: bool`
on `Placed` — the `hyper-ui` breakage this stage exists to not become.

**Where it lives.** [`SELECT_KEY`](src/editor/addresses.rs) in region 5 beside the
session camera, encoded as `SL1` + four key bytes in [`facade/record.rs`](src/facade/record.rs).
The write path is `off-gate` → `encode-selection` → a second `amend`/`commit` pair in
the behaviour composition; the portal only emits a one-shot [`RELEASE_PULSE_KEY`](src/editor/addresses.rs)
on button-up.

**Verified by.** `tests/selection.rs`.

---

## D51 — The inspector reads through the scene port · **locked** · 2026-08-28

**Property fields come from `Scene`, not from decoding store records in the panel
code.** `Store::selection_view` resolves the selected address against the same
`SceneSet` the canvas places from, and depth is `prefix_bits / 4` on the presenter
address (D45) — not a separate authored field that could drift.

**What forced it.** E13.2 is the first UI that displays authored geometry beside
the canvas. Reading `SpaceRecord` bytes directly in `editor/inspector.rs` would be
a second decode path and the beginning of the inspector naming store shapes (R2).

**Where it lives.** [`INSPECTOR_KEY`](src/editor/addresses.rs) and six text-primitive
children; [`editor/inspector.rs`](src/editor/inspector.rs) refreshes their runs from
[`SelectionView`](src/facade/present.rs) after each behaviour tick.

**Verified by.** `tests/inspector.rs`.

---

## D52 — Inspector writes go through the behaviour composition · **locked** · 2026-08-28

**Property edits amend gesture addresses only; the selected space is written by the
interpreted composition.** `editor/inspector.rs` may call `store.amend` on
[`EDIT_ORIGIN_KEY`](src/editor/addresses.rs) and [`EDIT_COMMIT_KEY`](src/editor/addresses.rs)
— never on the selected node's key. Five new behaviour blocks (`read`, `set-origin`,
`gate`, `amend`, `commit`) patch origin when the commit pulse is set.

**What forced it.** E13.3 is the second input surface (after the canvas). Calling
`store.amend` on the selection from panel code would be F-7's shape — two write paths
for one fact — and would bypass E6's discipline.

**Where it lives.** [`set-origin`](src/facade/ports/blocks.rs) primitive;
[`apply_origin`](src/editor/inspector.rs); edit path in
[`genesis.rs`](src/editor/genesis.rs) and [`run.rs`](src/editor/run.rs).

**Verified by.** `tests/inspector_write.rs`.

---

## D53 — The editor mints child addresses · **locked** · 2026-08-28

**New blocks receive the next free child nibble under the drop target (O28
single-session answer).** [`mint::next_child`](src/editor/mint.rs) scans the screen
range; the palette drag path writes [`PLACE_ADDR_KEY`](src/editor/addresses.rs) in
[`run.rs`](src/editor/run.rs) and the behaviour composition amends/commits the record
there — not `store.put` from panel code.

**What forced it.** E13.4 is the first gesture that creates geometry. Without D45's
nibble-per-level scheme, "under the parent" is meaningless; without editor-side minting,
the facade would need to know the editor's allocation policy (R2).

**Where it lives.** [`PALETTE_KEY`](src/editor/addresses.rs) panel and
[`PALETTE_PLAIN_KEY`](src/editor/addresses.rs) template in genesis; place chain in
[`genesis.rs`](src/editor/genesis.rs).

**Verified by.** `tests/palette.rs`.

---
