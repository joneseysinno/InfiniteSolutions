# Infinite Solutions — Building Vocabulary

> **Status:** draft 1, 2026-08-29. **Records nothing** (R29). Candidate alphabet for
> E18a — declare the set in prose before the registry and `check-rules.sh` land.
>
> Rules: [`RULES.md`](./RULES.md) · Decisions: [`DECISIONS.md`](./DECISIONS.md) ·
> Charter: [`CHARTER.md`](./CHARTER.md) · Ledger: [`SALVAGE.md`](./SALVAGE.md) ·
> Plan: [`plans/AUTHORING-STACK.md`](./plans/AUTHORING-STACK.md) §5 ·
> Specs: [`specs/COMPOSITOR.md`](./specs/COMPOSITOR.md) §5 ·
> [`specs/PRESENTER.md`](./specs/PRESENTER.md) §5

---

## 0 · Thesis

> A small population of registered keys — shapes that draw, effects that touch the
> store, and pure functions that are data — plus composition that closes, is enough to
> author every screen and every compute graph; anything else is composed, not minted.

DNA authors everything from a very small set. **Take the structure; leave the naming.**
R15 forbids metaphor names in the core (F-4: bion and biomimicry each maintained a
`VOCABULARY.md` whose only job was policing a metaphor's border). No `base`, no
`codon`, no `gene`, no `soma`, no `neuron`, no `cistron`.

Two structural properties matter more than any count:

1. **There are two alphabets, not one.** Encode keys are few; functional variety lives
   in *data* that one machine interprets — not in new native block types.
2. **The same alphabet encodes the thing and its regulation.** One linker; one
   substrate for interface graphs and compute graphs (D27, CHARTER).

This file is not a metaphor-border blotter. If a name needs policing, the name is wrong.

---

## 1 · Substrate words (locked)

Everything spatial is built from CHARTER / D20. Do not redefine them here.

| Word | Meaning |
|---|---|
| **space** | The unit. Coordinate region. Permanent address. |
| **node** | An object populating a space. May host its own space. |
| **graph** | One level: nodes in a space and the hyperedges among them. |
| **hyperedge** | Connects any number of nodes. Carries values. |
| **zoom** | Crosses the node/space seam. Primary navigation. |

The compositor adds three words ([`COMPOSITOR.md`](./specs/COMPOSITOR.md) §5.1):

| Word | Meaning |
|---|---|
| **block** | A space with declared ports — a space seen as a unit of composition |
| **port** | Named, directed, tagged attachment on a block |
| **plan** | Linked form of a composition — derived, never authored |

Used here, decided elsewhere: **tag**, **value** (D13); **wire** (hyperedge at a port);
**composition** (graph seen as a block — D14.6).

Presenter words (**view**, **placement**, **level**, **probe**) live in
[`PRESENTER.md`](./specs/PRESENTER.md) §5.1. They are not building alphabet.

---

## 2 · Building strata

Conflating these sets filled the native-block junk drawer (AUTHORING-STACK finding 28).

| Stratum | What it is | What it is not |
|---|---|---|
| **Shape** | Opaque draw key on a space (`area`, `text`, `link`) | Not a closed enum (D46); open string registry |
| **Arrangement** | Property of the parent over child `Extent`s (`across` / `down` / `absolute`) | **Not a shape key** (O35: parent property) |
| **Behavior** | Effectful block kinds with ports | Not pure arithmetic / format helpers |
| **Pure function** | Registered, serialisable, hashable data that `map` / `fold` dispatch on | Not a block — no address, no ports, no `addresses.rs` constant |
| **Composition** | Wired blocks → block (D14.6); use = **delegate** (D27) | No `Instance` primitive |

**Effects are blocks. Pure functions are data.**

A pure function needs no address, no ports, and no provenance edge, so it must not be a
block. `map` is one block kind dispatching on a registered function key — one machine
reading any codon. Variety lives in the table, not in new `.rs` natives.

```
shape key  ──► how a space draws
effect block ──► what touches the store / world
pure-fn key ──► data interpreted by map / fold
     │
     ▼
block + port + wire ──► composition closes ──► stored definition
     ▲                                              │
     └────────────── delegate (D27) ◄───────────────┘
```

---

## 3 · Candidate population (E18a)

Open registry, small population. Each effect block below must pass R32 (editor + a
second consumer with no shared purpose) and Rule 1 (§4). E20 builds
`panel` + `section_header` + two `field_row`s + `action_bar` from this set with
**zero additions**.

### 3.1 · Shape keys

| Key | Role | R32 |
|---|---|---|
| `area` | Filled region | Editor chrome; every facade that draws a region |
| `text` | Glyph run | Labels, field values, counter total |
| `link` | Stroke between two points | Wires; any relation drawn as a line |

Registered as opaque `Box<str>` (D46). Not an enum.

### 3.2 · Arrangement (not a key)

Parent property: `across` | `down` | `absolute`, applied by `arrange` over child
extents. A primitive that draws nothing is a smell; Innovator's `Stack` particle is
refused here (O35 closed as parent property for this draft).

### 3.3 · Effect blocks — keep

Today's registry has eleven natives. Five stay as alphabet.

| Key | Role | Why it stays |
|---|---|---|
| `read` | Address → value | Store load path; every interpreted graph |
| `amend` | (Address, value) → pending | The only write path (D24) |
| `commit` | Address → committed | Authorable commit boundary (D24) |
| `gate` | Pass value when flag set | Conditional flow; selection / place / wire behaviours |
| `probe` | Point → hit address | Hit-testing as authored data (`probe-at` today) |

`amend` + `commit` are the write alphabet — not a single `write` — because D24 makes
the pending/commit seam authorable.

### 3.4 · Effect blocks — retire (Rule 1)

One-offs. Become pure-function table entries, authored compositions, or both. Must not
keep a literal address / port declaration / `addresses.rs` constant after E18b.

| Today's key | Why Rule 1 fires | Destination |
|---|---|---|
| `increment-text` | One consumer: the counter | Pure chain: parse → add-one → format under `map` |
| `encode-selection` | One consumer: selection behaviour | Pure encode key, or fold into selection composition |
| `encode-wire` | One consumer: wire-draw behaviour | Pure encode key, or fold into wire composition |
| `set-origin` | One consumer: place / drag | Pure record-patch key under `map` |
| `offset` | Point delta; pairs only with drag/place | Pure point op under `map` |
| `displace` | Apply delta to origin; same feature family | Pure point/record op under `map` |

`offset` and `displace` are useful ops — they fail as *blocks* because each costs
structure for a single feature family. As pure-function keys they cost a row.

### 3.5 · Effect blocks — add (not yet in registry)

| Key | Role | Why |
|---|---|---|
| `map` | Apply a registered pure-fn key to a value | One machine; variety in data (biomimicry `TransductionFn` shape) |
| `fold` | Combine inputs under a registered pure-fn key + declared order | Home of PARALLELISM §6.3's combination order |

Without `map` / `fold`, every pure chain becomes another native (finding 28's run rate).

### 3.6 · Pure-function registry (data, not nodes)

Candidate keys — open table, not an enum. Exact set lands with E18a registration; the
stratum is load-bearing now.

| Kind | Examples (illustrative) |
|---|---|
| Arithmetic | `add`, `sub`, `mul`, `neg` |
| Compare | `eq`, `lt`, `gt` |
| Text | `parse-i64`, `format-i64`, `concat` |
| Point | `point-offset`, `point-displace` |
| Record | `set-field` (origin, and later other patches) |
| Encode | `selection-bytes`, `wire-bytes` |

Each entry: pure function of values, `PartialEq`, serialisable, content-hashable — no
closures, no trait objects as the authored form (biomimicry's discipline).

### 3.7 · Authored components — not alphabet

Built from §3.1–3.6. Stored definitions; use via **delegate** (D27). O33: a component
is a definition in the store plus a body that names it — not a second `instance_of`
ontology.

| Component | Built for |
|---|---|
| `panel` | E20 / O29 |
| `section_header` | E20 |
| `field_row` | E20; shared across screens (E18b check) |
| `action_bar` | E20 |

Editing a stored definition changes every use with no recompile. That is E18b's green
check, and it requires at least one block with `kind != "native"` (finding 29).

---

## 4 · Closure rules

Checkable bounds — not a feeling about "twenty".

> **Rule 1 — a primitive used by fewer than two components is a one-off.**

> **Rule 2 — a component that requires a new primitive means the alphabet was wrong**,
> and the primitive it needed names precisely what was missing.

> **Budget — declare at E18a, build the counter and the Innovator screen from it, count
> primitives added.** Target: **zero**. Two consumers with no shared purpose is R32
> applied to the alphabet.

Also locked:

- Composition closes in practice: `kind != "native"` must appear in `src/` (D14.6;
  finding 29).
- Both Rule 1 and Rule 2 run in `check-rules.sh` when E18a lands
  (AUTHORING-STACK E18a).

---

## 5 · Prior art → Infinite Solutions nouns

Mechanisms salvage; names do not import.

| Prior (biomimicry / bion / Innovator) | Here |
|---|---|
| Primitive / NeuronKernel | Shape key, pure-fn key, or effect block — by stratum |
| Cistron / Synapse / Binding | Hyperedge / wire |
| Block + Manifest / GanglionGenome | Composition + stored definition |
| Genome / OrganismGenotype | Deferred (version pin); plan / artifact when a second author or cache key appears |
| Spec → commit | Authoring sugar → flat addressed records (E16) |
| DNA vs Expression / DefinitionWriter vs StateWriter | Stored vs Derived / Pending (D8); R5 |
| IdSeed / IdSource | E15 / O32 minting *mechanism* — not a vocabulary word |
| Particle / ComponentKind catalogue | Authored component definitions (§3.7) |
| Valence emit/absorb | Port declaration (D14.1) |

---

## 6 · Forbidden and retired names

| Name / pattern | Status | Why |
|---|---|---|
| Biology metaphors in core (`gene`, `codon`, `soma`, `neuron`, `synapse`, `cistron`, `organism`, …) | Forbidden (R15) | Metaphor is intuition pump, bad type system |
| `Instance` as a primitive | Retired (D27) | Use is delegation |
| `ParticleKind` / widget toolkit types (`Button`, `Panel` as core types) | Forbidden | Panel is a space; button is a space that `accepts` |
| `chart` (cartographic) | Retired (D20) | — |
| Recycled names for a second structure | Forbidden (R17) | — |
| Closed enum for an open set of kinds | Defect (R16) | Registry + opaque string key |

Domain crates may use domain words. The prefix `infinite-` plus a plain functional noun
is the core pattern (R15).

---

## 7 · Defaults this draft commits

| Open | Choice here | Revisit |
|---|---|---|
| **O35** arrangement | Parent property, not a shape kind | Before E18b if wrong |
| **O33** component | Stored definition + delegate body | E18a registration |
| **O34** where vocabulary lives | Platform doctrine — this file | — |
| Naming | Plain functional nouns only | — |

Identity / address encoding (O32) is out of scope. Version pinning and genotype stay
deferred until a named consumer (SALVAGE / AUTHORING-STACK §4.1).

---

## 8 · What this document is not

It is not a port of biomimicry's four primitives or bion's Neuron / Fiber / Synapse
types. It is not the E16 authoring sugar, the E18 registry code, or the `check-rules.sh`
hooks — those stages implement what this file declares. It does not mint addresses.

It is the written alphabet E18a requires before anything is registered: the set named
up front, so a set named after the fact cannot describe whatever was built.
