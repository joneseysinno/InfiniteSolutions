# Infinite Solutions — E17–E20, record, alphabet, and the Innovator screen

> **Status:** draft 2, 2026-08-29. **E17–E20 landed** (open record, declared alphabet,
> stored components, focus, Innovator screen). R20 / D41: a stage is `landed`
> only when its **Verified by** cell names a check that can fail for the reason the
> claim would be false.
>
> O26 → **D59**, O33 → **D60**, O35 → **D61**, O29 → **D62** (stamped 2026-08-29).
>
> Rules: [`../RULES.md`](../RULES.md) · Decisions: [`../DECISIONS.md`](../DECISIONS.md) ·
> Charter: [`../CHARTER.md`](../CHARTER.md) · Stack:
> [`AUTHORING-STACK.md`](./AUTHORING-STACK.md) · Ledger: [`../SALVAGE.md`](../SALVAGE.md) ·
> Vocabulary: [`../vocabulary.md`](../vocabulary.md) · Predecessor:
> [`E15-E16-IDENTITY-AUTHORING.md`](./E15-E16-IDENTITY-AUTHORING.md) · Layer:
> [`../specs/EDITOR.md`](../specs/EDITOR.md)
>
> **This document is a specification, not code** (R28). E15–E16 removed the hard
> blockers. E17–E20 are the rest of the stack: open the record, declare then use the
> alphabet, make a field take keystrokes, and land O29. They are written before they
> are built.

---

## 0 · What these stages are for, in one sentence

> **E17 opens the record; E18a names the alphabet; E18b makes composition the normal
> case; E19 makes a field live; E20 is the product.**

