# infinite-solutions — Platform Facade Specification

> **Status:** draft 1, 2026-08-21. Satisfies R18 for the platform facade. R30: this
> is the compatibility surface; internals may churn, this document may not. A
> facade change requires a decision record; a layer change does not.
>
> Surface: **platform facade** (D10, D32). Rules: [`../RULES.md`](../RULES.md) ·
> Decisions: [`../DECISIONS.md`](../DECISIONS.md) · Charter:
> [`../CHARTER.md`](../CHARTER.md) · Layers: [`RUNTIME.md`](./RUNTIME.md) ·
> [`COMPOSITOR.md`](./COMPOSITOR.md) · [`PRESENTER.md`](./PRESENTER.md) ·
> [`DB.md`](./DB.md) · App: [`EDITOR.md`](./EDITOR.md) · Plan:
> [`../plans/EDITOR-BOOTSTRAP.md`](../plans/EDITOR-BOOTSTRAP.md)
>
> Records D32 and D33. The stages that implement this document are E0–E9 of the bootstrap
> plan; this specification does not own a stage table of its own.

---

## 1 · What this is

Four layers — store, compositor, runtime, presenter — behind **one** facade (R1, D10).
The facade is the only thing a domain facade or an app depends on (R30). It is not a
fifth layer: R1 says a fifth layer requires a decision record before it gets a
directory under `crates/`, and D32 records why the facade lives in the root package
instead.

The facade's whole job, stated once:

> **Implement the thirteen ports the three layers declare, convert the three `Addr`
> types and the two `Revision` types, register derived artifacts neither crate can
> see, and narrow `f64` to `f32` in exactly one file.**

It names every layer crate. Nothing else in the root package may. The portal
drives cadence and input through `Store::tick` and `Store::amend`, so
`src/portal/` never names a layer (D32, D24).

---

## 2 · The forcing consumer (R19, R26)

**The editor** — the platform's own graph editor (O11, [`EDITOR.md`](./EDITOR.md)).

R19 requires the consumer be *something that breaks if this surface is wrong*. Six
breakages, each of which is a seam the layers cannot close themselves:

| # | If the facade is wrong | What breaks |
|---|---|---|
| F1 | An `Addr` conversion is more than a newtype unwrap, and nobody notices | O13's trigger has fired and the deferral has become a permanent condition — D29's own words about how a deferral dies |
| F2 | `StoreWrite::submit` blocks | R14, D24, and the editor's input path. Typing stalls when the queue is full |
| F3 | `Definitions` reads the store and not stored ∪ pending | C4 is unimplementable: the editor cannot ask *"if I drop this wire here, does it link?"* about a wire that is not yet committed |
| F4 | Artifact registration is attempted in a layer crate | It cannot compile. D23 forbids the runtime naming another layer; D26 and D29 forbid the others naming the runtime. D30 records that `RUNTIME.md` §5.2 is wrong about this |
| F5 | `f32` appears anywhere but `src/facade/ports/surface.rs` | The precision floor (O14) becomes jitter instead of a finding. `hyper-ui` already paid this |
| F6 | A store error is mapped to an empty result | A failed query and an empty screen are indistinguishable (`PRESENTER.md` §13 finding 8) |

The editor is also the consumer that *uses* the facade as an app: `src/editor/` names
no layer crate and no graphics crate (R2). The grep in `scripts/check-rules.sh` is
what makes that true rather than intended. That is D32's cost, stated in D32.

---

## 3 · Thirteen ports, and which store facility backs each

The layers declare ports as traits. The facade implements them. **A sixth runtime
port, a sixth compositor port, or a fourth presenter port requires a decision
record** (D23, D26, D29). This document does not add any.

Implementations live under `src/facade/ports/`. The trait definitions live in the
layer crates. The facade does not re-declare them.

### 3.1 Runtime (five)

