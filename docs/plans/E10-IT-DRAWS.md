# Infinite Solutions — E10, it draws

> **Status:** draft 1, 2026-08-22. E10.0–E10.4 landed; E10.5 landed in part. R20: a
> status line is written by the change that lands the phase, never at authoring time,
> and never by the person who wrote the plan.
>
> Rules: [`../RULES.md`](../RULES.md) · Decisions: [`../DECISIONS.md`](../DECISIONS.md) ·
> Charter: [`../CHARTER.md`](../CHARTER.md) · Predecessor:
> [`EDITOR-BOOTSTRAP.md`](./EDITOR-BOOTSTRAP.md) · Layer spec:
> [`../specs/PRESENTER.md`](../specs/PRESENTER.md)
>
> This plan exists because of §1 below. It is a successor stage, not an edit (R21):
> `EDITOR-BOOTSTRAP.md` stays as written, and this document carries the correction.
>
> Requires D41–D44. Opens O18, O19.

---

## Stage table

| # | Stage | Status | Verified by | Green check |
|---|---|---|---|---|
| **E10.0** | The findings, and the two false status lines | landed | `docs/DECISIONS.md` D41–D44 | `EDITOR-BOOTSTRAP.md` §9 carries findings 11–15; `PRESENTER.md` S8 and `EDITOR-BOOTSTRAP.md` E3 carry a status the code earns, and each cites the test by name |
| **E10.1** | Headless readback, and the check that fails | landed | `tests/pixels.rs` | `tests/pixels.rs` renders to an offscreen texture, copies it back, and asserts the centre pixel of node A. **It must fail on today's `Surface`** before anything else in this plan is written |
| **E10.2** | The device reaches the window | landed | `tests/pixels.rs` + the running binary | The window holds an adapter, a device, a queue and a configured swapchain. The frame clears to a colour **read from the store** and presents. Edit the canvas style row, restart, the background changes |
| **E10.3** | The surface geometry is the window's | landed | `tests/pixels.rs` + the running binary | Resize the window; `SurfaceRect` follows within one tick, origin and scale factor included. `set_surface` has a non-test caller. The saturation test still passes |
| **E10.4** | The placement becomes pixels | landed — O22 is closed | `tests/pixels.rs::the_authored_screen_reaches_the_framebuffer` | E10.1's readback passes. Node A and node B are each the fill on their own authored style row, at their own authored position; the two are distinct pixels, not one stacked rectangle |
| **E10.5** | Pan and zoom | landed in part — the camera is a record; **the D20 multi-level claim is unverified, and O23 says the check may not be writable as specified** | `tests/camera.rs` | The camera is authored at a well-known address and resolved stored ∪ pending, exactly as `Definitions` resolves a composition (§3.6). `pan_and_zoom_are_visible_before_any_commit` and `a_crash_after_pan_and_zoom_replays_the_camera_before_the_first_tick` are the falsifiable pair. **Not done:** "zoom changes which level is the graph" — see the note below the table |

**E10.4 is the deliverable.** E10.1 is the one that must not be skipped, and E10.0 is
the one that will feel like paperwork and is not.

> **What landed, and what did not.** Pixels reach the screen, and the readback proves
> the chain from a store record to a framebuffer value: three authored style rows
> resolve to three distinct colours, and editing one row changes the picture with
> nothing recompiled. **Finding 18 and O22 are now closed, correcting this section**:
> `Placeable` carries a `position: Point` field, `Scene::placed_in` decodes it from
> the space record's `origin`, and `place_group` offsets the local rect by it
> (`crates/infinite-presenter/src/core/place.rs`). Genesis seeds node B at a distinct
> origin (`[0.5, 0.0]`, `src/editor/genesis.rs`), and
> `the_authored_screen_reaches_the_framebuffer` asserts the two nodes land on
> different pixels. E7's *"drag a node and it moves"* is now true on screen as well
> as in the store — this correction is itself an instance of R20/R23: the fix landed
> without the status line that should have accompanied it, and stayed undiscovered
> until this document was next read closely.

