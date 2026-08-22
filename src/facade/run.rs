//! Run the interpreted behaviour composition. The portal never names `interpret`.

use infinite_compositor::binding::interpret;
use infinite_compositor::binding::ports::{Definitions, Provenance as ProvenancePort, Values as ValuesPort};
use infinite_compositor::core::{link, PortRef, Tag, Value};
use infinite_presenter::binding::ports::Surface as SurfacePort;
use infinite_presenter::core::{Camera, Point, View};

use super::addr::{compositor_addr, runtime_revision};
use super::artifacts::{register, register_plan};
use super::open::Store;

impl Store {
    /// Binds the behaviour plan and registers `Placement` and `Plan` (D25).
    pub fn bind_plan(&self, start: &[u8], end: &[u8], root: &[u8]) {
        *self.inner.plan.lock().expect("plan lock") =
            Some((start.to_vec(), end.to_vec(), root.to_vec()));
        let view = View::new(
            Camera::new(Point::new(0.5, 0.5), 400.0),
            SurfacePort::geometry(&self.surface()),
            0.0,
        );
        let mut driver = self.inner.driver.lock().expect("driver lock");
        register(driver.artifacts(), view);
        register_plan(driver.artifacts(), start, end, root);
    }

    /// Writes a port slot so `interpret` can read an unbound input.
    pub fn write_slot(&self, block: &[u8], port: &str, payload: &[u8], tag: &str) {
        let at = PortRef {
            block: compositor_addr(block),
            port: port.into(),
        }
        .slot();
        let mut values = self.values();
        ValuesPort::write(
            &mut values,
            &at,
            Value::new(Tag::new(tag), payload.to_vec()),
        );
    }

    /// Links the bound plan and interprets it.
    pub fn run_linked(&self) {
        let root = {
            let plan = self.inner.plan.lock().expect("plan lock");
            match plan.as_ref() {
                Some((_, _, root)) => root.clone(),
                None => return,
            }
        };
        let defs = self.definitions().resolve(&compositor_addr(&root));
        let linked = link(&defs, &compositor_addr(&root));
        self.inner
            .findings
            .lock()
            .expect("findings lock")
            .extend(linked.findings.iter().cloned());
        if linked.value.steps.is_empty() {
            return;
        }
        let blocks = self.blocks();
        let mut values = self.values();
        let mut provenance = self.provenance();
        let ran = interpret(&linked.value, &blocks, &mut values, &mut provenance);
        self.inner
            .findings
            .lock()
            .expect("findings lock")
            .extend(ran.findings);
    }

    /// The exact declared input set of an executed output (S6).
    pub fn inputs_of(&self, output: &[u8]) -> Vec<Vec<u8>> {
        ProvenancePort::inputs_of(&self.provenance(), &compositor_addr(output))
            .into_iter()
            .map(|a| a.as_bytes().to_vec())
            .collect()
    }

    /// Outputs whose declared input set contains `input`. Identical in form to
    /// [`Self::inputs_of`] inverted — if they disagree, one layer is wrong about
    /// what a dependency is.
    pub fn stale_downstream(&self, input: &[u8]) -> Vec<Vec<u8>> {
        self.provenance()
            .downstream_of(&compositor_addr(input))
            .into_iter()
            .map(|a| a.as_bytes().to_vec())
            .collect()
    }

    /// R12 over every artifact registered on this store's driver.
    pub fn artifacts_pass_discard(&self) -> bool {
        let read = self.store_read();
        let at = runtime_revision(self.inner.db.stable_revision().legacy_sequence());
        let mut driver = self.inner.driver.lock().expect("driver lock");
        driver.artifacts().audit(&read, at).is_empty()
    }
}
