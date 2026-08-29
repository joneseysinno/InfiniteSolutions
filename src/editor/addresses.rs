//! The well-known addresses. The bootstrap ABI, in one file (D34).
//!
//! A change to a well-known address is a migration, and there is no migration
//! machinery. This is the only file in the repository in which a literal
//! well-known address may appear (`scripts/check-rules.sh`).
//!
//! # The layout is a hierarchy, one nibble per level (D45)
//!
//! Every key is **four bytes**, because the editor's space is one `infinite-db`
//! dimension of 32 bits and that is the width the store hands back. What changed in
//! D45 is what those bytes mean: they are read most-significant nibble first, four
//! bits per level of nesting, so that *a child's key is its parent's key with one
//! more nibble set*. The top nibble is the region.
//!
//! ```text
//!   1 _ _ _ _ _ _ _   screen        2 _ _ _ _ _ _ _   styles
//!   3 _ _ _ _ _ _ _   behaviour     4 _ _ _ _ _ _ _   gesture
//!   5 _ _ _ _ _ _ _   session       6 _ _ _ _ _ _ _   authored graph
//!
//!   1 1 _ _ _ _ _ _   the canvas          (8 bits significant)
//!   1 1 1 _ _ _ _ _   node A, on it       (12)
//!   1 1 1 1 _ _ _ _   a node inside A     (16)
//! ```
//!
//! **The invariant that makes the length recoverable: no level's nibble is zero.**
//! Children are numbered from one, so the significant length of a key is four times
//! the position of its last non-zero nibble, and `facade::presenter_addr` can hand
//! the presenter an address that knows how deep it is without the presenter learning
//! anything about this scheme. `a_well_known_key_is_a_hierarchy` in `tests/genesis.rs`
//! is the check; break the invariant and it fails.
//!
//! Before D45 these keys were flat — `0x10000001`, `0x10000010`, `0x10000020` — and
//! no one contained another under any reading. Finding 19 is what that cost.

/// Pointer position in surface coordinates. Amended every pointer move.
pub const POINTER_POSITION: &str = "/input/pointer/position";

/// Pointer button flags. Amended on transition only.
pub const POINTER_BUTTON: &str = "/input/pointer/button";

/// Key event. Amended on transition only.
pub const KEY: &str = "/input/key";

/// Surface size, scale factor, origin. Amended on resize.
pub const SURFACE: &str = "/input/surface";

/// Style-table root. Authored rows live under [`STYLE_ROOT_KEY`].
pub const STYLE_ROOT: &str = "/style/";

/// The editor's screen root. Genesis writes under this; E4 deletes under this.
pub const SCREEN_ROOT: &str = "/screen/";

/// Inclusive start of the screen range. [`SCREEN_END_KEY`] is exclusive.
pub const SCREEN_ROOT_KEY: &[u8] = &[0x10, 0x00, 0x00, 0x00];

/// Exclusive end of the screen range.
pub const SCREEN_END_KEY: &[u8] = &[0x20, 0x00, 0x00, 0x00];

/// Property inspector panel (E13.2). A space; field rows are text primitives under it.
pub const INSPECTOR_KEY: &[u8] = &[0x12, 0x00, 0x00, 0x00];

/// Inspector field: selected address.
pub const INSPECTOR_ADDR_KEY: &[u8] = &[0x12, 0x10, 0x00, 0x00];

/// Inspector field: style key.
pub const INSPECTOR_STYLE_KEY: &[u8] = &[0x12, 0x20, 0x00, 0x00];

/// Inspector field: across extent.
pub const INSPECTOR_ACROSS_KEY: &[u8] = &[0x12, 0x30, 0x00, 0x00];

/// Inspector field: down extent.
pub const INSPECTOR_DOWN_KEY: &[u8] = &[0x12, 0x40, 0x00, 0x00];

/// Inspector field: authored origin.
pub const INSPECTOR_ORIGIN_KEY: &[u8] = &[0x12, 0x50, 0x00, 0x00];

/// Inspector field: depth from the address (D45).
pub const INSPECTOR_DEPTH_KEY: &[u8] = &[0x12, 0x60, 0x00, 0x00];

/// Block palette panel (E13.4). A space; templates are children under it.
pub const PALETTE_KEY: &[u8] = &[0x13, 0x00, 0x00, 0x00];

