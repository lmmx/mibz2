# The decoder written as a hybrid system

*The Coordinator's interpretation, 25 November 2026. The book's notation (`../precis/ch01-introduction.md`, `../precis/ch02-solution-concept.md`); the decoder's facts from stage 1 (journal §66). A modelling proposal, not a validated model.*

## The format

The book has one object: $\mathcal H = (C, F, D, G)$. The state $x$ may flow, $\dot x \in F(x)$, while $x \in C$, and may jump, $x^+ \in G(x)$, while $x \in D$ (Definition 2.2). Modes, timers and counters are ordinary state components (Chapter 1). Where $C$ and $D$ overlap, both are possible. Set-valued $F$ and $G$ model genuine choice or uncertainty.

## The state

For the parallel decoder with k workers, one scanner and one reader, each component is measured in stage 1:

| component | meaning | stage 1's measure |
|---|---|---|
| $A, B, C_r$ | blocks issued, completed (ready), taken by the reader: counters | the trace's `q_issued`, `q_ready`, `q_taken` |
| $r_s$ | the scanner's remaining work on its current block | `s_A2` + `s_A3` per block |
| $m_s \in \{\text{scan}, \text{wait-admission}, \text{wait-input}\}$ | the scanner's mode | the ledger's states |
| $r_i, m_i$ ($i = 1..k$) | each worker's remaining work on its block, and whether it is busy | `w_A5`…`w_A9` per block; the starvation time |
| $r_c, m_c$ | the reader's remaining copy work, and its mode | the reader's copy and wait split |
| $R$ | which completed blocks wait in `ready`, with their bytes | the occupancy integrals |
| $\theta$ | contention: how many threads share cores and cache | placement, IPC, the memory controller's traffic |

$A - C_r$ is the admission quantity `issued − taken`. The conservation identities ($A - B$ = in flight, $B - C_r$ = ready, their sum) held exactly in all 840 decodes.

## Flow: work being done

Between events, each busy thread's remaining work falls: $\dot r_s \in -[\underline\mu_s(\theta), \overline\mu_s(\theta)]$, and the same for each $r_i$ and $r_c$. Counters and modes don't flow.

The *rates* are the analogue part, and they are set-valued, because contention makes them uncertain. Stage 1 shows how wide the set is:
- the scanner's Huffman stage costs 84 µs a block at 1 worker, 89 µs at 3–12, and 106 µs at 16 or more;
- emission costs 526 µs at 1, 673 µs at 10, and 816 µs at 14.

Writing $F$ as an inclusion is not a modelling convenience. It is what the data show. Neon's bimodal scanner (a fast and a slow mode across runs, from 10 workers up) is two branches of the same inclusion.

$C$ is the set where no event is forced: every remaining work is positive, or its thread is waiting.

## Jumps: events

$D$ is where an event may happen. $G$ says what it does:

| event | allowed when ($D$) | effect ($G$) |
|---|---|---|
| issue | $r_s = 0$, and admission is open: $A - C_r < W$, and not (ready bytes ≥ cap and $A - C_r \ge k+2$) | $A \mathrel{+}= 1$; the block joins the queue; $r_s$ ← the next block's scan demand |
| start | a worker is idle and the queue is non-empty | that worker's $r_i$ ← the block's demand. **Set-valued: any idle worker may take it** |
| finish | some $r_i = 0$ | $B \mathrel{+}= 1$; the block joins `ready`; the worker goes idle |
| take | the reader is idle and block $C_r + 1$ is in `ready` (**in order**) | $C_r \mathrel{+}= 1$; $r_c$ ← its copy work |
| consumed | $r_c = 0$ | the reader goes idle; buffers are recycled |

**Hybrid time** (Definition 2.3) is the pair $(t, j)$: elapsed time, and the count of events. Stage 1's trace is a hybrid arc, with timestamps for $t$ and the under-lock sequence numbers for $j$. Events sharing a timestamp are the zero-length steps that hybrid time domains allow. The causal-order problem the design review found (§59) is the reason $j$ must be recorded separately from $t$.

## The goal

The decode starts with everything unissued and ends in the target set
$$\mathcal T = \{x : C_r = N \text{ and } r_c = 0\},$$
with every block consumed. The goal is to reach $\mathcal T$ as early as possible, for every input the decoder should serve, robustly to the contention that makes $F$ uncertain.

The book's stability is about approaching a set (Chapter 3, pre-asymptotic stability of a closed set). The decoder *always reaches* its set, in finite time, so what we need is a bound on the remaining time. The book has exactly that: for systems that always terminate, a smooth function bounding the remaining flow time, or the remaining jumps, exists (Example 7.35; finite-time convergence in Example 7.6). That function is a certificate of completion time.

## The bottleneck bound is such a certificate

Take
$$V(x) = \max\Big( \text{scan work left},\ \tfrac{1}{k}\,\text{worker work left},\ \text{reader work left} \Big)$$
in seconds at the slowest admissible rates.
- **During flow,** whichever term is the maximum falls at rate 1 while its resource is busy.
- **Across jumps,** work only moves between stages and is not created, so $V$ does not increase.
- **The caveat:** the busy resource is not always busy. That is exactly the gap stage 1 measured: workers idle while the scanner is active, and the scanner waiting for admission.

So $V(x_0)$, adjusted for that idle time, bounds the decode's time from the start. This is the operational-law bound $X \le \min(1/D_\text{scan}, k/D_\text{wkr}, 1/D_\text{read})$ (§57, §66) read as a Lyapunov-type function on the hybrid system. Chapter 3's relaxed conditions (§3.3) are the tool when $V$ may rise briefly in one regime but falls overall. Our idle periods are such a case.

**Why this matters for the work, not just as theory.** A certificate judges a *whole design* without simulating it. Change $F$, $D$ or $G$ (`levers.md`) and ask whether the certificate's rate improves. Ma et al.'s enumeration (`../../../leads/structure-and-function.md`) needs exactly such a cheap scorer, and this is one.

## What the model leaves out (so far)

- Memory and cache contention enter only through $\theta$ in the rates. A better model might give $\theta$ its own dynamics, driven by how many threads are busy, which is where the "analogue" quantities live.
- Routing (Open, dense, refused, column) is a set-valued choice of demand per block, fixed by the block's content: a deterministic $G$, given the corpus.
- The feeder is negligible: about 0.006 ms a block against the scanner's 0.10.
