# Infinite Solutions — the editor, self-hosted from the first screen

> **Status:** draft 1, 2026-08-21. E0–E9 landed.
> R20 is precise about this: a status line is written by the change that lands the
> phase, never at authoring time, and never by the person who wrote the plan.
>
> Rules: [`../RULES.md`](../RULES.md) · Decisions: [`../DECISIONS.md`](../DECISIONS.md) ·
> Charter: [`../CHARTER.md`](../CHARTER.md) · Layer specs:
> [`../specs/RUNTIME.md`](../specs/RUNTIME.md) ·
> [`../specs/COMPOSITOR.md`](../specs/COMPOSITOR.md) ·
> [`../specs/PRESENTER.md`](../specs/PRESENTER.md)
>
> **This document is an execution plan for an assistant (Cursor).** It is not a
> specification. R18 says a layer gets a specification before it gets a crate, and
> stage **E0** below is where the two specifications this plan needs get written. This
> plan tells the assistant what to build and how the result is checked; the
> specifications say what the things *are*.
>
> Opens O15, O16; O17 closed by D37. Requires D32–D40 to be written as the stages land.

---

## Stage table

| # | Stage | Status | Verified by | Green check |
|---|---|---|---|---|
| **E0** | Specifications, root package, module skeleton, rule checks | landed | `scripts/check-rules.sh` | `cargo build` succeeds; `bash scripts/check-rules.sh` reports a **facade** section and an **editor** section and every check passes, including R18 for all four crate directories |
| **E1** | The facade seam — thirteen ports over the real store | landed | `tests/seam.rs` | One space written through the facade is readable through `StoreRead`, `Definitions` **and** `Scene`, and all three agree on its address at the same revision. Every `Addr`/`Revision` conversion is a newtype unwrap, asserted by a test |
| **E2** | The portal ticks | landed | `tests/saturation.rs` | A window opens and `tick` runs at cadence under a budget. The saturation test (`RUNTIME.md` §7.4) passes against the **real** store, not the fake. Kill the process mid-drag; on restart the journal replays and the pending set is intact |
| **E3** | One space on screen, and it answers a click | landed — **arithmetic only until E10.4**; finding 11 | `tests/agreement.rs` + `tests/pixels.rs` | The agreement test (`PRESENTER.md` §6.4) passes against a real surface with a **non-zero origin**. Clicking the space returns its address. The placement passes the runtime's generic discard harness with no per-artifact test code |
| **E4** | The first screen is authored, not coded | landed | `tests/genesis.rs` | **The genesis discard test.** Delete every space under the editor's screen root; restart: the portal still runs and the canvas is empty with a finding, not a black screen. Re-run genesis: the screen is bit-identical to before the delete |
| **E5** | It links | landed | `tests/link.rs` | The editor's behaviour composition links. The findings corpus yields **exactly one** finding per malformed composition, each carrying a site, a said, a wanted and a remedy. The closure test (`COMPOSITOR.md` §7.3) passes |
| **E6** | It runs | landed | `tests/behaviour.rs` | Dragging a space moves it, and the move is performed by the **interpreted composition**, not by Rust in the portal. Provenance recovers the exact declared input set; the store's staleness query returns exactly the downstream address set — no more, no fewer |
| **E7** | It edits itself | landed | `tests/self_edit.rs` | Drag a node **belonging to the editor's own screen** while the editor is running. The change persists. Restart. It is still there. Nothing was recompiled |
| **E8** | Wiring, and the finding surface | landed | `tests/wiring.rs` | A wire is drawn and `link` answers **before** it is committed (C4). A tag mismatch renders said / wanted / remedy, and the canvas zooms to the site |
| **E9** | Compilation, tier 0 | landed | `tests/tier0.rs` | The equivalence harness runs over a corpus drawn from the editor's own plans; tier 0 registers **by passing it**, with no per-backend test code |

**E7 is the deliverable.** E0 through E6 exist to make it possible and E8/E9 to make it
usable. If a stage can be cut, cut it from the E8 end.

**The `Verified by` column is new, and it is why finding 11 was possible** (D41). A
stage may not be marked `landed` while that cell is empty or names a test that cannot
fail for the reason the claim would be false. Read E3's row with the column filled in
and the mismatch is visible without anyone auditing anything: the words say *one space
on screen* and the test named beside them is eighty samples of `f64` arithmetic.

**Successor plan:** [`E10-IT-DRAWS.md`](./E10-IT-DRAWS.md). E10 exists because this
plan reached E9 without ever drawing a pixel. R21 — this document is not edited to
pretend otherwise; it carries the findings and the corrected status, and the successor
carries the work.

---

## 0 · What this plan decides, and what each decision costs

Four choices were made before writing it. Each is recorded here with its cost, because
a decision without its cost gets re-litigated (R22).

### 0.1 `src/` is one crate holding the facade, the portal, and the editor

Root `Cargo.toml` becomes a package **and** the workspace root. `src/` is
`infinite-solutions`.

R2 fixes the dependency direction — `layer → platform facade → domain facade → app` —
and says no app depends on a layer directly. With one crate, the manifest cannot
enforce that, so **a grep enforces it instead**: only `src/facade/**` may name a layer
crate, and `src/editor/**` may name neither a layer crate nor a graphics crate. §8 has
the check.

