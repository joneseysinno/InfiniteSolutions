//! Placeable and style records, as bytes the store holds.
//!
//! Genesis writes these; [`Scene`](super::ports::Scene) reads them. The layout is
//! fixed so a re-seed is bit-identical (E4).

/// One authored space, before the presenter sees it.
pub struct SpaceRecord {
    /// Extent across: min, ideal, weight.
    pub across: [f64; 3],
    /// Extent down: min, ideal, weight.
    pub down: [f64; 3],
    /// Opaque style key.
    pub style: String,
    /// Authored detail override, if any.
    pub detail_override: Option<i64>,
    /// Whether this space has an interior.
    pub hosts_space: bool,
    /// Whether a probe may land here.
    pub accepts: bool,
    /// Authored origin, across then down. Drag writes this (EDITOR.md §1).
    pub origin: [f64; 2],
    /// Opaque primitive key — what shape draws this (D46). Empty means `rect`.
    pub primitive: String,
    /// The two addresses this connects, when it is a link rather than an area (D46).
    pub link: Option<(Vec<u8>, Vec<u8>)>,
    /// The run to draw when `primitive` is `text` (E13.0, O26 option a).
    pub text: String,
}

const SPACE_MAGIC: &[u8] = b"IS1";
const STYLE_MAGIC: &[u8] = b"ST1";
const COMP_MAGIC: &[u8] = b"CM1";
const CAMERA_MAGIC: &[u8] = b"CA1";
const SELECTION_MAGIC: &[u8] = b"SL1";
const NONE: i64 = i64::MIN;

/// One port, as genesis writes it. Direction is a flag so the editor never
/// names the compositor's `Direction` (R2, D35).
pub struct PortRecord {
    /// Unique within the block.
    pub name: String,
    /// True when the block reads here.
    pub incoming: bool,
    /// Opaque tag (D13).
    pub tag: String,
    /// `None` is unbounded.
    pub arity: Option<u32>,
    /// Whether a missing internal wire is an unsatisfied import.
    pub required: bool,
}

/// One block instance inside a composition.
pub struct BlockRecord {
    /// The instance address.
    pub at: Vec<u8>,
    /// Body kind key (`native`, `composed`, …).
    pub kind: String,
    /// Native key bytes, or the delegated address.
    pub target: Vec<u8>,
    /// Declared ports.
    pub ports: Vec<PortRecord>,
}

/// One wire, n-ary on both ends.
pub struct WireRecord {
    /// Source ports.
    pub sources: Vec<(Vec<u8>, String)>,
    /// Sink ports.
    pub sinks: Vec<(Vec<u8>, String)>,
}

/// The editor's behaviour composition, as store bytes.
pub struct CompositionRecord {
    /// D19: a true value that reads outside its inputs is `not-pure`.
    pub compilable: bool,
    /// Instances, in address order once encoded.
    pub blocks: Vec<BlockRecord>,
    /// Authored wires (D22).
    pub wires: Vec<WireRecord>,
}

/// Encodes one space. Deterministic.
pub fn encode_space(record: &SpaceRecord) -> Vec<u8> {
    let mut out = Vec::from(SPACE_MAGIC);
    for n in record.across.iter().chain(record.down.iter()) {
        out.extend_from_slice(&n.to_le_bytes());
    }
    let style = record.style.as_bytes();
    out.extend_from_slice(&(style.len() as u16).to_le_bytes());
    out.extend_from_slice(style);
    let detail = record.detail_override.unwrap_or(NONE);
    out.extend_from_slice(&detail.to_le_bytes());
    out.push(u8::from(record.hosts_space));
    out.push(u8::from(record.accepts));
    for n in record.origin {
        out.extend_from_slice(&n.to_le_bytes());
    }
    // Appended after `origin`, and read back only if present, for the same reason
    // `origin` itself was: a record written by an earlier genesis still decodes, and
    // the fields it predates take their documented default. The layout is still fixed
    // — it only grows at the end — so a re-seed is bit-identical (E4).
    put_str(&mut out, &record.primitive);
    match &record.link {
        None => out.push(0),
        Some((from, to)) => {
            out.push(1);
            put_bytes(&mut out, from);
            put_bytes(&mut out, to);
        }
    }
    put_str(&mut out, &record.text);
    out
}

