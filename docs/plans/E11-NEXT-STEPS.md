# Infinite Solutions — after E10, what actually moves this forward

> **Status:** draft 1, 2026-08-22. Written after confirming E10.0–E10.4 are genuinely
> landed (a live window shows a real surface with correctly positioned, correctly
> coloured nodes) and E10.5 is landed in part (the camera is an authored record; the
> multi-level zoom claim is not verified and is blocked — see O23 below).
>
> This is a roadmap, not a stage plan. Two items (O23, O20) are **decisions**, not
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
- **The one thing still faked is the platform's actual thesis.** D20/D31 — *a node
  and a space are the same thing at two zoom levels, zoom reveals children* — has
  never been exercised, and per finding 19/O23, currently **cannot** be: every
  address the facade hands the presenter is canonicalized to exactly 4 bytes, so the
  bit-length comparison `place_group` uses to decide "descend into this child" can
  never fire. This is the single most important open item below.

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

1. **This week, in parallel:** decide O23 (write the failing test first), decide
   O20, and do the manual six-interaction desktop verification pass.
2. **Next:** E11 (wires render) — the second primitive, closes the last of the six
   interactions, and is the direct, visible answer to "does it apply the actual
   graph-like behaviour."
3. **Then:** E12 (undo), decided before E13 adds more state kinds to retrofit.
4. **Then:** E13 (property inspector / block palette / text) — the real authoring
   surface.
5. **Throughout:** the moment any of the above is enough to build one small real
   thing through the editor, do that, and let it be the forcing function for
   whatever E13 turns out to actually need, rather than guessing ahead of it.
