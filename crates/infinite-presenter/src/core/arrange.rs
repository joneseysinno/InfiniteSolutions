//! [`arrange`] — one axis, allocated over extents (spec §7).

use crate::core::extent::Extent;

/// Distributes `available` across `wants`, returning one size per entry.
///
/// **No graph, no store, no surface, and no identity.** That is the whole point of the
/// signature: a slice of extents in, a vector of sizes out, so the layout algorithms
/// are a pure core depending on nothing, exactly as D15 and R3 require.
///
/// Order of fill: floor (`min`), then up to `ideal`, then surplus by `weight`.
/// Overflow is the caller's: sizes never sum to more than `available`.
pub fn arrange(wants: &[Extent], available: f64) -> Vec<f64> {
    let n = wants.len();
    let mut out = vec![0.0; n];
    if n == 0 || !(available > 0.0) {
        return out;
    }

    let mins: Vec<f64> = wants.iter().map(|e| e.min.max(0.0)).collect();
    let sum_min: f64 = mins.iter().sum();
    if available <= sum_min {
        if sum_min <= 0.0 {
            return out;
        }
        let scale = available / sum_min;
        for (i, m) in mins.iter().enumerate() {
            out[i] = *m * scale;
        }
        return out;
    }

    out.clone_from(&mins);
    let mut rest = available - sum_min;

    let room: Vec<f64> = wants
        .iter()
        .enumerate()
        .map(|(i, e)| (e.ideal.max(e.min) - out[i]).max(0.0))
        .collect();
    let sum_room: f64 = room.iter().sum();
    if sum_room > 0.0 {
        if rest >= sum_room {
            for i in 0..n {
                out[i] += room[i];
            }
            rest -= sum_room;
        } else {
            let scale = rest / sum_room;
            for i in 0..n {
                out[i] += room[i] * scale;
            }
            rest = 0.0;
        }
    }

    if rest > 0.0 {
        let sum_w: f64 = wants.iter().map(|e| e.weight.max(0.0)).sum();
        if sum_w > 0.0 {
            for (i, e) in wants.iter().enumerate() {
                out[i] += rest * (e.weight.max(0.0) / sum_w);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::arrange;
    use crate::core::Extent;

    #[test]
    fn fixed_extents_take_exactly_their_size() {
        let sizes = arrange(&[Extent::fixed(10.0), Extent::fixed(20.0)], 100.0);
        assert_eq!(sizes, vec![10.0, 20.0]);
    }

    #[test]
    fn surplus_goes_to_weight() {
        let a = Extent::new(0.0, 10.0, 1.0);
        let b = Extent::new(0.0, 10.0, 3.0);
        let sizes = arrange(&[a, b], 40.0);
        assert!((sizes[0] - 15.0).abs() < 1e-12);
        assert!((sizes[1] - 25.0).abs() < 1e-12);
    }

    #[test]
    fn overflow_never_exceeds_available() {
        let sizes = arrange(&[Extent::fixed(30.0), Extent::fixed(30.0)], 20.0);
        let sum: f64 = sizes.iter().sum();
        assert!((sum - 20.0).abs() < 1e-12);
    }
}
