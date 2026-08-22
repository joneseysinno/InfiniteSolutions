//! [`Provenance`] — compositor. Records what was computed from what (D11).

use std::sync::Arc;

use infinite_compositor::binding::ports::Provenance as Port;
use infinite_compositor::core::Addr;

use crate::facade::addr::{compositor_addr, runtime_addr};
use crate::facade::open::Inner;

/// Provenance over the real store. Held by the facade, keyed by address (D38).
pub struct Provenance {
    pub(crate) inner: Arc<Inner>,
}

impl Port for Provenance {
    fn record(&mut self, outputs: &[Addr], inputs: &[Addr], _block: &Addr) {
        let rev = self.inner.db.revision().legacy_sequence();
        let mut lineage = self.inner.lineage.lock().expect("lineage lock");
        let mut dirty = self.inner.dirty.lock().expect("dirty lock");
        for output in outputs {
            lineage.insert(
                runtime_addr(output.as_bytes()),
                inputs.iter().map(|i| runtime_addr(i.as_bytes())).collect(),
            );
            dirty.push((runtime_addr(output.as_bytes()), rev));
        }
    }

    fn inputs_of(&self, output: &Addr) -> Vec<Addr> {
        self.inner
            .lineage
            .lock()
            .expect("lineage lock")
            .get(&runtime_addr(output.as_bytes()))
            .map(|ins| ins.iter().map(|a| compositor_addr(a.as_bytes())).collect())
            .unwrap_or_default()
    }
}

impl Provenance {
    /// Outputs whose declared input set contains `input`. The S6 query.
    pub fn downstream_of(&self, input: &Addr) -> Vec<Addr> {
        let lineage = self.inner.lineage.lock().expect("lineage lock");
        lineage
            .iter()
            .filter(|(_, ins)| ins.iter().any(|i| i.as_bytes() == input.as_bytes()))
            .map(|(o, _)| compositor_addr(o.as_bytes()))
            .collect()
    }
}
