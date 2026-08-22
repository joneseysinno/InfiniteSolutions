//! [`priority`] — the total order the scheduler works down.

use super::{Addr, Revision};

/// Scheduling priority. Higher sorts first.
///
/// Two components, and deliberately no third (R27):
///
/// - `focus_bits` — bits of address prefix shared with the focused address. This is
///   B6: zoom must not rebuild a distant space ahead of the one under the cursor.
/// - `arrived_at` — the revision at which the entry went stale. Older first.
///
/// `arrived_at` exists to make the order **total**, not because age is interesting.
/// A partial order would leave the schedule dependent on iteration order, and D19's
/// equivalence law needs a deterministic schedule to be checkable at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Priority {
    focus_bits: u32,
    arrived_at: Revision,
}

impl Priority {
    /// Bits of prefix shared with the focused address.
    pub const fn focus_bits(&self) -> u32 {
        self.focus_bits
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Nearer to focus first; then older first. Reversing `arrived_at` makes an
        // older (smaller) revision compare as *greater* priority.
        self.focus_bits
            .cmp(&other.focus_bits)
            .then_with(|| other.arrived_at.cmp(&self.arrived_at))
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Priority of `target` given the current `focus` and the revision it went stale at.
///
/// With no focus every entry scores zero on the first component and the order falls
/// back to arrival — still total, which is the property that matters.
pub fn priority(focus: Option<&Addr>, target: &Addr, arrived_at: Revision) -> Priority {
    Priority {
        focus_bits: focus.map_or(0, |f| f.shared_prefix_bits(target)),
        arrived_at,
    }
}

#[cfg(test)]
mod tests {
    use super::{priority, Addr, Revision};

    #[test]
    fn nearer_to_focus_outranks_farther() {
        let focus = Addr::new(vec![0xAB, 0xCD]);
        let near = Addr::new(vec![0xAB, 0xCE]);
        let far = Addr::new(vec![0x00, 0x00]);
        assert!(
            priority(Some(&focus), &near, Revision::new(9))
                > priority(Some(&focus), &far, Revision::new(1))
        );
    }

    #[test]
    fn at_equal_distance_older_wins() {
        let focus = Addr::new(vec![0xAB]);
        let a = Addr::new(vec![0xAB, 0x01]);
        let b = Addr::new(vec![0xAB, 0x02]);
        assert!(
            priority(Some(&focus), &a, Revision::new(1))
                > priority(Some(&focus), &b, Revision::new(2))
        );
    }

    #[test]
    fn order_is_total_without_a_focus() {
        let a = Addr::new(vec![0x01]);
        let b = Addr::new(vec![0x02]);
        let pa = priority(None, &a, Revision::new(1));
        let pb = priority(None, &b, Revision::new(2));
        assert!(pa > pb);
        assert_ne!(pa, pb);
    }
}