/// Decodes one space. `None` if the payload is not a space record.
pub fn decode_space(bytes: &[u8]) -> Option<SpaceRecord> {
    if bytes.len() < 3 + 48 + 2 || !bytes.starts_with(SPACE_MAGIC) {
        return None;
    }
    let mut i = 3;
    let mut take_f64 = || {
        let n = f64::from_le_bytes(bytes.get(i..i + 8)?.try_into().ok()?);
        i += 8;
        Some(n)
    };
    let across = [take_f64()?, take_f64()?, take_f64()?];
    let down = [take_f64()?, take_f64()?, take_f64()?];
    let slen = u16::from_le_bytes(bytes.get(i..i + 2)?.try_into().ok()?) as usize;
    i += 2;
    let style = std::str::from_utf8(bytes.get(i..i + slen)?).ok()?.to_string();
    i += slen;
    let detail = i64::from_le_bytes(bytes.get(i..i + 8)?.try_into().ok()?);
    i += 8;
    let hosts_space = *bytes.get(i)? != 0;
    let accepts = *bytes.get(i + 1)? != 0;
    i += 2;
    let origin = if bytes.len() >= i + 16 {
        i += 16;
        [
            f64::from_le_bytes(bytes.get(i - 16..i - 8)?.try_into().ok()?),
            f64::from_le_bytes(bytes.get(i - 8..i)?.try_into().ok()?),
        ]
    } else {
        [0.0, 0.0]
    };
    let (primitive, link, text) = match take_str(bytes, i) {
        Some((primitive, mut n)) => {
            let link = match bytes.get(n) {
                Some(1) => {
                    let (from, next) = take_bytes(bytes, n + 1)?;
                    let (to, next) = take_bytes(bytes, next)?;
                    n = next;
                    Some((from, to))
                }
                Some(0) => {
                    n += 1;
                    None
                }
                _ => None,
            };
            let text = take_str(bytes, n).map(|(t, _)| t).unwrap_or_default();
            (primitive, link, text)
        }
        None => (String::new(), None, String::new()),
    };
    Some(SpaceRecord {
        across,
        down,
        style,
        detail_override: if detail == NONE { None } else { Some(detail) },
        hosts_space,
        accepts,
        origin,
        primitive,
        link,
        text,
    })
}

/// Encodes one style row: its name, and a fill as four unit intervals.
///
/// **The name is part of the record** (D44). A space carries an opaque style *key*
/// and the table is addressed by *address*; without the name in the row there is
/// nothing to join them on, and the facade would have to know the editor's
/// addresses to resolve a colour — which is R2 backwards.
pub fn encode_style(name: &str, fill: [f64; 4]) -> Vec<u8> {
    let mut out = Vec::from(STYLE_MAGIC);
    for n in fill {
        out.extend_from_slice(&n.to_le_bytes());
    }
    put_str(&mut out, name);
    out
}

/// Decodes one style row as `(name, fill)`. `None` if the payload is not one.
pub fn decode_style(bytes: &[u8]) -> Option<(String, [f64; 4])> {
    if bytes.len() < 3 + 32 || !bytes.starts_with(STYLE_MAGIC) {
        return None;
    }
    let fill = [
        f64::from_le_bytes(bytes[3..11].try_into().ok()?),
        f64::from_le_bytes(bytes[11..19].try_into().ok()?),
        f64::from_le_bytes(bytes[19..27].try_into().ok()?),
        f64::from_le_bytes(bytes[27..35].try_into().ok()?),
    ];
    let (name, _) = take_str(bytes, 35)?;
    Some((name, fill))
}

/// Encodes the session camera: centre (x, y), then zoom (E10.5).
///
/// Plain `f64`s, not `Camera` — this file stays a byte codec and does not name the
/// presenter's core types, matching [`SpaceRecord`]'s split from `Placeable`/`Point`.
/// The caller (`facade::present`) converts.
pub fn encode_camera(centre_x: f64, centre_y: f64, zoom: f64) -> Vec<u8> {
    let mut out = Vec::from(CAMERA_MAGIC);
    for n in [centre_x, centre_y, zoom] {
        out.extend_from_slice(&n.to_le_bytes());
    }
    out
}

/// Decodes the session camera as `(centre_x, centre_y, zoom)`. `None` if the payload
/// is not a camera record.
pub fn decode_camera(bytes: &[u8]) -> Option<(f64, f64, f64)> {
    if bytes.len() < 3 + 24 || !bytes.starts_with(CAMERA_MAGIC) {
        return None;
    }
    let x = f64::from_le_bytes(bytes[3..11].try_into().ok()?);
    let y = f64::from_le_bytes(bytes[11..19].try_into().ok()?);
    let zoom = f64::from_le_bytes(bytes[19..27].try_into().ok()?);
    Some((x, y, zoom))
}

/// Encodes authored selection as the store key bytes of the selected space (E13.1).
pub fn encode_selection(selected: &[u8]) -> Vec<u8> {
    let mut out = Vec::from(SELECTION_MAGIC);
    put_bytes(&mut out, selected);
    out
}

/// Decodes a selection record. `None` if not `SL1`. Some(None) if empty (no selection).
pub fn decode_selection(bytes: &[u8]) -> Option<Option<Vec<u8>>> {
    if bytes.len() < 3 || !bytes.starts_with(SELECTION_MAGIC) {
        return None;
    }
    let (key, _) = take_bytes(bytes, 3)?;
    Some(if key.is_empty() { None } else { Some(key) })
}

