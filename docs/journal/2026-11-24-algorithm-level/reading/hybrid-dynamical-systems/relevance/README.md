# What the book is for, in this project

*The Coordinator's interpretation, kept apart from the précis in `../precis/`.*

Louis's aim: "not to be stuck in theory but to find the levers that allow me to reorganise". The files here turn the book into that.

| file | read it for |
|---|---|
| `decoder-as-hybrid-system.md` | the decoder written in the book's format, $\mathcal H = (C, F, D, G)$, each state variable tied to a stage-1 measurement; the goal restated as reaching the "all decoded" set early; the bottleneck bound read as a completion-time certificate (Example 7.35) |
| `levers.md` | **start here.** The five levers (the flow map, the flow set, the jump set, the jump map, the state space); each current lead placed on one; what the book says each lever risks |
| `by-chapter.tsv` | one row a chapter: priority, what we take, the key results by number, which file uses it |
| `measurements.md` | six computations on the traces we already have, or will from stage 2, in order of how directly they inform a decision, and who takes each |

**In one paragraph.** Every change to the decoder changes one of five things:
- how fast work proceeds;
- where work may proceed;
- when events may happen;
- what events do;
- what is represented at all.

Engineering pulls the first four; mathematics, mostly the fifth. The book supplies two things we lacked. The first is a precise language in which any proposal, and the shipped design, can be written the same way. The second is results about what makes such systems fragile (thin decision boundaries, Chapter 4) and how to certify their behaviour without simulating every run: Lyapunov-type functions, including the bound on remaining time for systems that always terminate (Example 7.35). Our operational-law bound turns out to be one of those certificates. That makes it the scorer a design-space search needs.

**Limits:** the book's main theorems are about solutions that run forever and approach a set. The decoder terminates. The results that transfer directly are the solution concept, perturbation and regularisation, well-posedness, and the finite-time certificate. The stability and invariance results apply to a steady-state model of a long decode, and each file says which.
