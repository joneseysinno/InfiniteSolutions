# Infinite Solutions — after E10, what actually moves this forward

> **Status:** draft 1, 2026-08-22. **Updated 2026-08-28 by the change that landed
> §1 and §3.** The two decisions are made (O23 → D45, O20 → D46), §3's wire rendering
> is landed, §4's undo decision is made (O16 → D48) and its stages moved to
> [`E12-UNDO.md`](./E12-UNDO.md), and §5's scope is specified in
> [`E13-AUTHORING-SURFACE.md`](./E13-AUTHORING-SURFACE.md). **§2 is the one item
> still open, and it is the one nobody can automate**: a person has to sit at the
> window.
>
> This is a roadmap, not a stage plan. Two items (O23, O20) were **decisions**, not
> code, and everything below them is sequenced by what those decisions unblock. In
> the spirit of R20/R23: nothing here gets a green check until it has one that can
> fail, and nothing gets called "landed" until the change that lands it says so.

---

## 0 · Where we actually are, corrected

`docs/plans/E10-IT-DRAWS.md` had two stale status lines going into this review —
E10.4 was marked "blocked on O22" when the fix had already landed (silently, no
status line), and E10.5's stage table didn't exist yet. Both are now corrected.
Actual state:

- **The render pipeline is real and tested end to end**: store record → style
  resolution → placement → GPU pixels, verified by `tests/pixels.rs` against an
  offscreen texture and (per your message) confirmed live in a window.
- **Position is authored and applied**: `Placeable.position`, `place_group`'s
  offset, genesis seeding two nodes at distinct origins — all real, all tested.
- **The camera is authored and persistent** (this session's work): pan/zoom amend a
  well-known record, resolved stored ∪ pending, survives a restart — `tests/camera.rs`.
- **The one thing still faked is the platform's actual thesis.** D20/D31 — *a space
  contains nodes, and a node may itself host its own space; zoom reveals it* — has
  never been exercised, and per finding 19/O23, currently **cannot** be: every
  address the facade hands the presenter is canonicalized to exactly 4 bytes, so the
  bit-length comparison `place_group` uses to decide "descend into this child" can
  never fire. This is the single most important open item below.

  > **Landed 2026-08-28 (D45).** The diagnosis was right and incomplete — see §1
  > below, which the same change annotates. `tests/nesting.rs` is the check, and it
  > was written first and seen to fail.

---

## 1 · Two decisions before more code (this week)

Both of these are architecture, not implementation. Writing code before deciding
either one risks building on an assumption that gets revisited, which is exactly the
pattern `docs/DECISIONS.md` exists to prevent.

### O23 — how does an address say "I am inside this other address"?

This is the one worth spending real thought on, because it's the difference between
*"Infinite Solutions demonstrates its own core idea"* and *"Infinite Solutions has
never tried."* Three shapes, roughly:

| | Approach | Cost |
|---|---|---|
| a | **Keep addresses variable-length past the facade boundary.** Stop canonicalizing every key to a `u32` in `Inner::coord`; carry the original byte length through to the presenter, so `Addr::prefix_bits()` means something real. | Touches the store's indexing scheme (`infinite-db`'s `DimensionVector`/coordinate space), which is the deepest layer in the stack. Real work, but it directly fixes the actual claim rather than working around it. |
| b | **Give `place_group` a different "descend" signal than bit-length.** E.g., an explicit `depth: u32` field on the space record, authored at genesis and by whatever creates a nested space, instead of deriving depth from address structure. | Smaller, contained to the presenter/editor. But it makes "level" a fact someone has to remember to set correctly rather than a structural property of the address — closer to `hyper-ui`'s failure mode than D20's "the store's invariant" framing wants. |
| c | **Accept flat addressing and reframe D20.** Stop claiming zoom reveals nested children through address containment; make "entering a space" an explicit navigation action (like a hyperlink) rather than an implicit zoom-triggered reveal. | Cheapest, but it's not a fix — it's declaring the original architectural bet void. Only take this if (a) turns out to be disproportionately expensive relative to what the platform actually needs. |

**My recommendation is (a).** It's more work, but it's the only option that makes
the claim in `CHARTER.md`/`DECISIONS.md` actually true rather than actually true *for
one level*. (b) would work today and quietly become a liability the moment someone
builds a UI (like a property inspector, below) that assumes depth is derivable from
the address the way the spec says it is.

**Before committing to a direction, write the test that would fail.** Seed a genesis
with a canvas containing a node containing two more nodes, assert `place`'s output at
increasing zoom reveals them level by level. Right now that test cannot be written in
a way that can fail — write it first, watch it fail for the *right* reason under the
current scheme, then pick (a) or (b) knowing exactly what makes it pass.

