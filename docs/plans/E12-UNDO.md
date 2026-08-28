# Infinite Solutions — E12, undo

> **Status:** draft 1, 2026-08-28. Nothing landed. R20: a status line is written by the
> change that lands the phase, never at authoring time, and never by the person who
> wrote the plan — so every **Verified by** cell below is empty on purpose, and D41
> forbids marking any stage `landed` while it stays empty.
>
> Rules: [`../RULES.md`](../RULES.md) · Decisions: [`../DECISIONS.md`](../DECISIONS.md) ·
> Charter: [`../CHARTER.md`](../CHARTER.md) · Predecessor:
> [`E11-NEXT-STEPS.md`](./E11-NEXT-STEPS.md) §4 · Layer specs:
> [`../specs/RUNTIME.md`](../specs/RUNTIME.md), [`../specs/EDITOR.md`](../specs/EDITOR.md)
>
> Requires D48, which closed O16 and is the shape this plan implements. Opens O24, O25.

---

## Stage table

| # | Stage | Status | Verified by | Green check |
|---|---|---|---|---|
| **E12.0** | The check that fails | not started | — | `tests/undo.rs` exists, drags a node, commits, undoes, and asserts the node's authored origin is what it was before the drag. **It must fail on today's store**, which has no `undo` at all, before anything else here is written. Record what the failure looked like |
| **E12.1** | The commit journal is readable as a stream | not started | — | `Store::committed_since(watermark)` returns `(address, revision)` in commit order for this session, and a test asserts it is empty at open, one entry after one committed drag, and unchanged by a pan (which never commits, D48) |
| **E12.2** | Undo is a commit | not started | — | `Store::undo()` writes the value read at the revision *before* the last committed edit, as a new commit, and returns the address it touched. `stable_revision()` **increases**; a test asserts no revision was rewound and that the derivation DAG still resolves the intermediate state |
| **E12.3** | Redo, and the stream's shape | not started | — | Undo, redo, and the record is byte-identical to before the undo. A *new* committed edit after an undo drops the redo tail, and a test asserts the tail is gone rather than silently reachable |
| **E12.4** | Discard is the other verb | not started | — | Escape during a drag discards the pending amend and the node returns to its committed origin, with **no entry added to the undo stream** — the assertion that tells the two verbs apart |
| **E12.5** | The stream passes the discard test | not started | — | The undo stream registers under D25 with a rebuild rule, and R12's *generic* harness — `Store::artifact_passes_discard` — drops and rebuilds it with no per-artifact test code, exactly as the placement and the compiled plan already do |
| **E12.6** | It is reachable from the window | not started | — | `Ctrl+Z` / `Ctrl+Shift+Z` in `portal/window.rs`, and the six-interaction desktop pass done again with undo as a seventh. Manual, and stated as manual (R23) |

**E12.2 is the deliverable.** E12.0 is the one that must not be skipped. E12.5 is the
one that will look like paperwork and is the reason there is no fourth state category.

---

## 1 · What D48 settled, restated once

Four sentences, because the rest of this document is only their consequences.

1. **Undo operates on committed history**, and writes the previous value as a **new
   commit**. It never rewinds a revision.
2. **The pending set is not undone, it is discarded.** Two verbs, two gestures, two
   mechanisms — and R13 already gives the pending set the bound and the enumeration
   the discard verb needs.
3. **The camera is outside the undo stream by construction.** `pan_by` and `zoom_by`
   amend `CAMERA_KEY` and nothing ever commits it (D5), so a pan cannot enter a stream
   it never reaches. No rule excludes it; there is nothing to exclude.
4. **The stream is a registered derived artifact** (D25, R12): a pure function of the
   store's commit history above a session watermark. D8's Stored / Derived / Pending
   stands, and this plan adds no category.

The reason to write them down again is R20's sibling failure: a plan that restates a
decision loosely is how the decision drifts while the record still reads correctly.

---

## 2 · The one thing that is genuinely unresolved

**What is one undo?** A drag produces one committed amend, so a drag is one undo, and
that is the easy case. Two harder ones exist and neither has a consumer yet:

- **A gesture that commits more than one record.** Wiring two nodes commits a
  composition record; if it ever also commits a position, one gesture becomes two
  entries and Ctrl-Z half-undoes it.
- **A gesture repeated quickly.** Ten small drags in two seconds is ten entries, and
  a person expects roughly one.

Both are the same question — *does the stream group?* — and R27 says a capability with
no named consumer is a defect, so **the answer for now is no: one committed amend is
one undo entry.** O24 records the trigger. Do not build grouping before something
needs it; do not build the stream so that grouping cannot be added.

---

## 3 · The stages

### 3.1 E12.0 — The check that fails