> **E10.5, split.** The camera stopped being a `Mutex<Option<Camera>>` field that
> `pan_by`/`zoom_by` wrote directly, and became `CAMERA_KEY` (`editor::addresses`), a
> well-known address `Scene::camera` resolves stored ∪ pending — the exact mechanism
> §3.6 asked for. `tests/camera.rs` is the falsifiable pair: pan and zoom are visible
> before any commit, and both survive a restart via journal replay, the same way a
> dragged node does (E7). What did **not** land is the stage's other claim, *"zoom
> changes which level is the graph."* `place_group`'s recursion into a `hosts_space`
> child needs `item.at.prefix_bits() < level` (`crates/infinite-presenter/src/core/
> place.rs`), and every address the facade hands the presenter is canonicalized to
> exactly 4 bytes by `Inner::coord`/`Inner::bytes_of` (`src/facade/open.rs`) —
> right-aligned for a short key, FNV1a-hashed for a longer one, either way always 32
> bits. `prefix_bits()` is therefore always 32, and `level` is clamped by the
> surface-size floor to roughly 9–12, so the recursion's guard can never be satisfied,
> for any genesis, at any depth. **Seeding a deeper genesis, as §3.6 instructs, will
> not make this check able to fail** — it is not a missing fixture, it is the address
> scheme itself. That is finding 19 and O23. It is a change to a locked layer's storage
> scheme, which R29 says is corrected rather than merged, so it is raised here and not
> patched around.

---

## 1 · What is wrong, stated once

The binary opens a window and shows nothing. That is not a bug in the window. It is
this, in `src/facade/ports/surface.rs`:

```rust
fn submit(&mut self, placement: &Placement) {
    // The narrowing point. f64 world → f32 device, once, here.
    let mut verts: Vec<[f32; 2]> = Vec::with_capacity(placement.placed.len() * 4);
    for item in &placement.placed { /* ... four corners ... */ }
    self.narrowed = verts.len();
    let _format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let _ = (_format, verts);          // ← the vertices are dropped here
}
```

`submit` computes the geometry correctly and throws it away. There is no adapter, no
device, no queue, no swapchain, no shader, no pipeline, no command encoder, no render
pass, and no present anywhere in the repository. `src/portal/device.rs` constructs a
`wgpu::Instance` and exposes `instance()`, and **`instance()` has no callers**;
`main.rs` binds the device to `_device` and hands the window a `Store` instead.

**Everything upstream of that line works.** The store opens, genesis seeds, `Scene`
resolves records to `Placeable`s, `place` produces correct rectangles with correct
transforms and a correct precision floor, `probe` answers points with addresses,
`link` links the behaviour composition, `interpret` runs it, the pending set and the
session WAL survive a kill, and tier 0 passes equivalence. The last hundred lines of
a four-thousand-line pipeline are missing. That is a better position than it looks.

### 1.1 Why nine green stages did not catch it

Every green check in `EDITOR-BOOTSTRAP.md` is satisfiable without a pixel.

- **E3's** check reads *"the agreement test passes against a real surface with a
  non-zero origin."* `tests/agreement.rs` asserts that `visible(&view)` and
  `View::embedding()` agree over eighty sample points. That is pure arithmetic on
  `f64`. The word *surface* in the check means `SurfaceRect` — a struct of three
  points — and not a drawable. The test is a good test. It is not a test of drawing.
- **`PRESENTER.md` S8** reads *"landed — the S3–S7 tests pass unchanged against real
  `infinite-db` and a real wgpu `Surface`."* The `Surface` names one `wgpu` type and
  uses none. **That status line is false**, and it is the exact mechanism
  `HISTORY.md` traces every recorded drift to.
- **E7's** check — the deliverable — is `tests/self_edit.rs`, which drags by calling
  `store.amend` directly and asserts on stored records. It never opens a window.

R23 says a claim about rendering or interaction states its verification method. Every
stage did state one. None of the methods could distinguish a working renderer from a
`let _ =`. **R23 needs the second half it never got: the method must be capable of
failing for the reason the claim would be false.** §5 makes that structural.

The ordering is the other tell. E9 — compilation, tier 0 — landed before a single
pixel, and the plan's own sentence was *"if a stage can be cut, cut it from the E8
end."* E8 and E9 landed while E3's actual claim did not.

---

## 2 · Findings for `EDITOR-BOOTSTRAP.md` §9

Recorded there, not here, when E10.0 lands. Stated here so E10.0 has its content.

**11. Nothing has ever drawn a pixel.** §1. `src/facade/ports/surface.rs:56–60`
discards the frame; `src/portal/device.rs::instance` has no callers; `main.rs` never
hands the device to the window. Two status lines are false and are corrected by
E10.0: `PRESENTER.md` S8, and this plan's E3.

