# The levers: what can be changed, and what each change risks

*The Coordinator's interpretation, 25 November 2026. Louis: "my goal is not to be stuck in theory but to find the levers that allow me to reorganise". With the decoder written as $\mathcal H = (C, F, D, G)$ (`decoder-as-hybrid-system.md`), every possible change is a change to one of five things. That makes the space of reorganisations explicit, and lets each proposal be placed and compared.*

## The five levers

| lever | what it changes | examples in our work | what the book says to watch |
|---|---|---|---|
| **1. The flow map $F$** | how fast each activity runs: its rates | the scanner plumbing fix (QUEUED 7147862124); faster Huffman decode; cheaper emission; contention reduced by placement | Shrinking an uncertain rate set (less contention) strengthens every robustness result (Chapter 7). Speeding one rate *moves the bottleneck*: the certificate's maximum changes term (§61's migration). |
| **2. The flow set $C$** | where work may proceed; where a thread must wait | the window W and the byte cap close flow for the scanner; the in-order reader closes flow for out-of-order blocks | Open, thin flow sets are fragile under perturbation (Chapter 4). Closed sets that overlap $D$ are what make behaviour robust (Proposition 4.10). |
| **3. The jump set $D$** | when an event may happen | admission's predicate; in-order taking; when lazy workers apply (`LAZY_FROM_THREADS`, §66); when a worker may take a job | Thin decision surfaces (an exact threshold) let tiny timing noise flip outcomes. Hysteresis turns them into robust regions (Examples 4.18, 4.19). |
| **4. The jump map $G$** | what an event does | dispatch (which idle worker); routing (Open, fallback, column); teams splitting one block; size-aware dispatch | A set-valued $G$ honestly models a free choice. Choosing well within it is a *policy*, and policies are judged by the certificate, not by one trace. |
| **5. The state space itself** | what is represented | a parallel or speculative scanner (several scanner states at once); decoding from all start states and composing (`../../../leads/structure-and-function.md`); an intermediate object between scanner and workers; Rauzy–Veech in place of LF traversal | This is "mathematical reorganisation". The book doesn't generate it, but it gives the format any proposal must be written in, so it can be compared with the shipped design by the same certificate. |

## Placing the current leads

| lead | lever | expected effect (from our records) |
|---|---|---|
| the scanner plumbing (input copy; symbol re-zeroing) | 1 | about 15 ms off the scanner's demand; about 6% wall at 8 workers, 8–9% at 9–10 (§62) |
| lazy workers below 8 threads (`LAZY_FROM_THREADS`) | 3 | 7 threads asked: 0.337 s against 0.209 s (§66) |
| the worker count (7 against 8 or 9) | 4 (the policy chooses k) | 0.209 s against 0.190–0.192 s through the public API (§66) |
| the scanner's placement (a favoured core, sibling idle) | 1, by reducing contention | removes the slowdown at 16 and above if it is SMT (stage 2a decides) |
| the window and byte cap | 2, 3 | rarely binding above the knee (§66): little to gain |
| within-block parallel Huffman (composed transition functions) | 5 | raises the scanner's ceiling, the lever that matters above the knee; conjectured (§64) |
| worker mathematics (cheaper inversion or emission) | 1, for workers | lowers the knee; buys the same speed with fewer cores (§61) |

## Rephrasing the goal and the work

**The goal:** reach the "all decoded" set as early as possible, for every input that matters, robustly to contention. That is, minimise a certificate of remaining time over the five levers.

**Each piece of work, rephrased:**
- **Measurement** (stage 1, stage 2) estimates $F$ (rates and their spread), observes which part of $C$ and $D$ the decoder actually visits, and checks the conservation laws that any $G$ must respect.
- **Engineering** pulls levers 1–4 within the existing state space.
- **Mathematics** pulls lever 5, a new state space, or lever 1 by a different computation of the same function.
- **A design-space enumeration** (Ma et al.) is a search over levers 2–5 for a fixed function, scored by the certificate.

**What this buys:** a proposal can be placed in one line ("changes $D$ at the admission boundary") and checked for its known risks (fragility at thin boundaries; bottleneck migration). It can be compared with others on one scale (the certificate's rate), before anything is built.
