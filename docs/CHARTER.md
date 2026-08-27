# Infinite Solutions — Charter

> Draft 1, 2026-08-20. The shortest true statement of what this is.
> Rules: [`RULES.md`](./RULES.md) · Decision record: [`DECISIONS.md`](./DECISIONS.md) ·
> Current status: [`STATUS.md`](./STATUS.md)

---

## What it is

**Infinite Solutions is a visual programming platform for building and deploying production apps, in which frontend, backend, and persistence are one substrate rather than three.**

Spaces subdivide and hyperedges connect them. Every space has a permanent address, and hyperedges carry values between spaces. Because of that, *"this runs on the server"* is a **placement** decision rather than an architectural one — the same graph, placed differently. Collapsing that seam is the point: most of what makes full-stack development hard is the boundaries between the three, not the three themselves.

**A child should be able to build a full stack app with it.** This is a design constraint, not a slogan. It is what rules out a text syntax in the core loop, rules out stack traces as the error surface, and requires that the four layers below are invisible to the person composing.

## Who it is for

Two tiers, and the platform must serve both without either noticing the other:

- **Block authors** write native primitives in Rust and publish them as blocks.
- **App authors** compose blocks visually. They never see Rust.

## The four layers

| Layer | Owns |
|---|---|
| **store** | Addresses, records, nested spaces, revisions, branches. The only writable model. |
| **compositor** | Blocks, ports, wiring, link-time validation, compilation. The static structure of a program. |
| **runtime** | Motion in time — scheduling, transport, staleness, cadence. Owns no storage and no thread pool. |
| **presenter** | The embedding layer: address ↔ screen, culling, level of detail, wgpu. Holds positions, never identity. |

Facades stack on the platform and house domain law. Apps sit on facades.

## Execution

A graph runs the moment it is drawn. There is no build step between composing and seeing it work.

1. **Interpreted — always available.** Every composition executes as drawn. Testing happens here.
2. **Compiled — opt-in, per composition.** Chosen when something is too slow, on the runtime's evidence rather than the author's guess. Many apps never need it.

Two laws govern the seam:

> **Equivalence.** The interpreted execution is the specification. A compiled block must be observationally identical to it. Differentially testable, and exactly so, because execution is deterministic.
>
> **Compiled artifacts are derived.** The graph is the definition. Delete every compiled block and rebuild from the graphs: the system is identical. Compilation is a cache and obeys the discard test.

A composition is compilable only if it is a pure function of its declared inputs — which is the same declaration the store already needs to compute staleness. One discipline, two payoffs.

**Composition closes.** A wired set of blocks is itself a block with ports, compiled or not. This is the property that turns primitives into blocks into whole applications; without it composition stops after one flat layer.

## Multi-platform

A platform boundary is a **portal**. Desktop and its server are two graphs glued at a portal; adding a new target adds a graph and a portal. The author draws an edge and the runtime decides whether it is a function call, an IPC message, or a network round-trip.

## Why it can work at scale

Visual programming fails when the canvas becomes unreadable. **Nested spaces** answer that structurally rather than as an editor feature.

> **We subdivide spaces and connect them with hyperedges. Zoom into a space and you see more detail; zoom out and it becomes a node in the graph.**

A space contains nodes, and a node may itself be a space — the same entity can be
both at once: an object populating its parent, and, if it hosts one, the root of its
own interior. Zoom is how you cross that seam: collapsed, a space renders as a node
in its parent's graph; entered, that node's own space is what you see, populated by
its own nodes. Addresses are permanent, so drilling in never breaks a reference.

Encapsulation, addressing, and navigation therefore come from one mechanism instead of three. And detail is **per space, not per camera**: zoom sets a default, and individual spaces are held open or closed against it, which is how several things stay legible at once.

## Vocabulary

Five words. Everything else is built from them.

| Word | Meaning |
|---|---|
| **space** | The unit. A coordinate region. Carries its own coordinates and a permanent address. |
| **node** | An object populating a space, at an address within it. May itself host its own space. |
| **graph** | What you see at one level: the nodes populating a space, and the hyperedges among them. |
| **hyperedge** | Connects any number of nodes. Carries values between them. |
| **zoom** | Crosses the node/space seam — enters a node's own space, or leaves it. The primary navigation. |

## Where "production" comes from

Not from what the author writes — from the substrate:

| Property | Supplied by |
|---|---|
| Persistence | everything is already in the store |
| Versioning | revisions |
| Rollback | branch merge |
| Audit | computation provenance |
| Observability | the derivation DAG *is* the trace |

All five exist in `infinite-db` today. **Auth is the open one** — no substrate answer yet.

## Named consumers

**SES** (physics), **Coach Assistant** (coaching), **mycelium** (numerical methods), and the structural / AEC work. Each is native blocks plus authored compositions, in whatever proportion suits it. No app is designed until the platform facade is workable.

The platform's own editor is the ultimate forcing consumer: **if the editor cannot be built in the platform, a child cannot build an app in it.**

## Out of scope

Kinds, units, math, domain law, and widgets. Payloads are opaque and their tags are defined by apps; the platform's only operation on a tag is *match*.
