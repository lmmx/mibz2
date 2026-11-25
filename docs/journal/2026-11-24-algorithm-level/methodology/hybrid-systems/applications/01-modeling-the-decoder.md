# 1. Modelling the decoder

This is original decoder analysis prompted by the [Chapter 1 reading](../precis/01-introduction.md).
It is not a theorem from the book. Only Chapter 1 has been read from the available
source; no result from Chapters 2-9 is assumed here.

The code baseline is `lmmx/mibz2` at
[`9c38ad6c18237a1b923f9b762a1f634864d3b8a4`](https://github.com/lmmx/mibz2/tree/9c38ad6c18237a1b923f9b762a1f634864d3b8a4).
The concrete mapping below covers the plain scanner-and-workers path. The
cooperative teams and service-core variants require their own phase and resource
states. Merely changing the worker count in this model does not represent them.

**Labels:** **proved** means a direct code fact or a deduction from stated model
premises; **argued** means a modelling choice with an explicit justification;
**conjectured** means an untested performance or implementation proposal.

## Start with the actual objects and ownership

**Proved from code.** A scanner job carries a stream position, bit position and
complete `Symbols` payload. `Scanner::next` reads an item before its ending bit
position is known; `worker` receives the whole symbols object and passes it to
`decode_owned`. A model that dispatches a cheap block descriptor instead changes
the scanner/worker boundary and must account for obtaining that descriptor.
Sources: [scanner](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/stream.rs#L81),
[job and worker](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L70),
[downstream decoding](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/lib.rs#L532).

**Argued model state.** Keep a discrete program layer and a separate timed
abstraction. The following records are a defensible starting point; they are not
a claim that every listed field is necessary in the smallest predictive model.

| State component | Why it changes the possible next events |
|---|---|
| Issued position, taken position, scanner position and scanner mode | Distinguishes an unscanned block, a scan in progress and an issued job. |
| Ordered job queue and each worker's current job, route and phase | Distinguishes available work from work already assigned; preserves route-specific costs. |
| Ready results keyed by position, including errors | The reader can take only its next position, even when later results are ready. |
| Reader's current buffer and consumed offset | Taking a block frees admission capacity before its bytes are consumed. |
| Buffer owners, lengths and capacities, including reusable pools | Ready-byte accounting does not describe all live allocations. |
| End position, cancellation and terminal/error state | Successful completion, ordered input failure and cancellation have different transitions. |
| Remaining service, input availability, clocks and resource mode | Gives the timed abstraction enough information to predict when an enabled event becomes possible. |

The record of each block need not contain all decoded bytes in a performance
simulator. It can refer to an immutable trace record containing its validated
size and service data. That is an abstraction decision, so it cannot establish
byte correctness by itself. Fixing the worker pool and window also permits
fixed-length arrays with inactive flags; changing those limits requires an
explicit representation rule.

Counts alone are insufficient: two states can have the same admitted count and
ready-byte total while one has the next required block ready and the other has
only later blocks ready. Their next reader action differs. Likewise, equal queue
lengths do not imply equal remaining work or resource contention. These are
constructive counterexamples to an occupancy-only state, not a claim about a
particular queueing distribution.

## Preserve the implemented guard and event boundaries

**Proved from code.** With $a=i-t$, the scanner's blocking predicate is

$$
a \ge W\quad\text{or}\quad(b_{ready}\ge B\ \text{and}\ a\ge k+2).
$$

Here $i$ and $t$ are `issued` and `taken`, $W$ is the slot limit, and
$B=12\,\mathrm{MiB}$ on this path. The capacity is not necessarily $4k$ for every
explicit constructor call; that is the usual slots-per-thread policy. The
constructor ensures at least one worker and at least as many slots as workers.
The guard is checked **before scanning**. `issued` increases **after** a block
has been scanned. Sources: [constants and cap](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L44),
[`Shared::full`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L401),
[`launch`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L760),
[`scanner`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L1047).

| Transition | State update to preserve |
|---|---|
| Scanner passes the admission guard | Begin scanning; do not yet increment issued. |
| Scanner returns a block | Allocate its ordered position and increment issued. Enqueueing is a subsequent operation. |
| Worker receives a job | Remove the oldest queued job and assign its downstream work to that worker. |
| Worker publishes its result | Insert the result, including an error if applicable, at the original position; update ready bytes. |
| Reader takes its next result | Remove that position, increment taken, and transfer successful output into the reader's current buffer. |
| Reader copies or transfers bytes | Update the consumed offset or ownership; do not repeat the taken increment. |
| Scanner ends or fails | Record the end position; a non-cancellation scanner error occupies an ordered result position. |
| Decoder cancels | Clear ready results, signal stopping and wake waiters; preserve the documented terminal error. |

The enqueue and publish operations must be modelled at their state-changing
points. Diagnostic markers are not automatically those points: this code emits
`Ready` after releasing the publication lock, and `Enqueued` after sending the
job. A replay must respect causal dependencies rather than give a later marker
priority over an earlier state change. Sources: [publication](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L1189),
[reader advance](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L927).

**Proved distinction.** $B$ is a threshold in the scanner's guard, not an
invariant upper bound on live output memory. Previously admitted workers can
publish after it is crossed. Ready bytes count successful result lengths;
capacities, current-reader ownership, worker scratch and reusable buffers are
separate. A proposed hard memory budget needs explicit reservations and release
events for those owners. Sources: [ready accounting](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L105),
[state and pools](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L280).

## Add elapsed work without inventing machine capacity

**Argued abstraction.** Between state-changing events, let each active activity
have remaining work $r_v$ and a rate $\nu_v$ in declared units:

$$
\dot r_v=-\nu_v(q,\text{placement},\text{pressure}).
$$

A blocked activity has zero progress rate while elapsed time can continue.
Completion enables a discrete transition. Service rates must be supplied by
measurement or explicitly bounded assumptions; an admission rule cannot provide
them. In particular, elapsed busy time is not automatically CPU demand, and a
worker-derived SMT factor is not a common rate multiplier for all roles.

If two completions can occur together, represent both permitted interleavings
or justify why a tie-breaking convention preserves the quantity being predicted.
An abstract model may permit more interleavings than the implementation; any
claim quantified over every abstract execution then needs that relationship
stated. Keeping memory bounded does not establish progress: a model can remain
bounded while all useful actions are blocked.

## Candidate changes made concrete by this model

These are **conjectured levers**, not outcomes of Chapter 1 or authorization for
another workstation run.

| Candidate | Concrete alteration | First useful check |
|---|---|---|
| Separate compute admission from ordered retirement | Introduce a compute credit released at a specified downstream completion, alongside an independent output reservation. | Replay the existing jobs and sizes; verify both credit conservation and every output owner's lifetime. Early compute-credit release alone must not remove the memory bound. |
| Extend ownership transfer | Use the existing whole-block path where the caller can accept it; identify when the caller returns a spent buffer. | Compare caller semantics and retained allocations. `read_block` already transfers a whole untouched block; it copies a remainder after partial `read`. Do not count the existing fast path as new work. |
| Add an adaptive execution mode | Specify the measured signal, allowed worker/kernel modes, switch guard, transition cost and retained state. | Replay candidate decisions and charge every transition. Current `launch` creates a fixed pool; changing its size online is an additional implementation. |

The ownership facts follow [`read` and `read_block`](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/src/parallel.rs#L837).
A caller can retain transferred buffers, so a decoder-internal bound must state
whether it excludes caller-owned output.

An adaptive policy can use distinct increase/decrease thresholds and a minimum
time between changes. Our elementary cost bound is: if successive changes are
at least $\delta>0$ apart, at most $1+\lfloor T/\delta\rfloor$ changes occur in an
interval of length $T$. With per-change cost at most $c$, their direct cost is at
most $c(1+\lfloor T/\delta\rfloor)$. This proves a conditional switching-cost
budget, **not** an optimal worker count, convergence guarantee, or speedup.
The threshold, $c$, and any effect on service rates remain to be identified.

## What is ready to use now

Use the existing Stage 1 traces to check this event/state mapping before adding
a larger continuous model. The [experiment programme](../../../briefs/experiment-programme.md)
already reports completed Stage 1, including reader and occupancy accounting; the earlier aggregate
scanner-only profile is not the current limit of the evidence. This note adds
semantic obligations to that work, not a competing experimental queue.

A proposed implementation still has to preserve ordered output and the same
errors on corrupt input, as required by [the correctness policy](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/docs/team/OWNERSHIP.md#2-what-never-bends).
A timed abstraction can suggest a change or expose an
incorrect resource claim; it cannot certify the decoder's byte-level behavior.
Shipping a production-default change remains subject to the repository's
[existing adoption policy](https://github.com/lmmx/mibz2/blob/9c38ad6c18237a1b923f9b762a1f634864d3b8a4/CLAUDE.md#delegated-as-a-class-design-acceptance-adoption).
No new default or benchmark is selected here.

