# Infinite Solutions — E15–E16, identity and authoring

> **Status:** landed 2026-08-29. All stages E15.0–E16.2 green. O32 → D57; O34 → D58.
>
> **Records nothing** (R29) beyond the decision stamps named above.
>
> Rules: [`../RULES.md`](../RULES.md) · Decisions: [`../DECISIONS.md`](../DECISIONS.md) ·
> Charter: [`../CHARTER.md`](../CHARTER.md) · Stack:
> [`AUTHORING-STACK.md`](./AUTHORING-STACK.md) · Ledger: [`../SALVAGE.md`](../SALVAGE.md) ·
> Vocabulary: [`../vocabulary.md`](../vocabulary.md) · Predecessor:
> [`E13-AUTHORING-SURFACE.md`](./E13-AUTHORING-SURFACE.md) · Layer:
> [`../specs/EDITOR.md`](../specs/EDITOR.md)
>
> **This document is a specification, not code** (R28). E15–E16 are the hard blockers
> for every later stage that authors more than fifteen children or refuses to name every
> node in Rust — so they are written before they are built.

---

## 0 · What these stages are for, in one sentence

> **E15 makes identity derived and uncapped; E16 makes nesting authorable sugar that
> flattens to addresses — so genesis shrinks and wide trees become expressible.**

AUTHORING-STACK §0 measured the symptom: 83 address constants, 675 lines of genesis for
19 spaces, fifteen children maximum per space. E15 removes the ceiling and the store
scan. E16 removes the hand-written `SpaceRecord` litter. Neither is the product; E20 is.
A plan that lands E15–E16 and stops has not answered the question.

---

## Stage table

| # | Stage | Status | Verified by | Green check — and what must **stop** passing |
|---|---|---|---|---|
| **E15.0** | Encoding helpers: `child`, `MintSeed`; carried bits, no inference | **landed** | `editor::mint` unit tests; `facade::addr` | `child(parent, slot)` is pure; `MintSeed::next_slot` threads without I/O. `presenter_addr` / bootstrap paths supply **carried** bits. *Must stop passing:* new keys whose depth comes only from `significant_bits` inference |
| **E15.1** | Replace `next_child`; palette and wire mint by derivation | **landed** | `tests/mint_derived.rs` | **(a)** One parent holds **200** children. **(b)** Two `MintSeed`s mint concurrently with **no shared slot**. **(d)** Delete a child, mint again, undo the delete — restored value lands on **its own** address (finding 25). *Must stop passing:* `mint.rs` store scan over `SCREEN_ROOT..SCREEN_END`; `next > 0x0F` as a hard ceiling |
| **E15.2** | Bootstrap + genesis keys under the new layout; hierarchy still true | **landed** | `tests/genesis.rs` hierarchy; `tests/nesting.rs` | Nesting, probe, and apparent-size descent still hold under carried-length keys. *Must stop passing:* the old 4-byte nibble scheme as the **only** layout the editor understands |
| **E15.3** | Same authored seed → byte-identical addresses on two machines | **landed** | `tests/mint_identical.rs` | Two fresh stores, same named Spec / same seed path → **byte-identical** child addresses. *Must stop passing:* address identity that depends on what is already in the store |
| **E16.0** | `Spec` + builders: nested sugar → flat addressed puts | **landed** | `tests/spec_flatten.rs` | A nested Spec flattens to N keys; containment is **prefix of addresses only** — no containment field on the committed record. *Must stop passing:* hand-built sibling keys as the only nesting API |
| **E16.1** | R-C: genesis as builders | **landed** | `tests/genesis.rs` (E4 discard unchanged) | `genesis.rs` **under 150 lines** while seeding **strictly more** spaces than today. *Must stop passing:* 19 hand-written `SpaceRecord { … }` literals as the authoring style |
| **E16.2** | R-A: bootstrap ABI only in `addresses.rs` | **landed** | `editor::addresses` unit tests; six `pub const …_ROOT_KEY` | Bootstrap = six region roots. Content addresses are `child_key` / `content_key!` derived. *Must stop passing:* ~83 `pub const …_KEY` literals carrying app content |

