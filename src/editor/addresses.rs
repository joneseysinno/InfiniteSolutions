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

/// Exclusive end of the behaviour range.
pub const BEHAVIOUR_END_KEY: &[u8] = &[0x40, 0x00, 0x00, 0x00];

/// Press-origin for a drag in progress. Latched while the button is down.
pub const DRAG_FROM_KEY: &[u8] = &[0x41, 0x00, 0x00, 0x00];

/// The session camera (E10.5, D5). Amended directly by the portal's pan/zoom —
/// session-scoped fact, not authored geometry, so it is never read by the
/// interpreted behaviour composition the way [`DRAG_FROM_KEY`] is.
pub const CAMERA_KEY: &[u8] = &[0x51, 0x00, 0x00, 0x00];

/// Authored graph being wired. A pending record here is C4's in-flight wire.
pub const GRAPH_ROOT_KEY: &[u8] = &[0x60, 0x00, 0x00, 0x00];
