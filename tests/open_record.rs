//! E17.2 — a fourth shape touches no SpaceRecord field and no decoder branch.

use infinite_solutions::editor::addresses;
use infinite_solutions::facade::{
    decode_space, encode_space, payload_key, SpaceRecord,
};

#[test]
fn space_record_has_no_link_or_text_fields() {
    let src = include_str!("../src/facade/record.rs");
    let record = src
        .split("pub struct SpaceRecord")
        .nth(1)
        .and_then(|s| s.split('}').next())
        .expect("SpaceRecord body");
    assert!(
        !record.contains("pub link"),
        "link must not be a SpaceRecord field"
    );
    assert!(
        !record.contains("pub text"),
        "text must not be a SpaceRecord field"
    );
}

#[test]
fn fourth_shape_uses_payload_key_only() {
    let space = addresses::canvas_key();
    let record = SpaceRecord {
        across: [0.1, 0.1, 0.0],
        down: [0.1, 0.1, 0.0],
        style: "plain".into(),
        detail_override: None,
        hosts_space: false,
        accepts: true,
        origin: [0.0, 0.0],
        primitive: "mark".into(),
    };
    let encoded = encode_space(&record);
    let decoded = decode_space(&encoded).expect("IS1");
    assert_eq!(decoded.primitive, "mark");
    let mark = b"dot-at-origin";
    assert!(
        decode_space(mark).is_none(),
        "shape payload is not an IS1 space"
    );
    assert_eq!(payload_key(space), addresses::payload_key(space));
    assert_eq!(
        payload_key(space),
        {
            let mut k = space.to_vec();
            k.extend_from_slice(&[0xFF, 0xFF]);
            k
        }
    );
}

#[test]
fn encode_space_has_no_field_per_primitive() {
    let src = include_str!("../src/facade/record.rs");
    let encode = src
        .split("pub fn encode_space")
        .nth(1)
        .and_then(|s| s.split("pub fn decode_space").next())
        .expect("encode_space");
    assert!(
        !encode.contains("record.link") && !encode.contains("record.text"),
        "encode_space must not write link/text columns"
    );
}
