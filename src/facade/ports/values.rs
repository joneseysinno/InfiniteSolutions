//! [`Values`] — compositor. Read an input, write an output.
//!
//! Port slots live in an address-keyed overlay. Store writes go through the
//! pending path, never straight to the queue (D24).

use std::sync::Arc;

use infinite_compositor::binding::ports::Values as Port;
use infinite_compositor::core::{Addr, Tag, Value};

use crate::facade::addr::runtime_addr;
use crate::facade::open::Inner;

/// Values over the real store, via pending and an address-keyed slot overlay.
pub struct Values {
    pub(crate) inner: Arc<Inner>,
}

impl Port for Values {
    fn read(&self, at: &Addr) -> Option<Value> {
        let origin = runtime_addr(at.as_bytes());
        {
            let slots = self.inner.slots.lock().expect("slots lock");
            if let Some((tag, payload)) = slots.get(&origin) {
                return Some(Value::new(Tag::new(tag.as_str()), payload.clone()));
            }
        }
        let payload = {
            let mut driver = self.inner.driver.lock().expect("driver lock");
            let found = driver
                .pending()
                .list()
                .find(|e| e.origin() == &origin)
                .map(|e| e.payload().to_vec());
            found
        };
        if let Some(payload) = payload {
            return Some(Value::new(Tag::new("value"), payload));
        }
        match self.inner.current_value(at.as_bytes()) {
            Some(payload) => Some(Value::new(Tag::new("value"), payload)),
            None => None,
        }
    }

    fn write(&mut self, at: &Addr, value: Value) {
        let origin = runtime_addr(at.as_bytes());
        self.inner.slots.lock().expect("slots lock").insert(
            origin,
            (value.tag().label().to_string(), value.payload().to_vec()),
        );
    }
}