/// Encodes one composition. Deterministic.
pub fn encode_composition(record: &CompositionRecord) -> Vec<u8> {
    let mut out = Vec::from(COMP_MAGIC);
    out.push(u8::from(record.compilable));
    out.extend_from_slice(&(record.blocks.len() as u16).to_le_bytes());
    for block in &record.blocks {
        put_bytes(&mut out, &block.at);
        put_str(&mut out, &block.kind);
        put_bytes(&mut out, &block.target);
        out.extend_from_slice(&(block.ports.len() as u16).to_le_bytes());
        for port in &block.ports {
            put_str(&mut out, &port.name);
            out.push(u8::from(port.incoming));
            put_str(&mut out, &port.tag);
            out.extend_from_slice(&port.arity.unwrap_or(u32::MAX).to_le_bytes());
            out.push(u8::from(port.required));
        }
    }
    out.extend_from_slice(&(record.wires.len() as u16).to_le_bytes());
    for wire in &record.wires {
        out.extend_from_slice(&(wire.sources.len() as u16).to_le_bytes());
        for (block, port) in &wire.sources {
            put_bytes(&mut out, block);
            put_str(&mut out, port);
        }
        out.extend_from_slice(&(wire.sinks.len() as u16).to_le_bytes());
        for (block, port) in &wire.sinks {
            put_bytes(&mut out, block);
            put_str(&mut out, port);
        }
    }
    out
}

/// Decodes a composition into the compositor's types. `None` if not `CM1`.
pub fn decode_composition(bytes: &[u8]) -> Option<infinite_compositor::core::Composition> {
    if bytes.len() < 6 || !bytes.starts_with(COMP_MAGIC) {
        return None;
    }
    let mut i = 3;
    let compilable = *bytes.get(i)? != 0;
    i += 1;
    let nblocks = u16::from_le_bytes(bytes.get(i..i + 2)?.try_into().ok()?) as usize;
    i += 2;
    let mut blocks = std::collections::BTreeMap::new();
    for _ in 0..nblocks {
        let (at, n) = take_bytes(bytes, i)?;
        i = n;
        let (kind, n) = take_str(bytes, i)?;
        i = n;
        let (target, n) = take_bytes(bytes, i)?;
        i = n;
        let nports = u16::from_le_bytes(bytes.get(i..i + 2)?.try_into().ok()?) as usize;
        i += 2;
        let mut ports = Vec::new();
        for _ in 0..nports {
            let (name, n) = take_str(bytes, i)?;
            i = n;
            let incoming = *bytes.get(i)? != 0;
            i += 1;
            let (tag, n) = take_str(bytes, i)?;
            i = n;
            let arity_raw = u32::from_le_bytes(bytes.get(i..i + 4)?.try_into().ok()?);
            i += 4;
            let required = *bytes.get(i)? != 0;
            i += 1;
            let dir = if incoming {
                infinite_compositor::core::Direction::In
            } else {
                infinite_compositor::core::Direction::Out
            };
            let mut port = infinite_compositor::core::Port::new(
                name,
                dir,
                infinite_compositor::core::Tag::new(tag),
            );
            port.arity = if arity_raw == u32::MAX {
                None
            } else {
                Some(arity_raw)
            };
            port.required = required;
            ports.push(port);
        }
        let at_addr = crate::facade::compositor_addr(&at);
        blocks.insert(
            at_addr,
            infinite_compositor::core::Block {
                signature: infinite_compositor::core::Signature { ports },
                body: infinite_compositor::core::Body {
                    kind: infinite_compositor::core::BodyKind::new(kind),
                    target: crate::facade::compositor_addr(&target),
                },
            },
        );
    }
    let nwires = u16::from_le_bytes(bytes.get(i..i + 2)?.try_into().ok()?) as usize;
    i += 2;
    let mut wires = Vec::new();
    for _ in 0..nwires {
        let (sources, n) = take_ends(bytes, i)?;
        i = n;
        let (sinks, n) = take_ends(bytes, i)?;
        i = n;
        wires.push(infinite_compositor::core::Wire { sources, sinks });
    }
    Some(infinite_compositor::core::Composition {
        blocks,
        wires,
        compilable,
    })
}

fn put_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
    out.extend_from_slice(bytes);
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    put_bytes(out, s.as_bytes());
}

fn take_bytes(bytes: &[u8], i: usize) -> Option<(Vec<u8>, usize)> {
    let n = u16::from_le_bytes(bytes.get(i..i + 2)?.try_into().ok()?) as usize;
    let start = i + 2;
    Some((bytes.get(start..start + n)?.to_vec(), start + n))
}

fn take_str(bytes: &[u8], i: usize) -> Option<(String, usize)> {
    let (raw, n) = take_bytes(bytes, i)?;
    Some((String::from_utf8(raw).ok()?, n))
}

fn take_ends(
    bytes: &[u8],
    mut i: usize,
) -> Option<(Vec<infinite_compositor::core::PortRef>, usize)> {
    let n = u16::from_le_bytes(bytes.get(i..i + 2)?.try_into().ok()?) as usize;
    i += 2;
    let mut ends = Vec::new();
    for _ in 0..n {
        let (block, next) = take_bytes(bytes, i)?;
        i = next;
        let (port, next) = take_str(bytes, i)?;
        i = next;
        ends.push(infinite_compositor::core::PortRef {
            block: crate::facade::compositor_addr(&block),
            port: port.into(),
        });
    }
    Some((ends, i))
}
