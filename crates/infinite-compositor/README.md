# infinite-compositor

The **compositor** layer (D2, D17). Blocks, ports, wiring, link-time validation, and
the contract a compiled form must satisfy.

**This file does not restate the specification** (R17, R21). Read
[`docs/specs/COMPOSITOR.md`](../../docs/specs/COMPOSITOR.md). Everything below is about
the crate as a crate.

---

## Status

Stages **S2–S5** of the specification's stage table, landed with E5. `link`,
`order`, `signature_of` and the findings corpus are live. S6–S8 remain ahead.

## Build

```
cargo build -p infinite-compositor                    # the core alone. [dependencies] is empty
cargo build -p infinite-compositor --features binding # adds ports, registries, execution
bash scripts/check-rules.sh                           # the mechanical checks, both layers
```

## Two structural choices worth knowing before reading the source

**1 · Determinism comes from the data structure, not from discipline.** A
`Composition` holds its blocks in a `BTreeMap`, so iteration order is address order and
a plan built by walking it is deterministic by construction. D19's equivalence law is
exact rather than statistical *only* because execution is deterministic; a plan whose
order depended on hash iteration would make the compile story unverifiable, and nothing
would announce it.

**2 · Open sets are string keys; the one closed set is pinned.** `check-rules.sh`
counts the enums in `src/core` and fails if the number changes, so R16's *"a new enum
requires a decision record"* has to be answered before the build goes green rather than
at review time.

## One thing this crate still raises rather than deciding

- **`Body`** in `core/block.rs` is deliberately **not** an enum (spec §14 finding 7):
  D18 added portals and D21 added iterative regions *after* the model was drawn,
  which is two new body kinds in two days. `Direction { In, Out }` is closed by D35.

## Conventions

Taken from `infinite-runtime` rather than re-decided (R17): `module.rs` plus a
directory of leaf files and no `mod.rs` (F-8); module files declare `mod` privately and
re-export; one public function per file for **free** functions, while a type with an
inherent impl is one file; `edition` / `rust-version` / `license` / `publish` inherited
from `[workspace.package]`.
