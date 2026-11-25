# What the book is for, in this project

*The Coordinator's interpretation, kept apart from the précis in `../precis/`.*

Louis's aim: "not to be stuck in theory but to find the levers that allow me to reorganise". The files here turn the book into that.

| file | read it for |
|---|---|
| `decoder-as-hybrid-system.md` | the decoder in the book's format, $\mathcal H = (C, F, D, G)$, with events matched to the code and what the records actually support; three quantities kept apart: lower bound, predictor and upper certificate (rewritten after ChatGPT's review) |
| `levers.md` | **start here.** The five levers (the flow map, the flow set, the jump set, the jump map, the state space); each current lead placed on one; what the book says each lever risks |
| `by-chapter.tsv` | one row a chapter: priority, what we take, the key results by number, which file uses it |
| `candidate-uses.md` | the chapter readers' speculative uses, moved out of the précis, with corrections marked |
| `measurements.md` | six computations on the traces we already have, or will from stage 2, in order of how directly they inform a decision, and who takes each |

**In one paragraph.** Every change to the decoder changes one of five things:
*(Corrected: these levers are a way of asking questions, not a split between engineering and mathematics.)*

- how fast work proceeds;
- where work may proceed;
- when events may happen;
- what events do;
- what is represented at all.

The book supplies two things we lacked. The first is a precise language in which any proposal, and the shipped design, can be written the same way. The second is a vocabulary for what makes such systems fragile (Chapter 4), and for certifying behaviour without simulating every run (Lyapunov-type functions; Example 7.35's remaining-time bound for suitably regular systems with no complete solutions). **Corrected after ChatGPT's review (journal §70):** the first version said our operational-law bound is such a certificate. It is not: it is a **lower bound**. Upper certificates are a separate proof obligation, and `decoder-as-hybrid-system.md` gives a concrete route to one for a small finite model.

**Limits:** the book's main theorems are about solutions that run forever and approach a set. The decoder terminates. The results that transfer directly are the solution concept, perturbation and regularisation, well-posedness, and the finite-time certificate. The stability and invariance results apply to a steady-state model of a long decode, and each file says which.

## The other set of notes, and what they add

ChatGPT worked on the same task at the same time. Its notes are merged here from `docs/hybrid-systems-notes` (894b676e) and live in `../../../methodology/hybrid-systems/`:
- `index.md`;
- a Chapter 1 précis;
- `applications/00…04`, its interpretation.

It read Chapter 1 only (the public sample). Its Chapters 2–9 are marked pending; the full précis are in `../precis/` here. Its report is recorded verbatim in `../../../external/chatgpt-hybrid-systems-commit.md`.

**Where the two agree.** The decoder is a hybrid system with explicit guards, transitions and ownership. A handful of levers covers every reorganisation. Prices must be conditional, and joint, because the bottleneck moves.

**What it adds, adopted here:**
- **A sixth lever, ownership.** ChatGPT separates *compute credits* from *memory reservations*, and taking a buffer from consuming and returning it. In `levers.md` these sit inside the jump map. They deserve their own row, because the byte cap gates new scanning but does not bound all live memory: worker buffers, ready results and the reader's current buffer are all outside it.
- **The scanner fix's price at 8 workers, re-done on stage 1's numbers.** Stage 1's instrumented worker busy time at 8 is 0.1847 s against the scanner's 0.1850 s. So a 15 ms scanner cut is almost entirely masked there (0.1941 s against 0.1944 s), but gives 7.5% at 10 workers. That revises §62's "about 6% at 8 workers", which used the scan profile's lower worker figure (0.1786 s). **Caveat:** stage 1's ledgers are on in those numbers, and may inflate worker time (PERTURBED at 10 and 20; untested at 8). Run 2's paired A/B at 8, 9 and 10 workers decides. Combined with a 20% cut in emission, the conditional figure at 8 is 0.180 s (−7.3%), a reason to price joint changes.
- **Replay is not prediction.** Re-running a trace with its own observed service times tests the bookkeeping. A model is validated only by predicting a withheld intervention without its realised durations. Stage 2's pulses are the test. `measurements.md` item 3 adopts this: the certificate must predict held-out decodes and pulses.
- **Termination is not a time bound.** Example 7.35's function exists for systems that always terminate, but turning it into a *time* bound needs explicit assumptions about input, the sink, fairness and service. They are stated with the certificate in `decoder-as-hybrid-system.md`.
- **A precise data note:** `service.tsv` is per run, role and activity, not per block. Per-block reconstruction uses the archived block traces (`blocks-r00..r03.tsv`). `measurements.md` already does.
