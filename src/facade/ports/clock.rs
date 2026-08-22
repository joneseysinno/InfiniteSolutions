//! [`Clock`] — runtime. `std::time::Instant`.
//!
//! The layers may not name `std::time` (R10). This file may.

use infinite_runtime::binding::ports::Clock as Port;
use infinite_runtime::core::Instant;

/// A monotonic instant source.
pub struct Clock {
    origin: std::time::Instant,
}

impl Clock {
    pub(crate) fn new() -> Self {
        Self {
            origin: std::time::Instant::now(),
        }
    }
}

impl Port for Clock {
    fn now(&self) -> Instant {
        Instant::from_nanos(self.origin.elapsed().as_nanos() as u64)
    }
}
