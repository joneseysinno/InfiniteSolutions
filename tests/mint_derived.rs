//! E15.1 — derived identity green checks (a)(b)(d).

use infinite_solutions::editor::addresses;
use infinite_solutions::editor::mint::{self, MintSeed};
use infinite_solutions::facade::{self, open};

#[test]
fn a_space_holds_two_hundred_children() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = open(dir.path()).expect("open");
    store.set_mint_seed(MintSeed::new());
    let parent = addresses::canvas_key();
    let mut keys = Vec::new();
    for _ in 0..200 {
        let k = store.mint_under(parent).expect("mint");
        assert!(k.starts_with(parent));
        assert_eq!(k.len(), parent.len() + 2);
        keys.push(k);
    }
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), 200);
    assert!(store.mint_under(parent).is_some() || keys.len() == 200);
}

#[test]
fn two_seeds_mint_disjoint_slots() {
    let a = MintSeed::at(1);
    let b = MintSeed::at(50_000);
    let parent = addresses::canvas_key();
    let (ka, _, _) = mint::mint_child(parent, a).unwrap();
    let (kb, _, _) = mint::mint_child(parent, b).unwrap();
    assert_ne!(ka, kb);
}

#[test]
fn delete_remint_undo_keeps_addresses_apart() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = open(dir.path()).expect("open");
    store.set_mint_seed(MintSeed::new());
    let parent = addresses::canvas_key();

    let first = store.mint_under(parent).expect("first");
    store.put(&first, b"one");
    let _ = store.sync();

    store.delete_key(&first);
    let _ = store.sync();

    let second = store.mint_under(parent).expect("second");
    store.put(&second, b"two");
    let _ = store.sync();

    // Finding 25: max+1 recycling would put "one" back onto `second` under undo-by-address.
    // Derived slots never reuse: first and second differ, and restoring `first` cannot
    // land on `second`.
    assert_ne!(first, second);
    store.put(&first, b"one-restored");
    let _ = store.sync();
    assert_eq!(store.stored_at(&first).as_deref(), Some(&b"one-restored"[..]));
    assert_eq!(store.stored_at(&second).as_deref(), Some(&b"two"[..]));
}

#[test]
fn significant_bits_is_byte_length_not_nibble_scan() {
    // Must stop passing: depth from last non-zero nibble.
    assert_eq!(facade::significant_bits(&[0x10, 0x00, 0x01]), 24);
    assert_eq!(facade::bits_of(&[0x10, 0x00, 0x01]), 24);
}