**12. The window's geometry never reaches the presenter.** `Store::set_surface` has
five callers and all five are tests. `portal/input.rs::on_resize` amends
`/input/surface` in the pending set, and **nothing reads that address** — grep for
`addresses::SURFACE` returns the constant, the tag, and the one amend. So the running
binary places every frame against the 800×600 default at origin `(0, 0)` installed by
`facade::open`, whatever size the window actually is. Fixed in E10.3.

**13. `Placed` carries no style and nothing resolves one.** `PRESENTER.md` is right
that it should not — *"what is deliberately not here: a colour"* — and `Placeable`
does carry a `style: Box<str>`. But `place` drops it, `frame` builds the `SceneSet`
internally and drops that too, and `editor::styles::bootstrap_default` has exactly one
caller: `genesis`, encoding the row it writes. **Nothing on a draw path ever reads a
style.** A renderer written today would have no colour to use. This is a real design
gap, not an oversight, and it wants D41 (§4.2).

**14. Two of the six interactions in §1 of `EDITOR-BOOTSTRAP.md` do not exist.** The
list is zoom, pan, select, drag, wire, read-a-finding. `src/portal/window.rs` handles
`CloseRequested`, `CursorMoved`, `MouseInput`, `KeyboardInput`, `Resized` and
`RedrawRequested`. There is no `MouseWheel` arm and no pan gesture. The camera is
`default_camera()` — a constant — except inside `zoom_to`, which only findings call.
Since zoom is *the primary navigation* (D20) and the argument for why visual
programming scales at all, its absence is not a missing convenience. Fixed in E10.5.

**16. `Placement` cannot express the grouping the presenter is said to own.** D15 and
D29 both give this layer *"what is uploaded, in what order, at what detail, **grouped
how**"*, and D29 leans on that word to argue the facade owns only the API. But
`Placement` is a flat `Vec<Placed>` of address, rect, level, clip, accepts. There is
no batch, no pipeline affinity, no notion that these things share a draw. With one
quad pipeline the gap is invisible and the split holds by luck. **E8's wires are
lines, and a label is a text run** — the moment a second primitive exists, either the
facade invents the grouping (and D29's split quietly moves, which is `hyper-ui`'s
failure relocated rather than avoided) or `Placement` grows a way to say it. Decide
before E10.4 hardcodes one pipeline, not after. See O20.

