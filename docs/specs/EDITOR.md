# The editor — Application Specification

> **Status:** draft 1, 2026-08-21. Satisfies R18 for the app that lives in
> `src/editor/`. The editor is not a layer; it is the forcing consumer named in
> all three layer specifications and in [`FACADE.md`](./FACADE.md).
>
> App: **the editor** (O11, D16, D32). Rules: [`../RULES.md`](../RULES.md) ·
> Decisions: [`../DECISIONS.md`](../DECISIONS.md) · Charter:
> [`../CHARTER.md`](../CHARTER.md) · Facade: [`FACADE.md`](./FACADE.md) · Plan:
> [`../plans/EDITOR-BOOTSTRAP.md`](../plans/EDITOR-BOOTSTRAP.md)
>
> Self-hosting is answered **yes** (bootstrap plan §0.3). O11 is closed by D36.

---

## 1 · What is being built, exactly

**The first screen is a canvas.** It shows the sibling spaces of one space as
rectangles, and the hyperedges among them as lines. You can:

- **zoom** — which changes which level is the graph (D20);
- **pan**;
- **select** a space by clicking it;
- **drag** a space, which changes its authored position;
- **draw a wire** between two spaces (E8);
- **read a finding** when something does not link (E8).

That is the whole list.

There is **no** property panel, no menu bar, no block palette, no toolbar, no file
dialog, no settings, no theme picker, no undo UI, and no text editing beyond what
E8's wire needs.

**R27 is the reason.** Generality is a defect unless a named consumer requires it.
The named consumer is D16's proof obligation — *if the editor cannot be built in
the platform, a child cannot build an app in it* — and it is satisfied by a canvas
that can compose. Everything on the list above is required to demonstrate that a
graph can be edited by a graph. Nothing else is.

**A screen element that is not on that list is a finding, not a commit.**

---

## 2 · What self-hosting means, and what it does not

### 2.1 Appearance is authored data. It is not blocks.

D15: widgets are **not code**. The presenter's `Scene` port reads, per address: an
authored extent, an opaque style key, a detail override, and whether the space
hosts a sub-space. All four are records in the store. Drawing a rectangle is not a
block invocation; it is `place` walking an address range and `Surface` accepting
the resulting work.

The editor's chrome is spaces, not a composition. A canvas is a space. A node's
box is a space. A selection highlight is a space with a style key. None of that is
linked, none of it is interpreted, and none of it costs a plan step.

**There is no `rectangle` block, no `label` block, no `panel` block, no `widget`
block.** `scripts/check-rules.sh` greps for those filenames under
`src/editor/blocks/`. If a visual element needs to exist, it is authored into the
store by genesis (§6) or by the editor itself (E7).

### 2.2 Behaviour is a linked composition. That is what is self-hosted.

What *is* a composition is the editor's behaviour: what happens when the pointer
moves, when a button goes down over a space, when a drag ends. That composition is
authored graph data, linked by `link`, executed by `interpret`. E6 and E7 prove
it. The verification method (R23) is: delete the composition, observe that
dragging stops working while the window still runs.

### 2.3 Three things stay native forever

D16 has two tiers of user and they are permanent: block authors write native
primitives in Rust; app authors compose visually and never see Rust. The editor is
in the second tier. It does not mean there is no Rust.

| | Where it lives | What reads it |
|---|---|---|
| **Appearance** | authored spaces in the store | the presenter, through `Scene` |
| **Behaviour** | an authored composition in the store | the compositor, through `Definitions` |
| **Primitives** | native Rust blocks, registered under string keys | the compositor, through `Blocks` |

The three native things, named so the boundary cannot drift:

1. **The portal** — window, GPU device, OS input, the tick loop. The platform
   boundary (D18). `src/portal/`.
2. **The native blocks** — §5. Six to start.
3. **Genesis** — the seed that writes the editor's own composition into an empty
   store. Native, forever, on one condition: **it writes graph data and nothing
   else.**

