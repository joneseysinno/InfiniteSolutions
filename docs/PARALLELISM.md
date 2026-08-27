# Infinite Solutions — Parallelism

> **Status:** draft 1, 2026-08-25. **Records nothing.** Every numbered item below is a
> *candidate* decision written by an assistant, and R29 says a proposal is corrected,
> not merged. Nothing here is locked until a change lands it and stamps it (R20, R22).
>
> Cross-cutting: touches all four layers plus the facade and the portal. Rules:
> [`RULES.md`](./RULES.md) · Decisions: [`DECISIONS.md`](./DECISIONS.md) · Charter:
> [`CHARTER.md`](./CHARTER.md)
>
> Written under R28: the idea is fuzzy, so the deliverable is a specification and not
> code. Written under R18: theory before anything gets built.

---

## 0 · The claim, in one sentence

> **This system is already parallel-shaped. Nothing new has to be invented — two
> places throw away information the design already computed, and one law has to be
> restated so that it says what it actually means.**

That is the whole document. Everything below is the argument for it, the price, and
the checks that would catch it being wrong.

The reason it comes out this way is not luck. Three things were decided for other
reasons — provenance is the **exact declared** input set (D11, D38), graphs are
**acyclic** by default (D21), and interpreted execution is the **specification** with
an equivalence harness as the registration gate (D19, D28) — and those three are, in
order, the dependency test, the schedulability condition, and the correctness proof
that a parallel executor needs. They were bought for staleness, for legibility, and
for compilation. They also buy this.

---

## 1 · "Parallel" is four asks, not one

*Run any or all of the graph in parallel* decomposes into four different requests
with four different homes. Separating them is most of the work, because three of the
four turn out to need no new platform concept at all.

| # | The ask | Grain | Home |
|---|---|---|---|
| **P1** | Two steps in one plan that do not depend on each other run at the same time | step | compositor emits it, runtime schedules it, driver executes it |
| **P2** | Two whole compositions — a solve and a screen redraw, two open spaces — run at the same time | composition | **the same mechanism**, because composition closes (D14.6) |
| **P3** | One block applied to a million elements at once | inside a step | **not the platform's.** Either inside the block (L3) or a registered backend (D28) |
| **P4** | Work runs on another core, another process, another machine | placement | **already decided.** D18: the author draws an edge, the runtime decides whether it is a call, an IPC message, or a round trip |

**P3 is the boundary worth stating loudly**, because getting it wrong is how the
compositor grows math. The platform parallelizes *between* blocks and never *inside*
one. A `map` primitive, a `reduce` primitive, a parallel-`for` node — all of them are
R31 violations wearing a scheduler's clothes, and all of them are F-1 (a closed set of
combinators standing in for an open one). §9 shows that P3 already has an authored
form that needs no primitive.

**P4 is the out-of-the-box half of the answer.** Multi-core is the degenerate case of
multi-machine with a transport cost near zero. Design *placement* and threads fall
out; design *threads* and you will have to redesign when the second machine arrives.
D16 already made this move once — *"this runs on the server"* is a placement decision
rather than an architectural one — and D28 made it again for compiled forms. This is
the third instance of the same move, and by this project's own standard three
instances is a pattern rather than a coincidence.

**R32, applied before going further.** A concept is platform only if both planned
consumers need it. SES needs P1–P4 obviously. Coach Assistant needs a practice plan
evaluated across twelve players, a season across thirty games, a drill library scored
against a roster — which is structurally the same fan as a quadrature rule over
integration points. Different vocabulary, identical shape. Parallelism passes R32.

---

## 2 · What is already decided, and what each piece turns out to be

Six locked decisions, each recorded for a reason that had nothing to do with
concurrency, and what each one is when read as a parallelism decision:

| Decided | For | What it also is |
|---|---|---|
| **D11 / D38** — provenance is the **exact** declared input set, recorded per step | staleness | **the dependency test.** Two steps' address sets are all a scheduler needs |
| **D21** — acyclic by default; cycles only inside a marked region, and a region is one step from outside | legibility; a spinning app is the worst failure | **the schedulability condition.** A DAG *is* a partial order. The scheduler never sees a cycle, so it can never deadlock on one |
| **D19** — interpreted execution is the specification; equivalence is exact, not statistical | verifiable compilation | **the correctness proof.** "Is the parallel run right?" is already a mechanized question |
| **D28** — the compiled form is a registered backend with a cost declaration; registration is *passing the harness* | refusing to name one compiled form | **the delivery vehicle.** A parallel executor is a backend. GPU and store-pushdown were already named as members of the same open set |
| **D25** — the runtime knows artifact lifecycle, never content | one generic discard harness | **the unit of independent work.** A registered artifact declares the ranges it derives from — which is a dependency declaration for something that is not a plan step |
| **D18** — a boundary is a portal; the runtime decides call vs. IPC vs. round trip | multi-target without a build matrix | **P4, already written** |

And two purity decisions that were bought for testability and pay again here:

- **L4** — the compositor is a function, not a place. It holds nothing across a call,
  owns no mutable state, has no interior mutability. So `link` is trivially callable
  from many threads at once.
- **L6** — the presenter authors nothing and has no write port; `place` takes `&SceneSet`,
  `probe` takes `&Placement`, and there is no `&mut` in either signature. So placement
  is trivially callable from many threads at once.

Both were argued from `hyper-ui`'s untested renderer and its `arrange` writing back
into the model it was laying out. Neither mentions threads. Both are the reason the
two most expensive pure functions in the system are already safe to run concurrently.

> **Purity was bought for testing, and it silently bought parallelism.** That is the
> single most useful sentence in this document, because it means the cost of this
> capability was already paid and the only question left is whether to collect.

---

## 3 · The two places the information is thrown away

### 3.1 `Plan` is a total order, and the partial order it came from is discarded

`COMPOSITOR.md` §9.2: *"A deterministic sequence of steps."* `core/order.rs` computes
a topological sort. A topological sort is a **projection** of a partial order onto a
line, and the projection is lossy — after it runs, "step 7 must follow step 3" and
"step 7 merely happens to be printed after step 3" are indistinguishable.

The fix is not to make the plan concurrent. It is to **stop deleting what link
already computed**:

> **A `Step` carries the set of steps it waits on. The canonical linearization is
> derivable from the plan and must remain so, because it is D19's specification.**

Read that carefully, because it is the whole of P1: *this is not adding parallelism.
It is ceasing to discard it.* `order.rs` already builds the dependency relation in
order to sort by it. Keeping it is a smaller change than any scheduler.

Two consequences fall straight out:

- The linearization must be **reproducible from the partial order** — same DAG, same
  line, every time — or D19's equivalence harness has an ambiguous baseline. Tie-break
  by address order, which is total (COMPOSITOR §3.2) and already required to be so.
- The closure test (§7.3) is unaffected: wrapping a linked composition as a block and
  re-linking must still produce an identical plan, now including identical dependency
  edges. That check gets *stronger* for free.

### 3.2 `tick` decides *and* does, in one call

`RUNTIME.md` §7.1: `tick(&mut self, now, budget) -> Outcome`. Today that call chooses
the work and performs it. Every parallel design dies at this line, and the usual death
is to relax **L1 — the runtime owns no thread pool** so `tick` can fan out internally.

Do not do that. L1 is not a stylistic preference; the runtime is the layer where all
three previous attempts died (D2), and D4 wrote L1 down precisely because the crate's
own name invites the opposite. §4 is the alternative.

---

## 4 · The law, restated so it says what it means

> **L7 (candidate) — The runtime decides what may run. It never runs it.**
>
> `tick` returns a **ready set**: the steps whose dependencies are satisfied and whose
> address ranges conflict with nothing in flight, in priority order. The driver takes
> as many as it wants, executes them however it likes, and hands the results back on a
> later tick.

This is not a weakening of L1. It is L1 finally being literal. *"Cadence in, work
out"* was always the sentence; today `tick` reads it as *"cadence in, work done"*.
A ready set is **work out**.

What it preserves, mechanically:

