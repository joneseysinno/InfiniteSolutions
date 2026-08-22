//! [`coalesce`] — one in-flight commit per address (D24.3).

use std::collections::{BTreeMap, BTreeSet};

use super::{Addr, Pending, Seq};

/// What a coalescing pass decided.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Coalesced {
    /// Entries to submit, oldest first. At most one per address.
    pub submit: Vec<Seq>,
    /// Entries a newer value for the same address has replaced. They are settled
    /// without ever reaching the store — not lost work, because the value that
    /// replaced them is the one the person meant.
    pub superseded: Vec<Seq>,
}

/// Chooses what to submit from the committed-but-unsent entries.
///
/// Two rules, and nothing else:
///
/// 1. **At most one commit in flight per address.** An address already in flight is
///    skipped entirely this pass.
/// 2. **A newer value supersedes an unsent older one** rather than queueing behind it.
///    This is what keeps the pending set bounded under sustained backpressure: a person
///    typing into one field for a minute produces one submission, not a minute of them.
///
/// Together these are why store backpressure degrades *durability latency* rather than
/// *input latency* (D24). Nothing here can block, and nothing here touches a store.
pub fn coalesce<'a>(
    committed: impl Iterator<Item = &'a Pending>,
    in_flight: &BTreeSet<Addr>,
) -> Coalesced {
    // Newest committed entry per address wins; the rest are superseded.
    let mut newest: BTreeMap<&Addr, Seq> = BTreeMap::new();
    let mut out = Coalesced::default();

    for entry in committed {
        debug_assert!(
            entry.is_committed(),
            "coalesce takes committed entries only"
        );
        if in_flight.contains(entry.origin()) {
            continue;
        }
        if let Some(older) = newest.insert(entry.origin(), entry.seq()) {
            out.superseded.push(older);
        }
    }

    out.submit = newest.into_values().collect();
    out.submit.sort_unstable();
    out.superseded.sort_unstable();
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{coalesce, Addr};
    use crate::core::PendingSet;

    fn addr(b: u8) -> Addr {
        Addr::new(vec![b])
    }

    fn committed_set(entries: &[(u8, &[u8])]) -> PendingSet {
        let mut p = PendingSet::new(64);
        for (a, payload) in entries {
            let seq = p.open(addr(*a), payload.to_vec()).unwrap();
            p.commit(seq);
        }
        p
    }

    #[test]
    fn newest_per_address_is_submitted_and_the_rest_superseded() {
        let p = committed_set(&[(1, b"a"), (1, b"b"), (1, b"c")]);
        let out = coalesce(p.committed(), &BTreeSet::new());
        assert_eq!(out.submit.len(), 1);
        assert_eq!(out.superseded.len(), 2);
        assert_eq!(out.submit[0].get(), 2, "the newest, not the oldest");
    }

    #[test]
    fn an_address_already_in_flight_is_skipped_entirely() {
        let p = committed_set(&[(1, b"a"), (2, b"b")]);
        let in_flight = BTreeSet::from([addr(1)]);
        let out = coalesce(p.committed(), &in_flight);
        assert_eq!(out.submit.len(), 1);
        assert!(
            out.superseded.is_empty(),
            "a skipped address supersedes nothing"
        );
    }

    #[test]
    fn distinct_addresses_all_submit() {
        let p = committed_set(&[(1, b"a"), (2, b"b"), (3, b"c")]);
        let out = coalesce(p.committed(), &BTreeSet::new());
        assert_eq!(out.submit.len(), 3);
        assert!(out.superseded.is_empty());
    }

    #[test]
    fn nothing_committed_means_nothing_to_do() {
        let p = PendingSet::new(8);
        assert_eq!(
            coalesce(p.committed(), &BTreeSet::new()),
            Default::default()
        );
    }
}
