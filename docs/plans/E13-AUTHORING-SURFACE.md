# Infinite Solutions — E13, the authoring surface

> **Status:** draft 1, 2026-08-28. Nothing landed. R20: every **Verified by** cell
> below is empty on purpose, and D41 forbids marking any stage `landed` while it stays
> empty.
>
> Rules: [`../RULES.md`](../RULES.md) · Decisions: [`../DECISIONS.md`](../DECISIONS.md) ·
> Charter: [`../CHARTER.md`](../CHARTER.md) · Predecessor:
> [`E11-NEXT-STEPS.md`](./E11-NEXT-STEPS.md) §5 · Layer spec:
> [`../specs/EDITOR.md`](../specs/EDITOR.md)
>
> **This document is a specification, not code, and that is R28**: *when the idea is
> fuzzy, the deliverable is a specification. Fuzzy plus code is the drift mechanism.*
> E13 is the largest remaining scope in the project and the least pinned down, so it
> gets written before it gets built.
>
> Requires D45, D46, D48. Opens O26, O27, O28.

---

## 0 · What this stage is for, in one sentence

Everything before E13 made the editor able to *show* a graph and *move* what is in it.
E13 is what lets a person **add** something that was not there — which is the
difference between a canvas of coloured rectangles and a thing a child could build an
app with.

The charter's constraint is the design brief, unchanged: *a child should be able to
build a full stack app with it.* That rules out a text syntax in the core loop, rules
out stack traces as the error surface, and requires the four layers to be invisible.

---

## Stage table

