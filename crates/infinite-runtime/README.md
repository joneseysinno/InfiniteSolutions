# infinite-runtime

The **runtime** layer of Infinite Solutions (D2, D17). It owns *motion in time* —
scheduling, cadence, staleness, priority, and pending state.

> **L1 — owns no thread pool.** Driven, never driving. Cadence in, work out.
> **L2 — owns no storage.** Nothing is authored here.

**The specification is [`docs/specs/RUNTIME.md`](../../docs/specs/RUNTIME.md).** This
file deliberately does not restate it — R17 (one name, one thing) and R21 apply to
documents as much as to types, and two descriptions of one layer is how they drift.

Read before touching this crate: [`docs/RULES.md`](../../docs/RULES.md) and
[`docs/DECISIONS.md`](../../docs/DECISIONS.md) (R24).

## Building

```sh
cargo build -p infinite-runtime                    # the pure core, alone. R3's check.
cargo build -p infinite-runtime --features binding # core + the part that knows a graph
```

The core is `std` but has no dependencies. It is not `no_std`: no named consumer
requires it, and R27 makes unrequired generality a defect.
