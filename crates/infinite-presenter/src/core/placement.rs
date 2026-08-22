//! [`Placement`] — the artifact (spec §4, §8.2).

use std::collections::BTreeMap;

use crate::core::addr::Addr;
use crate::core::placed::Placed;
use crate::core::revision::Revision;
use crate::core::transform::Transform;

/// The derived map from address to rectangle at a level, in draw order.
///
/// > **This type was called `RenderList` in D5 and D25, and that name is retired.**
///
/// Three reasons: it answers pointer queries, which is not rendering; it holds no draw
/// commands, so *list* describes the wrong thing; and it is what
/// [`crate::core::probe`] reads, which has no rendering in it at all. R17 permits a
/// rename and forbids a *recycle* — `RenderList` is retired, not reused — and D20 set
/// the precedent when it retired "chart" for "space", with the same convention:
/// citations of D5 and D25 keep the original word, nothing else uses it. The rename is
/// flagged in the specification's §13 rather than buried, because a rename proposed by
/// an assistant is the class of change R29 says to correct rather than merge.
///
/// # It is a registered derived artifact (D25)
///
/// The presenter owns the **function**; the runtime owns the **schedule**. A placement
/// is registered under a string key with the address ranges it derives from, a rebuild
/// function, and a validity watermark — so R12's generic discard harness drops it and
/// rebuilds it *without knowing what it is*, and this layer contributes no
/// invalidation machinery and no per-artifact test code.
///
/// That is what makes P5 impossible. `hyper-ui`'s render list claims in its module doc
/// that *"it is rebuilt only when the structure or layout changes"* and the type has
/// no dirty flag, no generation and no watermark; the protocol is a comment on an enum
/// variant saying *"the host must rebuild layout and the render list"*. A host that
/// forgets it gets stale rectangles and clicks that land on the wrong thing, silently.
///
/// **The registration itself happens in the facade, not here** — this crate may not
/// name the runtime (D29), and the runtime may not name this one (D23).
/// `binding::artifact` exposes the three parts; the facade hands them over.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Placement {
    /// Every placed thing, in draw order.
    ///
    /// Draw order is address order within a level and level order across levels. No
    /// z-index: `hyper-ui` arrives at the same order structurally, by depth-first
    /// pre-order over children, and it is the one part of that file that needed no
    /// argument.
    pub placed: Vec<Placed>,
    /// The embedding of each visited space, in address order.
    ///
    /// One per **space**, never one per thing (spec §6.1). This is the map that makes
    /// a pan O(1): change the root's transform and every descendant follows rigidly,
    /// which is `infinitedb-spatial-layer.md`'s scene-graph invariance used as it was
    /// written.
    pub spaces: BTreeMap<Addr, Transform>,
    /// The revision this placement is valid through — D25's watermark.
    pub through: Revision,
    /// The shallowest address at which the surface ran out of precision, if any.
    ///
    /// Reported as a fact, not as prose. The facade turns it into a finding with a
    /// site, a `said`, a `wanted` and a `remedy` (`COMPOSITOR.md` §6); this layer does
    /// not define a second `Finding` type, because two structures with one name is
    /// R17's failure and `Addr` is already carrying enough of that (O13).
    ///
    /// `None` is *"the screen can still tell these apart"*. Note what it must never
    /// mean: `hyper-ui`'s `cull_nodes_from_infinite_db` maps a database error to
    /// `Vec::new()`, so a failed query and an empty viewport are indistinguishable and
    /// nothing is logged. A condition the person should be told about must never be
    /// rendered as ordinary emptiness.
    pub precision_floor: Option<Addr>,
}