AUTHORING-STACK §0 measured the symptom after identity and sugar: `SpaceRecord` still
carries `link` and `text` (D46's rejected alternative, adopted at the store), eleven
natives of which six fail Rule 1, `kind: "native"` used zero times as anything else
(finding 29), and E13.3's property write with no focus. None of that is the product.
E20 is. A plan that lands E17–E19 and stops has not answered the question.

---

## Stage table

| # | Stage | Status | Verified by | Green check — and what must **stop** passing |
|---|---|---|---|---|
| **E17.0** | Payload address helper: `payload_key(space)` = `child(space, reserved_slot)` | landed | unit test beside `addresses.rs` / mint | Pure; reserved slot collides with neither Spec slots `1..N` nor `MintSeed` (`0x0100+`). Payload bytes are **not** an `IS1` space. *Must stop passing:* text/link living only on `SpaceRecord` |
| **E17.1** | `SpaceRecord` drops `link` and `text`; encode/decode keep `primitive` only | landed | `record.rs`; `tests/genesis.rs` (E4) | `encode_space` / `decode_space` have no `link`/`text` fields. `scene.rs` fills `Placeable::{link,text}` by reading `payload_key`. *Must stop passing:* `link` and `text` as `SpaceRecord` fields |
| **E17.2** | Fourth shape | landed | `tests/open_record.rs` | A fourth primitive (e.g. `mark`) stores its payload under the space and draws without adding a field or a decoder branch on `SpaceRecord`. *Must stop passing:* a field-per-primitive encode path |
| **E18a.0** | Declared set + Rule 1 / Rule 2 in `check-rules.sh` | landed | `scripts/check-rules.sh` | Each declared element has a two-domain (R32) line. Rule 1 flags a native used by fewer than two components. *Must stop passing:* the current eleven-key registry as the undeclared alphabet |
| **E18a.1** | Register `map` and `fold`; pure-fn table | landed | compositor `Blocks` + pure-fn registry | `map` / `fold` are the only new **effect** keys. Variety is table data (`PartialEq`, serialisable, no closures). *Must stop passing:* adding a native to do parse/add/format |
| **E18a.2** | Live natives ⊆ declared effects | landed | grep / Rule 1 on `native_signatures` | Keep: `read`, `amend`, `commit`, `gate`, `probe` (`probe-at`). Add: `map`, `fold`. Retire as **blocks**: `increment-text`, `encode-selection`, `encode-wire`, `set-origin`, `offset`, `displace` — they become pure-fn keys (or map chains). Existing graphs call `map` instead. *Must stop passing:* those six as `kind: native` entries |
| **E18b.0** | Stored definition + `kind != "native"` | landed | `src/` grep; compositor `closure.rs` already exists | At least one block in `src/` has `kind != "native"` (finding 29). *Must stop passing:* `kind: "native"` as the only kind in `src/` |
| **E18b.1** | Shared `field_row` definition | landed | `tests/stored_component.rs` | Two screens delegate to **one store address**. Edit the definition; both change; **no recompile**. *Must stop passing:* a Rust `field_row()` function as the only sharing mechanism |
| **E18b.2** | R-E: counter authored | landed | `tests/counter.rs` | `app::connect` / `classify` / `increment_graph` **deleted**. Pointer wiring authors the graph. *Must stop passing:* finding 24's pre-written installer |
| **E19.0** | Session focus record | landed | `tests/focus.rs` | Click an `accepts` text space writes focus (session region, derived like `select_key`). *Must stop passing:* keystrokes with nowhere to land |
| **E19.1** | Type → composition → commit | landed | `tests/focus.rs` | Characters amend the focused field's **payload** (D24: amend pending, explicit commit). Portal does not `put` the space. *Must stop passing:* E13.3's `apply_origin` as the only property-write path |
| **E19.2** | `Ctrl+Z` restores the typed value | landed | `tests/focus.rs` + existing undo | Undo restores the previous payload at the field's address (D49). *Must stop passing:* focus edits that bypass the commit stream |
| **E20.0** | Four stored components | landed | store contents / tests | `panel`, `section_header`, `field_row`, `action_bar` are **definitions in the store**, built from E18a's set only |
| **E20.1** | One screen, role-routed commit | landed | `tests/innovator_screen.rs` | Header + two field rows + action bar; commit goes through the interpreted composition, routed by role. *Must stop passing:* a screen that is only genesis builders |
| **E20.2** | Budget and re-seed | landed | same + check-rules Rule 2 | **Zero** primitives added to E18a's set. Delete the Rust builders and re-seed **from the store**: it still renders. *Must stop passing:* composed-block count of zero; a native added "just for this screen" |

**E17 before E18 is mandatory.** Opening the record while builders still write `text:` / `link:` on the struct is a scatter; after E16 it is a small diff.

**E18a before E18b is mandatory.** Declaring a set and building from it is a test; building then naming it is a description.

**E19 before E20 is mandatory.** `field_row` without focus is decoration.

**Out of scope.** Ownership (O10). Version pinning / genotype (AUTHORING-STACK §4.1). Promoting `Addr` to its own crate (O13). The parallel scheduler. A widget toolkit.

---

## 1 · The order, and why

**E17.0 before E17.1** so Scene and genesis swap onto a pure `payload_key` that already has tests, rather than inventing addressing inside encode/decode.

**E17.1 before E17.2** so the fourth-shape check is against the open record, not against a record that still has `text` and `link` columns.

**E18a.0 before E18a.1** so `map` / `fold` are registered *into* a declared set, not invented as two more undeclared natives.

**E18a.1 before E18a.2** so retiring the six one-offs has somewhere to go (pure-fn keys under `map`) rather than deleting capabilities.

**E18a.2 before E18b.0** so a composed block is built from the declared alphabet, not from leftovers that Rule 1 would flag.

**E18b.0 before E18b.1** so shared `field_row` is a stored definition with `kind != "native"`, not a second Rust helper.

**E18b.1 before E18b.2** so the counter's authored graph can delegate to a stored definition rather than inventing a one-off composition kind.

**E18b.2 before E19** so keystrokes land on an authored field, not on E13.7's installer.

**E19.0 before E19.1** so typing has a focus record before characters amend a payload.

**E19.1 before E19.2** so undo restores a commit that went through the composition, not a portal `put`.

**E20.0 before E20.1** so the screen is four stored definitions, not four builders that happen to look like components.

**E20.1 before E20.2** so the budget check counts primitives added *for that screen*, not for scaffolding that was never the product.

---

## 2 · O26 locked — where a text run lives (R-D) → **D59**

*Stamps as a decision record when E17.0–E17.1 land. Closes O26 as option **b**. Does not change D46's `Placeable::{link,text}` — those are the presenter artifact, not the store record.*

### 2.1 · Encoding

- `SpaceRecord` keeps extents, style, detail, `hosts_space`, `accepts`, origin, and an opaque **`primitive`** key (D46).
- It does **not** keep `link` or `text`.
- Per-shape payload is a derived address:

  `payload_key(space) = child(space, PAYLOAD_SLOT)`

  with `PAYLOAD_SLOT` reserved so it collides with neither authored Spec slots (`1..N`) nor interactive `MintSeed` (`0x0100+`). Slot `0` stays reserved (D57). A high reserved slot (`0xFFFF`) is the candidate; the unit test names the constant.
- Payload bytes are **not** an `IS1` space. `decode_space` returns `None`; Scene skips them as placeables.

### 2.2 · Why not keep columns on the record

D46 rejected a parallel list on `Placement` in these words: *a field per primitive closes an open set just as firmly as a variant per primitive, and a third primitive would need a third field.* `SpaceRecord` then grew `link` and `text`. That is the rejected alternative, relocated one layer down (AUTHORING-STACK §2.3).

A fourth shape that requires a fourth field has not opened the record. E17.2 is the check that can fail for that reason.

### 2.3 · Why option (b), and what it costs

O26 named two homes for a text run: (a) on the space record, (b) in a record it points at. AUTHORING-STACK answered (b) ahead of the length-measurement trigger because the argument is structural, not about run length. Cost, stated: one more indirection and one more thing that can dangle. Scene fills `Placeable` by a second read at `payload_key`. A dangling payload is an empty run or an unspanned link — a finding, not a decoder branch.

**D34 cost:** encode/decode change. No migration machinery. Acceptable because no data has shipped; genesis re-seeds (E4).

### 2.4 · What stays on the presenter

`Placeable::{primitive, link, text}` and `Placement::batches` are D46 and do not move. The façade selects a pipeline per batch. E17 changes **where the store holds** the run and the endpoints, not how the presenter groups them.

---

## 3 · O33 and O35 locked — component and arrangement → **D60**, **D61**

*Stamp when E18a.0 lands.*

### 3.1 · O33 — a component is a stored definition plus `delegate` (D60)

D27 says use is delegation and `Instance` is not a primitive. Innovator used an `instance_of` edge to a `component_def` node. They may be one thing said twice; E18a takes D27's reading for appearance as well as compute:

- A component **definition** is bytes in the store (a composition, or a space tree that flatten already knows how to write).
- A **use** is a `BlockRecord` with `kind: "delegate"` (or `"composed"`) and `target` the definition's address.
- [`BodyKind::DELEGATE`](../../crates/infinite-compositor/src/core/block.rs) already exists. E18b is the first use in `src/`.

Not a second ontology. Not a Rust function that returns a `Spec`. Guard 3: E18b.1 is not satisfiable by a builder.

### 3.2 · O35 — arrangement is a parent property (D61)

Shape keys draw (`area`, `text`, `link`). Arrangement does not draw. Innovator's `Stack` particle is refused: a primitive that draws nothing is a smell, and the record already carries extents. Parent property: `across` / `down` / `absolute`, applied by `arrange` over child `Extent`s.

Revisit before E18b only if a stored `field_row` cannot be laid out without a shape key. Wrong here is cheap; wrong after E18b is a migration of every definition.

### 3.3 · The declared effect set (E18a.2)

| Stay as effect blocks | Why |
|---|---|
| `read`, `amend`, `commit` | D24 write alphabet |
| `gate` | Conditional flow |
| `probe` (`probe-at`) | Hit-testing as authored data |
| `map`, `fold` | One machine; variety in the pure-fn table |

| Retire as blocks (Rule 1) | Destination |
|---|---|
| `increment-text` | Pure chain under `map` |
| `encode-selection`, `encode-wire` | Pure encode keys |
| `set-origin`, `offset`, `displace` | Pure record/point keys under `map` |

**R-H starts at E18a.2.** **R-E does not** — `app::connect` / `increment_graph()` may still install a Rust-built graph that now uses `map`. Deleting the installer is E18b.2.

[`vocabulary.md`](../vocabulary.md) §3 is the candidate set. E18a stamps it into the registry and `check-rules.sh`. Spec/builders stay in the editor (D58). This file stays platform doctrine.

---

## 4 · O29 locked — the forcing consumer → **D62**

*Stamps when E20.1 lands.*

The forcing consumer is **one Innovator screen** — `panel` + `section_header` + two `field_row`s + `action_bar` — authored as store data, commit routed by role through the interpreted composition, built with **zero** primitives added to E18a's set. Then the Rust builders are deleted and the screen re-seeds from the store.

SALVAGE §6 named this before E14. E14–E19 exist to make that table satisfiable. "The editor does more" is not a consumer that can break (R19). This screen is.

---

## 5 · What each stage changes in code

```
payload_key(space) ──► store put (not IS1)
SpaceRecord { primitive } ──► encode_space ──► store put
        │
        ▼
Scene reads payload_key ──► Placeable { link, text } ──► batches (D46)

declared alphabet ──► native_signatures ⊆ {read,amend,commit,gate,probe,map,fold}
pure-fn table ──► map / fold dispatch

stored definition ──► BlockRecord { kind: delegate, target }
field_row × 2 ──► same target; edit once

focus (session) ──► KEY amend payload ──► composition commit ──► Ctrl+Z

panel + header + field_row×2 + action_bar ──► store ──► re-seed without Rust builders
```

| Area | Files |
|---|---|
| Payload key | `src/editor/addresses.rs` — helper, not a new region root |
| Record | `src/facade/record.rs` — drop `link`/`text` |
| Scene | `src/facade/ports/scene.rs` — second read at `payload_key` |
| Builders | `src/editor/spec.rs`; inspector / toolbar / encode-wire call sites |
| Alphabet | `src/facade/ports/blocks.rs`; `src/editor/blocks.rs`; new pure-fn table |
| Checks | `scripts/check-rules.sh` — Rule 1, Rule 2; strengthen widget grep (finding 22) |
| Components | stored definitions; `kind` on `BlockRecord` |
| Counter | delete `src/editor/app.rs` installer; `tests/counter.rs` authors by pointer |
| Focus | session `child_key`; `src/portal/input.rs` / `window.rs` already emit `KEY` and Ctrl+Z |
| E20 | `tests/innovator_screen.rs`; four definitions in the store |
| Tests | `tests/open_record.rs`, `tests/stored_component.rs`, `tests/focus.rs`; keep E4 |

The façade does not name the editor (R2). Presenter L5 still holds: it mints no identity and authors no payload address.

---

## 6 · How this plan could fail the same way E13 did

**Guard 1 — the consumer is E20, named up front, and it does not move.** Every stage below is justified by E20, not by its own test. If a check can be met without advancing E20, the check is wrong.

**Guard 2 — every green check names what must stop passing.** The stage table's last column is load-bearing (D41).

**Guard 3 — a component is a stored definition that delegation points at (D27), not a Rust function.** E18b.1 and E20.2 are not satisfiable by a builder.

**Guard 4 — the alphabet is declared before it is used, and counted after.** E18a then E20.2. A set named after the fact describes whatever was built.

**Guard 5 — E17 must not put containment on the record** (already D58) **and must not grow a field per shape.** A `payload: Vec<u8>` column on `SpaceRecord` is the same defect as `text` / `link`.

**Guard 6 — E18 must not smuggle a widget toolkit.** No `Button` / `Panel` core types. `EDITOR.md` §2.1 still applies. Strengthen `check-rules.sh` so a one-off native cannot hide behind a name (finding 22 — the current grep is anchored and `increment_text.rs` passes).

---

## 7 · Decisions owed when stages land

| When | Record |
|---|---|
| E17.0–E17.1 | **D59** (O26): payload under the space; `SpaceRecord` has no `link`/`text` |
| E18a.0 | **D60** (O33): stored definition + `delegate`; **D61** (O35): arrangement is a parent property |
| E20.1 | **D62** (O29): forcing consumer is the Innovator screen |

O10 (ownership at creation) remains open. Version pinning and genotype stay deferred (AUTHORING-STACK §4.1) until a second author or a cache key for D28 tier 1.

---

## 8 · What this plan is not

It is not a port of `hyper-ui`. It is not biomimicry's genotype or Innovator's `instance_of` ontology — only the *mechanism* of stored definition plus delegate (D27), and the *discipline* that a pure function is data (AUTHORING-STACK §5.4). It is not E15's derivation or E16's Spec sugar; those have landed.

It is not a closed enum of kinds (R16). It is not ownership (O10). It is not implementation — this document is the written answer to: *how does a fourth shape touch no record field, how does composition become the normal case, and how does one Innovator screen get authored without adding a primitive?*