*What it costs.* R2's strongest enforcement — a manifest that cannot express the wrong
edge — is traded for a grep that can be circumvented by anyone who wants to. That is a
real weakening and it is stated rather than smoothed over. The mitigation is that the
grep runs in `check-rules.sh` alongside checks that have all been verified to fail
(`PRESENTER.md` §11's discipline), so it is not a check nobody knows the polarity of.

*Also costs:* R1 says *Infinite Solutions is a platform, not an application*, and the
root now builds a binary. `RUNTIME.md` §10 finding 2 recorded the previous root
`src/main.rs` as *a leftover of the abandoned three-layer scaffold* and asked for it to
go. It went, and this plan brings it back deliberately. **That requires D32** (§7.1) —
not a comment in `Cargo.toml`.

### 0.2 A vertical slice, not a horizontal one

The three layer crates are S2 skeletons: `link`, `place`, `probe`, `interpret` and
`tick` are all `todo!()`. This plan does **not** drive each layer's stage table to green
against fakes and then integrate. It builds the thinnest end-to-end path — one space
you can see, click, move and persist — and implements only the parts of S3–S8 that path
needs.

*Why.* The facade does not exist yet, and it is where four independently-designed seams
meet for the first time. Three `Addr` types, two `Revision` types, thirteen ports, and
one artifact registry that two layers hand into from opposite sides (D30 corrects
`RUNTIME.md` §5.2 on exactly this). Seam errors found on day one are cheap; found after
three stage tables are green they are a redesign.

*What it costs.* Each layer's stage table stays partly green for a long time, and R20
means nobody may stamp a status line early to make the table look tidy. A stage is
`not started` until the change that lands it says otherwise, even when half of it is
working.

### 0.3 Self-hosted from the first screen

O11 — *is the editor self-hosted?* — is answered **yes**, closed by D36 at E7.
The editor's screens are authored compositions in the store from E4 onward. There is
no native-Rust editor that gets migrated later.

*Why it is defensible.* All three layer specifications name the editor as their forcing
consumer, and D16's proof obligation is *if the editor cannot be built in the platform,
a child cannot build an app in it.* Building a native editor first and self-hosting
later means the layers are validated against a consumer that is not the one they were
specified for, and the migration is the rewrite the previous three attempts each hit.

*What it costs, stated plainly.* This is the highest-risk option available and §2
exists to make it survivable. You must hand-author graph data before an editor exists to
draw it, which means the **genesis seed** (§2.3) is native Rust and stays native
forever. The first screen is the hardest screen rather than the easiest, because
nothing renders until link, interpret, place, probe and tick are all alive at once. The
mitigation is that the first screen is made absurdly small — §1 says exactly how small,
and the assistant is not permitted to grow it.

### 0.4 The plan is file-by-file

Every stage names the files to create, the module they live in, the green check that
says the stage is done, and the decision records the stage owes. §5's stop list names
what the assistant must refuse to do and raise as a finding instead.

*What it costs.* Length, and a plan that will be wrong in places. R21: when a stage is
wrong, it gets a terminal status and a successor stage, not an edit.

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

That is the whole list. There is **no** property panel, no menu bar, no block palette,
no toolbar, no file dialog, no settings, no theme picker, no undo UI, and no text
editing beyond what E8's wire needs.

**R27 is the reason, and it is the rule most at risk in this plan.** *Generality is a
defect unless a named consumer requires it.* The named consumer here is D16's proof
obligation, and it is satisfied by a canvas that can compose. Everything on the list
above is required to demonstrate that a graph can be edited by a graph. Nothing else
is, and an assistant asked for "an editor" will produce a toolbar unless told not to.

**The assistant may not add a screen element that is not on that list.** If one seems
necessary, that is a finding for §9, not a commit.

---

## 2 · What self-hosting means, and what it does not

This section is the one most likely to be misread, so it is stated three ways.

### 2.1 Appearance is authored data. It is not blocks.

D15 already settled this and it is easy to forget: *widgets are **not code**.*
`Innovator` had already made them authored `Entity` nodes with `component_kind` props.

The presenter's `Scene` port reads, per address: an authored extent, an opaque style
key, a detail override, and whether the space hosts a sub-space (`PRESENTER.md` §3.1).
**All four are records in the store.** Drawing a rectangle is not a block invocation; it
is `place` walking an address range and `Surface` accepting the resulting work.

So: **the editor's chrome is spaces, not a composition.** A canvas is a space. A node's
box is a space. A selection highlight is a space with a style key. None of that is
linked, none of it is interpreted, and none of it costs a plan step.

*Consequence the assistant will get wrong if not told:* do **not** create a `rectangle`
block, a `label` block, or a `panel` block. There is no such thing. If a visual element
needs to exist, it is authored into the store by genesis (§2.3) or by the editor itself
(E7).

### 2.2 Behaviour is a linked composition. That is what is self-hosted.

What *is* a composition is the editor's behaviour: what happens when the pointer moves,
when a button goes down over a space, when a drag ends. That composition is authored
graph data, linked by `link`, executed by `interpret`, and it is what E6 and E7 prove.

The split, stated once:

| | Where it lives | What reads it |
|---|---|---|
| **Appearance** | authored spaces in the store | the presenter, through `Scene` |
| **Behaviour** | an authored composition in the store | the compositor, through `Definitions` |
| **Primitives** | native Rust blocks, registered under string keys | the compositor, through `Blocks` |

### 2.3 Native code does not disappear, and pretending otherwise is the failure mode

D16 has two tiers of user and they are permanent: **block authors write native
primitives in Rust; app authors compose visually and never see Rust.** Self-hosting
means the editor is in the second tier. It does not mean there is no Rust.

Three things stay native forever, and naming them now is what keeps the boundary from
drifting:

1. **The portal** (§3) — window, GPU device, OS input, the tick loop. This is the
   platform boundary, and D18 already says what a platform boundary is.
2. **The native blocks** — the primitives the editor's composition is built from. §7.6
   names the starting six.
3. **Genesis** — the seed that writes the editor's own composition into an empty store.

Genesis is the bootstrap and it is allowed to be native, on one condition: **it writes
graph data and nothing else.** It contains no layout algorithm, no behaviour, no
policy, and no conditional beyond "does this space already exist". It is a bootloader.
Its green check is E4's discard test, and if genesis ever grows a decision, that test is
how you find out.

---

## 3 · The OS is a portal

D18: *a platform boundary is a portal. Desktop and its server are two graphs glued at a
portal; the author draws an edge and the runtime decides whether it is a function call,
an IPC message, or a network round-trip.*

**Apply that to input.** The operating system is another graph. The window, the pointer
and the keyboard sit on the far side of a portal, and the module that owns that seam is
therefore called `src/portal/` rather than `src/shell/` — a name taken from the existing
vocabulary rather than invented (R15), and one that `Innovator`'s failed `app_shell`
does not already own (R17).

**Input arrives as pending amends, never as writes.** The portal converts an OS event
into an `amend` on a pending entry (D8, D24) at a well-known address:

| Address | Carries | Amended when |
|---|---|---|
| `/input/pointer/position` | a point in surface coordinates | every pointer move |
| `/input/pointer/button` | a flag per button | on transition only |
| `/input/key` | the key event | on transition only |
| `/input/surface` | size, scale factor, origin | on resize |

The editor's behaviour composition reads those addresses as ordinary inputs. Three
things follow, all free:

- **R14 holds by construction.** A keystroke is an amend to a bounded in-memory set plus
  a journal append. It never touches the store's write queue, so backpressure cannot
  reach the input path. D24 is not something the portal has to be careful about; it is
  the only thing the portal can do.
- **The editor can react to input that has not been committed**, which is C4 —
  `COMPOSITOR.md` §2's *"if I drop this wire here, does it link?"* — arriving at the
  input edge for the same reason.
- **A drag in progress is enumerable.** D8's pending category is doing real work rather
  than being paperwork, and an "unsaved" indicator (B4) is implementable because `list`
  returns everything.

*This is the reading that makes "frontend, backend and persistence are one substrate"
(D16) true at the input edge rather than only at the server edge.* Recorded here
because it is not written anywhere else in the corpus and E2 depends on it.

---

## 4 · Repository shape

### 4.1 The root becomes a package

```toml
# Cargo.toml — the workspace root, and now also a package.
#
# R1 says Infinite Solutions is a platform, not an application, and this root builds a
# binary. See D32: the binary is the *portal* — the platform's boundary with the
# operating system (D18) — plus the platform facade (D10) and the editor that is the
# forcing consumer named in all three layer specifications (O11). It is not "an app"
# in D12's sense; D12 defers SES, Coach Assistant and the structural work, and this
# defers all three still.

[package]
name = "infinite-solutions"
version = "0.0.0"
description = "The platform facade, the OS portal, and the editor."
edition.workspace = true
rust-version.workspace = true
license.workspace = true
publish.workspace = true

[workspace]
resolver = "2"
members = [
    "crates/infinite-runtime",
    "crates/infinite-compositor",
    "crates/infinite-presenter",
    "crates/infinite-db",       # see §9 finding 2 before adding this line
]
```

### 4.2 The module tree

`module.rs` plus a directory of leaf files. **No `mod.rs` anywhere** (F-8). A module
file holds docs, private `mod` declarations, and re-exports — nothing else. `pub mod`
in a module file gives every type two paths, which is one name for two things read
backwards (R17); the layer crates already hold this convention and a second house style
in one repository is the failure it is written against.

One public function per file for **free** functions. A type with an inherent impl is one
file. This is the reading all three layer specs record.

```
src/
  main.rs                  the binary. Opens the store, builds the facade, runs the portal.
                           Under 60 lines, and that is a check (§8).
  lib.rs                   module file: docs, mod declarations, re-exports.

  facade.rs                module file
  facade/                  ── THE ONLY PLACE A LAYER CRATE OR A GRAPHICS CRATE IS NAMED ──
    open.rs                open the store; one function.
    addr.rs                the three Addr conversions and the two Revision conversions.
                           Newtype unwraps, asserted (E1). O13's trigger lives here.
    ports.rs               module file
    ports/
      store_read.rs        runtime  · range reads at a revision
      store_write.rs       runtime  · submit; returns Accepted or Full, never blocks (D24)
      stale_feed.rs        runtime  · staleness_closure → "these went stale at rev N"
      clock.rs             runtime  · std::time::Instant. The layers may not name it; this may.
      journal.rs           runtime  · the session WAL, built in infinite-db and unused
      definitions.rs       compositor · a definition set at a revision, stored ∪ pending
      blocks.rs            compositor · the string-keyed native block registry
      values.rs            compositor · read an input, write an output
      provenance.rs        compositor · ComputationProvenance
      backends.rs          compositor · the backend registry. Tier 0 after E9.
      scene.rs             presenter  · extents, style keys, detail overrides, hosts-a-space
      surface.rs           presenter  · wgpu. The ONLY f64 → f32 narrowing in the repository.
      glyphs.rs            presenter  · text extent and raster
    artifacts.rs           registers Placement and Plan with the runtime's registry.
                           D30: neither crate may see the other's, so this is the seam.
    finding.rs             renders a Finding (COMPOSITOR.md §6) and turns the presenter's
                           precision-floor *fact* into one (PRESENTER.md §10).

  portal.rs                module file
  portal/                  ── THE OPERATING SYSTEM BOUNDARY (§3) ──
    window.rs              winit: create the window, own the event loop
    device.rs              wgpu: adapter, device, queue, swapchain configuration
    input.rs               OS event → amend at a well-known address. Never a write.
    drive.rs               the tick loop. Cadence in, work out (L1). Owns no thread pool.

  editor.rs                module file
  editor/                  ── THE APP. NAMES NO LAYER CRATE AND NO GRAPHICS CRATE. ──
    addresses.rs           the well-known addresses. The bootstrap ABI, in one file.
    tags.rs                the editor's tag convention (D13 — tags are the app's)
    styles.rs              style keys, and the bootstrap default table (§7.4)
    genesis.rs             the seed (§2.3). Writes graph data. Contains no policy.
    blocks.rs              module file
    blocks/
      probe_at.rs          a point → an address
      read.rs              an address → a value
      amend.rs             (address, value) → pending
      commit.rs            an address → committed
      offset.rs            (point, point) → point
      gate.rs              pass a value through when a flag is set

tests/
  seam.rs                  E1 — the three ports agree
  agreement.rs             E3 — cull agrees with draw, against a real surface
  genesis.rs               E4 — the discard test on the editor's own screen
  behaviour.rs             E6 — the drag is interpreted, and provenance is exact
  self_edit.rs             E7 — the editor edits itself and it persists
  wiring.rs                E8 — pending wire links (C4); mismatch zooms to site
  tier0.rs                 E9 — the editor's plan; tier 0 passes the harness
```

**Nothing in `src/editor/` may name a layer crate.** Its blocks are registered with the
compositor by `facade/ports/blocks.rs`, which hands them across; the block's own file
holds a plain Rust function over opaque payloads. This is what makes the editor an app
under R2 despite living in the same crate, and §8's grep is what makes it true rather
than intended.

---

## 5 · Rules the assistant must obey

### 5.1 Opening a session

**R24: a session opens by reading `docs/RULES.md` and `docs/DECISIONS.md`.** An
assistant that has not read them is not working on this project yet. Then read the
specification for whichever layer the stage touches. This plan does not restate the
rules and must not be treated as a substitute for them.

Every commit message names the stage (E0–E9) and the rules the change is checked
against.

### 5.2 The working rules, in the order they get broken

- **R20** — a stage's status is updated **in the change that lands it**. Not at
  authoring time, not in a later audit. Never pre-stamp this table.
- **R22** — a decision is recorded when it lands, **with its rejected alternative**. The
  rejected alternative is the part that gets lost and the part that stops the decision
  being re-litigated.
- **R21** — a document is never deleted. It gets a terminal status and stays.
- **R23** — a claim about rendering or interaction **states its verification method**,
  or is marked unverified. This plan's E3, E4 and E7 are all interaction claims, and
  `HISTORY.md`'s closing observation is that rendering and pointer interaction are
  exactly where the longest-lived drifts survived.
- **R27** — generality is a defect unless a named consumer requires it. §1 is the list
  of named consumers for this stage of work.
- **R28** — when the idea is fuzzy, the deliverable is a specification, not code.
- **F-8** — no `mod.rs`, anywhere, ever.

### 5.3 The stop list

**These are not judgment calls. Each one is a finding for §9 and a message to the
author, not a commit.** R29: a proposal that adds an enum, a second graph, or a
metaphor name is *corrected, not merged* — and these are the recurrences with the
highest historical rate.

| Stop | Why | Prior count |
|---|---|---|
| Adding an `enum` to a layer core | R16, F-1 — a closed enum standing for an open set | 5 |
| A `HashMap`/`BTreeMap` keyed by anything but an address | F-2 — a map keyed by id standing in for an edge | ~30 in one codebase |
| A second in-memory graph beside the store | F-3, D6 | 3 |
| A metaphor as a core type name | F-4, R15 | 2 codebases |
| Writing to a cache | F-7 — the mechanism behind F-2 | — |
| Naming a layer crate outside `src/facade/**` | R2, and 0.1's whole cost | — |
| Naming a graphics crate outside `src/facade/ports/surface.rs`, `src/facade/ports/glyphs.rs` and `src/portal/` | D29 | `hyper-ui` |
| A sixth runtime port, a sixth compositor port, a fourth presenter port | D23, D26, D29 each say a further port needs a decision record | — |
| `f32` anywhere but `src/facade/ports/surface.rs` | D29, `PRESENTER.md` §3.3 | `hyper-ui` |
| Math in the compositor | R31, L3 | — |
| A `rectangle`, `label`, `panel` or `widget` block | §2.1 — appearance is authored data | `Innovator` already got this right |
| A screen element not in §1's list | R27 | — |
| `std::thread`, `sleep`, `block_on`, or an async runtime in the tick path | R8, L1 | — |
| Mapping a store error to an empty result | `PRESENTER.md` §13 finding 8 — a failed query and an empty screen must never be indistinguishable | 1 |

### 5.4 When a stage is blocked

Do not improvise architecture. Write the finding into §9 of this document with the
evidence, mark the stage `blocked — <reason>` (R20 permits a status the change earns),
and stop. A fuzzy idea plus code is the drift mechanism (R28); a fuzzy idea plus a
written finding is how it comes into focus.

---

## 6 · How the stages relate to the layer stage tables

This plan does not own the layer stage tables. It **forces** entries in them, and each
one is landed by a change that updates that layer's own table (R20).

| This plan | Forces, in the layer specs |
|---|---|
| E1 | nothing — the ports are the facade's own code |
| E2 | `RUNTIME.md` S3 (ports and the fake store), S4 (pending set and journal) |
| E3 | `PRESENTER.md` S3 (ports and fakes), S4 (the embedding), S5 (arrangement), S6 (placement and probe) |
| E4 | `PRESENTER.md` S7 (discard), S8 (first real binding) |
| E5 | `COMPOSITOR.md` S3 (ports and fakes), S4 (link and findings), S5 (closure) |
| E6 | `COMPOSITOR.md` S6 (interpreted execution and provenance), S8 (first real binding); `RUNTIME.md` S5 (artifact registry), S6 (staleness frontier), S7 (first real binding) |
| E9 | `COMPOSITOR.md` S7 (backend contract, tier 0, equivalence harness) |

**Note what E1 forces: nothing.** That is deliberate and it is the argument for the
vertical slice. The seam can be built and tested before a single layer function stops
being `todo!()`, because a port implementation is the facade's code all the way down.
Do E1 first and the hardest integration risk in the project is retired before anything
depends on it.

---

## 7 · The stages

### 7.1 E0 — Specifications, root package, module skeleton, rule checks

**Deliverables.**

1. **`docs/specs/FACADE.md`** — R18 requires it, and the facade is the compatibility
   surface (R30), so it is the one document that may not churn. It must state: the
   thirteen ports and which store facility backs each; the `Addr`/`Revision` conversion
   contract and O13's trigger; where the artifact registrations happen and why neither
   crate can do it (D30); the single narrowing point for `f32`; and its own forcing
   consumer, which is the editor.
2. **`docs/specs/EDITOR.md`** — the app's specification. §1's screen list, §2's three-way
   split, §3's input portal, the native block set, and the tag convention. R26: it
   names what breaks if the editor is wrong, which is D16's proof obligation.
3. **`docs/specs/DB.md`** — short. §9 finding 1: `check-rules.sh`'s R18 loop expects a
   spec per crate directory and `crates/infinite-db` has none, so that check fails
   today. The document should say the store is D1-locked, point at
   `crates/infinite-db/SEMANTICS.md` and the spatial-layer doctrine, and state that this
   repository does not specify it. Do not restate it (R17, R21).
4. **Root `Cargo.toml`** per §4.1. Delete nothing; the existing comment gets replaced by
   one that cites D32.
5. **The module skeleton** per §4.2 — every file, each holding its doc comment and a
   `todo!()` body, in the manner the three layer crates were skeletoned. This is a
   large commit and it should be: a skeleton reviewed as one thing is reviewable, and a
   skeleton that arrives file-by-file over ten commits is not.
6. **`scripts/check-rules.sh`** grows a **facade** section and an **editor** section per
   §8, and the existing R18 check goes green.

**Decision record owed: D32 — the root is a package, and what that does to R1.**
State the choice (root builds the portal, the facade and the editor), what forced it
(the facade must exist for any of the four layers to be exercised together; the editor
is the forcing consumer named in all three layer specs; O11 is answered yes), and what
it costs (R1's sentence now needs the qualification in §4.1's comment; R2's enforcement
degrades from a manifest to a grep). **Rejected alternative:** the facade as a fifth
crate `crates/infinite-solutions` with the editor as a sixth — rejected because R1's
"a fifth layer requires a decision record before it gets a directory" makes `crates/`
the wrong home for something that is not a layer, and because two more crates buys
manifest-level R2 enforcement at the cost of two more `Addr` conversions and a second
place for the facade's own vocabulary to live.

**Green check.** `cargo build` succeeds. `bash scripts/check-rules.sh` prints a facade
section and an editor section, and every check in the file passes — **including R18 for
all four crate directories**, which is a check that fails today.

**Verification of the new checks (R23, `PRESENTER.md` §11's discipline).** Every check
added in §8 must be **verified to fail**: break it deliberately in the working copy,
confirm it fires, restore. A check that has never been seen to fail is a check nobody
knows the polarity of. Record in the commit message which seven were exercised.

---

### 7.2 E1 — The facade seam

Implement all thirteen ports over the real `InfiniteDb`. No layer function is called
yet; nothing renders. This stage exists to find out whether the four seams fit.

**The mapping.** The store's public surface (`crates/infinite-db/src/lib.rs`) supplies:

| Port | Backed by |
|---|---|
| `StoreRead` | `ReadTxn` plus `QueryOptions` over an address range at a revision |
| `StoreWrite` | `InfiniteDb::try_insert` (D33) — **see the risk below** |
| `StaleFeed` | `staleness_closure::{FreshnessReport, StaleTarget}`, `check_hyperedge_freshness`, `query_stale_downstream`, the `engine/derivation/` bus, watermarks |
| `Clock` | `std::time::Instant`. The layers may not name `std::time` and this file may |
| `Journal` | the session WAL — `engine::session_wal_store`, `WalDurability`. Built and currently unused (D8) |
| `Definitions` | a range read of a definition space, **unioned with the runtime's pending set** — that union is C4 and it is the port's whole point |
| `Blocks` | a `BTreeMap<String, Primitive>` in the facade, populated at startup by the editor |
| `Values` | read at an address; writes go through the pending path, never straight to the queue |
| `Provenance` | `ComputationProvenance`, `infinitedb_core::provenance` |
| `Backends` | the compiled-form registry; tier 0 after E9 |
| `Scene` | a range read mapping node props to `Placeable` — extent, style key, detail override, hosts-a-space |
| `Surface` | wgpu, in E3. A stub reporting a fixed size is correct for E1 |
| `Glyphs` | a text shaping crate, in E3. A stub is correct for E1 |

**The single biggest technical risk in this plan is `StoreWrite`.** D24 requires
`submit` to be **non-blocking and fallible** — `Accepted` or `Full`, never a wait — and
the reason D24 exists at all is that `infinite-db`'s write queue *blocks when full*. If
the store exposes no way to attempt a submission without blocking, the facade cannot
satisfy the contract by wrapping it.

Do not paper over this. Three responses, in order of preference:

1. **The store grows a `try_submit`.** Correct, and it is the store's concern. `D1`
   locks the store as the store; it does not forbid changing it.
2. **The facade owns a bounded try-channel in front of the queue**, drained by the tick
   loop. Costs a second place where writes are ordered, which is a smell.
3. **Blocked.** §5.4. Write the finding and stop.

Whichever happens, **it is a decision record — D33** — with its rejected alternatives,
because "a larger queue" and "an unbounded channel in front of it" were already
rejected by D24 and will be proposed again.

**`Addr` and `Revision` (O13).** `facade/addr.rs` holds five conversions: three `Addr`
(runtime, compositor, presenter) and two `Revision` (runtime, presenter). **Each must be
a newtype unwrap.** A test asserts it. The moment a conversion needs logic, O13's
trigger has fired — promote `Addr` to a zero-dependency crate, with a decision record.
Do not quietly add the logic; the trigger existing and never being watched is how a
deferral becomes a permanent condition (D29's own words).

**Green check.** `tests/seam.rs`: write one space through the facade; read it back
through `StoreRead`, through `Definitions`, and through `Scene`; assert all three
resolve the same address at the same revision. Assert every conversion in `addr.rs` is
an unwrap. Assert `StoreWrite::submit` returns `Full` rather than blocking when the
queue is saturated.

---

### 7.3 E2 — The portal ticks

**Files.** `portal/window.rs`, `portal/device.rs`, `portal/input.rs`, `portal/drive.rs`,
`main.rs`.

**Forces `RUNTIME.md` S3 and S4.** `PendingSet`, `coalesce`, `Frontier`, `tick`,
`Journal` replay all stop being `todo!()`.

**What `drive.rs` is.** Cadence in, work out (L1). The winit event loop calls
`tick(now, budget)`; `tick` never blocks, never sleeps, never spawns; `Outcome` reports
whether work remains and the loop decides whether to tick again. **The runtime owns no
thread pool** and `check-rules.sh` already greps for it.

**What `input.rs` is.** §3. Every OS event becomes an `amend` at a well-known address
from `editor/addresses.rs`. There is no other input path, and there is no code path from
an OS event to `StoreWrite`.

**Green check, and it has three parts.**

1. A window opens and the loop runs at cadence with no tick exceeding its budget.
2. **The saturation test runs against the real store.** `RUNTIME.md` §7.4 specifies it
   against `FakeStore`, and this is the same test with the real one behind it: saturate
   the queue, drive input at 60 Hz for 10 s, assert every keystroke reaches the pending
   set within one tick, the pending set stays within its bound, no tick exceeds budget,
   and after the queue drains the store's final value per address equals the last input
   value.
3. **Kill the process mid-drag.** On restart the journal replays and the pending set is
   intact before the first tick. This is D8's *"a crash does not lose a half-finished
   calculation"* exercised rather than asserted, and it uses a session WAL that has been
   built and never driven.

**Verification (R23).** Parts 2 and 3 are `tests/saturation.rs` (automated, against
the real store). Part 1 is the portal binary: `cargo run` opens a winit window;
`tick` is called from `RedrawRequested` and the budget is the same `tick_at` the
saturation test asserts. A unit test cannot construct a winit `EventLoop` off the
main thread on Windows, so part 1 is not a `#[test]`.

---

### 7.4 E3 — One space on screen, and it answers a click

**Forces `PRESENTER.md` S3–S6.** `visible`, `detail`, `arrange`, `place`, `probe` stop
being `todo!()`. `facade/ports/surface.rs` becomes real wgpu; `facade/ports/glyphs.rs`
becomes real shaping.

**The narrowing point.** `f64` runs from the presenter core to the surface
implementation, and narrows to `f32` **once**, inside `facade/ports/surface.rs`. §8's
check pins that. `hyper-ui` ran an `f64` world through an `f32` camera and narrowed and
widened twice a frame; an address space whose premise is unbounded refinement was being
projected through 24 bits of mantissa. That is the difference between a precision floor
that can be detected (O14) and one that shows up as jitter.

**The style table.** `editor/styles.rs` maps an opaque style key to a descriptor — fill,
stroke, corner, text run. The descriptor is data; `facade/ports/surface.rs` turns it
into wgpu. **The editor names no graphics crate**, which is what makes that split
checkable.

*Recommended, and worth doing now rather than later:* author the style table **into the
store**, under `/style/`, with `editor/styles.rs` holding only a bootstrap default for
the case where the store has no style space. Styling then falls under E4's discard test
for free, and a store that has been emptied renders a legible default rather than a
black screen — which is `PRESENTER.md` §13 finding 8's failure, avoided by construction
instead of by care.

**Green check.**

1. **The agreement test, against a real surface with a non-zero origin.**
   `PRESENTER.md` §6.4: for random cameras, surface sizes and non-zero origins, a point
   is inside the culled address range **iff** its projection is inside the surface
   rect, generated by different code paths. This is P1, and P1 is a live bug in
   `hyper-ui` that survived because nothing in `renderer/` is tested.
2. Clicking the space returns its address, and `probe` is called **with no port in
   scope at all** (`PRESENTER.md` §8.4).
3. The `Placement` passes the runtime's generic discard harness with **no per-artifact
   test code contributed by this stage**. If per-artifact code turns out to be needed,
   D25 is wrong and that is a finding, not a workaround.

---

### 7.5 E4 — The first screen is authored, not coded

**Files.** `editor/genesis.rs`, `editor/addresses.rs`, `tests/genesis.rs`.

**What genesis writes.** The editor's screen as spaces: a canvas space, a space per
node, the extents, the style keys, the detail overrides. No behaviour yet — E5 and E6
add the composition. Under §2.3 it contains no policy and no algorithm.

**Decision record owed: D34 — the well-known addresses are the bootstrap ABI.**
`editor/addresses.rs` is the one place a literal address appears in the whole
repository. State what forced it (something has to be findable in an empty store before
anything can be drawn) and what it costs (a change to a well-known address is a
migration, and there is no migration machinery). **Rejected alternative:** discovery by
convention — scan for a space with a marker prop — rejected because it makes an empty
store and a corrupt store indistinguishable, which is §5.3's last stop-list row.

**Green check — the genesis discard test, and it is the important one.**

> Delete every space under the editor's screen root. Restart. **The portal still runs
> and the canvas is empty, with a finding that says so** — not a crash, not a black
> screen. Re-run genesis. The screen is **bit-identical** to before the delete.

That is R12's discard test pointed at the editor itself. It proves three things at once:
the screen really is data; genesis really is deterministic; and an emptied store fails
legibly. If it passes, self-hosting is no longer an argument.

---

### 7.6 E5 — It links

**Forces `COMPOSITOR.md` S3–S5.** `link`, `order`, `signature_of`, `Finding` and the
findings corpus stop being `todo!()`.

**The native block set, and how it grows.** Six to start. Each gets a file in
`src/editor/blocks/`, holds a plain Rust function over opaque payloads, and is
registered under a string key by `facade/ports/blocks.rs`.

| Key | Signature | Why it is needed |
|---|---|---|
| `probe-at` | point → address? | the pointer has to become an address, and only the presenter can do that |
| `read` | address → value | the composition has to see the store |
| `amend` | (address, value) → pending | the write path, and the only one (D24) |
| `commit` | address → committed | the commit boundary has to be authored, not implicit |
| `offset` | (point, point) → point | a drag is a delta. Every number is inside a block (L3) |
| `gate` | (value, flag) → value? | "on press, not on move" has to be expressible |

**These are app blocks, not platform concepts, and R32 is why.** Ask whether Coach
Assistant needs `probe-at`: it does not. So it is not platform, and it does not want a
place in any layer. It is registered under a string key by the app that needs it, which
is exactly what R4's registry mechanism is for. **A seventh block needs a line in
`docs/specs/EDITOR.md` saying which of §1's six interactions requires it** — not a
decision record, but not silent either.

**`Direction { In, Out }`.** `COMPOSITOR.md` §14 finding 6 is still open: it is an enum
in a core crate and R16 owes it a decision record saying the set is genuinely closed.
E5 is the stage that reads it, so E5 is the stage that owes the answer. Write **D35**
either justifying the enum or replacing the type. *"It looks closed"* is the sentence
that preceded all five prior occurrences of F-1.

**Green check.**

1. The editor's behaviour composition links, and `link` returns an `Outcome` carrying a
   plan — not a `Result`.
2. **The findings corpus:** each malformed composition yields **exactly one** finding.
   One cause, one finding; no cascade. Every finding carries a site, a said, a wanted
   and a remedy, and a finding kind registered with no remedy sentence fails the test
   rather than the review.
3. **The closure test** (`COMPOSITOR.md` §7.3): link C, wrap it as a block B, build C′
   containing only B wired straight through, link C′; the plan must be identical.
   Without this, composition stops after one flat layer and the editor cannot be a
   composition of compositions (C2).

---

### 7.7 E6 — It runs

**Forces `COMPOSITOR.md` S6 and S8, and `RUNTIME.md` S5, S6, S7.** `interpret`,
provenance, the artifact registry, and the staleness frontier all come alive.

**`facade/artifacts.rs` is the integration surface.** Both `Placement` and `Plan` are
registered with the runtime's registry under string keys, with their address ranges,
their rebuild functions and their validity watermarks. **Neither layer can do this** —
D23 forbids the runtime naming another layer, D29 forbids the presenter naming the
runtime, D26 forbids the compositor naming either. D30 records that `RUNTIME.md` §5.2 is
wrong about this and the facade is where it happens. That is D25's mechanism reaching
its fourth and fifth instances, and if it works here with no per-artifact machinery,
D25 is vindicated in the place it was written for.

**Green check.**

1. **Dragging a space moves it, and the move is performed by the interpreted
   composition.** The way this is verified matters (R23): assert it by *deleting the
   composition* and observing that dragging stops working while the window still runs.
   A test that merely observes movement cannot distinguish an interpreted drag from a
   native one, which is the whole claim.
2. Provenance recovers the **exact** declared input set for every executed step.
3. **An input change at revision N yields exactly the downstream address set** — no
   more, no fewer. This is `RUNTIME.md` S6's green check and `COMPOSITOR.md` S6's,
   deliberately identical in form: if the two disagree, one layer is wrong about what a
   dependency is.

---

### 7.8 E7 — It edits itself

No new layer work. This stage is the demonstration, and it should be a small commit.

**Green check, and it is the one to record on video (R23 — an interaction claim states
its verification method, and this one is worth stating twice).**

> With the editor running, drag a node **that belongs to the editor's own screen** —
> one of the spaces genesis wrote. The change persists. Restart the process. It is
> still there. **Nothing was recompiled.**

That is D16's thesis executed rather than asserted: the editor is an app built in the
platform, and it edits the graph it is drawn from. O11 is answered by demonstration.

**Decision record owed: D36 — O11 is closed, yes.** With its cost: the editor's screen
is now data that can be broken by editing it, and there is no undo beyond the store's
revisions and branches. That is not a gap — revisions, rollback by branch merge and
provenance are what the charter says production comes from — but the editor exposes
none of it yet, and saying so is what keeps it from being discovered.

---

### 7.9 E8 — Wiring, and the finding surface

**The one thing this stage must get right is C4.** A wire is drawn and `link` answers
**before it is committed**, because `Definitions` resolves stored ∪ pending (E1). The
person finds out the wire will not link *while dragging it*, not after. `COMPOSITOR.md`
§2's C4: *"so validation can only happen after commit, and the child finds out what was
wrong after doing it."*

**The finding surface.** A tag mismatch renders said / wanted / remedy, and **the canvas
zooms to the site** — because every finding carries an address, "go to the error" is
not an editor feature, it is a zoom (D20). D16 rules out stack traces as the error
surface, and about half of the time a person is working, the finding *is* the primary
output.

**Green check.** Drag a wire between two spaces with mismatched tags; the finding
appears before release; releasing it still creates the edge (D21 — *a drawn cycle is
judged, not refused*; the same stance applies to a mismatch: the edge may exist and
derivation runs as far as it can); clicking the finding zooms to its site.

---

### 7.10 E9 — Compilation, tier 0

**Forces `COMPOSITOR.md` S7.** The backend contract, tier 0, and the equivalence
harness.

Tier 0 is the resolved plan: sources resolved to slots, invocations to function
pointers, order fixed. **No compiler, no toolchain, no new dependency.** It is the
honest first test of the harness, because a backend that *should* be equivalent failing
the harness means the harness is wrong.

**The plan corpus is a deliverable, not a fixture** (D28's cost 1). Draw it from the
editor's own plans, which by E7 are real ones.

**Green check.** Tier 0 registers **by passing the harness** over the corpus — outputs
bit-for-bit, provenance edge-for-edge, no per-backend test code. And the compiled
artifact passes R12's generic discard harness without anyone writing anything for it,
which is the second thing D25 gives away free.

---

## 8 · New checks for `scripts/check-rules.sh`

Two new sections. `srcgrep`/`nogrep` already strip comment lines; every new check uses
them, because the documentation that cites a rule trips the grep that enforces it.

```bash
FA=src/facade
ED=src/editor
PO=src/portal

echo "infinite-solutions · facade — rule checks"

# R2 · the dependency direction is one-way. Only the facade names a layer.
check R2  "no layer crate named outside src/facade" \
  bash -c 'for d in src/editor src/portal; do
             ! ( find $d -name "*.rs" -print0 | xargs -0 sed "s://[/!]*.*::" \
                 | grep -Eq "infinite_(db|runtime|compositor|presenter)" ) || exit 1
           done'
check R2  "main.rs is thin" \
  bash -c '[ "$(grep -vcE "^\s*(//|$)" src/main.rs)" -le 60 ]'

# D29 · one narrowing point, and it is the Surface implementation.
check D29 "f32 appears only in facade/ports/surface.rs" \
  bash -c '! ( find src -name "*.rs" ! -path "src/facade/ports/surface.rs" -print0 \
               | xargs -0 sed "s://[/!]*.*::" | grep -q "f32" )'

# D30 / L5 · the app mints no identity either. Every reference is an address.
check L5  "every map in src is keyed by an address" maps_keyed_by_addr src

# D24 · there is exactly one write path, and input is not on it.
check D24 "portal/input.rs never submits a write" \
  nogrep 'submit|StoreWrite' $PO/input.rs

# R8 / L1 · the portal drives; the runtime does not.
check R8  "no thread pool in the tick path" \
  nogrep 'std::thread|thread::spawn|\bsleep\b|block_on' $PO/drive.rs

# F-8 · no mod.rs.
check F-8 "no mod.rs anywhere" bash -c '! find src -name mod.rs | grep -q .'

echo
echo "infinite-solutions · editor — rule checks"

# R2 · the app depends on the facade, and on nothing below it.
check R2  "editor names no layer crate" \
  nogrep 'infinite_(db|runtime|compositor|presenter)' $ED

# D29 · the app names no graphics crate.
check D29 "editor names no graphics or windowing crate" \
  nogrep 'wgpu|winit|glyphon|cosmic_text|raw_window_handle|softbuffer|glam' $ED

# §2.1 · appearance is authored data. There is no widget block.
check §2.1 "no widget-shaped block" \
  bash -c '! ls src/editor/blocks/ | grep -Eq "^(rectangle|label|panel|widget|button|text)\.rs$"'

# D34 · addresses live in exactly one file.
check D34 "no literal well-known address outside addresses.rs" \
  bash -c '! ( find src -name "*.rs" ! -path "src/editor/addresses.rs" -print0 \
               | xargs -0 sed "s://[/!]*.*::" | grep -Eq "\"/(input|style|screen)/" )'

# E4 · the screen is data. An emptied store fails as a finding, not a black frame.
check E4  "genesis discard — empty canvas with a finding; re-seed is bit-identical" \
  cargo test --offline --test genesis

# E5 · the editor's behaviour composition links.
check E5  "the editor's behaviour composition links" \
  cargo test --offline --test link
```

**Every one of these must be verified to fail before it is committed** (E0's
verification clause). Break it, watch it fire, restore it, and say in the commit message
which ones were exercised.

---

## 9 · Findings in the existing repository

Recorded, not fixed, except where a stage above fixes one. R21's habit: a finding list
that is never re-read becomes a document describing a repository that no longer exists.

1. **`scripts/check-rules.sh`'s R18 check fails today.** Its loop runs over `crates/*/`
   and expects `docs/specs/<LAYER>.md`; `crates/infinite-db` exists and
   `docs/specs/DB.md` does not. Since `check` reports `FAIL` and sets `fail=1`, the
   script as a whole is currently red. **Fixed in E0** by writing the short `DB.md`,
   which is more in keeping with R18 and R21 than weakening the check.
2. **`crates/infinite-db` is back on disk, and `COMPOSITOR.md` §14 finding 3 says it is
   gone.** It is present, with its **own `Cargo.lock` and its own `target/`**, and it is
   **not a workspace member**. Three things need reconciling before the root becomes a
   package: whether it joins `members` (a path dependency of a workspace member is a
   member unless excluded, and its inner lock is then ignored); whether `target/` is
   gitignored; and whether the facade should depend on the vendored copy at all rather
   than on the published `infinite-db` 0.4.x that D1 names. **Reconciled in E0 / D32:**
   it joins `members`; the root package path-depends on the vendored copy; a root
   `.gitignore` covers workspace `/target` and the crate's own `.gitignore` still covers
   its leftover `target/`; the trigger for switching to the published crate is *when
   the facade needs no store change for two consecutive stages*.
3. **Root `Cargo.toml`'s comment asserts the root declares no package.** E0 makes that
   false. It is replaced, not deleted, and the replacement cites D32.
4. **`docs/plans/` is an empty directory.** This document lands in it. Worth noting that
   `HISTORY.md`'s convention — a stage table at the top with a per-stage status and a
   green check — is followed here, and that F-5 (*a plan written after the code it
   describes*, every plan in `Innovator/plans`) is avoided by this one being written
   first.
5. **Every layer function is `todo!()` and all three stage tables read `not started`,
   including S2, whose files exist.** That is correct under R20 and this plan must not
   tidy it. Three status tables get updated by the changes that land E2–E9, each by the
   person landing it.
6. **`Direction { In, Out }` still owes a decision record** (`COMPOSITOR.md` §14 finding
   6). E5 is the stage that reads it and therefore the stage that owes the answer (D35).
7. **`crates/infinite-db/src/lib.rs` exposes `engine::write_queue::WriteJob` and
   `WriteSession`, and nothing named `try_submit`.** Closed by D33: the store grew
   `try_insert` / `try_enqueue_write`, which return `QueueFull` rather than waiting.
   The published name is `try_insert`, not `try_submit`; the contract is the same.
8. **§8 has eleven `check` invocations, not seven.** E0's verification clause asked
   for the seven new checks to be exercised. The two new sections contain eleven
   `check` calls. All eleven were verified to fail (layer crate outside the facade,
   `main.rs` over 60 lines, `f32` outside `surface.rs`, a `HashMap<u32, _>`, `submit`
   in `input.rs`, `block_on` in `drive.rs`, a `mod.rs`, a layer crate named from the
   editor, a graphics crate named from the editor, a `rectangle.rs` block, a
   `"/input/"` literal outside `addresses.rs`) and then restored. The count in the
   clause was wrong; the checks were not. Left as a finding rather than silently
   edited (R21).
9. **The `Journal` port is in-process at E1.** Session WAL exists and is unused
   (D8). E1's green check does not include crash replay; E2's does. Wiring
   `Journal` through `insert_with_session` without a replay path would be a second
   write that nothing reads. Left until E2 rather than dual-written. **Closed by E2:**
   `Journal::append` writes the session WAL; `open` replays recovered records into
   the pending set before the first tick. Verified by
   `tests/saturation.rs` `a_crash_mid_drag_replays_the_pending_set_before_the_first_tick`.
10. **`Glyphs` did not grow a shaping crate in E3.** No font is in the store and the
    first screen has no text run. Adding `cosmic-text` without a font would invent a
    measurement the way `hyper-ui` invented `char_w = font_size * 0.55`. The port
    still returns a declared box. Real shaping waits for a style that names a font
    (E4). Recorded rather than guessed.

11. **Nothing had ever drawn a pixel, through nine landed stages.**
    `src/facade/ports/surface.rs` computed the frame's quads correctly and ended
    `let _ = (_format, verts);`. There was no adapter, device, queue, swapchain,
    shader, pipeline, encoder, render pass or present anywhere in the repository, and
    `portal::Device::instance` had no callers — `main.rs` bound the device to
    `_device` and handed the window a `Store` instead. **Two status lines were
    therefore false**: this plan's E3, and `PRESENTER.md` S8's *"a real wgpu
    `Surface`"*. Both are corrected in the change that lands E10.0, and D41 records
    the mechanism rather than the instance. Fixed in E10.2 and E10.4.
12. **The window's geometry never reached the presenter.** `Store::set_surface` had
    five callers and all five were tests; `portal/input.rs::on_resize` amended
    `/input/surface` and nothing read that address. The running binary placed every
    frame against the 800×600 default installed by `facade::open`, whatever size the
    window was. Fixed in E10.3 (D43): the portal calls `set_surface` on `Resized` and
    on `ScaleFactorChanged`, and `/input/surface` is not a second path to the same
    fact.
13. **`Placed` carried no style and nothing resolved one.** `PRESENTER.md` is right
    that it should not; `Placeable` carries the key. But `place` dropped it, `frame`
    built the `SceneSet` internally and dropped that too, and
    `editor::styles::bootstrap_default` had one caller — genesis, encoding the row it
    wrote. Fixed in E10.4 (D44): style rows carry their own name, the app binds the
    table's range, and `Store::draw_with` resolves address → fill from the set the
    placement was built from.
14. **Two of the six interactions in §1 do not exist.** No `MouseWheel` arm, no pan
    gesture, and the camera is `default_camera()` except inside `zoom_to`. Zoom is
    *the primary navigation* (D20) and the argument for why any of this scales.
    **Still open** — E10.5, not attempted in the E10.0–E10.4 batch.
15. **The portal's coordinate spaces were unreconciled.** `CursorMoved` delivers
    device pixels; `SurfaceRect::size` is logical and carries `scale_factor`
    separately; `probe_at` took the raw values. Correct at scale 1.0, wrong on the
    first HiDPI display. Fixed in E10.3: the division happens once, in
    `portal/window.rs`, and everything above the portal is logical.
16. **`Placement` cannot express the grouping the presenter is said to own.** D15 and
    D29 both give this layer *"what is uploaded, in what order, at what detail,
    grouped how"*, and `Placement` is a flat `Vec<Placed>`. With one quad pipeline the
    gap is invisible. E8's wires are lines and a label is a text run. **Open — O20.**
17. **`infinite_presenter::binding::frame` now has no caller.** `Store::draw_with`
    takes the three steps itself, because D44's fill resolution needs the `SceneSet`
    the placement was built from and `frame` builds its own and drops it. The function
    is four lines and is left in place rather than deleted (R21). A binding function
    nobody calls is R27's defect. **Open — O21.**
18. **There is no authored position, so the drag is invisible and two nodes are one.**
    `SpaceRecord` carries an `origin`, `editor::blocks::displace` writes it, and
    `Scene::placed_in` decodes it and **throws it away** — `Placeable` has no field
    for it. Every space is positioned by `place_group`'s stacking, so genesis's two
    nodes are drawn at exactly the same rectangle, and E7's *"drag a node and it
    moves"* is true in the store and false on screen. This is not an oversight in the
    binding: **the presenter has no notion of an authored position at all**, and a
    canvas whose whole purpose is that a person puts things where they want them
    cannot be expressed without one. Raised rather than patched, because `Placeable`
    is a locked layer's core type and R29 says a change of that shape is corrected,
    not merged. **Blocks E10.4's "the gap between A and B is background" and all of
    E7's visible claim. O22, and it wants a decision record before code.**

---

## 10 · Open, carried forward

| # | Item | Trigger |
|---|---|---|
| **O15** | **Does the store admit a non-blocking submit?** | **Closed by D33.** The store grew `try_insert` |
| **O16** | **Where does the editor's undo live?** The charter says rollback comes from branch merge and audit from provenance; the editor exposes neither. Is undo a branch, a pending-set operation, or an app concern? | **Visible as of E7 / D36.** Still open. |
| **O17** | **Is the style table authored or native?** | **Closed by D37.** Authored under the style root when present; `editor/styles.rs` is the bootstrap fallback |
| O11 | Is the editor self-hosted | **Closed by D36.** Yes. |
| O13 | Three `Addr` types and two `Revision` types | Unchanged: when the facade's conversion is more than a newtype unwrap. E1 is where it would first be seen, and `facade/addr.rs` is where the assertion lives |
| O10 | Ownership and capability | Not this plan's. But `facade/ports/scene.rs` is where *"may this viewer see that space"* goes and `facade/ports/blocks.rs` is where *"may this composition use that block"* goes. **Do not build either so that the check cannot be inserted** |
| O1 | Hot working set | Sharpened by D30 to a measurement. E3 is the first stage where the number can actually be taken: a warm prefix scan of ~1000 nodes against frame budget |
| O14 | The precision floor | Unchanged. E3 clamps and reports; `facade/finding.rs` turns the presenter's fact into a finding with a remedy |
| O12 | May an iterative region yield | Out of scope. The editor exercises no solve, which is the same cut all three layer specs made |
| — | Settling loops, large numeric geometry | Out of scope by the same trigger all three specs name: the first consumer with a solve in it (the crane mat) |

---

## Appendix · Driving this with Cursor

**One stage per session.** Not one stage per prompt — a stage is a unit of review, and
E0 in particular is a large commit that is only reviewable as one thing.

**Open every session with this**, which is R24, R25 and R26 in the form the assistant
needs them:

> Read `docs/RULES.md` and `docs/DECISIONS.md` in full before doing anything else.
> Then read `docs/plans/EDITOR-BOOTSTRAP.md` §5 (the rules and the stop list) and the
> stage you are about to work on. Then read the specification for the layer that stage
> touches.
>
> You are working on stage **E<n>**. Its layer is **<layer>**. The consumer that breaks
> if it is wrong is **<consumer>**.
>
> Build only what the stage names. If something outside it seems necessary, that is a
> finding for §9 and a message to me — not a commit. If you are about to add an enum, a
> second in-memory graph, a metaphor name, a map keyed by an id, or a port beyond the
> declared count, stop and say so instead: R29 says those are corrected, not merged.
>
> When the stage is done: run `bash scripts/check-rules.sh`, run the stage's green
> check, update the status line **in this change** (R20), and write the decision records
> the stage owes with their rejected alternatives (R22).

**What to watch for in the assistant's output**, drawn from the three recurrences with
the highest historical rate:

- an `enum` proposed as *"the set is obviously closed"* — five prior occurrences;
- a `HashMap<SomeId, _>` introduced to *"make the lookup easier"* — F-2, about thirty
  instances in one codebase;
- a helper struct that caches part of the store *"to avoid re-reading"* — F-3 and F-7 at
  once;
- a `mod.rs`, which is the conventional Rust answer to several things this repository
  does differently — F-8;
- and a status line written before the work lands, which is the single mechanism
  `HISTORY.md` traces every recorded drift to.