| Port | Backed by | Notes |
|---|---|---|
| `StoreRead` | `infinite-db`'s `ReadTxn` plus `QueryOptions` over an address range at a revision | Records pass through; they are never retained (R11). A failed query is a failed query, never an empty range (F6) |
| `StoreWrite` | `InfiniteDb::try_insert` (D33) | **Non-blocking and fallible.** Returns `Accepted` or `Full`, never a wait (D24). The store's blocking `enqueue` stops at this boundary |
| `StaleFeed` | `staleness_closure::{FreshnessReport, StaleTarget}`, `check_hyperedge_freshness`, `query_stale_downstream`, the `engine/derivation/` bus, watermarks | This is what makes an external writer cost responsiveness and never correctness (D6) |
| `Clock` | `std::time::Instant` | The layers may not name `std::time` (R10). This file may. Hands a monotonic count to `tick`; the runtime owns *now* but owns no thread (L1) |
| `Journal` | the session WAL — `engine::session_wal_store`, `WalDurability` | Driven at E2: `append` is `insert_with_session`; crash replay restores the pending set before the first tick. The runtime *calls* it and does not *implement* it (L2) |

### 3.2 Compositor (five)

| Port | Backed by | Notes |
|---|---|---|
| `Definitions` | a range read of a definition space, **unioned with the runtime's pending set** | That union is C4 and it is the port's whole point. Speculative sets are ordinary input |
| `Blocks` | a string-keyed native-block registry in the facade, populated at startup by the editor | R4. The compositor knows a block's *shape*, never what it computes (L3). O10: this is where *"may this composition use that block"* would be inserted; do not build it so that the check cannot be |
| `Values` | read at an address; writes go through the pending path, never straight to the queue | D24. A derivation result is not a keystroke, but it still must not stall the tick |
| `Provenance` | `ComputationProvenance`, `infinitedb_core::provenance` | D11. Half of this exists in the store and has never been driven |
| `Backends` | the compiled-form registry; tier 0 after E9 | D28. A backend registers *by passing the equivalence harness*, not by existing |

### 3.3 Presenter (three)

| Port | Backed by | Notes |
|---|---|---|
| `Scene` | a range read mapping node props to `Placeable` — extent, style key, detail override, hosts-a-space | Reads a set at a revision, same shape as `Definitions`, same reason: pending geometry (D8). O10: this is where *"may this viewer see that space"* would be inserted; a placement that has already been built is too late |
| `Surface` | wgpu. `f64` narrows to `f32` once, here | **The only f64 → f32 narrowing in the repository**, and the only file in `src/` that may name a graphics crate besides `src/portal/` and `src/facade/ports/glyphs.rs` (D29) |
| `Glyphs` | a declared box until a style names a font (plan §9 finding 10) | The one measurement the presenter cannot make itself. `hyper-ui` invented `char_w = font_size * 0.55`; a port is the correction |

---

## 4 · `Addr` and `Revision` (O13)

`src/facade/addr.rs` holds five conversions:

| Conversion | From | To |
|---|---|---|
| address | store bytes | `infinite_runtime::core::Addr` |
| address | store bytes | `infinite_compositor::core::Addr` |
| address | store bytes | `infinite_presenter::core::Addr` |
| revision | store revision | `infinite_runtime::core::Revision` |
| revision | store revision | `infinite_presenter::core::Revision` |

**Each conversion is a newtype unwrap.** A test in `tests/seam.rs` asserts it. The
compositor has no `Revision` (it has no *now* and no logical time of its own: linking
at revision N is a pure function of the definitions at N).

> **O13's trigger.** The moment a conversion needs logic — a byte swap, a length
> check, a version tag, anything that is not wrapping or unwrapping the inner value
> — promote `Addr` to a zero-dependency crate, with a decision record. Do not
> quietly add the logic.

That crate would not be a fifth layer (R1 is not engaged). It still needs a decision
record. The trigger existing and never being watched is how a deferral becomes a
permanent condition.

---

## 5 · Artifact registration (D25, D30)

The runtime knows artifact *lifecycle*, never artifact *content* (D25). A derived
artifact is registered under a string key with the address ranges it derives from, a
rebuild function, and a validity watermark. R12's generic discard harness then covers
it with no per-artifact test code.