- **The runtime stays single-threaded by construction.** It holds the frontier, the
  pending set and the artifact registry, and nothing else touches them. No `Mutex`, no
  `Arc`, no `Send`/`Sync` bound anywhere in the layer. `check-rules.sh`'s existing
  greps for `thread`, `sleep` and an executor dependency keep passing **unchanged**,
  and gain three siblings (§14).
- **The scheduler becomes unit-testable with zero threads.** Assert the ready set
  directly. This is D23 / D26 / D29's argument — *a layer whose central function
  cannot be exercised in a unit test is a layer whose central function is untested* —
  running for the fourth time, and it is the reason to prefer this shape even if
  nothing is ever executed in parallel.
- **R11 holds**: a ready set is addresses and step identities. No records.

**The worker pool lives in `src/portal/`.** D42 already argued the placement for a
different OS-shaped resource: the portal owns the `wgpu` instance, adapter, device,
queue and swapchain because *a window is an operating system object and the portal is
the seam with the operating system*. A thread pool is an operating system object. Same
seam, same argument, no new reasoning required — and `src/portal/` may not name a
layer crate (D32), so the pool physically cannot leak into one.

**Rejected: a sixth port, `Workers`.** D23 permits a sixth port with a decision record,
so this was available. It is worse. A `dispatch(work) -> handle` trait behind which the
facade hides a pool is L1 violated by indirection: the runtime would once again own a
pool, just one it cannot see. And the check degrades exactly the way D23 said it would
when it rejected *"depend on `infinite-db` for its types only"* — R8 stops being a
manifest grep and becomes a judgment about what is behind the trait. A ready set is
**data**; a port is **capability**. Return the data.

---

## 5 · The condition, and why it costs nothing

Two steps may run simultaneously iff:

1. their declared **output** ranges do not intersect, and
2. neither's declared **output** range intersects the other's declared **input** range.

That is Bernstein's condition, and it is stated here in addresses because addresses are
what the system has. Three properties already relied on make it cheap:

- Every declaration is a **range**, not a list, because a subtree is contiguous in key
  order **by construction** (O2, D20) rather than by luck.
- `Addr` is **totally ordered**, so range intersection is interval overlap on a byte
  key — and the runtime's `Addr` already carries `in_range` and prefix truncation.
- The declaration is **exact**, because D38 requires it for staleness and §6.2's
  `not-pure` finding already fires at link time when a composition reads something it
  did not declare.

> **Parallel scheduling reduces to interval arithmetic over the addresses the runtime
> already holds.**

The frontier is already a priority-ordered set of addresses. The ready set is the
frontier filtered by one predicate. That is the entire scheduler.

And the reason it is that cheap is the reason addresses exist — the same result D30
recorded for the probe descending in O(depth) instead of scanning: *nobody optimized
it; it falls out of addresses being permanent and locality-ordered.*

---

## 6 · Determinism, which is where systems like this usually die

Three rules, in dependency order.

**6.1 Determinism is a property of the result, never of the schedule.** The plan's
canonical linearization defines the answer (D19). A parallel run is correct iff its
output is bit-identical and its provenance is edge-for-edge identical. Nothing is
promised about the order in which work happens.

**6.2 The parallel executor is a registered backend, so registration is the gate.**
D28 §10.3 already says a backend is registered *by passing the equivalence harness over
a maintained plan corpus*, not because someone wrote it. A parallel executor registers
the same way, with one addition: **the worker count and the dispatch order are part of
the corpus sweep.** Same plan at 1, 2, 7 and 64 workers, plus a seeded adversarial
dispatch order, must produce identical bytes. Worker count must not be observable.

The platform therefore cannot grow a subtly-wrong parallel backend for the same
structural reason it cannot grow a subtly-wrong compiler. That is the second time
D28's registration gate does load-bearing work for something it was not written for.

**6.3 The two things that will actually break it, named in advance:**

