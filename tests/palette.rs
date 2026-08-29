//! E13.4 — drag a block from the palette; the editor mints a child address.

use infinite_solutions::editor;
use infinite_solutions::editor::addresses;
use infinite_solutions::editor::mint;
use infinite_solutions::facade::{self, decode_space, significant_bits};

fn point(x: f64, y: f64) -> Vec<u8> {
    let mut p = Vec::with_capacity(16);
    p.extend_from_slice(&x.to_le_bytes());
    p.extend_from_slice(&y.to_le_bytes());
    p
}

fn seeded() -> (tempfile::TempDir, facade::Store) {
    let dir = tempfile::TempDir::new().unwrap();
    let store = facade::open(dir.path()).expect("open store");
    editor::seed(|k| store.has(k), |k, v| store.put(k, v));
    editor::bind(&store);
    store.set_surface(0.0, 0.0, 800.0, 600.0, 1.0);
    let _ = store.place_now();
    (dir, store)
}

fn drain(store: &facade::Store) {
    for _ in 0..32 {
        if store.committed_len() == 0 {
            break;
        }
        store.tick();
    }
    store.sync().expect("sync");
}

fn drag_palette_to(store: &facade::Store, drop_x: f64, drop_y: f64) -> Vec<u8> {
    let placement = store.place_now();
    let item = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::PALETTE_PLAIN_KEY)
        .expect("palette template is placed");
    let start_x = (item.rect.min.x + item.rect.max.x) * 0.5;
    let start_y = (item.rect.min.y + item.rect.max.y) * 0.5;
    let minted = mint::next_child(store, addresses::CANVAS_KEY).expect("mint before drop");

    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(start_x, start_y));
    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[1]);
    editor::run(store);

    store.amend(addresses::POINTER_POSITION.as_bytes(), &point(drop_x, drop_y));
    editor::run(store);

    store.amend(addresses::POINTER_BUTTON.as_bytes(), &[0]);
    store.amend(addresses::RELEASE_PULSE_KEY, &[1]);
    editor::run(store);

    store.commit_at(&minted);
    drain(store);
    minted
}

#[test]
fn dragging_a_palette_block_mints_a_child_under_the_canvas() {
    let (_dir, store) = seeded();
    let placement = store.place_now();
    let canvas = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::CANVAS_KEY)
        .expect("canvas is placed");
    let drop_x = canvas.rect.min.x + (canvas.rect.max.x - canvas.rect.min.x) * 0.6;
    let drop_y = canvas.rect.min.y + (canvas.rect.max.y - canvas.rect.min.y) * 0.6;

    let minted = drag_palette_to(&store, drop_x, drop_y);

    assert_eq!(
        minted,
        vec![0x11, 0x40, 0x00, 0x00],
        "the fourth canvas sibling is minted after A, B, and the wire"
    );
    assert_eq!(
        significant_bits(&minted),
        significant_bits(addresses::CANVAS_KEY) + 4,
        "the new block sits one level under the canvas"
    );
    assert_eq!(
        mint::parent_key(&minted),
        addresses::CANVAS_KEY.to_vec(),
        "the address names the canvas as parent"
    );

    let space = decode_space(&store.stored_at(&minted).expect("placed block stored")).expect("IS1");
    assert_eq!(space.style, "plain");
    assert!(store.has(&minted));
}

#[test]
fn a_palette_block_survives_restart() {
    let (dir, store) = seeded();
    let placement = store.place_now();
    let canvas = placement
        .placed
        .iter()
        .find(|p| p.at.as_bytes() == addresses::CANVAS_KEY)
        .expect("canvas is placed");
    let drop_x = canvas.rect.min.x + (canvas.rect.max.x - canvas.rect.min.x) * 0.6;
    let drop_y = canvas.rect.min.y + (canvas.rect.max.y - canvas.rect.min.y) * 0.6;
    let minted = drag_palette_to(&store, drop_x, drop_y);
    assert!(store.has(&minted), "must persist before reopen");
    drop(store);

    let store = facade::open(dir.path()).expect("reopen");
    editor::bind(&store);
    assert!(
        store.has(&minted),
        "genesis put_if must not overwrite a user-placed block"
    );
    let space = decode_space(&store.stored_at(&minted).expect("stored after reopen")).expect("IS1");
    assert_eq!(space.style, "plain");
}

#[test]
fn mint_skips_existing_siblings() {
    let (_dir, store) = seeded();
    assert_eq!(
        mint::next_child(&store, addresses::CANVAS_KEY).expect("mint"),
        vec![0x11, 0x40, 0x00, 0x00]
    );
}