**19. Address canonicalization makes multi-level nesting structurally unreachable.**
`Inner::coord`/`Inner::bytes_of` (`src/facade/open.rs`) map every address the facade
ever hands the presenter to exactly 4 bytes — a short key is right-aligned into a
`u32`, a longer one is FNV1a-hashed, and either way `Addr::prefix_bits()` comes back
32. `place_group`'s recursion into a `hosts_space` child (`crates/infinite-presenter/
src/core/place.rs`) fires only when `level > item.at.prefix_bits()`, and `level` is
clamped by the surface-size floor to roughly 9–12 (`crates/infinite-presenter/src/
core/place.rs::surface_floor`) — never 32. No genesis, however deep, changes this: the
guard compares against a constant. §3.6's instruction to "seed a deeper genesis...
before writing the check, or the check cannot fail" cannot be satisfied by seeding
alone. Fixed by neither E10.5 nor any stage before it; see O23.

**15. The portal's coordinate spaces are unreconciled.** `CursorMoved` delivers
physical pixels; `SurfaceRect` carries a `scale_factor` the placement multiplies by;
`probe_at` takes the raw values. On a 1.0-scale display these agree, which is why no
test noticed. On the first HiDPI display they will not. Fixed in E10.3.

---

## 3 · The stages

### 3.1 E10.0 — The findings, and the two false status lines

No code. Write findings 11–15 into `EDITOR-BOOTSTRAP.md` §9, and change two status
lines to what the code earns.

**Do this first and do not fold it into E10.2.** A repository whose status table is
wrong in a known place, while work proceeds elsewhere, is the condition every prior
attempt died in. The correction costs an hour now; discovering it a third time costs
the codebase.

**Decision record owed: D41 — a status line cites the test that could fail.** §5.
**Rejected alternative:** a reviewer checklist — rejected because the four green
checks that passed here were each reviewed and each read as adequate. The defect is
not that nobody looked; it is that looking could not distinguish the two cases.

### 3.2 E10.1 — Headless readback, and the check that fails

**Write the test before the renderer.** `tests/pixels.rs`:

1. Open a store, seed genesis, set a surface, place.
2. Create a `wgpu` device with no window — request an adapter with no compatible
   surface, render into an offscreen `Texture` with `RENDER_ATTACHMENT |
   COPY_SRC`, `copy_texture_to_buffer`, map, read.
3. Assert the pixel at the centre of node A's rectangle is node A's authored fill.
4. Assert a pixel in the gap between A and B is the background.

**Run it against today's `Surface` and watch it fail.** That is the whole point of
writing it first, and it is the discipline `PRESENTER.md` §11 already states for
`check-rules.sh` — a check that has never been seen to fail is a check nobody knows
the polarity of. Record in the commit message what the failure looked like.

This test needs no window, no display server and no GPU vendor: `lavapipe` or
`llvmpipe` serves it, so it runs in CI. **It is the check E3 should have had.**

*Cost, stated:* a software rasteriser's output is not bit-identical to a discrete
GPU's, so the assertion is a tolerance on a small number of sampled pixels rather
than a golden image hash. That is enough to distinguish *drew the thing* from *drew
nothing*, which is the failure that actually happened, and §6 (O18) carries the
golden-image question forward rather than pretending it is settled.

### 3.3 E10.2 — The device reaches the window

**First, the ownership question, because it is why `Device::instance` has no callers.**

`wgpu` is legally in two places under D32: `src/portal/` may name graphics crates, and
so may `src/facade/ports/surface.rs`. Today the device lives in the portal and the
thing that needs a device lives in the facade, **and there is no path between them.**
That is not an oversight in E3; it is a seam nobody has had to draw yet. Draw it now.

| | Who owns what | Cost |
|---|---|---|
| a | **Portal owns instance, adapter, device, queue and swapchain. Per frame it acquires the texture and lends `&Device`, `&Queue` and a `&TextureView` to the facade's `Surface`, which encodes everything including the clear.** | `Surface` is constructed per frame with borrowed handles, so pipeline and buffers need somewhere to live across frames — the facade, cached on `Inner`, or rebuilt each frame at a cost |
| b | Facade owns every `wgpu` object; the portal hands it a raw window handle | The facade names `raw-window-handle` and owns swapchain resize, which is an OS event arriving on the portal's side. Puts an OS-shaped resource behind the wrong seam |
| c | Both keep what they have and something ferries between them | Two owners for one device. F-7's shape |

**Take (a), and note that `check-rules.sh` already argues for it.** The `f32` check is
`find src -name "*.rs" ! -path "src/facade/ports/surface.rs"` — it covers the portal
too. So the portal may not hold a vertex, an instance buffer, or a clear colour
without tripping a rule check, and under (a) it never needs to: it holds a window, a
device, a queue and a texture view, and every float belongs to the facade. Under (b)
the check survives as well but D18 is strained. Under (c) the check fires the first
time the portal touches a colour.

The division that falls out is the one D18 already states: **the portal owns the
OS-shaped resources — window, surface, swapchain, and the device born from them. The
facade owns the drawing.** Nothing about that contradicts D29; it is the same
organization/API split one level further down.

`portal/device.rs` grows adapter, device and queue. `portal/window.rs` creates a
surface from the window and configures it on resize. `RedrawRequested` acquires a
frame, hands the view to `Store::draw`, presents.

**The first pixel is already self-hosted, and this is the cheap trick worth taking.**
Do not clear to a hardcoded colour. Clear to the fill on the canvas space's style row,
read through `Scene`. It costs one lookup and it means the very first pixel the
project ever draws is proof of the whole chain — store → scene → place → surface →
screen — end to end. A person can then verify the entire pipeline by editing one
record and restarting. Compare a hardcoded clear colour, which proves that `wgpu`
works.

**Do not put `wgpu` types in `portal/drive.rs`.** The tick path is checked by
`check-rules.sh` for `block_on`, and adapter and device requests are async in `wgpu`.
Resolve them once at window creation, inside `window.rs`'s `resumed`, where blocking
is the event loop's own concern and not the runtime's. If that turns out to need a
`block_on` where the grep can see it, that is a finding, not a `#[allow]`.

**Decision record owed: D42 — the portal owns the device; the facade owns the
drawing; and the device is resolved at window creation, not in `drive`.** State the
`f32` check as the structural argument, since it is one that cannot be argued with.
**Rejected alternatives:** (b) and (c) above; and an async tick — rejected by R8 and
L1, since the runtime owns no thread pool and no executor and D24's entire argument is
that the input path never waits on anything.