*Floating-point fan-in.* Addition is not associative. A fan-in combined in worker
completion order produces schedule-dependent bits and 6.1 fails, silently and rarely —
the worst available failure. So: **an n-ary fan-in declares its combination order, and
the default is address order** (a deterministic tree reduction over the fan's range).
This is a real authoring obligation, and it is the honest cost of this section. It is
also *checkable* rather than a matter of care: a fan-in with no declared order and a
non-commutative combine is a **finding kind** (§6.2 of `COMPOSITOR.md` — the kinds are
open, and this is a good one), not a bug to be found later in a residual.

*Finding order.* D21 says derivation runs as far as it can and emits findings for what
it cannot. With N workers, findings arrive in completion order. The error surface is
the thing a child reads (D16); it must not depend on how many cores they have. So
**findings are sorted by site address before they are presented**, and the S4 corpus
check ("exactly one finding") grows a sibling ("the same sequence, at every worker
count").

**6.4 Non-determinism has exactly one legitimate home** and it is already built:
input arrival, wall-clock, network. Those are the runtime's `now` and the pending set,
and they are outside the plan by construction (D5, D8). Nothing in a plan may observe
them. That is not a new restriction; it is what `not-pure` already means.

---

## 7 · Where N results go, and what stops the machine eating itself

If eight steps finish at once, eight sets of outputs want to be written. This is
where a parallel design normally breaks D6 or D24. Here it does neither, because both
were already answered for a different reason.

**Results land in the pending set, not the store.** D24: the store is written only at a
commit boundary, through a non-blocking `try_insert`, coalesced per address, and what
has not landed stays pending and is retried on a later tick. A completed step's outputs
are exactly that shape. Nothing about parallel completion touches the store's write
queue, so D24's guarantee — backpressure degrades *durability latency*, never *input
latency* — holds unchanged and for the same reason.

**The pending set is the throttle.** R13 makes it bounded and enumerable. So the
scheduler's stop condition is not a new concept: **stop emitting ready steps as the
pending set approaches its bound.** Parallelism needs no backpressure mechanism of its
own, and D24's overflow policy (commit the oldest, never drop) is unchanged.

**Only the runtime touches the pending set, and it is single-threaded.** Workers return
results; the *next tick* accepts them. No lock, no queue discipline, no lost update.

**Reads are free, and this is the load-bearing detail.** A plan runs against a **pinned
revision** — `COMPOSITOR.md` §1: *linking a composition at revision N is a pure function
of the definitions at revision N* — and `StoreRead` already reads *records in an address
range, **at a revision***. A snapshot is immutable. Concurrent readers of an immutable
snapshot need no synchronisation at all. **The door was already open**: the revision
argument is in the port signature today, for purity, and it is what makes read
parallelism lock-free tomorrow.

**Where the cost actually lands: the facade.** The port implementations get called from
many threads at once, so they must be shareable at a pinned revision. That is a facade
change, and R30 says a facade change requires a decision record. The four layer crates
require no concurrency vocabulary whatsoever. This is the correct place for the cost —
the facade is where `wgpu` lives, where `Addr` conversion lives, and where the store's
MVCC is already visible.

---

## 8 · Nesting, and why one flat scheduler is enough

The classic failure of nested parallelism is oversubscription: an outer parallel region
containing an inner one, each sizing itself to the machine, producing N² workers.

**D14.6 removes the problem structurally.** Composition closes: a wired set of blocks
*is* a block with ports. So P2 (whole compositions in parallel) is P1 at a coarser
grain and needs no second mechanism. A subgraph is a step. A step is a step.

That leaves one genuine choice, and it should be named rather than discovered:

> **By default a region or sub-composition is one step and runs to completion on
> whichever worker took it** (D21: from outside, a region is a block). **A
> sub-composition may instead be *flattened* — its inner steps published into the same
> single ready set — when its steps are pure.**

Flattening is a **placement decision**, made by the runtime from cost declarations and
its own measurements, exactly as D28 §10.4 decided which compiled tier is used. It is
not a property of the block and not the author's guess. One flat ready set means one
scheduler, one priority order, and no possibility of N².

---

## 9 · P3 already has an authored form: a fan is a hyperedge

The temptation with data parallelism is to add a `map` block, and then a `reduce`
block, and then `zip`, and then the platform owns a combinator algebra it cannot close.

It does not need one, and the reason is already drawn on the canvas:

- A **hyperedge connects any number of nodes** (D20) and hyperedges are n-ary
  (direction read §3.2).
- **Every wire is drawn** (D22), so a fan is authored, visible, and never inferred.
- A **subtree is contiguous in address order** (O2), so a fan over a subtree declares
  an address **range**, not a list of a million addresses.

Therefore: **one pure block wired to a hyperedge covering a range is N independent
steps whose declared input ranges are disjoint slices of one range.** It is
data-parallel by §5's condition, with no new primitive, no combinator, no closed set,
and a picture a child can point at (D16).

And the payoff compounds: a fan of pure reads over an address range is a *range scan*,
which is D28's already-named **pushdown into the store** backend. The same authored
shape is CPU-parallel, GPU-parallel (a fan over a range is a dispatch grid) and
store-pushable, chosen per placement. Three of the four asks in §1 collapse into one
drawn gesture.

---

## 10 · Placement is the single concept; threads are one of its answers

Collecting §1's P4, §8's flattening and §9's fan:

> **Every question of the form "where does this run" is one question, answered by the
> runtime from cost declarations plus its own measurements, and never by the author.**

The answers, all in the same table, none of them a new mechanism:

| Answer | Existing mechanism |
|---|---|
| inline, this tick | today |
| another worker on this machine | §4's ready set + the portal's pool |
| the GPU | D28's named backend; §9's fan is the dispatch grid |
| inside the store | D28's named pushdown; §9's fan is a range scan |
| another process or machine | D18's portal; the runtime already decides call vs. IPC vs. round trip |

D28 §10.5 already handles the lying-backend problem — *the cost declaration is data
rather than a promise, so the runtime measures and may demote a backend whose measured
cost contradicts its declaration.* That mechanism covers "this step said it was cheap
and took 40 ms" with nothing added.

---

## 11 · What this closes, sharpens, or threatens

**O12 — may an iterative region yield between iterations? — is answered by placement,
not by a yielding scheduler.** O12 exists because *"a fifty-step contact solve that must
complete inside one scheduler pass will block a frame."* Under §4 it does not run on the
frame path at all. The region is one step, the driver hands it to a worker, and `tick`
returns immediately with the frame's work. **A step does not need to yield if it is not
on the thread that has the deadline.** Both `RUNTIME.md` §7.1 and `COMPOSITOR.md` §9.2
were careful to keep O12's door open at no cost; this is what walks through it. O12
should be re-read as a *placement* question and probably closed as one.

**O1 — the hot working set — changes shape.** The measurement (a warm prefix scan of
~1000 nodes against frame budget) can be taken off the frame path entirely, so the
threshold that decides it moves. The measurement is still owed; what it has to clear
is now different.

**B6 — priority is flat — gets a second failure mode.** RUNTIME §2's B6 is *"zoom
rebuilds distant spaces before the one under the cursor."* With workers there is a new
version: every worker is busy with distant, cheap work when a focused step becomes
ready. Two consequences: the ready set is **re-emitted every tick** rather than being a
queue the driver drains, and the driver's contract is *"take up to K, return what you
finished"* rather than *"take everything."* A long step still occupies its worker —
that is the residual, and pre-emption is not on the table (it needs a runtime that owns
threads).

**D41 is the live threat.** Nine stages were once marked landed while nothing had drawn
a pixel, because every green check was satisfiable without the claim being true. A
parallel scheduler is *more* susceptible to this than a renderer: a scheduler that emits
a correct ready set and a driver that runs it one-at-a-time passes every test in §14
except the one that measures wall-clock. **Any stage claiming speed names the
measurement that fails if it is absent**, and the measurement is not "the test passed."

---

## 12 · Rejected, with the reason each one dies

| # | Alternative | Why not |
|---|---|---|
| 1 | **A thread pool inside the runtime** | L1, D4. The runtime is the layer where three attempts died. The pool is an OS object; D42 already placed OS objects in the portal |
| 2 | **`async` through the layers** | R8's manifest grep, and R3's empty `[dependencies]` — a pure core cannot depend on an executor. It also colours every signature in the system to buy concurrency the driver can provide without it |
| 3 | **A sixth `Workers` port** | §4. L1 violated by indirection; R8 degrades from a grep to a judgment, which is D23's exact rejected shape |
| 4 | **Discover dependencies at run time by tracing reads** — "automatic parallelization" | Makes provenance schedule-dependent, so D19's exactness dies and D38's staleness with it. It is also D22 with the sign flipped: dependencies are **drawn**, not discovered, and the ambiguity argument that killed tag-matched auto-binding kills this identically |
| 5 | **A `parallel: bool` on a plan or a block** | F-1's shape, and it contradicts D28 directly — placement is the runtime's decision from measured evidence, never the author's guess |
| 6 | **Speculative execution with rollback** | Requires transactions across steps and makes provenance conditional. Deferred with a trigger rather than refused: revisit only if a measured workload is dominated by a single long serial chain |
| 7 | **A parallel in-memory graph the workers share** | F-3, three prior occurrences, and D6's stated ground — two things that can disagree |
| 8 | **Locking the store's model so writers can be concurrent** | D6/R7: the store is the only writable model and D24 already makes writes non-blocking. Adding locks to solve a problem that was solved by not blocking is F-7's neighbourhood |

---

## 13 · What it costs, stated so none of it is discovered

1. **`tick`'s shape changes**, and everything since E1 is built on the current one.
   This is the single reason to decide it now rather than when a workload forces it
   (§16).
2. **`Plan` grows dependency edges** — a bigger derived artifact, and `order.rs` must
   produce both the partial order and a reproducible linearization from it.
3. **The facade grows a concurrency requirement** (ports callable from many threads at
   a pinned revision), including the fakes, which means D24's saturation test has to be
   re-run at every worker count. R30: this is a facade change and needs a record.
4. **Fan-in combine order becomes an authored property**, because floating-point
   addition is not associative (§6.3).
5. **Findings must be canonically sorted**, or the error surface varies by core count.
6. **Small graphs get slower.** Dispatch overhead exceeds step cost below some
   threshold. That is a *measurement*, not an architecture question — the same
   distinction D30 drew for O1, and it should be recorded as an open measurement rather
   than guessed at.
7. **Debugging a parallel failure is harder than debugging a serial one**, and the
   mitigation is §6's worker-count invariance: any failure that reproduces at one
   worker is a normal bug, and any failure that does not is a determinism bug with a
   test that names it.

---

## 14 · Green checks — every one able to fail (D41)

| # | Check | Fails when |
|---|---|---|
| G1 | **Worker-count invariance.** The plan corpus at 1, 2, 7 and 64 workers, with a seeded adversarial dispatch order: byte-identical outputs, edge-identical provenance, identical finding sequence after canonical sort | anything is schedule-dependent |
| G2 | **Conflict exclusion.** Two steps with overlapping declared output ranges are never in one ready set. Asserted against the scheduler **with no threads at all** | the interval test is wrong |
| G3 | **Linearization reproducibility.** The plan's canonical order is identical across runs and identical to today's `order.rs` output for every corpus plan | the tie-break is not total |
| G4 | **Closure under dependencies.** §7.3's closure test, extended: the wrapped-and-re-linked plan has identical dependency edges, not merely identical steps | nesting loses an edge |
| G5 | **Input latency under load.** D24 §7.4's saturation test, re-run with every worker saturated by long steps: no dropped keystroke, no tick over budget | compute starves the input path — B1 and B6, the reason the runtime exists |
| G6 | **No pool in a layer.** `check-rules.sh` greps `crates/` for `thread`, `Mutex`, `RwLock`, `Arc`, `rayon`, `tokio`, and asserts the pool appears only under `src/portal/` | L1 or L7 is being violated |
| G7 | **The speed claim.** A named plan, wall-clock, N workers versus 1, with the number written down | the scheduler is correct and does nothing — §11's D41 threat |

G6 must be broken deliberately once and watched to fire, per `PRESENTER.md` §11's
discipline. G7 is the one that is easy to skip and is the only check in the table that
tests the actual purpose.

---

## 15 · Candidate decisions, and open items

**Candidates — numbered provisionally, recorded by nobody.** R29: a proposal from an
assistant that adds a concept is corrected, not merged.

- **D45? — The plan carries the partial order; the total order is a projection of it.**
  Forced by §3.1. Rejected alternative: compute dependencies in the scheduler from the
  plan's step declarations — which is re-deriving at run time what link already knew,
  once per tick.
- **D46? — L7: the runtime decides what may run and never runs it; `tick` returns a
  ready set; the worker pool lives in the portal.** Forced by §3.2 and §4. Rejected
  alternatives: a pool in the runtime; a sixth `Workers` port; `async`.
- **D47? — A parallel executor is a registered backend, and worker count and dispatch
  order are part of the equivalence corpus.** Forced by §6.2. Rejected alternative: a
  parallel *mode* on the interpreter, which has no registration gate and therefore no
  proof.

**Open.**

- **O24? — Fan-in combination order.** Default address order; a non-commutative combine
  with no declared order is a finding. Needs the finding kind written.
- **O25? — The dispatch threshold.** Below what step cost is dispatch a loss? A
  measurement, in the manner of O1, not an architecture question.
- **O12 — likely closed by placement** (§11). Needs re-reading, not re-deciding.
- **O16 — undo** is untouched by any of this, but note that a ready set makes
  "cancel work in flight" a thing that exists. Whether undo and cancel are one concept
  is worth asking while both are still undecided.

---

## 16 · What to actually do now, and what not to

**R27 says generality is a defect unless a named consumer requires it, and R19 says no
layer is built without a forcing consumer that breaks if it is wrong.** Applied
honestly here, they say: **do not build a parallel scheduler.**

The editor is the forcing consumer for all three layer specs, and the editor's plans
are a handful of steps. Nothing in the system today runs long enough to be worth
splitting. Building a scheduler now would be a capability with no consumer, which is
F-6, three prior occurrences.

What *is* required now is the same move O10 made for ownership — **doors that are cheap
to keep open and expensive to retrofit**, decided before more code assumes they are
shut:

| Door | State today | What keeping it open costs now |
|---|---|---|
| 1. The plan retains its dependency edges | closed — `order.rs` projects and discards | small: keep what is already computed |
| 2. `tick` separates deciding from doing | closed — `tick` does both | **this is the real cost, and it grows with every stage built on the current shape** |
| 3. A step's declared inputs are exact | **open** — D38 requires it, `not-pure` enforces it | nothing |
| 4. Reads are at a pinned revision | **open** — already in `StoreRead`'s signature | nothing |
| 5. Cost declarations exist per backend | **open** — D28's fourth contract item | nothing |

Three of five doors are already open, and were opened for other reasons. Door 1 is
cheap. **Door 2 is the decision**, and it is the one that gets more expensive every
week — E11 (wires), E12 (undo) and E13 (the authoring surface) all build on `tick`'s
current shape, and each one added is another caller to change.

**The recommendation, therefore:** decide D46 now on interface grounds alone — because
a `tick` that returns a ready set is *better without any parallelism at all* (it makes
the scheduler unit-testable, which is D23's argument for the fourth time) — and defer
the scheduler itself until a named consumer with a real workload exists. That consumer
is already named and already the trigger for the settling-loop scope cut in both layer
specs: **the crane mat.** Same trigger, same reason, one fewer thing to remember.

And before any of it, the check that could fail: **write G1 first and watch it pass
trivially at one worker.** A determinism test written after a parallel executor exists
is a test written to the implementation.

---

## 17 · The shortest version

1. A plan is a DAG. Today it is stored as a line and the DAG is thrown away. Keep it.
2. The runtime should hand out work, not perform it. That is L1 being literal, not L1
   being relaxed.
3. Two steps commute iff their declared address ranges do not overlap — which is
   interval arithmetic over keys the runtime already holds, and the declarations
   already exist because staleness needed them.
4. Correctness is D19's equivalence harness with worker count added to the corpus.
   Determinism is of the result, never of the schedule.
5. Results land in the pending set, which is already bounded, already enumerable, and
   is therefore already the throttle.
6. Threads, GPU, store pushdown and another machine are one decision — placement — made
   by the runtime from measured cost, never by the author.
7. Do not build it yet. Decide `tick`'s shape now, because that door closes a little
   more with every stage.