**E15 before E16 is mandatory.** Builders that seed more than fifteen children fail under today's ceiling. E16.2 may land in the same change as E15.2 if rewriting keys once is cheaper; it must not land *before* derivation works.

**Out of scope.** E17 (open `SpaceRecord`), E18a/b (alphabet registration and stored components), E19 (focus), E20 (Innovator screen). Spec here is authoring sugar, not a stored component definition.

---

## 1 · The order, and why

**E15.0 before E15.1** so palette/wire swap onto a pure API that already has tests, rather than inventing encoding inside gesture code.

**E15.1 before E15.2** so the 200-child and concurrent-seed checks do not depend on rewriting every well-known key first.

**E15.2 before E15.3** so “identical on two machines” uses the layout the product will actually ship, not a side encoding.

**E16.0 before E16.1** so genesis is a consumer of Spec, not the place Spec is invented.

**E16.1 before E16.2** (if not simultaneous) so shrinking `addresses.rs` is deleting constants that builders no longer need, not orphaning genesis mid-rewrite.

---

## 2 · O32 locked — depth and minting (R-B) → **D57**

*Stamped as D57 when E15.0–E15.1 landed. Supersedes D53's store-scan mint. Amends D45's uniform-nibble layout and `significant_bits` as depth authority. O28 remains subsumed.*

### 2.1 · Encoding

- Addresses are **opaque byte strings**.
- Significant length is **carried** (`Addr.bits` on the presenter path; façade conversion passes bits from mint/bootstrap — **never** re-inferred by scanning for a last non-zero nibble).
- Child address = pure

  `child(parent_bytes, parent_bits, slot: u32) -> (bytes, bits)`

  that **appends a fixed-width slot field**: **2 bytes / 16 bits**. Enough for green check (a) (200 children). Slot **`0` is reserved / unused** so an empty suffix stays unambiguous if a length-prefix reading is ever needed for debugging — depth itself still comes from carried `bits`, not from forbidding zero in the payload.

### 2.2 · Why not keep four-bit nibbles

Breadth stays fifteen forever (`next > 0x0F` in `editor/mint.rs`). Widening the key to 128 bits buys depth only; per-level slice still caps siblings (AUTHORING-STACK §2.1). That is F-1's shape in the address scheme: enumeration with a small enum, not subdivision.

### 2.3 · Why D45's rejection of (a) no longer holds

D45 rejected variable-length keys *with no change to the descend rule*: eight bits per level would mean 256× magnification to enter a space. D45's other half already replaced that rule — descent is **apparent size** against `View::opening_extent`. Carried length and longer keys do not change when interiors open. Pairing (a) with the new descend rule was never evaluated; E15 evaluates it and takes it.

### 2.4 · Derivation (bion `IdSeed` shape, plain nouns)

| Path | How the slot is chosen | Which green check |
|---|---|---|
| Authored Spec / genesis | Slot from a **stable local name** (or an explicit index in the sugar) — deterministic, no store read | (c) byte-identical across machines |
| Interactive mint (palette / wire) | Session **`MintSeed`** threads `next_slot()` — no store scan, no entropy required for determinism within a session | (b) two sessions ⇒ two seeds ⇒ no collision |
| Delete + remint + undo | Slots are **never** recycled from `max+1` over live rows; D49 undo restores by address without aliasing a newly minted node | (d) |

Must die:

- `next_child`'s scan of `SCREEN_ROOT_KEY..SCREEN_END_KEY`
- `significant_bits` as the depth authority for new keys (delete it, or leave it only behind a test that nothing production calls it)

**D34 cost:** rewrite well-known keys in `addresses.rs`. No migration machinery. Acceptable because no data has shipped.

