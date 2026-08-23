//! The well-known addresses. The bootstrap ABI, in one file (D34).
//!
//! A change to a well-known address is a migration, and there is no migration
//! machinery. This is the only file in the repository in which a literal
//! well-known address may appear (`scripts/check-rules.sh`).
//!
//! Screen and style keys are **four bytes** so a subtree is one u32 range under
//! the facade's left-padded identity mapping. Path strings name the ABI; the
//! keys are what genesis writes.

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
pub const SCREEN_END_KEY: &[u8] = &[0x11, 0x00, 0x00, 0x00];

/// The canvas space.
pub const CANVAS_KEY: &[u8] = &[0x10, 0x00, 0x00, 0x01];

/// First sibling node on the canvas.
pub const NODE_A_KEY: &[u8] = &[0x10, 0x00, 0x00, 0x10];

/// Second sibling node on the canvas.
pub const NODE_B_KEY: &[u8] = &[0x10, 0x00, 0x00, 0x20];

/// Inclusive start of the style range.
pub const STYLE_ROOT_KEY: &[u8] = &[0x20, 0x00, 0x00, 0x00];

/// Exclusive end of the style range.
pub const STYLE_END_KEY: &[u8] = &[0x21, 0x00, 0x00, 0x00];

/// The `plain` style row. Nodes.
pub const STYLE_PLAIN_KEY: &[u8] = &[0x20, 0x00, 0x00, 0x01];

/// The `canvas` style row. The canvas, and therefore the background (E10.2).
pub const STYLE_CANVAS_KEY: &[u8] = &[0x20, 0x00, 0x00, 0x02];

/// The editor's behaviour composition. Genesis writes this; E5 links it.
pub const BEHAVIOUR_ROOT_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x00];

/// `probe-at` instance inside the behaviour composition.
pub const BEHAVIOUR_PROBE_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x01];

/// `read` instance.
pub const BEHAVIOUR_READ_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x02];

/// `amend` instance.
pub const BEHAVIOUR_AMEND_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x03];

/// `commit` instance.
pub const BEHAVIOUR_COMMIT_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x04];

/// `offset` instance.
pub const BEHAVIOUR_OFFSET_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x05];

/// `gate` instance.
pub const BEHAVIOUR_GATE_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x06];

/// `displace` instance.
pub const BEHAVIOUR_DISPLACE_KEY: &[u8] = &[0x30, 0x00, 0x00, 0x07];

/// Exclusive end of the behaviour range.
pub const BEHAVIOUR_END_KEY: &[u8] = &[0x31, 0x00, 0x00, 0x00];

/// Press-origin for a drag in progress. Latched while the button is down.
pub const DRAG_FROM_KEY: &[u8] = &[0x40, 0x00, 0x00, 0x01];

/// The session camera (E10.5, D5). Amended directly by the portal's pan/zoom —
/// session-scoped fact, not authored geometry, so it is never read by the
/// interpreted behaviour composition the way `DRAG_FROM_KEY` is.
pub const CAMERA_KEY: &[u8] = &[0x50, 0x00, 0x00, 0x01];

/// Authored graph being wired. A pending record here is C4's in-flight wire.
pub const GRAPH_ROOT_KEY: &[u8] = &[0x60, 0x00, 0x00, 0x00];
