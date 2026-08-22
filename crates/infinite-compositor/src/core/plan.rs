//! [`Plan`] — the linked form, and this layer's third word.

use crate::core::addr::Addr;
use crate::core::port::PortRef;

/// One step of a plan.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Step {
    /// The block to invoke.
    pub block: Addr,
    /// Native registry key. Empty when this step is a region (inner plan).
    ///
    /// `interpret` looks the primitive up by this string (R4). Link copies it from
    /// the body's target so execution does not need the definition set.
    pub key: Box<str>,
    /// Where each input comes from, resolved. An unbound port is a self-ref, so
    /// the declared input set is exactly this list.
    pub inputs: Vec<PortRef>,
    /// Where each output goes, resolved.
    pub outputs: Vec<PortRef>,
    /// A region carries its own inner plan and is still one step from outside (D21).
    ///
    /// This is also the shape a *yielding* region would need, which is why O12 stays
    /// open at no cost: closing it later adds no new scheduler concept. The runtime's
    /// `Outcome::work_remains` is the other half of the same door.
    pub inner: Option<Plan>,
}

/// A deterministic order of steps with sources and sinks resolved.
///
/// > **The compositor decides what runs and in what order. The runtime decides when
/// > and how much.**
///
/// Already built once: `bion`'s pure library emits an `ExecutionPlan` and an external
/// runtime owns the clock and the threads. This is D25's core/binding split across a
/// layer boundary for the third time — the presenter owns the function and the runtime
/// owns the schedule; the compositor owns the function and the runtime owns the
/// schedule.
///
/// A plan is **derived** (D8, R12) and registers with the runtime's `ArtifactRegistry`
/// (D25), which is why this layer needs no cache machinery of its own: the generic
/// discard harness drops and rebuilds a plan without knowing what one is.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Plan {
    /// In execution order. Deterministic, because `Composition` iterates by address.
    pub steps: Vec<Step>,
}
