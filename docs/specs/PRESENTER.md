# infinite-presenter — Layer Specification

> **Status:** draft 1, 2026-08-21. S3–S7 landed (E3); S8 remains not started.
>
> Layer: **presenter** (D2, D15, D17). Rules: [`../RULES.md`](../RULES.md) · Decisions:
> [`../DECISIONS.md`](../DECISIONS.md) · Charter: [`../CHARTER.md`](../CHARTER.md) ·
> Siblings: [`RUNTIME.md`](./RUNTIME.md) · [`COMPOSITOR.md`](./COMPOSITOR.md)
>
> Satisfies R18 for this layer, and closes `RUNTIME.md` §10 finding 1 — the stale
> `crates/infinite-ux` directory, which D17 renamed and which had no spec.
> Records D29, D30, D31. Opens O14. Adds a third member to O13.

---

## Stage table

| # | Stage | Status | Green check |
|---|---|---|---|
| S1 | This specification | draft 1 | Recorded as D29–D31; reviewed line by line against R3, R5, R11, R15, R23, R27, D15, D20, D25 |
| S2 | Crate skeleton, pure core | not started | `cargo build -p infinite-presenter` with **no features** succeeds, the manifest's `[dependencies]` is empty, and `f32` appears nowhere in the crate |
| S3 | Ports and the fakes | landed | The whole test suite passes against fakes; no crate belonging to another layer, and no graphics crate, is named anywhere in the crate |
| S4 | The embedding | landed | The **agreement test** (§6.4): for random cameras and surface origins, a point is inside the culled range **iff** its projection is inside the surface rect. One transform, used twice, never two |
| S5 | Arrangement | landed | `arrange` runs with no graph, no surface and no store present; the hysteresis sweep (§7.3) crosses each boundary in both directions and settles |
| S6 | Placement and probe | landed | The **self-sufficiency test** (§8.4): `probe` answers with no port in scope at all, in O(depth), over a corpus of overlapping, clipped and collapsed cases |
| S7 | Discard | landed | `Placement` passes the runtime's generic R12 harness (D25) with **no** per-artifact test code contributed by this layer |
| S8 | First real binding, in the facade | landed — see the correction below | The S3–S7 tests pass unchanged against real `infinite-db` and a real wgpu `Surface`, **and `tests/pixels.rs` reads the frame back** |

---

## 1 · What this layer is

The store owns **records**. The compositor owns **structure**. The runtime owns
**time**. The presenter owns **one view of one screen**, and owns it for exactly as
long as that view lasts.

D15 names it the *embedding layer*, and its doctrine was written three years early,
while designing a database. `infinitedb-spatial-layer.md` §6:

> **Address layer (curve-side).** One-dimensional, exact-rational, permanent. All
> identity lives here. Nothing in this layer ever changes after issuance.
>
> **Embedding layer (space-side).** *d*-dimensional, epoch-dependent. Seats drift,
> subtrees are rigidly re-transformed, precision is finite. **Nothing in this layer
> carries identity.**

Membership test, alongside D5's and `COMPOSITOR.md` §1's:

> If it survives a restart it belongs to the store. If it is true of a program before
> it runs it belongs to the compositor. If it is only true while something is running
> it belongs to the runtime. **If it is only true of one view of one screen, it
> belongs to the presenter** — and it is discarded the moment the view changes.

Two laws, carried in the manner of D4's L1/L2 and `COMPOSITOR.md`'s L3/L4:

> **L5 — The presenter mints no identity.** Every reference to a thing is the store's
> address. No id of its own, no handle, no index standing in for a node, and no map
> keyed by anything but an address.
>
> **L6 — The presenter authors nothing.** It has no write port. Camera, collapse,
> selection and focus are read; hover and a drag in progress are the runtime's pending
> set (D8). Nothing about a thing is ever written where that thing's geometry is
> computed.

The layer's whole job, stated once:

> **Given a view, decide which addresses are on the screen, where each one is, and how
> much of each to show — and answer, for a point on the screen, which address is under
> it.**

The second half is not a secondary feature. It is half the layer, it is the half every
prior attempt got wrong, and §8 specifies it with the same care as the first.

---

## 2 · The forcing consumer (R19, R26)

**The editor** — the platform's own graph editor (O11). The same consumer as
`RUNTIME.md` §2 and `COMPOSITOR.md` §2, deliberately, so the three specifications can
be read against each other.

R19 requires the consumer be *something that breaks if the layer is wrong*. Six
breakages. Unlike the sibling specifications, every one of these has a prior occurrence
**in one crate** — `Innovator/crates/hyper-ui`, which is the only serious presenter
anyone in this corpus has built, and which was read line by line while writing this.

