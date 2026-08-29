//! User-authored apps — a bump-to-total link runs the stored increment definition.

use crate::editor::addresses;
use crate::editor::tags;
use crate::facade::Store;

/// Encodes the bump/total pair the increment graph reads (E13.7 / E18b).
pub fn encode_app_link(bump: &[u8], total: &[u8]) -> Vec<u8> {
    let mut link = Vec::with_capacity(4 + bump.len() + total.len());
    link.extend_from_slice(&(bump.len() as u16).to_le_bytes());
    link.extend_from_slice(bump);
    link.extend_from_slice(&(total.len() as u16).to_le_bytes());
    link.extend_from_slice(total);
    link
}

fn decode_link(link: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    if link.len() < 4 {
        return None;
    }
    let bn = u16::from_le_bytes(link[0..2].try_into().ok()?) as usize;
    if link.len() < 2 + bn + 2 {
        return None;
    }
    let bump = link[2..2 + bn].to_vec();
    let rest = &link[2 + bn..];
    let tn = u16::from_le_bytes(rest[0..2].try_into().ok()?) as usize;
    if rest.len() < 2 + tn {
        return None;
    }
    let total = rest[2..2 + tn].to_vec();
    Some((bump, total))
}

/// Runs the stored increment definition when the bump block is clicked.
pub fn try_run(store: &Store) {
    let Some(link) = store.stored_at(addresses::app_link_key()) else {
        return;
    };
    let Some((bump, total)) = decode_link(&link) else {
        return;
    };
    let Some(pos) = store.pending_at(addresses::POINTER_POSITION.as_bytes()) else {
        return;
    };
    let Some(hit) = store.probe_at(
        f64::from_le_bytes(pos[0..8].try_into().unwrap_or([0; 8])),
        f64::from_le_bytes(pos[8..16].try_into().unwrap_or([0; 8])),
    ) else {
        return;
    };
    if hit != bump && !hit.starts_with(&bump) {
        return;
    }
    let total_payload = crate::facade::payload_key(&total);
    store.write_slot(
        addresses::increment_read_key(),
        "addr",
        &total_payload,
        tags::ADDRESS,
    );
    store.write_slot(
        addresses::increment_map_key(),
        "fn",
        b"increment-text",
        tags::VALUE,
    );
    store.write_slot(
        addresses::increment_map_key(),
        "aux",
        &[],
        tags::VALUE,
    );
    store.write_slot(
        addresses::increment_amend_key(),
        "addr",
        &total_payload,
        tags::ADDRESS,
    );
    store.write_slot(
        addresses::increment_commit_key(),
        "addr",
        &total_payload,
        tags::ADDRESS,
    );
    store.run_at(addresses::increment_def_key());
}