### 3.4 E10.3 — The surface geometry is the window's

Close finding 12. `Resized` and `ScaleFactorChanged` must reach `Store::set_surface`.

**The question this stage actually asks is which path the geometry takes**, and the
plan's §3 already answered it for input in general: OS events become amends at
well-known addresses, and the composition reads them as ordinary inputs. `/input/surface`
exists, is amended, and is read by nothing. So either:

1. **`set_surface` is called directly from the portal**, and `/input/surface` is
   deleted as an address nothing uses. Honest, and it makes the surface a portal
   concern like the window title.
2. **The tick reads `/input/surface` from the pending set and applies it**, which is
   what §3 promised and is one more place the runtime is doing work. Then a
   composition could react to a resize, which is a real capability the editor does not
   yet want.

**Take (1) now and record the trigger for (2)** — *when a composition needs to read
the surface size*. Do not build both. Two write paths for one fact is F-7's shape.

**Decision record owed: D43**, with the above as its rejected alternative.

Fix finding 15 in the same change: divide `CursorMoved` by the scale factor at the
portal boundary, so everything above it is in logical coordinates, or carry physical
throughout — one of the two, stated in `EDITOR.md`, checked by a test at scale 2.0.

### 3.5 E10.4 — The placement becomes pixels

The real work, and it is smaller than it looks: one instanced quad pipeline, one
vertex buffer of unit-square corners, one instance buffer of `[x, y, w, h, r, g, b, a]`
per `Placed`, one draw call. Draw order is already correct — `Placement::placed` is in
address order within a level and level order across levels, and that is what the
buffer order is.

**The style resolution is the design decision, not the shader.** Finding 13:
`Placed` has no style, deliberately. Three ways to get one, and the choice is D44:

| | How | Cost |
|---|---|---|
| a | The facade's `Surface` holds `Arc<Inner>` and reads the style row per `Placed.at` | A store read on the draw path, per thing, per frame. F-7's neighbourhood |
| b | `place` copies `Placeable::style` into `Placed` | Contradicts `PRESENTER.md`'s explicit *"no colour here"*, and needs a decision amending it |
| c | The facade builds an address → style-key map once from the `SceneSet` it already resolves, and hands it to `Surface` alongside the `Placement` | The `SceneSet` is already in hand in `Store::draw`; nothing is re-read; `Placed` stays clean |

**(c) is the one to take**, and note *why* it is available: `Store::draw` already
calls `placed_in` itself for `place_now`, so the set is there. The only obstacle is
that `frame(scene, surface, view, at)` builds its own set internally and drops it —
so `Store::draw` should stop calling `frame` and do the three steps itself, which it
half does already. **Whether `frame` survives at all is worth asking**: it is four
lines, the facade duplicates them, and a binding function nobody calls is R27's
defect.

*What (c) costs:* the style key is resolved to a descriptor by `editor::styles`, which
is the app, and the `Surface`, which is the facade, must not name the editor. So the
descriptor — four `f64` — crosses at `Store::draw`, and `Surface::submit` takes
colours, not keys. Say that in `FACADE.md` or it will drift.

**Green check.** E10.1's readback passes. Add: a probe at the centre of the brightest
readback region returns node A's address. That is the one assertion that ties the
picture and the pointer to the same rectangle, and P1 in `hyper-ui` is precisely the
two of them disagreeing.

### 3.6 E10.5 — Pan and zoom

`MouseWheel` → zoom, drag-with-space or middle-drag → pan. Both change the camera.

**The camera is not a field on the portal.** D5 says the camera is session-scoped, and
`Inner::camera` is already a `Mutex<Option<Camera>>` that `Scene::camera` returns —
so the authored path exists and is unused. Amend the camera at a well-known address
and let `Scene::camera` resolve stored ∪ pending, exactly as `Definitions` does. Then
zoom is undoable, replayable and inspectable for free, and the editor's viewport
survives a restart because it is a record like everything else.

*This is the stage where D20's claim gets tested for the first time.* Zoom changes
which level is the graph; `detail` and `place`'s `level > at.prefix_bits()` recursion
already implement it; nothing has ever exercised it with more than three spaces at one
level. Seed a deeper genesis — a canvas, two nodes, and two spaces inside node A —
before writing the check, or the check cannot fail.

