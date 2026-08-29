//! [`Blocks`] — compositor. The string-keyed native block registry.
//!
//! Populated at startup by the editor. Stored as a vec of pairs, not a map keyed
//! by anything but an address (L5). O10: this is where *"may this composition use
//! that block"* would be inserted.

use std::sync::Arc;

use infinite_compositor::binding::ports::{Blocks as Port, Primitive};
use infinite_compositor::core::{
    Block, Body, BodyKind, DefinitionSet, Direction, Port as CPort, Signature, Tag, Value,
};
use infinite_presenter::core::{probe, Point};

use crate::editor::blocks::{
    amend as amend_fn, commit as commit_fn, gate as gate_fn, probe_at as probe_at_fn,
    read as read_fn,
};
use crate::facade::addr::{compositor_addr, runtime_addr};
use crate::facade::open::Inner;
use crate::facade::ports::pure_fn;

/// The editor's native blocks, registered under string keys.
pub struct Blocks {
    entries: Vec<(Box<str>, Signature, Arc<dyn Primitive>)>,
}

impl Blocks {
    pub(crate) fn new(inner: Arc<Inner>) -> Self {
        let mut entries: Vec<(Box<str>, Signature, Arc<dyn Primitive>)> = Vec::new();
        for (key, sig) in native_signatures() {
            let primitive: Arc<dyn Primitive> = match key {
                "probe-at" => Arc::new(ProbeAt {
                    inner: Arc::clone(&inner),
                }),
                "read" => Arc::new(Read {
                    inner: Arc::clone(&inner),
                }),
                "amend" => Arc::new(Amend {
                    inner: Arc::clone(&inner),
                }),
                "commit" => Arc::new(Commit {
                    inner: Arc::clone(&inner),
                }),
                "gate" => Arc::new(Gate),
                "map" => Arc::new(Map),
                "fold" => Arc::new(Fold),
                _ => Arc::new(Idle),
            };
            entries.push((key.into(), sig, primitive));
        }
        Self { entries }
    }

    pub(crate) fn hoist(&self) -> infinite_compositor::binding::Tier0 {
        infinite_compositor::binding::Tier0::new(
            self.entries
                .iter()
                .map(|(k, sig, p)| {
                    (
                        k.clone(),
                        Arc::clone(p),
                        sig.outputs().map(|port| port.name.clone()).collect(),
                    )
                })
                .collect(),
        )
    }
}

impl Port for Blocks {
    fn signature(&self, key: &str) -> Option<Signature> {
        self.entries
            .iter()
            .find(|(k, _, _)| &**k == key)
            .map(|(_, s, _)| s.clone())
    }

    fn primitive(&self, key: &str) -> Option<&dyn Primitive> {
        self.entries
            .iter()
            .find(|(k, _, _)| &**k == key)
            .map(|(_, _, p)| p.as_ref())
    }
}

/// Adds the native registrations to a definition set so `link` can resolve them.
pub(crate) fn inject_natives(set: &mut DefinitionSet) {
    for (key, signature) in native_signatures() {
        let at = compositor_addr(key.as_bytes());
        set.blocks.insert(
            at.clone(),
            Block {
                signature,
                body: Body {
                    kind: BodyKind::new(BodyKind::NATIVE),
                    target: at,
                },
            },
        );
    }
}

/// Declared live natives (E18a). Byte strings so Rule 1 counts the registry as a
/// use site, not only the graphs that name a key.
const LIVE_NATIVE: &[&[u8]] = &[
    b"probe-at",
    b"read",
    b"amend",
    b"commit",
    b"gate",
    b"map",
    b"fold",
];

fn native_signatures() -> Vec<(&'static str, Signature)> {
    let pairs = vec![
        (
            "probe-at",
            sig(&[
                ("at", true, "point", false),
                ("hit", false, "address", true),
            ]),
        ),
        (
            "read",
            sig(&[
                ("addr", true, "address", true),
                ("val", false, "value", false),
            ]),
        ),
        (
            "amend",
            sig(&[
                ("addr", true, "address", true),
                ("val", true, "value", false),
                ("pending", false, "flag", false),
            ]),
        ),
        (
            "commit",
            sig(&[
                ("addr", true, "address", true),
                ("done", false, "flag", false),
            ]),
        ),
        (
            "gate",
            sig(&[
                ("val", true, "value", false),
                ("on", true, "flag", false),
                ("pass", false, "value", false),
            ]),
        ),
        (
            "map",
            sig(&[
                ("fn", true, "value", true),
                ("val", true, "value", true),
                ("aux", true, "value", false),
                ("out", false, "value", false),
            ]),
        ),
        (
            "fold",
            sig(&[
                ("fn", true, "value", true),
                ("left", true, "value", true),
                ("right", true, "value", true),
                ("out", false, "value", false),
            ]),
        ),
    ];
    assert!(
        pairs
            .iter()
            .zip(LIVE_NATIVE)
            .all(|((key, _), bytes)| key.as_bytes() == *bytes)
            && pairs.len() == LIVE_NATIVE.len()
    );
    pairs
}

