//! [`Frontier`] — addresses known stale and not yet recomputed.

use std::collections::BTreeMap;

use super::{priority, Addr, Revision};

/// The staleness frontier: a declared derived artifact (D6, spec §5.1).
///
/// **Rebuild rule:** `StaleFeed::stale_since(last_durable_watermark)`.
/// **Discard test:** drop it, re-query from the watermark, obtain an identical set.
///
/// It holds addresses and revisions only — never records (R11).
#[derive(Clone, Debug, Default)]
pub struct Frontier {
    stale: BTreeMap<Addr, Revision>,
    focus: Option<Addr>,
}

impl Frontier {
    /// An empty frontier with no focus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks `addr` stale as of `at`.
    ///
    /// Re-marking keeps the **earlier** revision. The frontier records when something
    /// first went stale, so that a stream of edits to one address cannot starve
    /// anything behind it by continually refreshing its arrival.
    pub fn mark(&mut self, addr: Addr, at: Revision) {
        self.stale
            .entry(addr)
            .and_modify(|first| {
                if at < *first {
                    *first = at;
                }
            })
            .or_insert(at);
    }

    /// Sets the focused address, which reorders the frontier (spec §5.1).
    pub fn focus_on(&mut self, addr: Option<Addr>) {
        self.focus = addr;
    }

    /// The highest-priority stale address, removed.
    ///
    /// Linear in the frontier size. Deliberate for now: the scheduler's shape is what
    /// this stage is establishing, and a heap keyed on priority is invalidated by every
    /// focus change. The replacement is a measurement, not a guess — S6's test bed is
    /// where it gets one.
    pub fn take_next(&mut self) -> Option<(Addr, Revision)> {
        let best = self
            .stale
            .iter()
            .max_by(|(a_addr, a_rev), (b_addr, b_rev)| {
                priority(self.focus.as_ref(), a_addr, **a_rev).cmp(&priority(
                    self.focus.as_ref(),
                    b_addr,
                    **b_rev,
                ))
            })
            .map(|(addr, rev)| (addr.clone(), *rev))?;
        self.stale.remove(&best.0);
        Some(best)
    }

    /// How many addresses are stale.
    pub fn len(&self) -> usize {
        self.stale.len()
    }

    /// Whether anything is stale.
    pub fn is_empty(&self) -> bool {
        self.stale.is_empty()
    }

    /// Drops everything. The discard half of R12's test.
    pub fn discard(&mut self) {
        self.stale.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{Addr, Frontier, Revision};

    #[test]
    fn takes_the_entry_nearest_focus_first() {
        let mut f = Frontier::new();
        f.mark(Addr::new(vec![0x00, 0x00]), Revision::new(1));
        f.mark(Addr::new(vec![0xAB, 0xCE]), Revision::new(5));
        f.focus_on(Some(Addr::new(vec![0xAB, 0xCD])));
        assert_eq!(f.take_next().unwrap().0, Addr::new(vec![0xAB, 0xCE]));
    }

    #[test]
    fn remarking_keeps_the_earlier_arrival() {
        let mut f = Frontier::new();
        let a = Addr::new(vec![0x01]);
        f.mark(a.clone(), Revision::new(7));
        f.mark(a.clone(), Revision::new(9));
        assert_eq!(f.take_next().unwrap().1, Revision::new(7));
    }

    #[test]
    fn drains_to_empty() {
        let mut f = Frontier::new();
        f.mark(Addr::new(vec![0x01]), Revision::new(1));
        f.mark(Addr::new(vec![0x02]), Revision::new(2));
        assert_eq!(f.len(), 2);
        while f.take_next().is_some() {}
        assert!(f.is_empty());
    }
}
