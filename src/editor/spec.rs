//! Authoring sugar: nested [`Spec`] flattens to addressed records (E16 / O34).
//!
//! Nesting at authoring time does not make the runtime a tree — containment is
//! address prefix only after flatten.

use crate::editor::mint::{bits_of, child, slot_for_name};
use crate::facade::{encode_space, SpaceRecord};

/// Transient nested authoring value. Flattened by [`flatten`].
#[derive(Clone)]
pub struct Spec {
    /// Local name (documentation / debugging).
    pub name: String,
    /// Explicit child slot under the parent (`1..=0xFFFF`). Prefer this over hashing
    /// when the address must match a well-known key.
    pub slot: u32,
    /// Space payload.
    pub record: SpaceRecord,
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
            children: Vec::new(),
        }
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
    /// Encoded [`SpaceRecord`] bytes.
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
        if !spec.children.is_empty() {
            flatten_into(&key, &spec.children, out);
        }
    }
}

/// Convenience builders for common space shapes (not a widget toolkit).
pub mod build {
    use super::Spec;
    use crate::facade::SpaceRecord;

    /// An empty area space.
    pub fn area(
        name: impl Into<String>,
        slot: u32,
        across: [f64; 3],
        down: [f64; 3],
        origin: [f64; 2],
        hosts: bool,
    ) -> Spec {
        Spec::leaf(
            name,
            slot,
            SpaceRecord {
                across,
                down,
                style: "plain".into(),
                detail_override: None,
                hosts_space: hosts,
                accepts: true,
                origin,
                primitive: String::new(),
                link: None,
                text: String::new(),
            },
        )
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
            SpaceRecord {
                across,
                down,
                style: "canvas".into(),
                detail_override: None,
                hosts_space: true,
                accepts: false,
                origin,
                primitive: String::new(),
                link: None,
                text: String::new(),
            },
        )
    }

    /// A text run.
    pub fn text_run(
        name: impl Into<String>,
        slot: u32,
        across: [f64; 3],
        down: [f64; 3],
        origin: [f64; 2],
        text: impl Into<String>,
    ) -> Spec {
        Spec::leaf(
            name,
            slot,
            SpaceRecord {
                across,
                down,
                style: "plain".into(),
                detail_override: None,
                hosts_space: false,
                accepts: false,
                origin,
                primitive: "text".into(),
                link: None,
                text: text.into(),
            },
        )
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
            SpaceRecord {
                across: [stroke, stroke, 0.0],
                down: [stroke, stroke, 0.0],
                style: "wire".into(),
                detail_override: None,
                hosts_space: false,
                accepts: false,
                origin: [0.0, 0.0],
                primitive: "wire".into(),
                link: Some((from, to)),
                text: String::new(),
            },
        )
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
                link: None,
                text: String::new(),
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
