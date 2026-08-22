# infinite-presenter

The **presenter** layer (D2, D15, D17). The embedding: address to screen, screen to
address, culling, and level of detail.

**This file does not restate the specification** (R17, R21). Read
[`docs/specs/PRESENTER.md`](../../docs/specs/PRESENTER.md). Everything below is about
the crate as a crate.

> **L5 — mints no identity.** Every reference to a thing is the store's address.
> **L6 — authors nothing.** There is no write port.

---

## Status

Stage **S2** of the specification's stage table. The types and the exact arithmetic
are here; the stage functions are `todo!()` and each names the stage that lands it.
Per R20 the stage table's status column is stamped by the change that lands the stage,
not by the change that writes the files.

## Build

```
cargo build -p infinite-presenter                    # the core alone. [dependencies] is empty
cargo build -p infinite-presenter --features binding # adds the three ports and the frame path
bash scripts/check-rules.sh                          # the mechanical checks, all three layers
```

## Four structural choices worth knowing before reading the source

**1 · There is no GPU in this crate.** D15 gives the presenter *"wgpu resource
organization"*; the organization is here and the API is the facade's, with
`binding::ports::Surface` as the seam. The reason is not purity. `hyper-ui` names
`wgpu` and `winit` beside its camera and its layout, so testing the embedding would
mean standing up a device and a window — and of the 21 files of that crate read while
writing the specification, `renderer/` has **zero** tests and `geom/` has **zero**.
That is where its one live embedding bug lives.

**2 · One transform per space, never one per thing.** A thing's position is the fold of
the transforms along its path. `infinitedb-spatial-layer.md` §7 states this as a
property of the store — a subtree moves rigidly and only its one embedding transform
changes — and holding one per thing would be a map from identity to geometry, which is
F-2's shape in this layer. It is also why a pan is O(1).

**3 · One scalar, and it is `f64`.** `check-rules.sh` greps for `f32` and fails.
Narrowing happens inside the `Surface` implementation, once, in the facade.
`hyper-ui` runs an f64 world through a 32-bit camera and converts both ways across the
same boundary twice a frame, which projects an address space whose whole premise is
unbounded refinement through 24 bits of mantissa.

**4 · The core has zero enums, and the count is pinned.** The obvious candidate —
`Visibility { Shown, Collapsed, Hidden }` — is a number here instead, because under
D20 *a node is a space seen from one level out*, so collapse **is** zoom. A fourth rung
later is a different number rather than a new variant.

## One thing this change raises for decision, rather than deciding

R29: a proposal is corrected, not merged, and a rename proposed by an assistant is
exactly that class of change.

- **`RenderList` is renamed `Placement`.** D5 and D25 use the old name. It answers
  pointer queries, which is not rendering; it holds no draw commands, so *list*
  describes the wrong thing. R17 permits a rename and forbids a recycle — `RenderList`
  is retired, not reused — and D20 set the precedent when it retired "chart" for
  "space". Flagged in `docs/specs/PRESENTER.md` §13 finding 2, and one `sed` from being
  reversed.

## Conventions

Taken from `infinite-runtime` and `infinite-compositor` rather than re-decided (R17):
`module.rs` plus a directory of leaf files and no `mod.rs` (F-8); module files declare
`mod` privately and re-export; one public function per file for **free** functions,
while a type with an inherent impl is one file; `edition` / `rust-version` / `license`
/ `publish` inherited from `[workspace.package]`; `autotests = false` with explicit
`[[test]]` targets so `tests/fakes.rs` can be a shared helper without becoming a test
target.