> **Decided 2026-08-28 — D45, and it is (a) plus a third thing this table did not
> name.** The failing test was written first, as instructed, and it failed for two
> reasons rather than one: node A's interior nodes were placed as *siblings* at the
> resting camera, and `canvas.contains(node A)` was false. Fixing containment alone
> — option (a) as written above — would have left one nesting level costing eight
> address bits, and one bit is one doubling of zoom, so entering a space would take
> 256× magnification and entering two would take 65 536×. **True and unusable is a
> worse outcome than false and known-false.**
>
> So D45 takes (a) for *containment* — `Addr` carries a significant bit length the
> facade computes from the key scheme, and the editor's keys become a nibble-per-level
> hierarchy — and replaces the descend *trigger* with the space's apparent size in
> device pixels against `View::opening_extent`. Address depth answers *who is inside
> whom*; apparent size answers *when can you see in*. The recommendation above was
> right about which half was structural and wrong to assume one test could be both
> halves. `tests/nesting.rs`, five assertions, all of which can fail.

### O20 — where does draw grouping live?

`Placement` is a flat list of rectangles with one implicit pipeline. The moment a
second primitive exists — and wires are that primitive — either the facade invents
grouping ad hoc (which is `hyper-ui`'s exact failure, relocated) or `Placement`
grows a way to express it. This has to be decided before wire rendering starts, not
discovered while writing it.

Concretely: does `Placed` grow a `kind: PrimitiveKind` (rect vs. line vs. text) and
`Placement` group by kind for the facade to batch, or does the facade own grouping
entirely and the presenter stays ignorant of what a "batch" is? The plan's own O20
entry favors the former — D29 gives the presenter "grouped how," and a facade that
invents its own grouping quietly moves that boundary.

> **Decided 2026-08-28 — D46: the former, but not with an enum.** The presenter
> authors the grouping, as this paragraph argues it should. `Placement::batches`
> partitions `placed` into contiguous runs sharing a `primitive` key, and the key is a
> `Box<str>`, not a `PrimitiveKind` — the set of primitives is open by construction (a
> block author publishes a new one), R16 makes a closed enum a defect wherever the set
> is open, F-1 counts five prior instances, and R29 names an added enum as exactly the
> class of proposal to correct rather than merge. The suggestion above was one word
> away from being the fifth. `Placed` gains `span: Option<(Point, Point)>` for the
> primitives that run between two points, because a bounding box cannot say which
> diagonal a line takes.

---

## 2 · Cheap and parallel: close the verification gap

`docs/STATUS.md` has said this since before this session and it's still true:
**"Complete desktop verification of all authored-position editing gestures"** has
never been done. Every claim so far is proven by an automated test against an
offscreen texture or a fake surface — nobody has actually sat at the real window and
dragged, panned, zoomed, wired, and clicked a finding at both 1.0 and a HiDPI scale
factor. This costs an afternoon and it's the check that would have caught the O22
staleness immediately instead of leaving it for the next person who read closely.

Do this now, in parallel with the O23/O20 decisions — it needs no new code, only the
binary you already confirmed shows a surface.

> **Still open, 2026-08-28, and deliberately not claimed.** This is the one item in
> this document that an automated change cannot close, and writing a status line for
> it would be exactly the defect D41 exists to prevent. What the same change *did*
> add is more for a person to look at: node A now opens when you zoom into it, and a
> wire is drawn between node A and node B. The pass to do, unchanged: drag, pan,
> zoom, select, wire and click a finding, at scale factor 1.0 and at 2.0, against the
> running binary. `docs/STATUS.md` keeps it under **Not Yet** until someone has done
> it and says so.

---

## 3 · E11 — wires become visible

Once O20 is decided, this is the next real user-visible feature and the second half
of "six interactions, all six work." Right now a wire exists as data — linking,
validation, mismatch findings, zoom-to-site — but nothing draws a line. Scope:

- One more `Placed`-shaped thing (or a variant, per whatever O20 decided) carrying
  two endpoints instead of a rect.
- A second draw call or an extended instance buffer in `facade/ports/surface.rs`
  (still the one file allowed `f32`, per D42).
- **Green check, stated the E10 way**: a test that draws a known composition with one
  wire, reads back the framebuffer, and asserts a line of the wire's colour crosses
  the pixel path between the two endpoints — not just "the draw call didn't panic."

This is also where O21 gets resolved for free: if wire rendering needs the `SceneSet`
alongside the `Placement` (which it will, for endpoint resolution), that's the same
shape `binding::frame` already has and dropped — worth checking whether reviving it
properly, rather than deleting it, is the right call once there are two consumers of
"a placement plus its scene."

> **Landed 2026-08-28.** `tests/wires.rs`, five tests, and both failure modes were
> run and seen: with the wire record not written the test fails at *"the wire is
> placed with two endpoints"*, and with D46's batching defeated so the link falls
> through to the quad pipeline it fails with `channel 0 was 242, wanted 31` forty
> pixels off the line — a bounding box where a line should be. Genesis authors node B
> **down as well as across** for that second check: with the two nodes in a row the
> box *is* the line and no sample could tell them apart.
>
> **O21 did resolve here, and the answer was retire-and-replace** (D47). `frame` is
> gone; `binding::compose(scene, view, at) -> (SceneSet, Placement)` is what the two
> consumers wanted, and `Store::draw_with` and `Store::place_now` call it, so R27 is
> satisfied by a caller rather than by a deletion. The name is retired, not reused
> (R17): `frame` named a function that also submitted, and `compose` does not.

---

## 4 · E12 — undo, decided before more state piles onto an undecided model

O16 has been open since before this session, and this session made it sharper: the
camera is now a pending record, exactly like a drag-in-progress or a half-drawn
wire, and none of it has an undo story. Every new authored-state type (positions,
now camera, soon wires, eventually text) makes retrofitting undo more expensive.
Decide the shape now:

- Is undo a property of the **pending set** (discard an uncommitted amend) or the
  **committed history** (revert a commit), or does it need to distinguish the two —
  panning shouldn't enter the same undo stream as a committed drag, probably?
  D8's Stored/Derived/Pending split already gives the vocabulary; O16 is asking
  which of those three undo actually operates on, and whether that answer differs by
  record kind (camera vs. geometry vs. composition wiring).
- This is a decision record before it's code, same as O23/O20.

> **Decided 2026-08-28 — D48. The answer is the commit boundary, and it makes the
> per-record-kind question go away.** Undo operates on **committed** history and
> writes the previous value as a new commit; it never rewinds a revision, because the
> charter's audit and observability both rest on revisions being append-only.
> Abandoning a gesture in progress is `discard` — a different verb, on the pending
> set, which R13 already bounds. The camera is outside the undo stream not by a rule
> excluding it but because `pan_by` and `zoom_by` amend and **nothing ever commits**
> it (D5), so the worry in the second bullet answers itself structurally. And the
> stream is a registered derived artifact (D25, R12) rebuilt from the commit journal
> above a session watermark, so D8's three categories stand and there is no fourth.
>
> A policy per record kind was the alternative, and it is the same distinction
> restated as a table someone has to maintain — wrong the first time a kind is added
> and nobody updates it. Stages: [`E12-UNDO.md`](./E12-UNDO.md). No status line for
> any of them until the change that lands it writes one.

---

## 5 · E13 — the actual authoring surface

`docs/STATUS.md`'s "Not Yet" list names this directly: **property inspector, block
palette, toolbar, general text editing.** This is the largest remaining scope and
the one that turns "a canvas that draws coloured rectangles and lines" into
"something a person can build an app with." It should come after E11/E12, not
before, because:

- A property inspector needs something to inspect properties *of* — which is more
  useful once wires exist, so a wired composition's ports and types are visible
  objects, not just a canvas of shapes.
- Text editing is a new native block category (or several) — worth designing after
  O23 settles what depth/nesting really means, so text-in-a-space isn't designed
  against an addressing scheme about to change.

> **Specified 2026-08-28: [`E13-AUTHORING-SURFACE.md`](./E13-AUTHORING-SURFACE.md).**
> Both preconditions this section named are met — wires exist (§3) and O23 has
> settled what nesting means (D45) — so the deliverable is a stage plan rather than
> code, per R28: the idea is still fuzzy at the edges, and fuzzy plus code is the
> drift mechanism. Every stage there carries a green check that can fail and an empty
> **Verified by** cell, which under D41 is what forbids marking any of them landed.

---

## 6 · The horizon: close O11 for real

All of the above is in service of one thing the charter already states: **the editor
is self-hosting, and self-hosting only means something once it builds something that
isn't itself.** `docs/DECISIONS.md` D12 names the eventual consumers (SES, Coach
Assistant) and defers them; O11 stays open until one of them — or even a smaller
proof than either — gets built *through* the editor rather than around it. That's
the test that actually settles the worry this whole review started from: not "does
presenter apply graph-like behaviour" in the abstract, but "did someone build a real
thing with it." Nothing above is worth doing except as a path to being able to try
that honestly.

---

## Suggested order

*Struck through where done, 2026-08-28. The one item not struck is the one that needs
a person.*

1. ~~**This week, in parallel:** decide O23 (write the failing test first), decide
   O20~~ — both decided, D45 and D46, with the failing test written first — **and do
   the manual six-interaction desktop verification pass** (§2, still owed).
2. ~~**Next:** E11 (wires render)~~ — landed, `tests/wires.rs`.
3. **Then:** E12 (undo) — ~~decided~~ (D48), stages in `E12-UNDO.md`, not yet built.
4. **Then:** E13 (property inspector / block palette / text) — ~~scoped~~
   (`E13-AUTHORING-SURFACE.md`), not yet built.
5. **Throughout:** the moment any of the above is enough to build one small real
   thing through the editor, do that, and let it be the forcing function for
   whatever E13 turns out to actually need, rather than guessing ahead of it.
