# Rephrase the goal so that reorganisation becomes visible

**Status: our design argument, not a theorem imported from the book.** Context: the [current experiment programme](../../../briefs/experiment-programme.md) and [structure-and-function lead](../../../leads/structure-and-function.md), at the [pinned source revision](../source-notes.md).

## 1. The object of design

The useful question is: **which organisation of exact decoding makes the required output available sooner, within an acceptable CPU and memory budget?**

The current scanner, workers and in-order reader are one implementation of that requirement. A measured bottleneck tells us where a change would matter in that implementation. It does not make the existing activity boundary necessary. The distinction matters: speeding up an unchanged scanner and arranging for less work to pass through that scanner are different candidates.

For a valid finite input, define completion by the actual output contract: the required bytes have been delivered and the terminal checks relevant to that interface have completed. For corrupt input, preserve the corresponding error and externally observable behaviour. An empty queue is not completion if a worker still owns a block; a full ready map is not useful output if the next required block is missing.

Let a design be described by four choices:

\[
\theta=(\text{representation},\text{work placement},
        \text{admission policy},\text{ownership policy}).
\]

For an input \(w\) and an execution environment \(\xi\), measure wall time \(T_\theta(w,\xi)\), process CPU time and peak owned memory. Initially compare these as separate objectives. A weighted score would require weights justified by the deployment; it should not be invented to make a preferred design win.

**Argued:** the proper first model is a finite completion model with resource constraints. Regulating an occupancy around a target can be a means to that goal, but neither a steady queue level nor asymptotic stability is the product requirement. A decoder that keeps a bounded queue forever while delivering nothing has failed.

## 2. Separate the obligations from their current location

Start with required information and correctness obligations, then ask which representation and execution location can satisfy each. This gives a bounded design exercise rather than an unrestricted search over diagrams.

| obligation | present arrangement | question that can produce a new arrangement |
|---|---|---|
| Determine valid block structure and entropy symbols | The scanner constructs a complete `Symbols` object before a worker receives it | Can a validated descriptor or compact transition summary permit downstream work to start earlier? |
| Recover the block's byte order | A worker applies the selected inversion path | Can a different intermediate object reduce dependent traversal or make useful work composable? |
| Undo remaining coding and validate output | Several downstream steps already share information and buffers | Can retained information remove another pass, without breaking existing fusion? |
| Deliver bytes in order | Ready blocks wait for the reader's next position | Can execution order change while preserving the external order and error contract? |
| Bound live resources | A window and ready-byte guard throttle scanner admission | Can reservations describe actual ownership more accurately, so useful concurrency is permitted without uncontrolled retention? |

These are proposals, not claims that all placements are feasible. Block boundaries and table-selector state may prevent a cheap scanner split. A summary that is mathematically composable can be more expensive to construct than the original decode. An output buffer is still live when ownership has moved outside the ready map.

Source anchors: [`Scanner::next` and block parsing](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/stream.rs), [`decode_owned`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/lib.rs), and [`Shared::full`, `read_block`, `advance`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs).

## 3. Use dynamics to price consequences of a structural change

A state model records information whose present value affects what can happen next: unfinished work, block identities, retained memory, reader position and execution mode. Between completion events, work receives service. At events, jobs change owner, buffers appear or disappear, admission may become possible and the next ordered block may become available.

That description provides a precise place to put a proposed change. A new representation changes how much work remains. Moving work changes which resource provides service. Changing credits changes admission guards. Transferring a buffer changes ownership and later allocation/copy work. A mode change can alter all four and must have an explicit cost.

**Proved for a simple state counterexample:** an admitted-block count cannot be the complete exact state. Consider two executions with the same count of ready blocks and the same reader position. In one the next required block is ready; in the other only later blocks are ready. The reader has an enabled transfer in the first and must wait in the second. Equal counts do not imply equal enabled transitions. This does not rule out a useful approximate scalar predictor under a restricted workload or regime.

**Measured:** stage 1 also makes an occupancy-only performance explanation doubtful: mean admitted occupancy is 19.4695 at eight workers and 18.7298 at twenty, while steady worker starvation is 3.68489% and 56.1376%. These are different configurations and summary statistics, so they are evidence motivating added state, not a proof about every possible model reduction. [Per-count TSV](../../../math-targets/data/character-stage1/per_k.tsv).

The useful reduction is the smallest state that predicts the consequences of the interventions we care about. Predicting an existing trace when all its future service times are supplied is a replay check. Predicting a withheld intervention from information available before it runs is a stronger achievement.

## 4. A small candidate-design record

For each candidate, write six short entries before building substantial machinery:

1. **Function preserved:** exact bytes, validation, error and ownership behaviour.
2. **Object changed:** the representation, work placement, guard or transfer event; name the source location.
3. **Work removed and added:** include construction, extra metadata, synchronisation, copies and fallback.
4. **Regime where it can matter:** worker-limited, scanner-limited, startup, drain or memory-constrained; use current measurements.
5. **Observable consequence:** which event or demand changes, and the predicted whole-decoder effect under explicit assumptions.
6. **Cheapest falsifier:** a counterexample, offline count, trace replay or paired implementation comparison that can reject it promptly.

This is a design aid, not an additional approval procedure. Follow the existing [experiment programme](../../../briefs/experiment-programme.md) and [ownership policy](../../../../../team/OWNERSHIP.md).

## 5. What would count as progress

**Argued:** start with the already queued scanner plumbing work and the existing pulse/response experiments. They offer a real change of operating regime against which the model can be challenged. Then use the new regime to reconsider worker-side transformations previously hidden by scanner demand.

**Conjectured:** a compact scanner transition summary or a better intermediate object could change the feasible organisation more substantially. The price of construction and the selector-state obstruction decide whether that possibility is real. The hybrid description makes its consequences explicit; the algebraic construction and machine measurements still have to supply the gain.

The desired outcome of reading is therefore a sharper candidate or a cheaper rejection: a boundary that can move, a pass that can disappear, a reservation that can be made exact, or a counterexample that closes an unproductive direction. The notes should be revised whenever they fail to change one of those decisions.