fn sig(ports: &[(&str, bool, &str, bool)]) -> Signature {
    Signature {
        ports: ports
            .iter()
            .map(|(name, incoming, tag, required)| {
                let dir = if *incoming {
                    Direction::In
                } else {
                    Direction::Out
                };
                let mut port = CPort::new(*name, dir, Tag::new(*tag));
                port.required = *required;
                port
            })
            .collect(),
    }
}

struct Idle;

impl Primitive for Idle {
    fn invoke(&self, _inputs: &[Value]) -> Vec<Value> {
        Vec::new()
    }
}

struct ProbeAt {
    inner: Arc<Inner>,
}

impl Primitive for ProbeAt {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let Some(point) = inputs.first().and_then(|v| probe_at_fn(v.payload())) else {
            return vec![Value::new(Tag::new("address"), Vec::new())];
        };
        let x = f64::from_le_bytes(point[0..8].try_into().unwrap_or([0; 8]));
        let y = f64::from_le_bytes(point[8..16].try_into().unwrap_or([0; 8]));
        let hit = {
            let placed = self.inner.last_placement.lock().expect("placement lock");
            placed
                .as_ref()
                .and_then(|p| probe(p, Point::new(x, y)))
                .map(|h| h.at.as_bytes().to_vec())
                .unwrap_or_default()
        };
        vec![Value::new(Tag::new("address"), hit)]
    }
}

struct Read {
    inner: Arc<Inner>,
}

impl Primitive for Read {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let Some(addr) = inputs.first().map(|v| read_fn(v.payload())) else {
            return vec![Value::new(Tag::new("value"), Vec::new())];
        };
        if addr.is_empty() {
            return vec![Value::new(Tag::new("value"), Vec::new())];
        }
        let origin = runtime_addr(&addr);
        let pending = {
            let mut driver = self.inner.driver.lock().expect("driver lock");
            let found = driver
                .pending()
                .list()
                .filter(|e| e.origin() == &origin)
                .last()
                .map(|e| e.payload().to_vec());
            found
        };
        if let Some(payload) = pending {
            return vec![Value::new(Tag::new("value"), payload)];
        }
        let payload = self
            .inner
            .current_value(&addr)
            .unwrap_or_default();
        vec![Value::new(Tag::new("value"), payload)]
    }
}

struct Amend {
    inner: Arc<Inner>,
}

impl Primitive for Amend {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let addr = inputs.first().map(Value::payload).unwrap_or(&[]);
        let val = inputs.get(1).map(Value::payload).unwrap_or(&[]);
        let (addr, val) = amend_fn(addr, val);
        if addr.is_empty() || val.is_empty() {
            return vec![Value::new(Tag::new("flag"), vec![0])];
        }
        Inner::amend_pending(&self.inner, &addr, &val);
        vec![Value::new(Tag::new("flag"), vec![1])]
    }
}

struct Commit {
    inner: Arc<Inner>,
}

impl Primitive for Commit {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let addr = inputs
            .first()
            .map(|v| commit_fn(v.payload()))
            .unwrap_or_default();
        if addr.is_empty() {
            return vec![Value::new(Tag::new("flag"), vec![0])];
        }
        let ok = Inner::commit_pending(&self.inner, &addr);
        vec![Value::new(Tag::new("flag"), vec![u8::from(ok)])]
    }
}

struct Gate;

impl Primitive for Gate {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let val = inputs.first().map(Value::payload).unwrap_or(&[]);
        let on = inputs.get(1).map(Value::payload).unwrap_or(&[]);
        match gate_fn(val, on) {
            Some(pass) => vec![Value::new(Tag::new("value"), pass)],
            None => vec![Value::new(Tag::new("value"), Vec::new())],
        }
    }
}

struct Map;

impl Primitive for Map {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let key = std::str::from_utf8(inputs.first().map(Value::payload).unwrap_or(&[]))
            .unwrap_or("");
        let val = inputs.get(1).map(Value::payload).unwrap_or(&[]);
        let aux = inputs.get(2).map(Value::payload).unwrap_or(&[]);
        vec![Value::new(Tag::new("value"), pure_fn::apply(key, val, aux))]
    }
}

struct Fold;

impl Primitive for Fold {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let key = std::str::from_utf8(inputs.first().map(Value::payload).unwrap_or(&[]))
            .unwrap_or("");
        let left = inputs.get(1).map(Value::payload).unwrap_or(&[]);
        let right = inputs.get(2).map(Value::payload).unwrap_or(&[]);
        vec![Value::new(
            Tag::new("value"),
            pure_fn::fold_apply(key, left, right),
        )]
    }
}