`tests/undo.rs`, written before `Store::undo` exists:

1. Open a store, seed genesis, read node A's authored origin.
2. Drag it — through the interpreted composition, the way `tests/self_edit.rs` does,
   not by calling `amend` — and commit.
3. Assert the origin changed.
4. `store.undo()`.
5. Assert the origin is the original, and that `stable_revision()` is **higher** than
   it was at step 3.

Step 5's second clause is the one that makes this a test of D48 rather than of undo in
general: a rewind would pass the first clause and fail the second.

### 3.2 E12.1 — The commit journal is readable as a stream

The store already has revisions; what it has no accessor for is *"which addresses did
this session commit, in order"*. `Inner::commit_pending` is the one place a commit
happens, so the stream is derivable there.

**The watermark is the session's opening revision**, and it is why the stream is
per-session without anyone maintaining a per-session structure. Two sessions get one
stream each, which is the honest scope until multi-user exists.

**Decision owed: none.** This is D48's consequence, not a new choice. If it turns out
to need one — for instance if the store cannot answer *"the value at the previous
revision"* for a tombstone — that is a finding, not an `#[allow]`.

### 3.3 E12.2 — Undo is a commit

Read the value at the revision before the last entry; commit it at the same address.
Two subtleties, both worth stating before they are discovered:

- **An undo is itself a committed edit**, so it would enter the stream and undoing
  twice would oscillate. The stream carries a cursor rather than popping: undo moves
  the cursor back and commits, redo moves it forward and commits.
- **Undoing a create is committing a tombstone**, and undoing a delete is committing
  the value read at the previous revision. Both are the same operation with different
  inputs, which is the test that the mechanism is general rather than a special case
  per verb.

### 3.4 E12.3 — Redo, and the stream's shape

Standard, and the only decision in it is what a new edit does to the redo tail. **Drop
it**, and assert it is gone: a redo tail that survives a divergent edit is a branch,
and D48 rejected branch-per-keystroke explicitly. If someone later wants the tail kept,
that is a branch and the charter already has one.

### 3.5 E12.4 — Discard is the other verb

The assertion that keeps the two mechanisms from quietly merging. Escape during a drag
discards the pending amend; the node returns to its committed origin; **the undo stream
is unchanged**. Then, in the same test: pan, and assert the stream is still unchanged —
which is D48's third clause, checkable.

### 3.6 E12.5 — The stream passes the discard test

R12's harness is generic and already runs against the placement and the compiled plan.
If the undo stream needs a line of per-artifact test code to pass it, the stream is not
actually derived and D48's fourth clause is wrong — so this stage is a check on the
decision, not only on the code.

### 3.7 E12.6 — It is reachable from the window

`Ctrl+Z`, `Ctrl+Shift+Z`. The green check is manual and says so (R23). Fold it into the
desktop verification pass `E11-NEXT-STEPS.md` §2 still owes, rather than doing a second
one.

---

## 4 · Effort, stated honestly

| Stage | Shape | Rough size |
|---|---|---|
| E12.0 | one test file | an afternoon |
| E12.1 | an accessor and a watermark | 60–100 lines |
| E12.2 | read-at-revision, commit, cursor | 100–150 lines |
| E12.3 | the cursor's other direction | an afternoon |
| E12.4 | one test, and whatever it finds | an afternoon |
| E12.5 | registration, and the harness | 40 lines, plus whatever the harness says |
| E12.6 | two key arms | an hour, plus the manual pass |

Call it a week of evenings. The risk is concentrated in E12.2, and specifically in
whether `infinite-db` can be asked for *"the value at the revision before this one"*
cheaply for an address that was tombstoned. Check that before writing E12.1.

---

## 5 · Open

| # | Item | Trigger |
|---|---|---|
| **O24** | **Does the undo stream group?** §2. One committed amend is one entry, which is right until a gesture commits two records or a person makes ten small drags in two seconds. Candidates: a gesture token carried on the amend, or a time window — the first is honest and needs the input path to mint one, the second is a heuristic that will be wrong at some cadence | The first gesture that commits more than one record. E13's property inspector is the likely one: editing a field may touch a record and its layout |
| **O25** | **What does undo mean across a restart?** The watermark is the session's opening revision, so the stream is empty at open and yesterday's edits are not undoable. That is deliberate (R10: a thing that only means something while running belongs to the runtime) and it is also not obviously what a person expects of a document editor | A consumer that wants it. Note that the *capability* exists either way — the commit history is there — so this is a question about scope, not about whether it is possible |
| O10 | Ownership and capability | Unchanged, and undo touches it: an undo is a write, and *"may this viewer undo that space's edit"* is the same question the port was told to stay insertable for |
