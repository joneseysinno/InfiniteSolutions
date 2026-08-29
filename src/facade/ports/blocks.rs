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
    amend as amend_fn, commit as commit_fn, displace as displace_fn,
    encode_selection as encode_selection_fn, gate as gate_fn,
    increment_text as increment_text_fn, offset as offset_fn, probe_at as probe_at_fn,
    read as read_fn,
};
use crate::facade::addr::{compositor_addr, runtime_addr};
use crate::facade::open::Inner;
use crate::facade::{decode_space, encode_space};

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
                "offset" => Arc::new(Offset),
                "gate" => Arc::new(Gate),
                "encode-selection" => Arc::new(EncodeSelection),
                "encode-wire" => Arc::new(EncodeWire),
                "displace" => Arc::new(Displace),
                "set-origin" => Arc::new(SetOrigin),
                "increment-text" => Arc::new(IncrementText),
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

fn native_signatures() -> Vec<(&'static str, Signature)> {
    vec![
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
            "offset",
            sig(&[
                ("from", true, "point", false),
                ("to", true, "point", false),
                ("delta", false, "point", false),
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
            "encode-selection",
            sig(&[
                ("hit", true, "address", true),
                ("out", false, "value", false),
            ]),
        ),
        (
            "encode-wire",
            sig(&[
                ("from", true, "address", false),
                ("to", true, "address", false),
                ("out", false, "value", false),
            ]),
        ),
        (
            "displace",
            sig(&[
                ("record", true, "value", true),
                ("delta", true, "point", true),
                ("out", false, "value", false),
            ]),
        ),
        (
            "set-origin",
            sig(&[
                ("record", true, "value", true),
                ("origin", true, "point", false),
                ("out", false, "value", false),
            ]),
        ),
        (
            "increment-text",
            sig(&[
                ("val", true, "value", true),
                ("out", false, "value", false),
            ]),
        ),
    ]
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
        let end = {
            let mut c = Inner::coord(&addr);
            c = c.saturating_add(1);
            Inner::bytes_of(c)
        };
        let at_rev = self.inner.db.stable_revision().legacy_sequence();
        let payload = match self.inner.records_in_range(&addr, &end, at_rev) {
            Ok(mut rows) => rows.pop().map(|(_, p)| p).unwrap_or_default(),
            Err(e) => panic!("value read failed (not a missing value): {e}"),
        };
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

struct Offset;

impl Primitive for Offset {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let from = inputs.first().map(Value::payload).unwrap_or(&[]);
        let to = inputs.get(1).map(Value::payload).unwrap_or(&[]);
        vec![Value::new(Tag::new("point"), offset_fn(from, to))]
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

struct EncodeSelection;

impl Primitive for EncodeSelection {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let hit = inputs.first().map(Value::payload).unwrap_or(&[]);
        vec![Value::new(Tag::new("value"), encode_selection_fn(hit))]
    }
}

struct EncodeWire;

impl Primitive for EncodeWire {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let from = inputs.first().map(Value::payload).unwrap_or(&[]);
        let to = inputs.get(1).map(Value::payload).unwrap_or(&[]);
        if from.is_empty() || to.is_empty() {
            return vec![Value::new(Tag::new("value"), Vec::new())];
        }
        let record = super::super::record::SpaceRecord {
            across: [0.012, 0.012, 0.0],
            down: [0.012, 0.012, 0.0],
            style: "wire".into(),
            detail_override: None,
            hosts_space: false,
            accepts: false,
            origin: [0.0, 0.0],
            primitive: "wire".into(),
            link: Some((from.to_vec(), to.to_vec())),
            text: String::new(),
        };
        vec![Value::new(
            Tag::new("value"),
            encode_space(&record),
        )]
    }
}

struct Displace;

impl Primitive for Displace {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let origin = inputs.first().map(Value::payload).unwrap_or(&[]);
        let delta = inputs.get(1).map(Value::payload).unwrap_or(&[]);
        let Some(mut space) = decode_space(origin) else {
            return vec![Value::new(Tag::new("value"), origin.to_vec())];
        };
        let next = displace_fn(
            &{
                let mut b = Vec::with_capacity(16);
                b.extend_from_slice(&space.origin[0].to_le_bytes());
                b.extend_from_slice(&space.origin[1].to_le_bytes());
                b
            },
            delta,
        );
        if next.len() >= 16 {
            space.origin[0] = f64::from_le_bytes(next[0..8].try_into().unwrap_or([0; 8]));
            space.origin[1] = f64::from_le_bytes(next[8..16].try_into().unwrap_or([0; 8]));
        }
        vec![Value::new(Tag::new("value"), encode_space(&space))]
    }
}

struct SetOrigin;

impl Primitive for SetOrigin {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let record = inputs.first().map(Value::payload).unwrap_or(&[]);
        let origin = inputs.get(1).map(Value::payload).unwrap_or(&[]);
        let Some(mut space) = decode_space(record) else {
            return vec![Value::new(Tag::new("value"), record.to_vec())];
        };
        if origin.len() >= 16 {
            space.origin[0] = f64::from_le_bytes(origin[0..8].try_into().unwrap_or([0; 8]));
            space.origin[1] = f64::from_le_bytes(origin[8..16].try_into().unwrap_or([0; 8]));
        }
        vec![Value::new(Tag::new("value"), encode_space(&space))]
    }
}

struct IncrementText;

impl Primitive for IncrementText {
    fn invoke(&self, inputs: &[Value]) -> Vec<Value> {
        let val = inputs.first().map(Value::payload).unwrap_or(&[]);
        vec![Value::new(
            Tag::new("value"),
            increment_text_fn(val),
        )]
    }
}