/// Toolbar panel (E13.6). Three affordances as children — not a widget layer.
pub const TOOLBAR_KEY: &[u8] = &[0x14, 0x00, 0x00, 0x00];

/// Undo/redo affordance. Left half undo, right half redo.
pub const TOOLBAR_HISTORY_KEY: &[u8] = &[0x14, 0x10, 0x00, 0x00];

/// Zoom level readout from the session camera.
pub const TOOLBAR_ZOOM_KEY: &[u8] = &[0x14, 0x20, 0x00, 0x00];

/// Run/pause toggle for the tick loop.
pub const TOOLBAR_RUN_KEY: &[u8] = &[0x14, 0x30, 0x00, 0x00];

/// Palette template: a plain node block to drag onto the canvas.
pub const PALETTE_PLAIN_KEY: &[u8] = &[0x13, 0x10, 0x00, 0x00];

/// Label for [`PALETTE_PLAIN_KEY`].
pub const PALETTE_PLAIN_LABEL_KEY: &[u8] = &[0x13, 0x11, 0x00, 0x00];

/// Palette template: a counter total starting at zero.
pub const PALETTE_TOTAL_KEY: &[u8] = &[0x13, 0x20, 0x00, 0x00];

/// Label for [`PALETTE_TOTAL_KEY`].
pub const PALETTE_TOTAL_LABEL_KEY: &[u8] = &[0x13, 0x21, 0x00, 0x00];

/// Palette template: a bump control wired to a total.
pub const PALETTE_BUMP_KEY: &[u8] = &[0x13, 0x30, 0x00, 0x00];

/// Label for [`PALETTE_BUMP_KEY`].
pub const PALETTE_BUMP_LABEL_KEY: &[u8] = &[0x13, 0x31, 0x00, 0x00];

/// The canvas space. The one root of the screen (D45).
pub const CANVAS_KEY: &[u8] = &[0x11, 0x00, 0x00, 0x00];

/// First sibling node on the canvas. Hosts a space of its own.
pub const NODE_A_KEY: &[u8] = &[0x11, 0x10, 0x00, 0x00];

/// First node inside node A's own space (D20/D31 — a node may host a space).
pub const NODE_A1_KEY: &[u8] = &[0x11, 0x11, 0x00, 0x00];

/// Second node inside node A's own space.
pub const NODE_A2_KEY: &[u8] = &[0x11, 0x12, 0x00, 0x00];

/// Second sibling node on the canvas.
pub const NODE_B_KEY: &[u8] = &[0x11, 0x20, 0x00, 0x00];

/// The wire drawn between node A and node B (E11). A link on the canvas, so it is
/// a sibling of the two nodes it joins and is placed after them.
pub const WIRE_AB_KEY: &[u8] = &[0x11, 0x30, 0x00, 0x00];

/// Inclusive start of the style range.
pub const STYLE_ROOT_KEY: &[u8] = &[0x20, 0x00, 0x00, 0x00];

/// Exclusive end of the style range.
pub const STYLE_END_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x00];

/// The `plain` style row. Nodes.
pub const STYLE_PLAIN_KEY: &[u8] = &[0x21, 0x00, 0x00, 0x00];

/// The `canvas` style row. The canvas, and therefore the background (E10.2).
pub const STYLE_CANVAS_KEY: &[u8] = &[0x22, 0x00, 0x00, 0x00];

/// The `wire` style row (E11).
pub const STYLE_WIRE_KEY: &[u8] = &[0x23, 0x00, 0x00, 0x00];

/// The editor's behaviour composition. Genesis writes this; E5 links it.
pub const BEHAVIOUR_ROOT_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x00];

/// `probe-at` instance inside the behaviour composition.
pub const BEHAVIOUR_PROBE_KEY: &[u8] = &[0x31, 0x00, 0x00, 0x00];

/// `read` instance.
pub const BEHAVIOUR_READ_KEY: &[u8] = &[0x32, 0x00, 0x00, 0x00];

/// `amend` instance.
pub const BEHAVIOUR_AMEND_KEY: &[u8] = &[0x33, 0x00, 0x00, 0x00];

/// `commit` instance.
pub const BEHAVIOUR_COMMIT_KEY: &[u8] = &[0x34, 0x00, 0x00, 0x00];