| # | If the presenter is wrong | What the person sees | Prior occurrence |
|---|---|---|---|
| P1 | The transform used to cull is not the transform used to draw | Content pops in at the edge of the canvas, or draws behind the tab strip | `renderer/camera.rs`: `screen_to_world` honours `viewport_origin`; `visible_world_rect` probes at window coordinates and does not. The field's own doc comment describes this exact bug being fixed **in the draw path** — and left in the cull path |
| P2 | The presenter holds identity of its own | A deleted node is still hoverable, focusable, or drawn | `pgraph/input.rs` retains `focused` / `hovered` / `pressed` / `scroll_drag` across frames, so `prune(&store)` exists to check four fields against `store.contains` after every rebuild. Dangling-reference maintenance became the router's job |
| P3 | Hit testing needs the store | The pointer cannot be answered at pointer rate, and cannot be answered at all for geometry that is still pending (D8) | `pgraph/render_list.rs`'s `viewport_at(&self, store: &ParticleStore, pos)` — a positional query that takes the store as an argument. The positional structure is not self-sufficient |
| P4 | The placement is a cache that is written to (F-7) | Laying out changes the thing being laid out; running it twice is not running it once | `pgraph/layout.rs`'s `arrange` writes measured `content_extent` and a clamped `offset` back into `ParticleData::Viewport`; `pgraph/input.rs` writes `TriggerState::Hover` into the model on mouse move. This is R5, in the layer R5 was written for |
| P5 | Invalidation is a convention rather than a mechanism | A missed event and clicks land on the wrong thing, silently | `render_list.rs`'s module doc claims *"it is rebuilt only when the structure or layout changes"* — and the type has no dirty flag, no generation, no watermark. The protocol is a comment on an enum variant: *"the host must rebuild layout and the render list"* |
| P6 | Detail is per camera rather than per space | Zoom is all-or-nothing: you cannot hold one space open while the rest stay collapsed | D20 requires the opposite. `hyper-ui` has the ingredients — a demotion ladder and `SizeClass` hysteresis — but drives them from **width pressure in a one-dimensional allocator**, not from zoom, and a grep of the crate for `lod` returns nothing |

**The verification finding, and it belongs at the top of this section.**
`Innovator/plans/HISTORY.md` closes with the observation that *the graph model is well
covered, rendering and pointer interaction are covered by nothing, and that is exactly
where the longest-lived drifts survived unnoticed.* That is a prediction, and this
layer is where it can be checked. Of the 21 `hyper-ui` files read for this document —
2,747 lines — there are **24 tests, in 6 files. `renderer/` has zero. `geom/` has
zero.** The world↔screen embedding, which is the entire subject of this layer, has no
test anywhere in the corpus. P1 is not a hypothetical: it is a live bug that survived
because nothing could have caught it.

*Verification method (R23):* the counts above are from reading the files, not from
running the suite; `hyper-ui` was not built. Four further test files
(`layout/resolve/tests.rs`, `page/layout_tests.rs`, `pod/layout_tests.rs`, and the
unread remainder) are not in the count, which is why it is stated as *of the files
read* rather than as the crate's total. The zero counts for `renderer/` and `geom/`
are complete: every file in both directories was read.

**Scope consequence of choosing the editor alone.** The editor draws a graph: spaces,
hyperedges, text, and chrome. It does not draw a mesh, a field plot, or a
sixty-thousand-element contour. Therefore **the presenter specifies no drawing
primitive beyond the `Surface` port's opaque work item** (§9), and the question of
instancing, batching and buffer residency for large numeric geometry is deliberately
out of scope. The trigger to extend this document is the same as the trigger in both
sibling specifications: the first named consumer with a solve in it (the crane mat).
Recorded so the gap is deliberate rather than discovered.

---

## 3 · The seam: the presenter depends on no other layer (D29)

**The presenter names no crate belonging to another layer, and no graphics crate.** It
declares the ports it needs as traits; the platform facade supplies the
implementations.

*Why.* D3 established that a computation must not depend on the runtime, or the two
become mutual hostages. D23 ran the argument for the runtime, D26 for the compositor.
The fourth run is the one with the most evidence behind it, because the alternative was
built: `hyper-ui` names `wgpu` and `winit` in its core, and the consequence is
§2's finding — the embedding cannot be tested, because testing it would mean standing
up a GPU device and a window. A layer whose central function needs a display to
exercise is a layer whose central function is untested, and this is the layer where
that has already cost the most.

**D15 says the presenter owns *"wgpu resource organization"*, and this does not
contradict it.** The presenter owns the *organization* — what is uploaded, in what
order, at what detail, grouped how. The facade owns the *API*. The seam between those
two is the `Surface` port, and putting it there is what makes the organization
testable without a device.

### 3.1 The ports

**Three, and a fourth requires a decision record.** Plain functional nouns (R15).

| Port | The presenter asks it for | Notes |
|---|---|---|
| `Scene` | what is placed in an address range at a revision: each address's authored extent, its opaque style key, its detail override, and whether it hosts a sub-space | Reads a **set**, resolved against a revision — the same shape as `Definitions` (D26), and for the same reason: the editor must place geometry that is still pending (D8) |
| `Surface` | its pixel size, its scale factor, and its origin; and acceptance of a frame's work | The **only** place a GPU could ever exist. Named in the facade, never here |
| `Glyphs` | the extent of a text run; its raster | The one measurement the presenter cannot make itself. `hyper-ui` proves the cost of pretending otherwise: `char_w = font_size * 0.55`, with the in-source admission *"approximate text extent until glyphon measures precisely"* |

**There is no `Clock`.** Stated positively so it stays true: *the presenter has no
`now`*. Hysteresis (§7.3) is a function of zoom, not of time. Animation, when a
consumer asks for it, is the runtime driving `place` with a changing `View` — not the
presenter growing a clock. If this layer ever needs one, R10 has been violated.

**There is no write port, and that is L6 made structural.** The presenter cannot write
to the store because it has nothing to write with. Compare `RUNTIME.md` §3.1, where
`StoreWrite` exists and is the whole subject of D24. The absence here is the check.

Three is fewer than either sibling's five. R27 is the reason: a fourth port is a
defect until a named consumer requires it. The count going 5 → 5 → 3 across the three
layers is worth noticing rather than smoothing over — the presenter is the thinnest
seam in the platform, and D14's one-sentence statement of the platform's job says
nothing about drawing at all.

