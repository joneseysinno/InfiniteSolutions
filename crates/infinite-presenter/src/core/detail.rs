//! [`detail`] — the level a space is actually drawn at (spec §7).

/// Half-level of log-zoom. Promote at `bound + SLOP`, demote at `previous_bound - SLOP`,
/// so each boundary is a dead band of `2 * SLOP` (spec §7.3).
const SLOP: f64 = 0.25;

/// Resolves how much detail one space gets: the view's zoom, the space's override,
/// and the clamp the surface imposes.
///
/// `previous` is the level this space was drawn at last time. All hysteresis state
/// is in the arguments — which is why a replay of the same sweep is bit-identical.
pub fn detail(
    zoom: f64,
    override_steps: Option<i64>,
    floor: u32,
    previous: Option<u32>,
) -> u32 {
    let log = zoom_log(zoom) + override_steps.unwrap_or(0) as f64;
    let log = log.max(0.0);
    let naive = (log.floor() as u32).min(floor);
    let Some(prev) = previous else {
        return naive;
    };
    let prev = prev.min(floor);
    if naive > prev {
        if log >= lower_bound(naive) + SLOP {
            naive
        } else {
            prev
        }
    } else if naive < prev {
        if log < lower_bound(prev) - SLOP {
            naive
        } else {
            prev
        }
    } else {
        naive
    }
}

fn zoom_log(zoom: f64) -> f64 {
    if !zoom.is_finite() || zoom <= 0.0 {
        0.0
    } else {
        zoom.log2().max(0.0)
    }
}

fn lower_bound(level: u32) -> f64 {
    f64::from(level)
}

#[cfg(test)]
mod tests {
    use super::detail;

    #[test]
    fn no_previous_follows_the_naive_level() {
        assert_eq!(detail(4.0, None, 32, None), 2);
    }

    #[test]
    fn a_negative_override_holds_the_space_closed() {
        assert_eq!(detail(16.0, Some(-8), 32, None), 0);
    }

    #[test]
    fn the_floor_clamps() {
        assert_eq!(detail(1e9, None, 3, None), 3);
    }

    #[test]
    fn slop_delays_a_promote() {
        let just_inside = 2f64.powf(2.0 + 0.1);
        assert_eq!(detail(just_inside, None, 32, Some(1)), 1);
        let past = 2f64.powf(2.0 + 0.26);
        assert_eq!(detail(past, None, 32, Some(1)), 2);
    }
}
