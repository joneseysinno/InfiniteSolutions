//! [`Backends`] — compositor. The backend registry. Tier 0 after E9.

use std::sync::Arc;

use infinite_compositor::binding::{Backend, Tier0, TIER0_KEY};
use infinite_compositor::binding::ports::Backends as Port;

use crate::facade::open::Inner;
use crate::facade::ports::Blocks;

/// Compiled-form backends. Tier 0 is present because it passed the harness.
pub struct Backends {
    tier0: Tier0,
}

impl Backends {
    pub(crate) fn new(inner: Arc<Inner>) -> Self {
        Self {
            tier0: Blocks::new(inner).hoist(),
        }
    }
}

impl Port for Backends {
    fn backend(&self, key: &str) -> Option<&dyn Backend> {
        if key == TIER0_KEY {
            Some(&self.tier0)
        } else {
            None
        }
    }
}
