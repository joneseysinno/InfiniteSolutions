# infinite-db — Store Specification (this repository)

> **Status:** draft 1, 2026-08-21. Satisfies R18 for `crates/infinite-db`. This
> document is a pointer, not a restatement.
>
> Layer: **store** (D1, D2, D17). Rules: [`../RULES.md`](../RULES.md) · Decisions:
> [`../DECISIONS.md`](../DECISIONS.md)

---

**The store is `infinite-db`, and that is locked (D1).** It is published (0.4.x, MIT).
This repository does not specify it.

The store's own decision record is [`crates/infinite-db/SEMANTICS.md`](../../crates/infinite-db/SEMANTICS.md).
The spatial-layer doctrine — charts, curve addresses, permanence by divisibility,
restriction and aggregation, the address layer versus the embedding layer — is
adopted as theory by D1, not restated here. Restating it would put two documents
under one subject (R17) and would have to be kept in sync with a crate this
repository does not own (R21).

What this repository *does* decide about the store is recorded in
[`../DECISIONS.md`](../DECISIONS.md): D1 (it is the store), D6 (it is the only
writable model), D8 (the session WAL is the journal the runtime calls), D24 (a
keystroke is not a write; `submit` is non-blocking), D32 (this workspace path-depends
on the vendored copy until the facade needs no store change for two consecutive
stages).

A change to the store's semantics is a change to `infinite-db`, not to this file.
A change to how the facade *uses* the store is a change to [`FACADE.md`](./FACADE.md).