---

## 3 · The OS is a portal

D18: a platform boundary is a portal. Apply that to input. The operating system is
another graph. The window, the pointer and the keyboard sit on the far side of a
portal, and the module that owns that seam is `src/portal/`.

**Input arrives as pending amends, never as writes.** The portal converts an OS
event into an `amend` on a pending entry (D8, D24) at a well-known address from
`src/editor/addresses.rs`:

| Address | Carries | Amended when |
|---|---|---|
| `/input/pointer/position` | a point in surface coordinates | every pointer move |
| `/input/pointer/button` | a flag per button | on transition only |
| `/input/key` | the key event | on transition only |
| `/input/surface` | size, scale factor, origin | on resize |

The editor's behaviour composition reads those addresses as ordinary inputs. Three
things follow, all free:

- **R14 holds by construction.** A keystroke is an amend to a bounded in-memory
  set plus a journal append. It never touches the store's write queue.
- **The editor can react to input that has not been committed**, which is C4
  arriving at the input edge.
- **A drag in progress is enumerable.** D8's pending category is doing real work;
  an "unsaved" indicator (B4) is implementable because `list` returns everything.

There is no other input path, and there is no code path from an OS event to
`StoreWrite`. `scripts/check-rules.sh` greps `portal/input.rs` for both.

---

## 4 · What breaks if the editor is wrong (R26)

D16's proof obligation: *if the editor cannot be built in the platform, a child
cannot build an app in it.* Instantiated:

| # | If the editor is wrong | What that proves |
|---|---|---|
| E-a | The first screen is coded in Rust rather than authored | Self-hosting is a claim, not a property. The genesis discard test (E4) is the check: delete every space under the screen root, restart, the portal still runs and the canvas is empty *with a finding*, not a black screen; re-run genesis, bit-identical |
| E-b | Dragging a space is performed by Rust in the portal | The composition is decoration. E6 deletes it and the drag must stop |
| E-c | Editing a node of the editor's own screen does not persist across restart | The editor is not an app in the platform. E7, recorded on video (R23) |
| E-d | A tag mismatch is a stack trace, or has no site, or has no remedy | D16 ruled those out. E8: said / wanted / remedy, and zoom to the site (D20) |
| E-e | A seventh native block appears with no line here saying which of §1's six interactions requires it | R27, R32. Coach Assistant does not need `probe-at`; therefore `probe-at` is not platform. A silent seventh is how the native set becomes a widget toolkit |
| E-f | `src/editor/` names a layer crate or a graphics crate | The editor has stopped being an app under R2. The grep is the check |

---

## 5 · The native block set

Seven effect keys (E18a). Variety that is not store I/O lives in the pure-fn
table (`map` / `fold` dispatch), not in new natives. Registration is
`src/facade/ports/blocks.rs`. A plain Rust function over opaque payloads may
still live in `src/editor/blocks/` as table data; it is not a native.

| Key | Signature | Why it is needed | R32 (two domains) | §1 interaction |
|---|---|---|---|---|
| `probe-at` | point → address? | the pointer has to become an address, and only the presenter can do that | editor hit-test; any authored pointer tool | select, drag, wire |
| `read` | address → value | the composition has to see the store | editor; every interpreted graph | all six |
| `amend` | (address, value) → pending | the write path, and the only one (D24) | drag; field typing | drag, wire, type |
| `commit` | address → committed | the commit boundary has to be authored, not implicit | drag release; role-routed field commit | drag, wire, type |
| `gate` | (value, flag) → value? | "on press, not on move" has to be expressible | drag; selection / place / wire / type | drag, wire, type |
| `map` | (fn, val, aux?) → out | one machine; variety is a registered pure-fn key | offset/displace/set-origin; increment / append-char | drag, place, type |
| `fold` | (fn, left, right) → out | combination order (PARALLELISM home) | field_row; action_bar / panel | compose |