| # | Stage | Status | Verified by | Green check |
|---|---|---|---|---|
| **E13.0** | Text reaches the screen | not started | — | A third primitive under D46 — `primitive: "text"` — with a glyph run resolved through the `Glyphs` port that has been a stub since E3. A readback test asserts a known string's pixels differ from the background in the cells the layout says they occupy, and match at a second scale factor |
| **E13.1** | Selection is authored, not a flag | not started | — | Selecting a node writes a record; a second window (or a restart) shows the same selection. **The check that can fail**: `Placed` still carries no selection field, and `check-rules.sh`'s L5 identity-shape check still passes. `hyper-ui`'s `SceneNode { selected: bool }` is the thing this stage exists to not become |
| **E13.2** | The property inspector reads | not started | — | Select a node; a panel shows its address, its style key, its extent, its origin, **and its depth read from the address** (D45 — the thing option (b) would have made a lie). Every field is read through the same `Scene` port the canvas uses; the panel names no store type |
| **E13.3** | The property inspector writes | not started | — | Edit a field; the record changes; the canvas follows in the same frame; `Ctrl+Z` puts it back (E12). The write goes through the interpreted behaviour composition, not through `store.amend` — E6's discipline, applied to the second input surface |
| **E13.4** | The block palette | not started | — | Drag a block from a palette onto the canvas and a new record exists at a **new address the editor minted under the parent's** (D45's nibble-per-level scheme, which is what makes "under" meaningful). Restart: it is still there. Its address says which space it is in |
| **E13.5** | Wiring by pointer | not started | — | Drag from one node to another and a link record is committed — the authoring half of E11, which only drew a wire that genesis had written. A mismatch raises a finding and zooms to its site (E8's path, reached by a gesture) |
| **E13.6** | The toolbar, and what it is not | not started | — | Whatever survives §4's test. A toolbar is the easiest place in this project to accrete a widget toolkit, which `EDITOR.md` §2.1 forbids and `check-rules.sh` greps for |
| **E13.7** | One small real thing, built through the editor | not started | — | O11's actual closure. Not the editor, not a demo — something with a use, authored entirely by pointer, that runs. See §5 |

**E13.7 is the deliverable.** Everything above it is scaffolding for it, and a plan
that lands E13.0–E13.6 and stops has not answered the question this stage was written
to answer.

---

## 1 · The order, and why it is this order

E13.0 first because **text is the input every other stage needs**: a property
inspector with no text can display nothing, a palette with no text is a row of
unlabelled squares, and a toolbar is entirely labels. It is also the stage that tests
D46's claim that the primitive set is genuinely open — a third primitive, added by the
mechanism rather than by amending it, is the check that the mechanism is real. If
adding text requires touching `Placement`, D46 was wrong and should be corrected
rather than worked around (R29).

E13.1 before E13.2 because an inspector inspects a *selection*, and selection is where
the corpus's most-cited breakage lives. `hyper-ui`'s geometry record carries
`selected: bool`, so selecting a thing means re-deriving and re-uploading its geometry,
and a click has no path back to a node at all. The presenter already refuses to hold
identity-shaped state (L5, checked); this stage is where the pressure to relent
arrives.

E13.4 after D45 rather than before it, which is the sequencing `E11-NEXT-STEPS.md` §5
asked for and could not have: minting an address *under* a parent needs "under" to
mean something, and until D45 it did not.

E13.7 last only in the table. In practice it should start the moment E13.0–E13.5 are
enough to try, and be allowed to rewrite the rest — see §5.

---

## 2 · The three things most likely to go wrong

Stated now, so that recognising them later is cheap.

**A widget toolkit.** `EDITOR.md` §2.1 forbids a widget-shaped native block and
`check-rules.sh` greps `src/editor/blocks/` for `rectangle|label|panel|widget|button|
text`. E13 is the stage that will want every one of those names. The discipline that
replaces them: a panel is a **space** with children, a label is a **text primitive**,
and a button is a space that `accepts`. If a stage needs a block called `button`, the
composition model is being routed around.

**A second input path.** E6 established that a drag is performed by the interpreted
behaviour composition, and `check-rules.sh` checks that `portal/input.rs` never
submits a write. An inspector field is the obvious place to shortcut that — read the
keystroke, call `store.amend` — and it would be a second write path for one fact,
which is F-7's shape. E13.3's green check says *through the composition* for that
reason.

**A closed enum.** A field kind (number, text, colour, address), a block category, a
tool. Three chances to add the sixth instance of F-1. Every one of them is a
string-keyed registry (R4, D46's precedent) or it is a defect.

---

## 3 · What text actually needs (E13.0, expanded)

The stage with the most unknowns, so it gets the most words.

`infinite_presenter::binding::ports::Glyphs` has existed since E3 and is a stub. What
it must answer is *"how much room does this run take"* — an extent, in the same units
`Placeable::across` is in — and nothing else. What it must **not** answer is what the
glyphs look like: rasterisation is the facade's, in the one file allowed an `f32`,
alongside the two pipelines D46 already selects between.

The open question is where the string lives, and there are two candidates:

| | Where | Cost |
|---|---|---|
| a | **On the space record**, beside `style` and `primitive` — a text run is a space whose payload happens to be text | Simple, and consistent with "a panel is a space". Makes every record carry a field that is empty for almost all of them |
| b | **In a separate record the space points at**, addressed under it | Keeps the space record small, and a long document is not re-read every time its box moves. One more indirection, and one more thing that can dangle |

**(a) until something needs (b)**, and the trigger is a text run long enough that
re-reading it per frame is measurable — which is O1's shape and probably O1's
measurement. Recorded as O26 rather than decided here, because R28's whole point is
that this document is where the fuzzy parts get named, not resolved by guess.

---

## 4 · The toolbar test (E13.6)

Before building a toolbar, answer: **what is on it that is not either a block in the
palette or a property in the inspector?** If the answer is "nothing", there is no
toolbar, and that is a good outcome — R27 makes an unrequired capability a defect.

Plausible survivors: undo/redo (E12.6 puts them on keys, and a child does not know the
keys), the zoom level, and whether the graph is running. That is three affordances, not
a chrome layer, and three affordances are three spaces on the screen record.

---

## 5 · E13.7, and why the rest of the plan is provisional

`CHARTER.md`: *the platform's own editor is the ultimate forcing consumer.* D36 closed
O11 on the editor being self-hosted, and `E11-NEXT-STEPS.md` §6 is right that this is
not the same claim as *"someone built a real thing with it"* — **self-hosting only
means something once it builds something that is not itself.**

So E13.7 is not a demo. It is one small real thing with an actual use, authored
entirely by pointer, that runs. Not SES and not Coach Assistant — D12 defers both, and
either would take longer than the platform can afford to go unvalidated. Something
smaller: a form that stores what is typed into it, a counter with a persisted total, a
graph that reads one input and writes one record.

**And it should start early and be allowed to rewrite this plan.** Every stage above
E13.7 is a guess about what building something needs. The moment E13.0–E13.5 are
enough to try, trying is worth more than finishing the guesses — `E11-NEXT-STEPS.md`
§6's closing point, and the reason this document's stage table is provisional in a way
`E10-IT-DRAWS.md`'s was not.

---

## 6 · Open

| # | Item | Trigger |
|---|---|---|
| **O26** | **Where does a text run's string live?** §3 — on the space record, or in a record it points at | A run long enough that re-reading it per frame is measurable. Likely the same measurement as O1 |
| **O27** | **Is selection one address or many?** E13.1 writes a selection record. A single address is enough for an inspector and wrong for a marquee, and retrofitting a set is cheap only if the record is a set from the start — which R27 says not to build until something selects two things | E13.3, if editing a property of several nodes at once is wanted; otherwise E13.6 |
| **O28** | **Who mints an address for a new block?** E13.4. The editor knows the parent and the nibble scheme (D45), so it can pick the next free child index — but "next free" is a read of the parent's subtree, and two sessions doing it at once pick the same one. Candidates: a per-session prefix, or the store issuing addresses | E13.4 for the single-session answer; multi-user for the real one, which `STATUS.md` already lists under Not Yet |
| O24 | Does the undo stream group? (`E12-UNDO.md`) | E13.3 is the likely first gesture that commits two records |
| O10 | Ownership and capability | E13 is where a person first creates something, and *"who owns this space"* wants an answer at creation rather than a backfill |