---

## 4 · Effort, stated honestly

| Stage | Shape | Rough size |
|---|---|---|
| E10.0 | documentation | an hour |
| E10.1 | one test file, offscreen wgpu | 150–250 lines, a day if wgpu is unfamiliar |
| E10.2 | adapter/device/surface/present | 150–200 lines |
| E10.3 | two event arms, one call, one test | an afternoon |
| E10.4 | shader, pipeline, buffers, style path | 200–300 lines, plus D44 |
| E10.5 | two event arms, camera as a record, deeper genesis | a day |

Call it a week of evenings to a running, visible, draggable editor. Everything hard is
already done — that is the finding worth holding onto. **Check `wgpu` 30's signatures
on docs.rs before writing any of it**; the manifest pins 30, `device.rs`'s comment
already records that 29 removed `Instance::default()`, and surface configuration has
moved more than once.

---

## 5 · The change to how a stage lands

Finding 11's mechanism, not its instance. The stage table grows a fourth column.

| # | Stage | Status | **Verified by** | Green check |

`Verified by` names the test function, by name, that fails if the claim is false.
A stage may not be marked `landed` while that cell is empty or names a test that
cannot fail for the stated reason.

Applied retroactively to `EDITOR-BOOTSTRAP.md`, E3's cell reads
`cull_agrees_with_draw_on_a_real_surface`, and the moment it is written down next to
the words *"one space on screen"* the mismatch is visible to anyone reading the table.
That is the whole value: **the audit becomes a thing you can see rather than a thing
you have to do.**

`PRESENTER.md` §11 already holds the sibling discipline for `check-rules.sh` — break
it, watch it fire, restore it. This extends it from the rule checks to the stage
table, which is where it was needed and did not reach.

---

## 6 · Open

| # | Item | Trigger |
|---|---|---|
| **O18** | **Is the frame a registered derived artifact?** It is a pure function of a `Placement`, the style rows and the surface geometry — which is the definition D25 uses. If it registers, R12's generic discard harness audits *the screen*, and "the picture is correct" becomes a store-level property rather than a person looking at it. It is also the most literal possible test of D25's claim that the mechanism needs no per-artifact code | E10.4. Cheap to try once a readback exists; a redesign later |
| **O19** | **Does the readback belong in `check-rules.sh` or in `tests/`?** It needs a GPU adapter, which no other check does | E10.1 |
| ~~O22~~ | **Closed.** `Placeable` grew `position: Point`; `place_group` offsets the local rect by it; genesis seeds distinct origins; `tests/pixels.rs::the_authored_screen_reaches_the_framebuffer` verifies two nodes land on distinct pixels. Landed silently — no status line accompanied it, which is what re-flagged it | — |
| **O23** | **Can `place_group` ever recurse into a nested space?** Finding 19. `Inner::coord`/`bytes_of` canonicalize every address to 4 bytes, so `prefix_bits()` is always 32 and `level` (clamped to the surface-size floor, ~9–12) can never exceed it — the recursion's guard is unsatisfiable regardless of genesis depth. Candidates: keep addresses variable-length past the facade boundary instead of canonicalizing to a fixed `u32`; or give `place_group` a different signal for "descend" than bit-length comparison. Either is a storage- or presenter-core change, not a genesis fixture | **Now.** It is what E10.5's own green check (D20's multi-level claim) needs to be writable at all |
| **O21** | **Does `infinite_presenter::binding::frame` survive?** Finding 17: no caller since `Store::draw_with` took the three steps itself, because D44 needs the `SceneSet` the placement was built from and `frame` builds its own and drops it. Four lines. R27 makes an uncalled binding function a defect | E10.5, or the first consumer wanting a frame without a fill map |
| **O20** | **Where does draw grouping live?** Finding 16. D15 and D29 give the presenter *"grouped how"* and `Placement` cannot say it. Either the artifact grows a grouping the presenter authors, or D29's split is amended to give the facade batching as well as API — one or the other, written down | E10.4 for the decision; **forced** by E8's wires, which are the second primitive |
| O16 | Where does the editor's undo live | Unchanged. E10.5 makes it sharper: if the camera is a record, panning enters the undo stream, and that is probably wrong |
| O14 | The precision floor | E10.4 is the first stage where `Placement::precision_floor` can be *seen* rather than asserted |
| O1 | Hot working set | E10.4. The measurement D30 asked for needs a frame that actually costs something |
