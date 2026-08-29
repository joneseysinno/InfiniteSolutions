//! Authoring sugar: nested [`Spec`] flattens to addressed records (E16 / O34).
//!
//! Nesting at authoring time does not make the runtime a tree — containment is
//! address prefix only after flatten. Shape payload is a second put at
//! [`crate::facade::payload_key`] (E17 / O26).

use crate::editor::mint::{bits_of, child, slot_for_name};
use crate::facade::{encode_space, payload_key, SpaceRecord};

/// Transient nested authoring value. Flattened by [`flatten`].
#[derive(Clone)]
pub struct Spec {
    /// Local name (documentation / debugging).
    pub name: String,
    /// Explicit child slot under the parent (`1..=0xFFFF`). Prefer this over hashing
    /// when the address must match a well-known key.
    pub slot: u32,
    /// Space payload (no `link` / `text` columns — E17).
    pub record: SpaceRecord,
    /// Per-shape payload bytes (text run, link endpoints). Not an `IS1` record.
    pub shape_payload: Option<Vec<u8>>,
    /// Nested children (authoring sugar only).
    pub children: Vec<Spec>,
}

impl Spec {
    /// A leaf space with an explicit slot.
    pub fn leaf(name: impl Into<String>, slot: u32, record: SpaceRecord) -> Self {
        Self {
            name: name.into(),
            slot,
            record,
            shape_payload: None,
            children: Vec::new(),
        }
    }

    /// Attach a shape payload (text or encoded link).
    pub fn with_payload(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.shape_payload = Some(bytes.into());
        self
    }

    /// Attach children.
    pub fn with_children(mut self, children: Vec<Spec>) -> Self {
        self.children = children;
        self
    }
}

/// One flattened put: address + encoded payload.
#[derive(Debug, Clone)]
pub struct FlatPut {
    /// Store key.
    pub key: Vec<u8>,
    /// Encoded [`SpaceRecord`] bytes, or raw shape payload.
    pub payload: Vec<u8>,
}

/// Flatten a nested Spec under `parent` into addressed puts. No containment field.
pub fn flatten(parent: &[u8], specs: &[Spec]) -> Vec<FlatPut> {
    let mut out = Vec::new();
    flatten_into(parent, specs, &mut out);
    out
}

fn flatten_into(parent: &[u8], specs: &[Spec], out: &mut Vec<FlatPut>) {
    for spec in specs {
        let slot = if spec.slot == 0 {
            slot_for_name(&spec.name)
        } else {
            spec.slot
        };
        let (key, _) = child(parent, bits_of(parent), slot)
            .unwrap_or_else(|| panic!("flatten: slot {slot} under {parent:02x?}"));
        out.push(FlatPut {
            key: key.clone(),
            payload: encode_space(&spec.record),
        });
        if let Some(shape) = &spec.shape_payload {
            out.push(FlatPut {
                key: payload_key(&key),
                payload: shape.clone(),
            });
        }
        if !spec.children.is_empty() {
            flatten_into(&key, &spec.children, out);
        }
    }
}

/// Convenience builders for common space shapes (not a widget toolkit).
pub mod build {
    use super::Spec;
    use crate::facade::{encode_link_payload, SpaceRecord};

    /// An empty area space.
    pub fn area(
        name: impl Into<String>,
        slot: u32,
        across: [f64; 3],
        down: [f64; 3],
        origin: [f64; 2],
        hosts: bool,
    ) -> Spec {
        Spec::leaf(name, slot, bare(across, down, "plain", origin, hosts, true, ""))
    }

    /// A panel-like host space (canvas style).
    pub fn panel(
        name: impl Into<String>,
        slot: u32,
        across: [f64; 3],
        down: [f64; 3],
        origin: [f64; 2],
    ) -> Spec {
        Spec::leaf(
            name,
            slot,
            bare(across, down, "canvas", origin, true, false, ""),
        )
    }

    /// An accepting text field (E19). Same shape as [`text_run`], `accepts` set.
    pub fn field(
        name: impl Into<String>,
        slot: u32,
        across: [f64; 3],
        down: [f64; 3],
        origin: [f64; 2],
        text: impl Into<String>,
    ) -> Spec {
        let text = text.into();
        Spec::leaf(
            name,
            slot,
            bare(across, down, "plain", origin, false, true, "text"),
        )
        .with_payload(text.into_bytes())
    }

    /// A text run. Payload is the run bytes, not a record field.
    pub fn text_run(
        name: impl Into<String>,
        slot: u32,
        across: [f64; 3],
        down: [f64; 3],
        origin: [f64; 2],
        text: impl Into<String>,
    ) -> Spec {
        let text = text.into();
        Spec::leaf(
            name,
            slot,
            bare(across, down, "plain", origin, false, false, "text"),
        )
        .with_payload(text.into_bytes())
    }

    /// A wire/link between two addresses.
    pub fn link_wire(
        name: impl Into<String>,
        slot: u32,
        from: Vec<u8>,
        to: Vec<u8>,
        stroke: f64,
    ) -> Spec {
        Spec::leaf(
            name,
            slot,
            bare(
                [stroke, stroke, 0.0],
                [stroke, stroke, 0.0],
                "wire",
                [0.0, 0.0],
                false,
                false,
                "wire",
            ),
        )
        .with_payload(encode_link_payload(&from, &to))
    }

    fn bare(
        across: [f64; 3],
        down: [f64; 3],
        style: &str,
        origin: [f64; 2],
        hosts: bool,
        accepts: bool,
        primitive: &str,
    ) -> SpaceRecord {
        SpaceRecord {
            across,
            down,
            style: style.into(),
            detail_override: None,
            hosts_space: hosts,
            accepts,
            origin,
            primitive: primitive.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::addresses;

    #[test]
    fn nested_spec_flattens_to_prefix_containment() {
        let parent = addresses::canvas_key();
        let tree = vec![Spec::leaf(
            "a",
            1,
            SpaceRecord {
                across: [0.4, 0.4, 0.0],
                down: [0.4, 0.4, 0.0],
                style: "plain".into(),
                detail_override: None,
                hosts_space: true,
                accepts: true,
                origin: [0.0, 0.0],
                primitive: String::new(),
            },
        )
        .with_children(vec![build::area(
            "a1",
            1,
            [0.15, 0.15, 0.0],
            [0.15, 0.15, 0.0],
            [0.05, 0.05],
            false,
        )])];
        let flat = flatten(parent, &tree);
        assert_eq!(flat.len(), 2);
        assert_eq!(flat[0].key, addresses::node_a_key());
        assert_eq!(flat[1].key, addresses::node_a1_key());
        assert!(flat[1].key.starts_with(&flat[0].key));
    }
}
