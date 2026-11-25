# The levers: what can be changed, and what each change risks

*The Coordinator's interpretation, 25 November 2026. Louis: "my goal is not to be stuck in theory but to find the levers that allow me to reorganise". With the decoder written as $\mathcal H = (C, F, D, G)$ (`decoder-as-hybrid-system.md`), every possible change is a change to one of five things. That makes the space of reorganisations explicit, and lets each proposal be placed and compared.*

## The five levers

| lever | what it changes | examples in our work | what the book says to watch |
|---|---|---|---|
| **1. The flow map $F$** | how fast each activity runs: its rates | the scanner plumbing fix (QUEUED 7147862124); faster Huffman decode; cheaper emission; contention reduced by placement | Shrinking an uncertain rate set (less contention) strengthens every robustness result (Chapter 7). Speeding one rate *moves the bottleneck*: the certificate's maximum changes term (§61's migration). |
| **2. The flow set $C$** | where work may proceed; where a thread must wait | the window W and the byte cap close flow for the scanner; the in-order reader closes flow for out-of-order blocks | Open, thin flow sets are fragile under perturbation (Chapter 4). Closed sets that overlap $D$ are what make behaviour robust (Proposition 4.10). |
| **3. The jump set $D$** | when an event may happen | admission's predicate; in-order taking; when a worker may take a job. (`LAZY_FROM_THREADS` is construction-time configuration, not a jump guard: it picks the kernel once per decoder.) | Preserve the discrete state and the implemented predicates: the admission guard is an exact integer test under a lock, not a noisy threshold, and widening it (< W to ≤ W) would break the window. Chapter 4's fragility and hysteresis apply to *future adaptive policies* driven by noisy measurements, once their signal, switching cost and failure mode are specified (corrected after ChatGPT's review). |
| **4. The jump map $G$** | what an event does | dispatch (which idle worker); routing (Open, fallback, column); teams splitting one block; size-aware dispatch | A set-valued $G$ honestly models a free choice. Choosing well within it is a *policy*, and policies are judged by the certificate, not by one trace. |
| **4b. Ownership** (from ChatGPT's notes) | who holds which buffer, and when it is released: compute credits apart from memory reservations; taking a block apart from consuming and returning it | the byte cap gates new scanning but not live memory; worker, ready and reader buffers lie outside it | a redesign must account for every allocation owner, and keep the earliest unfinished block able to get resources |
| **5. The state space itself** | what is represented | a parallel or speculative scanner (several scanner states at once); decoding from all start states and composing (`../../../leads/structure-and-function.md`); an intermediate object between scanner and workers; Rauzy–Veech in place of LF traversal | This is "mathematical reorganisation". The book doesn't generate it, but it gives the format any proposal must be written in, so it can be compared with the shipped design by the same certificate. |

## Placing the current leads

| lead | lever | expected effect (from our records) |
|---|---|---|
| the scanner plumbing (input copy; symbol re-zeroing) | 1 | about 15 ms off the scanner's demand. On stage 1's instrumented numbers it is almost entirely masked at 8 workers, where the workers are within 0.3 ms of the scanner, and gives about 7.5% at 10 (ChatGPT's re-pricing; §62's 6% at 8 used the scan profile). Run 2's A/B decides |
| lazy workers below 8 threads (`LAZY_FROM_THREADS`) | configuration (which flow map the decoder is built with): lever 1 at construction | 7 threads asked: 0.337 s against 0.209 s (§66) |
| the worker count (7 against 8 or 9) | 4 (the policy chooses k) | 0.209 s against 0.190–0.192 s through the public API (§66) |
| the scanner's placement (a favoured core, sibling idle) | 1, by reducing contention | removes the slowdown at 16 and above if it is SMT (stage 2a decides) |
| the window and byte cap | 2, 3 | rarely binding above the knee (§66): little to gain |
| within-block parallel Huffman (composed transition functions) | 5 | raises the scanner's ceiling, the lever that matters above the knee; conjectured (§64) |
| worker mathematics (cheaper inversion or emission) | 1, for workers | lowers the knee; buys the same speed with fewer cores (§61) |

## Rephrasing the goal and the work

**The goal:** reach successful completion as early as possible, within a CPU and memory budget, for the inputs that matter, robustly to contention. **Corrected after ChatGPT's review:** the bottleneck maximum is a *lower bound*, not a certificate. A design search uses three separate quantities (`decoder-as-hybrid-system.md`): lower bounds to prune, validated predictors to rank, and proved upper certificates to guarantee.

**Each piece of work, rephrased:**
- **Measurement** (stage 1, stage 2) estimates $F$ (rates and their spread), observes which part of $C$ and $D$ the decoder actually visits, and checks the conservation laws that any $G$ must respect.
- **Engineering and mathematics are not split by lever** (corrected after ChatGPT's review). Scheduling mathematics can change $G$, an algebraic reformulation can cut the work in $F$, and an engineering change can add state. Changes often pull several levers at once, and which lever a change pulls depends on how the model is written.
- **A design-space enumeration** (Ma et al.) searches over the levers for a fixed function. It is pruned by lower bounds, ranked by validated predictors, and certified only where an upper bound is proved.

**What this buys:** a proposal can be placed in one line ("changes $D$ at the admission boundary") and checked for its known risks (fragility at thin boundaries; bottleneck migration). It can be compared with others on a common footing before anything is built: first by its lower bound, then by a validated predictor.
