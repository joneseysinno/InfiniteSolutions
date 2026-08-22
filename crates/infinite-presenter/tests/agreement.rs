//! The agreement test (S4) — this layer's equivalent of R12's discard test.
//!
//! > **The transform that culls is the transform that draws. One function, called
//! > twice.**

use infinite_presenter::core::{Camera, Point, SurfaceRect, View};

/// A tiny LCG so this crate's `[dev-dependencies]` stays empty (R3).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.0
    }

    fn f64(&mut self, lo: f64, hi: f64) -> f64 {
        let t = (self.next() >> 11) as f64 / ((1u64 << 53) as f64);
        lo + (hi - lo) * t
    }
}

fn view(rng: &mut Rng) -> View {
    View::new(
        Camera::new(
            Point::new(rng.f64(-200.0, 200.0), rng.f64(-200.0, 200.0)),
            rng.f64(0.25, 8.0),
        ),
        SurfaceRect::new(
            Point::new(rng.f64(1.0, 120.0), rng.f64(1.0, 80.0)),
            Point::new(rng.f64(200.0, 900.0), rng.f64(150.0, 700.0)),
            1.0,
        ),
        rng.f64(0.0, 32.0),
    )
}

#[test]
fn what_is_culled_is_what_is_drawn() {
    let mut rng = Rng(0xC0FFEE);
    for _ in 0..400 {
        let v = view(&mut rng);
        let embedding = v.embedding();
        let seen = infinite_presenter::core::visible(&v);
        let drawn = v.surface.rect().inflate(v.margin);
        for _ in 0..20 {
            let world = Point::new(rng.f64(seen.min.x - 50.0, seen.max.x + 50.0), rng.f64(seen.min.y - 50.0, seen.max.y + 50.0));
            let screen = embedding.apply(world);
            let in_cull = seen.contains(world);
            let in_draw = drawn.contains(screen);
            assert_eq!(
                in_cull, in_draw,
                "cull and draw disagreed at world={world:?} screen={screen:?} origin={:?}",
                v.surface.origin
            );
        }
    }
}