/// `offset` instance.
pub const BEHAVIOUR_OFFSET_KEY: &[u8] = &[0x35, 0x00, 0x00, 0x00];

/// `gate` instance.
pub const BEHAVIOUR_GATE_KEY: &[u8] = &[0x36, 0x00, 0x00, 0x00];

/// `displace` instance.
pub const BEHAVIOUR_DISPLACE_KEY: &[u8] = &[0x37, 0x00, 0x00, 0x00];

/// Press-origin for a drag in progress. Latched while the button is down.
pub const DRAG_FROM_KEY: &[u8] = &[0x41, 0x00, 0x00, 0x00];

/// One-shot pulse on pointer release. Cleared after each behaviour tick.
pub const RELEASE_PULSE_KEY: &[u8] = &[0x42, 0x00, 0x00, 0x00];

/// Pending origin for an inspector edit (E13.3). Sixteen bytes: x then y.
pub const EDIT_ORIGIN_KEY: &[u8] = &[0x43, 0x00, 0x00, 0x00];

/// One-shot pulse when an inspector edit is committed. Cleared after each tick.
pub const EDIT_COMMIT_KEY: &[u8] = &[0x44, 0x00, 0x00, 0x00];

/// Latched palette template while a block drag is in progress (E13.4).
pub const PALETTE_FROM_KEY: &[u8] = &[0x45, 0x00, 0x00, 0x00];

/// Drop origin for a palette placement. Sixteen bytes: x then y in parent space.
pub const PLACE_ORIGIN_KEY: &[u8] = &[0x46, 0x00, 0x00, 0x00];

/// Minted address for the block being placed.
pub const PLACE_ADDR_KEY: &[u8] = &[0x47, 0x00, 0x00, 0x00];

/// One-shot pulse when a palette drop commits. Cleared after each tick.
pub const PLACE_COMMIT_KEY: &[u8] = &[0x48, 0x00, 0x00, 0x00];

/// Shift-wire mode (E13.5). Set while shift is held during a pointer press.
pub const WIRE_MODE_KEY: &[u8] = &[0x49, 0x00, 0x00, 0x00];

/// Latched source node while wiring by pointer.
pub const WIRE_FROM_KEY: &[u8] = &[0x4A, 0x00, 0x00, 0x00];

/// Probed target node while wiring.
pub const WIRE_TO_KEY: &[u8] = &[0x4B, 0x00, 0x00, 0x00];

/// Minted address for the wire space record under its parent.
pub const WIRE_ADDR_KEY: &[u8] = &[0x4C, 0x00, 0x00, 0x00];

/// One-shot pulse when a wire drag commits.
pub const WIRE_COMMIT_KEY: &[u8] = &[0x4D, 0x00, 0x00, 0x00];

/// Force a tag-mismatch preview graph (tests). Cleared after each tick.
pub const WIRE_MISMATCH_KEY: &[u8] = &[0x4E, 0x00, 0x00, 0x00];

/// The session camera (E10.5, D5). Amended directly by the portal's pan/zoom —
/// session-scoped fact, not authored geometry, so it is never read by the
/// interpreted behaviour composition the way [`DRAG_FROM_KEY`] is.
pub const CAMERA_KEY: &[u8] = &[0x51, 0x00, 0x00, 0x00];

/// Authored selection (E13.1). Points at the selected space's key.
pub const SELECT_KEY: &[u8] = &[0x52, 0x00, 0x00, 0x00];

/// Whether the graph tick loop is active (E13.6). Non-zero means running.
pub const RUN_KEY: &[u8] = &[0x53, 0x00, 0x00, 0x00];

/// Exclusive end of the session range (camera + selection + run).
pub const SESSION_END_KEY: &[u8] = &[0x54, 0x00, 0x00, 0x00];

/// `gate` instance for selection — passes on [`RELEASE_PULSE_KEY`].
pub const BEHAVIOUR_SELECT_GATE_KEY: &[u8] = &[0x38, 0x00, 0x00, 0x00];

/// `encode-selection` instance.
pub const BEHAVIOUR_ENCODE_SELECTION_KEY: &[u8] = &[0x39, 0x00, 0x00, 0x00];

/// Second `amend` instance — writes [`SELECT_KEY`], not the probed node.
pub const BEHAVIOUR_SELECT_AMEND_KEY: &[u8] = &[0x3A, 0x00, 0x00, 0x00];

