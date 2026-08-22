//! The contract a compiled form must satisfy (D28, closes O9).

use crate::core::{Plan, Value};

/// What a backend costs, declared as **data** rather than promised.
///
/// D19 requires compilation be chosen *"on the runtime's evidence rather than the
/// author's guess"*, and the runtime cannot choose without this. The runtime's
/// `Outcome::budget_exhausted` is the other half of that evidence.
///
/// Declarations are self-reported, so a lying backend misroutes. Mitigated
/// structurally rather than by trust: the runtime measures, and **may demote a backend
/// whose measured cost contradicts its declaration**. Demotion is only possible
/// because this is data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cost {
    /// Roughly what it costs to produce an artifact from a plan.
    pub compile: u64,
    /// Roughly what it costs to cross into the artifact, per invocation.
    pub crossing: u64,
}

/// A compiled artifact: bytes plus a way to invoke them.
pub trait Artifact {
    /// Invokes the compiled form.
    ///
    /// **The same signature as interpreted invocation** ([`super::ports::Primitive`]).
    /// That identity is what makes D19's equivalence law testable rather than
    /// aspirational.
    fn invoke(&self, inputs: &[Value]) -> Vec<Value>;

    /// The bytes, for the discard test (R12).
    ///
    /// A compiled artifact is derived state (D19) registered with the runtime (D25),
    /// so the generic harness drops and rebuilds it without knowing what it is — which
    /// is why compilation needs no invalidation machinery of its own.
    fn bytes(&self) -> &[u8];
}

/// A compiled form.
///
/// > **The compiled form is a registered backend under a string key. The compositor
/// > owns the contract a backend must satisfy, and never the backend.**
///
/// D25's shape one layer over: *the runtime knows artifact lifecycle, never artifact
/// content* becomes *the compositor knows compiled-form lifecycle, never the form*.
///
/// **Registration is the gate.** A backend is not registered because someone wrote it;
/// a backend is registered **by passing the equivalence harness** — for every plan in a
/// maintained corpus, run interpreted, run compiled, compare outputs bit-for-bit and
/// provenance edge-for-edge. D19 says the interpreted execution is the specification;
/// `tests/equivalence.rs` makes that sentence executable, and it means the platform
/// cannot grow a subtly-wrong backend. Adding a form in 2027 costs a registration, not
/// a redesign.
///
/// Three tiers, in the order they should be built:
///
/// | Tier | Form | Removes | Costs |
/// |---|---|---|---|
/// | 0 | resolved plan — **no compiler** | lookup and dispatch | nothing |
/// | 1 | native — generate Rust, build it with the toolchain block authors already have | per-edge value boxing | a toolchain at author time; a per-target artifact |
/// | 2 | portable — WASM | — | a call-boundary cost SES may refuse |
///
/// **Tier 0 is built first, and the first compiled form requires no compiler.** It adds
/// no dependency to a crate whose green check is an empty `[dependencies]`, and its
/// equivalence is the easiest of the three to argue — same code, same order, lookups
/// hoisted. So it is also the honest first test of the harness: a backend that *should*
/// be equivalent failing the harness means the harness is wrong, not the backend.
pub trait Backend {
    /// Whether this backend can compile this plan.
    ///
    /// Refusal is reachable: a plan can be compilable in principle and refused by every
    /// registered backend. The author is told so in a finding, rather than left with a
    /// composition that silently stayed interpreted.
    fn accepts(&self, plan: &Plan) -> bool;

    /// Compiles a plan.
    ///
    /// *Tier 0 lands in stage **S7**.*
    fn compile(&self, plan: &Plan) -> Option<Box<dyn Artifact>>;

    /// What it costs. See [`Cost`].
    fn cost(&self) -> Cost;
}
