//! The hysteresis sweep (S5).

use infinite_presenter::core::detail;

fn sweep(up: bool) -> Vec<(f64, u32)> {
    let mut zoom = if up { 1.0 } else { 32.0 };
    let end = if up { 32.0 } else { 1.0 };
    let step = if up { 1.02 } else { 1.0 / 1.02 };
    let mut previous = None;
    let mut out = Vec::new();
    let mut guard = 0;
    while guard < 10_000 {
        let level = detail(zoom, None, 16, previous);
        if previous != Some(level) {
            out.push((zoom, level));
        }
        previous = Some(level);
        if up && zoom >= end {
            break;
        }
        if !up && zoom <= end {
            break;
        }
        zoom *= step;
        guard += 1;
    }
    out
}

#[test]
fn detail_settles_on_both_edges_of_every_boundary() {
    let up = sweep(true);
    let down = sweep(false);

    let up_levels: Vec<u32> = up.iter().map(|(_, l)| *l).collect();
    let down_levels: Vec<u32> = down.iter().map(|(_, l)| *l).collect();

    assert!(up_levels.len() > 2, "the up-sweep never left level 0");
    for w in up_levels.windows(2) {
        assert_eq!(w[1], w[0] + 1, "a boundary was skipped or crossed twice going up");
    }
    for w in down_levels.windows(2) {
        assert_eq!(w[1] + 1, w[0], "a boundary was skipped or crossed twice going down");
    }

    let mut previous = None;
    let mut last = 0u32;
    let mut zoom = 1.0;
    for _ in 0..200 {
        let level = detail(zoom, None, 16, previous);
        if previous == Some(level) {
            // still inside a dead band: one more tiny step must not skip a level
            assert!(level == last || level == last + 1 || last + 1 == level);
        }
        last = level;
        previous = Some(level);
        zoom *= 1.02;
        if zoom > 32.0 {
            break;
        }
    }

    let up_again = sweep(true);
    assert_eq!(up, up_again, "a replay of the sweep must be identical");
}