These are **app blocks, not platform concepts** (R32). Coach Assistant does not
need `probe-at`. So it is not platform, and it does not want a place in any layer.
It is registered under a string key by the app that needs it, which is exactly
what R4's registry mechanism is for.

**A new effect key needs a line in this section saying which of §1's six
interactions requires it, and a second consumer with no shared purpose (R32).**
Not a decision record, but not silent either. `increment-text`, `encode-selection`,
`encode-wire`, `set-origin`, `offset`, and `displace` are pure-fn keys, not natives.

---

## 6 · Genesis

`src/editor/genesis.rs` writes the editor's screen as spaces: a canvas space, a
space per node, the extents, the style keys, the detail overrides. From E5 it
also writes the behaviour composition.

It contains **no layout algorithm, no behaviour, no policy, and no conditional
beyond "does this space already exist".** It is a bootloader. If genesis ever
grows a decision, E4's discard test is how you find out: an emptied store that
renders a different screen on re-seed, or that crashes, or that goes black, is
genesis that stopped being a bootloader.

Well-known addresses live in `src/editor/addresses.rs` and nowhere else (D34).
A change to a well-known address is a migration, and there is no migration
machinery. Discovery by convention — scan for a marker prop — is rejected in
D34 because it makes an empty store and a corrupt store indistinguishable,
which is `PRESENTER.md` §13 finding 8.

---

## 7 · Tag convention (D13)

Tags are the app's. The platform's only operation on a tag is **match**.

The editor's tags, and this list grows only when a §1 interaction requires a
distinction the existing tags cannot make:

| Tag | Payload | Used by |
|---|---|---|
| `point` | two `f64`s, surface coordinates | pointer position, `offset`, `probe-at` |
| `address` | an `Addr`'s bytes | `probe-at` output, `read` / `amend` / `commit` input |
| `flag` | a byte, zero or not | pointer buttons, `gate` |
| `key` | a key event | `/input/key` |
| `surface` | size, scale factor, origin | `/input/surface` |
| `extent` | min / ideal / weight per axis, as stored | authored geometry, read through `Scene` not through a block |
| `value` | opaque bytes | `read` output, `amend` / `gate` input — E5, required by those three blocks |

The platform will not help the editor lay out a solver block differently from a
view block (D27's cost). That convention, when it exists, is authored here and
the compositor will not interpret it.

---

## 8 · Style keys

`src/editor/styles.rs` maps an opaque style key to a descriptor — fill, stroke,
corner, text run. The descriptor is data; `src/facade/ports/surface.rs` turns it
into wgpu. The editor names no graphics crate.

O17 is closed by D37: the table is authored under the style root when the store
has rows. `styles.rs` holds only the bootstrap default, so a store that has been
emptied still renders a legible colour rather than a black screen.

---

## 9 · How this is checked

| Rule | Check | Lives in |
|---|---|---|
| R2 — the app depends on the facade, not on a layer | no `infinite_(db\|runtime\|compositor\|presenter)` in `src/editor` | `scripts/check-rules.sh` |
| D29 — the app names no graphics crate | no `wgpu`, `winit`, `glyphon`, `cosmic_text`, `raw_window_handle`, `softbuffer`, `glam` in `src/editor` | same |
| §2.1 — no widget-shaped block | no `rectangle`, `label`, `panel`, `widget`, `button`, `text` file under `src/editor/blocks/` | same |
| D34 — addresses live in one file | no `\"/(input\|style\|screen)/` literal outside `src/editor/addresses.rs` | same |
| E4 — genesis discard | delete the screen root; restart; finding, not crash; re-seed; bit-identical | `tests/genesis.rs` |
| E6 — the drag is interpreted | delete the composition; dragging stops; the window still runs | `tests/behaviour.rs` |
| E7 — it edits itself | drag a genesis node; restart; it is still there; nothing was recompiled | `tests/self_edit.rs` |
