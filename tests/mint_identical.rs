//! E15.3 — same seed path → byte-identical addresses on two machines.

use infinite_solutions::editor::addresses;
use infinite_solutions::editor::mint::{self, MintSeed};
use infinite_solutions::facade::open;

#[test]
fn same_seed_yields_identical_addresses_on_two_stores() {
    let d1 = tempfile::tempdir().expect("t1");
    let d2 = tempfile::tempdir().expect("t2");
    let s1 = open(d1.path()).expect("open1");
    let s2 = open(d2.path()).expect("open2");
    s1.set_mint_seed(MintSeed::at(1));
    s2.set_mint_seed(MintSeed::at(1));
    let parent = addresses::canvas_key();
    let mut a = Vec::new();
    let mut b = Vec::new();
    for _ in 0..32 {
        a.push(s1.mint_under(parent).expect("m1"));
        b.push(s2.mint_under(parent).expect("m2"));
    }
    assert_eq!(a, b);
}

#[test]
fn named_slots_are_store_independent() {
    let parent = addresses::canvas_key();
    let slot = mint::slot_for_name("node-a");
    let (k1, _) = mint::child(parent, mint::bits_of(parent), slot).unwrap();
    let (k2, _) = mint::child(parent, mint::bits_of(parent), slot).unwrap();
    assert_eq!(k1, k2);
}