/// Second `commit` instance — commits [`SELECT_KEY`].
pub const BEHAVIOUR_SELECT_COMMIT_KEY: &[u8] = &[0x3B, 0x00, 0x00, 0x00];

/// `read` instance for inspector edits — addr is the selection, not the probe hit.
pub const BEHAVIOUR_EDIT_READ_KEY: &[u8] = &[0x3C, 0x00, 0x00, 0x00];

/// `set-origin` instance.
pub const BEHAVIOUR_SET_ORIGIN_KEY: &[u8] = &[0x3D, 0x00, 0x00, 0x00];

/// `gate` instance — passes on [`EDIT_COMMIT_KEY`].
pub const BEHAVIOUR_EDIT_GATE_KEY: &[u8] = &[0x3E, 0x00, 0x00, 0x00];

/// Third `amend` instance — writes the selected space after an inspector edit.
pub const BEHAVIOUR_EDIT_AMEND_KEY: &[u8] = &[0x3F, 0x00, 0x00, 0x00];

/// Third `commit` instance — commits the selected space after an inspector edit.
pub const BEHAVIOUR_EDIT_COMMIT_KEY: &[u8] = &[0x40, 0x00, 0x00, 0x00];

/// `read` instance for palette placement — addr is [`PALETTE_FROM_KEY`].
pub const BEHAVIOUR_PLACE_READ_KEY: &[u8] = &[0x32, 0x10, 0x00, 0x00];

/// Third `amend` instance — writes a minted block onto the canvas.
pub const BEHAVIOUR_PLACE_AMEND_KEY: &[u8] = &[0x33, 0x10, 0x00, 0x00];

/// Fourth `commit` instance — commits a minted block.
pub const BEHAVIOUR_PLACE_COMMIT_KEY: &[u8] = &[0x34, 0x10, 0x00, 0x00];

/// Second `set-origin` instance — positions a palette template at the drop point.
pub const BEHAVIOUR_PLACE_SET_ORIGIN_KEY: &[u8] = &[0x3D, 0x10, 0x00, 0x00];

/// Fourth `gate` instance — passes on [`PLACE_COMMIT_KEY`].
pub const BEHAVIOUR_PLACE_GATE_KEY: &[u8] = &[0x3E, 0x10, 0x00, 0x00];

/// `encode-wire` instance.
pub const BEHAVIOUR_ENCODE_WIRE_KEY: &[u8] = &[0x32, 0x20, 0x00, 0x00];

/// Fifth `amend` instance — writes a minted wire record.
pub const BEHAVIOUR_WIRE_AMEND_KEY: &[u8] = &[0x33, 0x20, 0x00, 0x00];

/// Fifth `commit` instance — commits a minted wire record.
pub const BEHAVIOUR_WIRE_COMMIT_KEY: &[u8] = &[0x34, 0x20, 0x00, 0x00];

/// Fifth `gate` instance — passes on [`WIRE_COMMIT_KEY`].
pub const BEHAVIOUR_WIRE_GATE_KEY: &[u8] = &[0x3E, 0x20, 0x00, 0x00];

/// Exclusive end of the behaviour range.
pub const BEHAVIOUR_END_KEY: &[u8] = &[0x41, 0x00, 0x00, 0x00];

/// Authored graph being wired. A pending record here is C4's in-flight wire.
pub const GRAPH_ROOT_KEY: &[u8] = &[0x60, 0x00, 0x00, 0x00];

/// User-authored app composition (E13.7).
pub const APP_ROOT_KEY: &[u8] = &[0x61, 0x00, 0x00, 0x00];

/// Bump and total keys the app graph serves — eight bytes.
pub const APP_LINK_KEY: &[u8] = &[0x61, 0x00, 0x00, 0x01];

/// `read` instance in the counter app.
pub const APP_READ_KEY: &[u8] = &[0x61, 0x10, 0x00, 0x00];

/// `increment-text` instance in the counter app.
pub const APP_INCREMENT_KEY: &[u8] = &[0x61, 0x20, 0x00, 0x00];

/// `amend` instance in the counter app.
pub const APP_AMEND_KEY: &[u8] = &[0x61, 0x30, 0x00, 0x00];

/// `commit` instance in the counter app.
pub const APP_COMMIT_KEY: &[u8] = &[0x61, 0x40, 0x00, 0x00];