**R-A** rides with E15.2 / E16.2: bootstrap ABI (~6 region roots) stays literal; content keys become derived. Stop growing eighty-three constants.

---

## 3 · O34 locked — where Spec lives → **D58**

*Stamped as D58 when E16.0 landed.*

Authoring sugar lives in the **editor**: `src/editor/spec.rs` (builders beside it or in the same module). It calls façade `encode_space` / mint. The façade does **not** name the editor (R2).

[`vocabulary.md`](../vocabulary.md) remains **platform doctrine for the alphabet** (E18a). Spec is the **authoring seam**, not the alphabet. Conflating them recreates the junk drawer at a different layer.

Innovator precedent (SALVAGE §2): `Spec` is authoring sugar only — a transient nested value that `commit` flattens into nodes and edges. **Nesting at authoring time does not make the runtime a tree.** Infinite Solutions keeps that seam: flatten to addressed `SpaceRecord`s; containment is address prefix only.

---

## 4 · What each stage changes in code

```
Spec / builders ──► flatten ──► Addr + SpaceRecord ──► encode_space ──► store put
                         ▲
MintSeed / named slot ───┴── child(parent, slot) ──► presenter Addr { bytes, bits }
```

| Area | Files |
|---|---|
| Mint / seed | Replace `src/editor/mint.rs`; call sites `palette.rs`, `wire.rs` |
| Depth | `src/facade/addr.rs`; `crates/infinite-presenter/src/core/addr.rs` |
| ABI | `src/editor/addresses.rs` — R-A shrink |
| Genesis | `src/editor/genesis.rs` — R-C |
| Sugar | New `src/editor/spec.rs`; genesis becomes its first consumer |
| Tests | `tests/mint_derived.rs`, `tests/mint_identical.rs`, `tests/spec_flatten.rs`; keep E4 in `tests/genesis.rs`; update palette / wire / counter / nesting fixtures |

Presenter L5 still holds: the presenter mints no identity; it only carries bits the façade already knew.

---

## 5 · How this plan could fail the same way E13 did

**Guard 1 — the consumer is E20, named up front.** E15–E16 are justified only if they unblock wide authored trees and short genesis. A green check that can pass without advancing that is the wrong check.

**Guard 2 — every green check names what must stop passing.** The stage table's last column is load-bearing (D41).

**Guard 3 — E16 must not smuggle a widget toolkit.** Builders produce spaces, shapes, and extents — not `Button` / `Panel` core types. `EDITOR.md` §2.1 and `check-rules.sh` still apply.

**Guard 4 — E16 must not put containment on the record.** Addresses only. A parent vector on `SpaceRecord` is the Innovator failure mode dressed as convenience.

**Guard 5 — E15 must not reintroduce inference.** If `presenter_addr` still calls `significant_bits` to fill `bits` for keys minted under the new layout, E15.0 has not landed even if unit tests for `child` are green.

---

## 6 · Decisions owed when stages land

| When | Record |
|---|---|
| E15.0–E15.1 | **D57** (O32): carried length + `child` + `MintSeed`; supersede D53 scan mint; amend D45 nibble/inference |
| E16.0 | **D58** (O34): Spec/builders in editor; façade encode-only |
| E15.2 / E16.2 | Note D34 rewrite of well-known keys (fourth time); R-A bootstrap vs content split — in D57 |

O10 (ownership at creation) remains open; E15 creates addresses and should not pretend ownership is answered. O33 / O35 stay with E18a / [`vocabulary.md`](../vocabulary.md).

---

## 7 · What this plan is not

It is not a port of `hyper-ui`. It is not biomimicry's genotype or bion's `NeuronId` types — only the *mechanism* of pure derivation (SALVAGE / AUTHORING-STACK §1). It is not E17's open record, not E18's alphabet registration, and not stored component definitions (E18b).

It is the written answer to: *how does a space hold two hundred children, and how does genesis stop being six hundred lines of literals?*
