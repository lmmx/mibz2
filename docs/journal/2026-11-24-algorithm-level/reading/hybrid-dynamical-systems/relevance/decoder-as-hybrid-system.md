# The decoder written as a hybrid system

*The Coordinator's interpretation. Written 25 November 2026 and **rewritten the same day after ChatGPT's review** (`../../../external/chatgpt-hybrid-systems-review.md`; journal §70). The first version called the bottleneck maximum a completion-time certificate. **That was wrong:** it is a lower bound. Its state table also misread logged sequence numbers as counts, and its event table did not match the code. The book's notation is from `../precis/ch01-introduction.md` and `../precis/ch02-solution-concept.md`. This is a modelling proposal, not a validated model.*

## The format

The book's object is $\mathcal H = (C, F, D, G)$. The state may flow, $\dot x \in F(x)$, while $x \in C$, and may jump, $x^+ \in G(x)$, while $x \in D$ (Definition 2.2). Modes, timers and counters are ordinary state components.

## The state, and what the records actually support

The admission counter $a = \text{issued} - \text{taken}$ is an integer, and the code evaluates its guard under a lock. The state space is partly discrete, and the model must keep it so.

| quantity | what stage 1's records support |
|---|---|
| cumulative issued, ready and taken counts $A, B, C_r$ | **reconstructible** by counting each event type in the causal order. The trace's `q_issued`, `q_ready` and `q_taken` are each event's *position in the shared linear order*, not counts, so they must not be substituted directly |
| the ready set $R$, with block identities and bytes | reconstructible from the per-block ready and taken records and sizes. Occupancy integrals alone cannot recover it |
| which block each worker, the scanner and the reader *owns*, and each one's mode | reconstructible from the per-block events (`worker`, the got, ready, taken and consumed times) |
| remaining work $r_s$, $r_i$, $r_c$ | **not observed.** The recorded elapsed service supports a *retrospective* reconstruction under an explicit model. It is not a measurement of future remaining work |
| contention $\theta$ | **partly observed**: placement, achieved frequency, IPC and memory traffic. Its dynamics and identification are assumptions to state |

**Why the distinction matters:** a replay that feeds each block its own observed service time reconstructs that run, but it predicts nothing. Prediction means producing the service times, or their consequences, without being given them.

## Flow: work being done

Between events, each busy thread's remaining work falls at a rate set by contention. Writing the rate as an interval (an inclusion) is a *modelling choice*. The rate histories it permits need justification. Run-to-run bimodality, Neon's fast and slow scanner, does not by itself identify a freely switching rate: persistent hidden modes or correlated conditions produce the same observations.

## Events, matched to the code

| event | allowed when ($D$) | effect ($G$) |
|---|---|---|
| **admit and start a scan** | the scanner is idle; input is available; $a < W$ and not (ready bytes ≥ cap and $a \ge k+2$) | the scanner owns the next block; its remaining scan work is set. **Admission is checked here, before scanning**, as in the code |
| **issue** | the scanner owns a block and its scan work is done | $A$ += 1; the block joins the job queue; the scanner goes idle. `issued` advances *after* scanning |
| **start** | a worker is idle and the queue is non-empty | that worker owns the block (any idle worker may take it, so $G$ is set-valued) |
| **finish** | a worker is **busy, owns a job,** and its work is done | $B$ += 1; the block joins `ready`; the worker goes idle |
| **take** | the reader is idle and the *next block in order* is in `ready` | $C_r$ += 1 (`taken` advances on taking, before copying); the reader owns the block |
| **consumed** | the reader **owns a block** and its copy work is done | the reader goes idle; its buffers are recycled |

The ownership and mode guards on *finish* and *consumed* matter. Without them, "consumed" would be enabled for an idle reader, and the model would allow an infinite run of idle-to-idle jumps.

**Terminal outcomes are kept apart:**
- **successful completion:** the last output byte is delivered, end of input is reached, and the stream's checks pass;
- **error termination**;
- **cancellation**;
- **deadlock:** a non-terminal state with nothing enabled.

**Hybrid time** (Definition 2.3, as corrected by the errata) is the pair $(t, j)$: elapsed time and event count. Stage 1's ledger gives the event skeleton. It becomes a hybrid arc only once the state between events is defined, and a *solution* only once the chosen $C, F, D, G$ are checked against it.

## The goal

The aim is to reach successful completion as early as possible, within a CPU and memory budget, for the inputs that matter, robustly to contention.

## Three quantities, kept apart

| quantity | what it establishes | how it is obtained | use |
|---|---|---|---|
| **lower bound** $L$ | no execution can finish earlier, under a stated demand model | $L = \max(S, W/k, R)$: total scanner, worker and reader service (the operational-law bound, §§57, 66) | prune designs whose *optimistic* time already loses to a feasible incumbent |
| **predictor** $\hat T$ | an estimated completion time | a trace-driven simulation or a reduced model | rank experiments. **Valid only after held-out validation**: stage 2's pulses and withheld rounds, without their realised durations |
| **upper certificate** $U$ | every permitted execution completes within $U$ | proved: a function decreasing at rate ≥ 1 during flow and not increasing at jumps, for every allowed rate and jump, plus no non-terminal deadlock, no endless instantaneous jumps, and correct terminal states | guarantees and robust comparisons |

**The bottleneck maximum is a lower bound, not an upper certificate.** ChatGPT's counterexample: one block, a 1 s scan, then a 1 s decode, then a 1 s delivery. $L = 1$, but completion takes 3 s. The maximum ignores serial dependencies, starvation, indivisible jobs and blocking.

**Example 7.35, read correctly.** It concerns a constrained differential inclusion on a *compact* flow set that meets the basic regularity conditions and has no complete solutions. For such a system, a smooth function bounding the remaining time exists. It is not "any system that always terminates". ChatGPT's example shows termination alone is not enough: with speed $p$ for $p > 0$ and speed 1 at $p = 0$, every run terminates, but the time from $(1, 1/n)$ is $n$, so no continuous bound exists near $p = 0$. Existence also doesn't identify which function, or how tight. The example *motivates* searching for a lifetime bound for a suitably regular terminating model of the decoder. Its hypotheses must be established, its maximal lifetime connected to successful completion, and a candidate constructed.

**A concrete route to an upper certificate** (ChatGPT's construction, proved conditionally). Take a finite, acyclic expanded state graph in which every non-terminal state has a successor and every edge has a defensible upper duration $\bar d$. Then $U(s) = \max_{s \to s'} [\bar d(s, s') + U(s')]$, with $U = 0$ at terminal states, bounds completion time by backward induction. It includes the dependencies the maximum ignores. It is a model-relative scorer for a small family of dispatch, credit or placement policies, to be built beside the trace-driven simulator and tested against it.

## What the model leaves out

- Contention enters only through $\theta$. Its dynamics are unmodelled.
- Routing per block is fixed by the block's content: a deterministic choice of demand, given the corpus.
- The feeder is negligible: about 0.006 ms a block against the scanner's 0.10.
