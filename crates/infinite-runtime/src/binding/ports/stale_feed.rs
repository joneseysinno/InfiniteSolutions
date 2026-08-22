//! [`StaleFeed`] — how change arrives.

use crate::core::{Addr, Revision};

/// Reports which addresses have gone stale.
///
/// The store already has the machinery and it has never been driven: the derivation
/// bus, watermarks, `check_hyperedge_freshness`, and `query_stale_downstream` (D11).
/// This port is the runtime's end of it.
///
/// # Why this makes B5 impossible
///
/// An external writer — the MCP server writing definitions — is D6's second rejection
/// ground: a live projection must be *told*, or it is silently wrong, which makes the
/// watch loop a correctness dependency. Here a change arrives as **staleness**, so a
/// missed notification costs responsiveness and never correctness. The next read simply
/// sees it.
pub trait StaleFeed {
    /// Addresses that went stale after `watermark`, with the revision at which each
    /// did.
    ///
    /// This is also the frontier's rebuild rule (spec §5.1), which is what makes the
    /// frontier pass R12's discard test: drop it, call this with the last durable
    /// watermark, obtain an identical set.
    fn stale_since(&self, watermark: Revision) -> Vec<(Addr, Revision)>;

    /// The newest revision this feed has observed.
    fn watermark(&self) -> Revision;
}
