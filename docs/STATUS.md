# InfiniteSolutions Status

> Current as of 2026-08-28. A capability is listed as implemented only when the repository has an executable check or an explicit manual verification path for it.

## Implemented

- Persistent spatial graph storage with addressed records, revisions, snapshots, branches, merges, crash recovery, directed hyperedges, typed payloads, derivation, and staleness queries.
- Four-layer architecture: store, compositor, runtime, and presenter, exposed through one platform facade.
- Native Rust block registration with opaque ports and values.
- Composition linking, link-time validation, structured findings, nested composition closure, interpreted execution, provenance, and tier-0 compiled-plan equivalence checking.
- Runtime pending state, journal recovery, non-blocking writes, stale-frontier scheduling, and derived-artifact discard/rebuild behavior.
- Presenter culling, arrangement, camera transforms, level of detail, address-based hit testing, authored styles, GPU rendering, and headless pixel readback.
- An authored session camera (E10.5): pan and zoom amend a well-known record resolved stored ∪ pending, and survive a restart via journal replay, verified by `tests/camera.rs`.
- Nested spaces, and zoom as the way into them (E10.5's second half, D45): an address carries its significant bit length so containment is structural, and a space opens when its extent on the surface reaches the view's opening extent. Node A hosts a space with two nodes; at rest they are not drawn, and one zoom later they are drawn inside it and answer a probe. Verified by `tests/nesting.rs`, written before the fix and seen to fail.
- Placement grouping (D46): the placement partitions what it placed into runs sharing an opaque primitive key, and the facade selects a pipeline per run. Verified by `tests/wires.rs::the_placement_groups_the_wire_apart_from_the_rectangles`.
- Wire rendering (E11): a link record with two endpoint addresses is drawn as a line between wherever its ends landed, through a second pipeline. Verified by `tests/wires.rs`, with both failure modes run and seen.
- Self-hosted editor bootstrap: authored screen data, selection, basic dragging, wire preview while pending, persistence, and finding navigation.
- Desktop portal with window creation, GPU setup, resize and scale-factor reconciliation, OS input conversion, and portal-driven ticks.

## Current Direction

InfiniteSolutions is its own primary consumer. The editor must become capable of creating and persisting the compositions needed to build the other applications. Each new platform capability should be forced by that self-hosting workflow and verified before it is declared complete.

## Not Yet

- Complete no-code application-authoring workflow.
- Property inspector, block palette, toolbar, settings, and general text editing.
- General widget toolkit or reusable application templates.
- **Complete desktop verification of all authored-position editing gestures.** Unchanged, and now the oldest item on this list: every claim above is proven by an automated check against an offscreen texture, and nobody has sat at the running window and dragged, panned, zoomed, wired and clicked a finding at both 1.0 and a HiDPI scale factor. `docs/plans/E11-NEXT-STEPS.md` §2.
- Text as a drawn primitive, and therefore labels of any kind. The primitive mechanism is in place (D46); the third primitive is not — `docs/plans/E13-AUTHORING-SURFACE.md` E13.0.
- Authoring a wire by pointer. E11 draws a link record; nothing yet writes one from a gesture.
- Undo and redo. **The model is decided** (D48: undo is a new commit, the pending set is discarded rather than undone, the camera is outside the stream because it never commits, and the stream is a registered derived artifact). No code — stages in `docs/plans/E12-UNDO.md`.
- Authentication and authorization.
- General solver support, iterative regions, convergence, damping, and stopping findings.
- Large-scale numerical geometry and domain-specific units or types.
- Broad domain facades and applications created entirely through the visual editor.
- Production multi-user deployment, server orchestration, replication workflows, and broader platform portals.

## Verification

Run `cargo test --workspace --all-targets` where all workspace feature requirements are enabled, and run `bash scripts/check-rules.sh`. For the editor, also exercise the focused genesis, nesting, wiring, wires, behavior, self-edit, tier-0, and pixel tests, then manually verify the six editor interactions at normal and HiDPI scale — that last one is the item under **Not Yet** that has never been done.

One workspace test does not compile and predates this work: `crates/infinite-db/tests/space_tower.rs` names `infinite_db::infinitedb_server`, which is gated behind the vendored store's `server` feature. Scope test runs to `-p infinite-solutions -p infinite-presenter -p infinite-compositor -p infinite-runtime` until that is either enabled or the test is gated to match.