**Registration happens in `src/facade/artifacts.rs`, not in either crate.**

| Artifact | Key | Function owned by | Schedule owned by |
|---|---|---|---|
| `Placement` | `"placement"` | presenter (`place`) | runtime |
| `Plan` | `"plan"` | compositor (`link`) | runtime |

Neither layer can do this:

- D23 forbids the runtime naming another layer.
- D26 forbids the compositor naming the runtime (or the presenter).
- D29 forbids the presenter naming the runtime (or the compositor).

D30 records that `RUNTIME.md` §5.2 is wrong about this — it says the artifact is
*"registered by the presenter's binding"*; it cannot be. The presenter's
`binding::artifact` exposes the three parts D25 asks for; the facade hands them over.
The compositor's plan is the same shape one layer over.

This is D25's fourth and fifth instances. If they work here with no per-artifact
machinery, D25 is vindicated in the place it was written for. If per-artifact code
turns out to be needed, D25 is wrong and that is a finding, not a workaround.

---

## 6 · The single narrowing point (D29, `PRESENTER.md` §3.3)

> **`f32` appears in this repository in exactly one file:
> `src/facade/ports/surface.rs`.**

`f64` runs from the presenter core to the surface implementation and narrows once, at
the last possible moment, inside that file. `scripts/check-rules.sh` pins it.

*Why.* `hyper-ui` ran an `f64` world through an `f32` camera and narrowed and widened
twice a frame. An address space whose premise is unbounded refinement was being
projected through 24 bits of mantissa. That is the difference between a precision
floor that can be detected (O14) and one that shows up as jitter.

The presenter's `Placement::precision_floor` reports the *fact* — the shallowest
address at which the surface ran out of bits. `src/facade/finding.rs` turns that fact
into a `Finding` (`COMPOSITOR.md` §6) with a site, a said, a wanted and a remedy.
The presenter defines no second `Finding` type (R17). A failed query is not an empty
screen (F6).

---

## 7 · What this facade does not own

Stated so the list cannot grow by accident (R27).

- **Appearance.** Authored spaces in the store, read through `Scene`. Not blocks.
- **Behaviour.** An authored composition, linked through `Definitions`. Not Rust in
  the portal, from E6 onward.
- **The window, the device, the event loop.** `src/portal/` — the operating-system
  boundary (D18). The portal names graphics and windowing crates; the facade's
  `Surface` and `Glyphs` implementations do too. The editor names neither.
- **Well-known addresses.** `src/editor/addresses.rs` is the bootstrap ABI (D34).
  The facade converts addresses; it does not mint them.
- **Tag meaning.** Tags are matched, never interpreted (D13). The editor's
  convention lives in `src/editor/tags.rs`.
- **A sixth / sixth / fourth port.** A decision record, or it does not exist.

---

## 8 · How this is checked

| Rule | Check | Lives in |
|---|---|---|
| R2 — only the facade names a layer | no `infinite_(db\|runtime\|compositor\|presenter)` in `src/editor` or `src/portal` | `scripts/check-rules.sh` |
| R2 — `main.rs` is thin | ≤ 60 non-comment non-blank lines | same |
| D29 — one narrowing point | no `f32` in `src/` except `facade/ports/surface.rs` | same |
| L5 — no map keyed by anything but an address | `maps_keyed_by_addr src` | same |
| D24 — input is not a write | `portal/input.rs` never names `submit` or `StoreWrite` | same |
| R8 / L1 — the portal drives | no `std::thread`, `sleep`, or `block_on` in `portal/drive.rs` | same |
| F-8 — no `mod.rs` | none under `src/` | same |
| O13 — conversions are unwraps | `tests/seam.rs` | E1 |
| D24 — `submit` returns `Full` rather than blocking | `tests/seam.rs`, saturated queue | E1 |

R30: a change to this document, or to the port count, or to where `f32` is legal,
requires a decision record.