### 3.2 `Addr`, and what this layer needs of it

The pure core depends on nothing (R3), so — exactly as in `RUNTIME.md` §3.2 and
`COMPOSITOR.md` §3.2 — it cannot use the store's key type.

> **`Addr` is an opaque, totally-ordered byte key.**

| Property | Runtime | Compositor | Presenter | Why here |
|---|---|---|---|---|
| Equality | yes | yes | **yes** | a placed thing is that thing or it is not |
| Total order | yes | yes | **yes** | a subtree is one contiguous key range (`infinitedb-spatial-layer.md` §10), which is exactly the range a cull asks for |
| Prefix truncation | yes | **no** | **yes** | the runtime truncates for *priority by distance from focus*; the presenter truncates because **truncation is level**, and level is detail (§7). The compositor deliberately refuses it |
| Permanence | relied on | relied on | relied on | the store's invariant; no layer verifies it |

The runtime and the presenter therefore want the *same three operations* — and the
compositor wants strictly fewer. That is new evidence for **O13**, recorded in §13
rather than acted on: two of three cores now agree exactly, which is a stronger case
for promoting `Addr` to a zero-dependency crate than existed when O13 was opened. The
trigger is unchanged.

**Level is measured in bits, not in levels.** Level ℓ is the key truncated to ℓ·*D*
bits, and *D* is the chart's dimension — which the presenter does not know and must
not ask for, since charts need not share dimension (`infinitedb-spatial-layer.md` §2).
So the core's operations are `truncate(bits)` and `prefix_bits()`, both pure byte
arithmetic. The presenter never learns *D*, never learns what a dimension is, and
consequently cannot be wrong about one.

### 3.3 One scalar, and it is `f64`

> **`f32` appears nowhere in this crate.** The embedding is `f64` throughout.
> Narrowing happens inside the `Surface` implementation, in the facade, at the last
> possible moment.

*Checked by:* a comment-stripped grep for `f32` over the crate. It is the mirror of
`COMPOSITOR.md` §12's *no float anywhere* — that layer forbids both because it has no
math (L3); this layer permits one because the embedding **is** arithmetic, and forbids
the other because mixing them is a defect the corpus already contains.

*Why.* `hyper-ui` runs an `f64` world through an `f32` camera: `WorldRect` and
`SceneNode.world_pos` are `[f64; 2]`, `SceneCamera.center` and both transform methods
are `f32`, and `fit_to_content` narrows while `visible_world_rect` widens back. An
address space whose whole premise is *unbounded refinement* is being projected through
24 bits of mantissa, and the round-trip is lossy exactly where §10's precision floor
says it will be. Choosing one scalar is not a style preference; it is the difference
between a precision floor that can be *detected* (§10) and one that shows up as
jitter.

---

## 4 · What the presenter may hold (L5, L6)

`RUNTIME.md` §4 states the runtime's version — *a record may pass through within a
single tick; it may not be retained across ticks, except inside a declared derived
artifact.* This layer's version:

> **A screen position may be held only inside the placement, and the placement is
> discarded when the view changes.** Nothing is held per-thing anywhere else.

The three state categories (D8), instantiated:

| Category | In the presenter | Discardable |
|---|---|---|
| **Stored** | nothing (L6) | — |
| **Derived** | the placement, and the per-space embedding transforms it is built from | yes, by definition |
| **Pending** | nothing. Hover, a drag in progress and a half-finished pan are the runtime's pending set (D8, D24); the presenter is *handed* the resulting view and never holds one | — |

Four checkable claims:

1. **No identity of the presenter's own** (L5). *Checked by:* no type in the crate has
   a field that is an integer id, a handle, a generational index, or a `usize` used as
   a key. Every reference is an `Addr`.
2. **No map keyed by anything but an address.** *Checked by:* the same grep. F-2 — *a
   map keyed by id standing in for an edge* — has roughly thirty instances in one
   codebase in this corpus, and `hyper-ui`'s `Overrides` shows the presentation-layer
   form: `HashMap<(ContainerId, SizeClass), f32>`, persisted, synced, and with nothing
   staged that ever evicts an entry, so it grows monotonically with every container
   that has ever existed.
3. **No write port** (L6). *Checked by:* the port list is three long and none of them
   writes.
4. **The placement is a derived artifact and it is registered with the runtime.**

Point 4 is D25 for the fourth time, and it is the one the sibling specifications
already predicted:

> The runtime knows artifact *lifecycle*, never artifact *content*. The presenter owns
> the **function**; the runtime owns the **schedule**.

D25 names the presenter's artifact as its *first* instance. `COMPOSITOR.md` §4 calls
D25's split *"the third instance, which is a pattern rather than a coincidence."* This
is the fourth, and it is the one D25 was written for. The presenter contributes **no**
invalidation machinery and **no** per-artifact test code; R12's generic discard harness
covers the placement on the day it is written.

*P5 dies here.* Invalidation stops being a comment on an enum variant and becomes a
registered rebuild rule with a validity watermark. A missed notification costs a
frame's freshness, never a wrong answer — which is `StaleFeed`'s property (RUNTIME
§3.1) inherited rather than re-argued.

---

## 5 · The model

### 5.1 The presenter adds four words to D20's five

D20's vocabulary is `space`, `node`, `graph`, `hyperedge`, `zoom`. `COMPOSITOR.md` §5
adds `block`, `port`, `plan`. This layer adds:

| Word | Meaning | Not a new noun because |
|---|---|---|
| **view** | where you are looking from: a camera and a surface | it is the argument to `place`, the way a `DefinitionSet` is the argument to `link` |
| **placement** | the derived map from address to rectangle at a level, in draw order | derived, never authored — the artifact of §4 |
| **level** | how many bits of an address are significant | genuinely new *as a number*, but it is D20's **zoom**, made arithmetic |
| **probe** | a point on the screen, answered with an address | genuinely new; it is the half of the layer everything else got wrong |

**A view carries no focus, and the absence is deliberate** (R27). Focus drives
*priority* — which space is rebuilt first — and priority is the runtime's
(`RUNTIME.md` §5.1, breakage B6). Nothing this layer computes depends on where
attention is: detail depends on zoom and on a space's own override (§7), which is
exactly what D20 means by *detail is per space, not per camera*. A focus in the view
would be a second, quieter way for the camera to decide detail, which is P6 arriving
through the door marked "convenience".

And two words already decided elsewhere, only used here: **transform** (the similarity
that embeds one space in its parent, `infinitedb-spatial-layer.md` §7) and **extent**
(`{ min, ideal, weight }`, salvaged verbatim from `hyper-ui/src/container/extent.rs`,
which is 36 lines, has no identity in it, and is correct).

### 5.2 There is no `Visibility` enum, and detail is not a mode

`hyper-ui` has `enum Visibility { Shown, Collapsed, Hidden }`, with the derived `Ord`
doing real work — *"`Shown < Collapsed < Hidden` is the demotion ladder, so demote one
step is a successor."* That is a good design inside a one-dimensional space allocator
and it is the wrong shape here, for a reason D20 already supplies:

> **A node is a space seen from one level out.**

So *collapsed* is not a third state beside *shown* and *hidden*. **Collapse is zoom.**
A space rendered at its own level is a node; a space rendered one level deeper is a
graph; a space below the visible range is absent. Three enum variants collapse into
one number, and the number is the same prefix arithmetic the runtime uses for
priority.

*Consequences.* The core carries **zero enums**, and `scripts/check-rules.sh` pins the
count at zero (§11) — a stronger position than the compositor's one, which owes a
decision record. The ladder becomes a clamp. And a fourth rung, when some consumer
wants one, is a different number rather than a new variant, which is F-1 avoided
without spending a decision record on it.

---

## 6 · The embedding

### 6.1 One transform per space, not one per thing

`infinitedb-spatial-layer.md` §7, invariant 7 — written while designing a database,
and it is a rendering doctrine:

> Because every chart's interior is expressed in its own local coordinates, its
> relationship to the outside world is a single similarity transform. When an ancestor
> densifies and the host cell shrinks and drifts, **the only thing that changes is that
> one embedding transform; the subtree moves rigidly**, and every address, seat
> relation, and sub-chart inside it is untouched.

Therefore:

> **The presenter holds one `Transform` per *visited space*. A thing's position is the
> fold of the transforms along its path. There is no per-thing transform, ever.**

This is not an optimization. It is what structurally forbids the per-node transform
table, which is F-2's shape in this layer, and it is why a pan is O(1) rather than
O(nodes): panning changes the root transform and nothing else.

A `Transform` is a similarity — scale and offset. Not a matrix, because rotation and
shear have no named consumer (R27), and because a similarity composes as two
multiplications and stays exact in a way a general matrix does not.

### 6.2 What a camera is

`Camera { center: [f64; 2], zoom: f64 }`. Nothing else.

`hyper-ui`'s `SceneCamera` additionally carries `screen_px`, `viewport_origin` and
`user_adjusted`. The first two are the surface's, not the camera's, and keeping them in
one struct is precisely how the cull path and the draw path came to disagree (P1). The
third is a policy flag — *"stops auto-fit taking over"* — whose enforcement lives
entirely at the call site, which makes it a comment with a type.

The camera is **stored, session-scoped**, in the manner of D5's *"which pod has focus
— store, session-scoped; the session WAL exists and is unused."* The presenter reads
it through `Scene`. It does not own it, because it survives a restart if the person
wants it to, and D5 settles that.

### 6.3 Culling is a range, not a filter

Under §3.2's total order a subtree is one contiguous key range, so *what is on screen*
is a **range of addresses**, computed once, not a predicate evaluated per thing.

`hyper-ui`'s `InMemorySpatial` is the counter-example and it names itself:

```rust
fn query_nodes_in_rect(&self, rect: WorldRect) -> Vec<SceneNode> {
    self.nodes.iter().filter(|n| rect.contains(n.world_pos)).cloned().collect()
}
```

Despite the type's name there is no spatial index — a linear scan, cloning each hit,
allocating a fresh `Vec` per frame. And it tests each node's **centre point** while
ignoring `size_world`, so a thing straddling the edge is culled while visibly on
screen. The camera's 64-pixel margin is what has been covering for that.

### 6.4 The agreement test, and P1

> **The transform that culls is the transform that draws. One function, called twice.**

*Green check for S4, and this layer's equivalent of the closure test:*

> For randomly generated cameras, surface sizes and **non-zero surface origins**:
> a point is inside the culled address range **if and only if** its projection is
> inside the surface rectangle, up to the declared margin. Generate the two by
> different code paths and assert they agree.

That property is what `hyper-ui` violates, and the violation is instructive because the
fix for the *same* bug already landed in the draw path. `viewport_origin` was added
with a doc comment explaining it:

> *"Without this the camera centred content on `screen_px * 0.5` measured from the
> window origin, while the viewport it was sizing to sat somewhere else — so the scene
> drew behind the tab strip."*

`screen_to_world` was corrected. `visible_world_rect` still probes at window
coordinates `(-margin, -margin)` … `(screen_px + margin)` and therefore returns a world
rectangle translated by `−viewport_origin / zoom`. Two paths, one concept, one of them
fixed — which is R17's failure mode wearing geometry, and it survived because §2's
finding says nothing in `renderer/` is tested.

---

## 7 · Detail is a level (D31)

### 7.1 Zoom resolves a default; a space overrides it; the override sticks

D20 states the requirement and even names the mechanism:

> *Detail is per space, not per camera.* Zoom sets a default; individual spaces are
> held open or closed against it, which is how several things stay legible at once and
> what tabs and pod-collapse actually are. The mechanism already exists: `hyper-ui`'s
> resolved-default-versus-sticky-override model with hysteresis, built for responsive
> layout and generalizing directly to semantic zoom.

Instantiated:

1. **The view resolves a default level** from zoom — how many bits of address are
   significant at this magnification.
2. **A space may carry an override**, held open or closed against that default. The
   override is **authored**, so it lives in the store (D5: *"which pod is collapsed —
   store, authored"*) and the presenter reads it through `Scene`. It is never written
   here (L6).
3. **The effective level is the default, overridden, then clamped** to the range the
   surface can actually resolve (§10).

### 7.2 Detail is per space because *level* is per address

The reason this works is not an editor feature. Level ℓ is the address truncated to
ℓ·*D* bits, so **every address already carries its own level**; asking "how much detail
does this space get" is asking about that address and not about the camera. P6 is
therefore impossible by construction rather than by policy.

### 7.3 Hysteresis, and the one thing worth salvaging wholesale

A boundary crossed by a continuous quantity needs a dead band or it chatters.
`hyper-ui/src/layout/viewport.rs` has the best-designed and best-tested code in the
crate — six tests including full 600 → 1100 → 600 sweeps — and its rule is
**asymmetric**:

- promote when `width >= naive.lower_bound() + CLASS_SLOP`
- demote when `width < previous.lower_bound() - CLASS_SLOP`

with `CLASS_SLOP = 32.0`, which makes the boundary a **64-unit dead band, not a
32-unit one**, and which uses the *previous* class's bound on the way down rather than
the naive one. That is subtle, correct, and exactly transferable: substitute *zoom* for
*width* and *level* for `SizeClass` and the algorithm is unchanged.

*Green check for S5:* sweep zoom up through every boundary and back down, and assert
(a) each boundary is crossed exactly once in each direction, (b) no level changes twice
within one dead band, and (c) the sequence is identical when the sweep is replayed —
which D19's equivalence law needs and which a hysteretic function only has if its
state is entirely in the view it is handed.

**Not salvaged, and it is worth saying why.** `hyper-ui`'s `DemotionLadder::demote`
returns the current visibility unchanged when the current value is not in the ladder's
`steps`, so a container carrying a visibility from a different arrangement is a silent
no-op rather than a finding. Under §5.2 there is no ladder membership to be wrong
about. And `InputClass::hit_slop()` — 4 units for a pointer, 12 for touch — exists, is
unit-tested, and is called by nothing but its own test, while the pointer path does an
exact both-edges-inclusive containment test with no slop at all. A tested dead function
is worse than an untested one, because the test says it works.

---

## 8 · Placement, and the probe

### 8.1 The two functions

```rust
pub fn place(scene: &SceneSet, view: &View) -> Placement
pub fn probe(placement: &Placement, at: Point) -> Option<Probe>
```

Both pure. No I/O, no clock, no store, no surface. `SceneSet` is whatever the `Scene`
port resolved — stored, pending, or a mix, which is C4's argument (`COMPOSITOR.md` §2)
arriving in this layer for the same reason: the editor must be able to place and probe
a shape the person is still dragging.

### 8.2 What a placement is

An ordered sequence of `Placed { at: Addr, rect: Rect, level: u32, clip: Option<Rect>,
accepts: bool }`, in draw order, plus the per-space transforms it was built from.

`accepts` is the fix for P3. `hyper-ui`'s `viewport_at` had to take the store as an
argument in order to ask what *kind* of thing it had found; here the question is
answered once, at place time, from the `Scene` declaration, and baked in as a bit. The
placement is then self-sufficient by construction.

What is **not** in a `Placed`: a colour, a selection flag, a hover flag, a style. The
counter-example is `hyper-ui/src/renderer/scene_node.rs`, an 11-line struct that
correctly holds no identity — and therefore had nowhere to put selection except
`selected: bool` inside the geometry record, so selecting a thing means re-deriving and
re-uploading its geometry. That struct is the honest demonstration of what happens when
"holds no identity" is read as "holds no address": the layer became unhittable. `Addr`
is the resolution. L5 forbids the presenter *minting* identity; it does not forbid it
*referring* to the store's.

### 8.3 The probe descends; it does not scan

`hyper-ui` hit-tests by reverse linear scan of a flat list —

```rust
self.items.iter().rev()
    .find(|item| item.interactive && item.contains(pos))
    .map(|item| item.node)
```

— on every `CursorMoved`, again on press, and again on release, with no index of any
kind. Its `get(node)` is worse: an O(n) `find` by id inside a positional vector.

Under §3.2 and §6.1 the presenter does better without trying, because the doctrine pays
for it. A subtree is contiguous in address order, and a space's children are embedded
rigidly inside it, so *address order is spatial order*. `probe` therefore **descends**:
find the space containing the point, then descend into it, until nothing deeper
contains the point. That is O(depth), not O(n), and among overlapping siblings at one
level it takes the last in draw order.

This is the argument for the whole spatial model arriving as a performance
consequence, which is worth noticing: the reason it is fast is the reason addresses are
permanent.

### 8.4 The self-sufficiency test

*Green check for S6:*

> `probe` is called with **no port in scope at all** — no `Scene`, no `Surface`, no
> `Glyphs` — over a corpus that includes overlapping siblings, a clipped subtree, a
> collapsed space, a point in a gutter, and a point outside every space. Every case
> answers, and the answers are the ones the corpus declares.

If `probe` ever needs a port, P3 has recurred and the placement stopped being
self-sufficient. The test is the detector, and it is a compile-time one.

---

## 9 · Drawing

> **Correction, 2026-08-22 (E10.0, D41).** From the change that landed S8 until the
> one that landed E10.4, this document's S8 row read *"a real wgpu `Surface`"* and no
> such thing existed: `src/facade/ports/surface.rs` computed the frame's quads and
> discarded them, and no adapter, device, pipeline or render pass existed anywhere in
> the repository. The row was not corrected for four stages because every check
> capable of failing was arithmetic. It is now verified by `tests/pixels.rs`, which
> renders into a texture and reads the pixels back, and which was confirmed to fail
> against the discarding implementation before the replacement was written. Recorded
> here rather than silently edited (R21), because the mechanism is more useful than
> the instance: see `docs/plans/EDITOR-BOOTSTRAP.md` §9 finding 11 and D41.


The `Surface` port takes a frame's work as an opaque sequence and reports back its
size, scale factor and origin. The presenter decides *what* is uploaded, in what order,
grouped how, at what level. The facade decides *how*, and that is where `wgpu` is
named.

Note what is absent: no shader, no pipeline, no buffer, no device, no window, no event
loop. `hyper-ui` has all of them in the same crate as its layout algorithms, and §2's
finding is the price.

**Draw order is address order within a level, and level order across levels.** No
z-index. `hyper-ui`'s paint order is DFS pre-order over children, which is the same
thing arrived at structurally, and it is the one part of that file that needed no
argument.

---

## 10 · Precision, and O14

This is the layer where exactness ends, and the store's own design document says so
(`infinitedb-spatial-layer.md` §9, *density governors*):

> **Precision floor (embedding layer).** Rendering a seat requires Σ dᵢℓᵢ bits of fixed
> point along the path. **Addresses remain exact forever; only renderability is
> bounded.** Detector: minimum embedded segment length approaching 2^(−P).

Three consequences that are this layer's, and one that is not:

1. **The clamp in §7.1 step 3 is this.** The effective level is clamped to what the
   surface can resolve, and the bound is arithmetic rather than a guess.
2. **Running out of bits is a finding, not a glitch** — and the presenter reports the
   *fact*, not the prose. `Placement::precision_floor` carries the shallowest address
   at which the surface ran out of bits, and the facade turns it into a finding with a
   site, a `said`, a `wanted` and a `remedy` (`COMPOSITOR.md` §6) — *"this space is
   refined past what the screen can distinguish at this zoom; zoom in to work in it."*

   The presenter defines **no second `Finding` type**. It cannot use the compositor's
   (D29), and defining its own would put two structures under one name, which is R17's
   failure — the mechanism that produced three `PageTree`s. `Addr` is already carrying
   as much of that as this workspace should tolerate (O13). Reporting an address and
   letting the facade format it costs nothing and duplicates nothing.

   What it must never do is report the floor as ordinary emptiness. §13 finding 8 has
   the corpus's example.
3. **§3.3's single scalar is what makes the detector work.** You cannot detect a
   precision floor while silently crossing between two precisions.

**O14 is opened rather than answered:** *what does the presenter do when the floor is
reached* — clamp and report, or re-base the transform stack on the deepest common
ancestor and carry on. The second is what a renderer of an unbounded space eventually
has to do, it is the presentation-side twin of the store's ratchet-versus-breathing
decision (§12 of the spatial document), and it needs a consumer deep enough to
measure. Trigger: the first composition where the minimum embedded segment length
approaches 2^(−P).

---

## 11 · How every rule is checked, in this layer

Every check lives in `scripts/check-rules.sh`, and **all three layers now use the
comment-stripping grep** — see §13, finding 3.

| Rule | Check | Lives in |
|---|---|---|
| R3 — pure core depends on nothing | `cargo build -p infinite-presenter` with no features; `[dependencies]` empty | CI |
| D29 — no graphics crate | manifest grep: no `wgpu`, `winit`, `glyphon`, `raw-window-handle`, `cosmic-text` | CI |
| D29 — no other layer is named | source grep: no `infinite_(db\|runtime\|compositor\|physics\|ux)` | CI |
| §3.3 — one scalar | source grep: no `f32` anywhere in the crate | CI |
| R10 / D29 — no `now` | source grep: no `std::time`, `Instant`, `SystemTime`. There is no `Clock` port and there must be no clock | CI |
| L6 — authors nothing | source grep: no `std::fs`, no file handle; and the port list is three long, none of which writes | CI + review |
| L5 — mints no identity | source grep: no field typed `u32`/`u64`/`usize` named `id`, no `Handle`, no map keyed by anything but `Addr` | CI |
| R5 — derived state never writes back | `place` takes `&SceneSet` and `probe` takes `&Placement`; neither has a `&mut` argument. P4 becomes a compile error | the type signatures |
| R16 — registries, not enums | the core's enum count is pinned at **zero** (§5.2) | CI |
| R12 — artifacts pass the discard test | the runtime's generic harness (D25); this layer contributes **no** per-artifact test code | runtime test suite |
| §6.4 — cull agrees with draw | the agreement test | test suite |
| §7.3 — hysteresis settles | the sweep | test suite |
| §8.4 — the probe is self-sufficient | called with no port in scope | test suite |
| R23 — rendering claims state their verification | every claim in this document about `hyper-ui` names the file it came from, and §2 states what was read and what was not | this document |
| F-8 — no `mod.rs` | file listing | CI |

**Every check in this table was verified to fail.** A check that has never been seen
to fail is a check nobody knows the polarity of, and this project has the failure mode
on record — `HISTORY.md` traces its drifts to statements that were true when written
and never re-run. So each one above was exercised twice: once against the crate, where
it passes, and once against a deliberately broken copy, where it must fail. An `f32`
added to `point.rs`; an `id: usize` field; a `BTreeMap<u64, _>`; an `enum` in the core;
a `&mut` on `place`; `infinite_runtime` named in code; `wgpu` added to the manifest.
All seven fired. Two further runs confirmed the reverse property that
`COMPOSITOR.md` §14 finding 4 is about: the doc comments that *cite* these rules —
`core.rs` explaining why there is no `f32`, `placed.rs` explaining what an id would
have cost — do **not** trip the greps that enforce them.

*Verification method (R23):* by injection, in the working copy, restored afterward;
`bash scripts/check-rules.sh` reports 37 checks across three layers and all pass on the
landed tree.

**L5's grep is the weakest check in the table and it is worth saying so.** It catches
the shapes the corpus actually produced — a `NodeId` field, a `HashMap<(Id, _), _>` —
and it would not catch a cleverer one. The compile-time half is stronger: with `Addr`
as the only key type in the crate's public surface, an id has nowhere to live. Where
the check is weak, say it is weak (RULES preamble: *a rule that cannot be checked is a
preference*), rather than letting a green tick imply more than it proves.

---

## 12 · Crate layout

One crate, `crates/infinite-presenter`. Core/binding split (D7) is a **module and
feature** boundary, following `bion`'s proven shape, `RUNTIME.md` §9 and
`COMPOSITOR.md` §13 — inverted so the strict build is the default.

- **default features: none.** The core builds alone, `[dependencies]` empty.
- **`binding`**, off by default, adds the ports and the frame path.

Conventions taken from the sibling crates rather than re-decided, because R17's failure
mode is two houses in one repository: module files declare `mod` privately and
re-export; `edition` / `rust-version` / `license` / `publish` inherited from
`[workspace.package]`; `autotests = false` with explicit `[[test]]` targets so
`tests/fakes.rs` can be a shared helper without becoming a test target — the
conventional Rust answer is `tests/common/mod.rs`, and F-8 forbids it.

```
crates/infinite-presenter/
  Cargo.toml
  README.md              → points at this document; does not restate it (R17, R21)
  src/
    lib.rs
    core.rs              module file: docs, mod declarations, re-exports only
    core/
      addr.rs            Addr — opaque ordered key; truncation is level (§3.2)
      point.rs           Point — a position in some space's coordinates
      rect.rs            Rect — min/max, half-open (§6)
      extent.rs          Extent { min, ideal, weight } — salvaged from hyper-ui
      transform.rs       Transform — one similarity per space (§6.1)
      camera.rs          Camera — centre and zoom, and nothing else (§6.2)
      surface_rect.rs    SurfaceRect — pixels, scale factor, origin (§6.2)
      view.rs            View — camera + surface + margin (§5.1)
      revision.rs        Revision — the store's logical clock, as this layer sees it
      level.rs           level — zoom resolved to significant bits (§7.1)
      detail.rs          detail — default, override, clamp, with hysteresis (§7.3)
      visible.rs         visible — the inverse image of the surface rect (§6.3)
      arrange.rs         arrange — pure 1-D allocation over extents (§7)
      placeable.rs       Placeable — what `Scene` says about one thing
      scene_set.rs       SceneSet — what place is handed (§8.1)
      placed.rs          Placed (§8.2)
      placement.rs       Placement — the artifact (§4, §8.2)
      place.rs           place — the function (§8.1)
      probe.rs           probe, Probe — a point, answered with an address (§8.3)
    binding.rs           module file
    binding/
      ports.rs           module file
      ports/
        scene.rs  surface.rs  glyphs.rs
      frame.rs           frame — place, then hand the work to Surface (§9)
      artifact.rs        KEY, ranges, rebuild — the three parts D25 asks for (§4)
  tests/
    agreement.rs         §6.4 — cull agrees with draw
    hysteresis.rs        §7.3 — the sweep
    probe.rs             §8.4 — self-sufficiency
    fakes.rs             the only implementations of the ports this layer ever names
```

`module.rs` plus a directory of leaf files, no `mod.rs` (F-8). One public function per
file for **free** functions; a type with an inherent impl is one file — the reading
both sibling specifications record, restated here only because it is the rule most
likely to be silently reinterpreted.

**`binding/artifact.rs` does not register anything.** It *exposes* the three parts D25
requires — the address ranges the placement derives from, the rebuild function, and the
validity watermark — in this layer's own vocabulary. The facade registers them with the
runtime, because this crate may not name the runtime. See §13, finding 1.

---

## 13 · Findings

`RUNTIME.md` §10 recorded three findings and `COMPOSITOR.md` §14 recorded that all
three were resolved. Continuing the habit, because a finding list that is never re-read
becomes a document describing a repository that no longer exists.

1. **`RUNTIME.md` §5.2 says `RenderList` is *"registered by the presenter's
   binding"* — and the presenter's binding cannot do that.** D23 forbids the runtime
   naming another layer and D29 forbids the presenter naming the runtime, so neither
   crate can see the other's registry. **The facade registers it.** A one-line
   correction to a sibling specification, recorded here rather than silently edited
   there (R21, R22). §12's `binding/artifact.rs` is the shape that makes it work.

2. **The artifact is renamed `RenderList` → `Placement`, and the old name is
   retired.** Three reasons: it answers pointer queries, which is not rendering; it
   holds no draw commands, so "list" describes the wrong thing; and it is the thing
   `probe` reads, which has no rendering in it at all. R17 permits a rename and forbids
   a *recycle* — `RenderList` is retired, not reused — and D20 set the precedent when
   it retired "chart" for "space", with the same convention: **citations of D5 and D25
   keep the original word; nothing else uses it.** This is the one rename this change
   makes, it is one `sed` from being reversed, and it is flagged here rather than
   buried because a rename proposed by an assistant is exactly the class of change R29
   says to correct rather than merge.

3. **`scripts/check-rules.sh`'s runtime section grepped raw source** — recorded as
   `COMPOSITOR.md` §14 finding 4, *"not urgent, no current comment collides, which is
   exactly why it is worth writing down now."* **Fixed in this change.** The presenter
   section needs comment stripping on its first day, because §3.3's `f32` check and
   §11's identity check both fire on the doc comments that explain them, and leaving
   one layer of three on raw greps is how the difference gets forgotten.

4. **`Addr` is now defined three times, and `Revision` twice.** Correct under R3, D23, D26 and D29 — and the
   presenter's needs are *identical* to the runtime's (§3.2), which is new evidence for
   **O13**: the case for promoting `Addr` to a zero-dependency crate is now two-thirds
   agreement rather than one instance. The trigger is unchanged (when the facade's
   conversion is more than a newtype unwrap), and it is not a fifth layer, so R1 is not
   engaged.

   `Revision` is the second type to duplicate — the runtime's and this layer's are the
   same twelve lines. Recorded here rather than folded in quietly, because O13 was
   opened about one type and is now about a *set*, and a deferred decision whose scope
   grows without anyone noticing is how a deferral becomes a permanent condition.

5. **The core carries zero enums**, and `check-rules.sh` pins the count at zero. This
   is a stronger position than the compositor's, which pins at one and owes a decision
   record for `Direction { In, Out }` (`COMPOSITOR.md` §14 finding 6, still open).
   §5.2 records why the obvious candidate here — `Visibility` — is a number instead.

6. **D15 says the presenter owns *"wgpu resource organization"*, and this document puts
   `wgpu` behind a port.** Not a contradiction, and §3 says why, but it reads like one
   on a fast skim and someone will eventually skim it. Recorded so the reconciliation
   is on the record: the presenter owns the organization, the facade owns the API, and
   the `Surface` port is the seam.

7. **`hyper-ui` is salvaged in pieces, not ported.** Taken: `Extent { min, ideal,
   weight }` verbatim; the asymmetric hysteresis rule from
   `layout/viewport.rs`; DFS-pre-order paint order. Deliberately not taken: two
   unrelated layout systems that never call each other (`pgraph/layout.rs` takes a
   `&Graph` and re-runs `measure` at every depth on every pass; `layout/resolve.rs` is
   pure but one-axis, one-arrangement, and neither mentions the other's types — R17's
   failure with two houses in one crate); `SceneNode`; `InMemorySpatial`; the
   `Overrides` side maps; and every magic sentinel in `pgraph/layout.rs` — `120.0`,
   `400.0`, `10_000.0`, `1_000_000.0`, none named, none tested, none documented.

8. **`cull_nodes_from_infinite_db` maps a database error to an empty viewport**
   (`Err(_) => Vec::new()`), so a failed query and an empty screen are indistinguishable
   and nothing is logged. Not this repository's code and not fixed here, but recorded
   because it is the exact shape §10 forbids: a condition the person should be *told
   about* rendered as ordinary emptiness. The precision-floor finding exists so that
   this layer never does the same thing.

---

## 14 · Open, carried forward

| # | Item | Trigger |
|---|---|---|
| **O14** | **The precision floor.** Clamp and report, or re-base the transform stack on the deepest common ancestor | The first composition where minimum embedded segment length approaches 2^(−P). §10 |
| O1 | Hot working set | **Sharpened, not closed.** Under §4 the placement *is* the candidate, and it is a registered artifact either way, so the architecture question is settled and only the measurement is owed: a warm prefix scan of ~1000 nodes against frame budget, in the runtime's S6 test bed |
| O13 | Three `Addr` types, one per layer core | Unchanged: when the facade's conversion is more than a newtype unwrap. §3.2 adds evidence, not a trigger |
| O11 | Is the editor self-hosted | **Closed by D36.** Yes. |
| O10 | Ownership and capability | Not this layer's — but `Scene` is where a *"may this viewer see that space"* check would go, and a placement that has already been built is too late. Do not build `Scene` so the check cannot be inserted |
| — | Large numeric geometry | Out of scope by §2. Instancing, batching and buffer residency wait for the first consumer with a solve in it — the same trigger as both siblings |
