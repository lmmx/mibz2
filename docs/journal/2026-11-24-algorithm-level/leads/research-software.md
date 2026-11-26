# Research software worth making fast (or using today)

*The Coordinator's index, 26 November 2026.*
- **The source:** Antigravity's survey, `research/neon-movement/SOFTWARE-SURVEY.md` at e5df35c0 (#145, REVIEW 7179215140), answering `../briefs/antigravity-software-survey-prompt.md`.
- **The occasion:** Vidit Nanda's fast rewrites, Perseus and Helmer (`../ideal-parts/antigravity/notes-for-neon.md`).
- **What this file is:** a pointer and summary. The survey is the authority, and its claims are its own, not yet checked.

## Its ranking (port to Rust, with the smallest test before porting)

| # | package (today) | what it would answer here | the pre-port test |
|---|---|---|---|
| 1 | **Deterministic network calculus:** Nancy (C#), Panco (Python), NCorg DNC (Java) | proved upper bounds on the wall (§70's missing certificates), with the admission window's feedback | run the stock tools on stage 1's traces at k 1, 6, 8 and 10. Pass if the bound is ≤ 1.5 × the measured wall and it reproduces the knee near 7–8 |
| 2 | **uiCA** (Python; a clean-room Rust port to avoid AGPL) | a fast, accurate cycle model for the i9's core, as a cost oracle for kernel search in the scan, construction and MTF | stock uiCA against llvm-mca on the scan, construction and runs kernels. Pass at ≤ 5% error, with the right sign on the 3-symbol regression (§82) |
| 3 | **surface_dynamics** (Sage/Python), an integer-IET and Rauzy subset | corpus-wide search over induction orders on real blocks (the Rauzy–Veech line, construction) | — |
| 4 | **IMITATOR** (OCaml) | certified polyhedral regime maps over (scanner rate, worker rate, W, k), via parametric timed and stopwatch automata | — |
| 5 | **Rosette / Alloy + Kodkod** with symmetry breaking | design-space (referee-class) enumeration, and §72's SAT necessity proof | — |

An honourable mention: TPLib (tropical polyhedra).

## Its corrections to the Coordinator's first guesses

- **Hybrid-reachability tools:** the Coordinator ranked them first. The survey puts CORA, SpaceEx, Flow\* and JuliaReach in the wrong sub-class: they are built for continuous ODE flowpipes, while the decoder's dynamics are piecewise-constant rates, which suit timed and stopwatch automata (IMITATOR, TChecker).
- **Also not worth porting:** Helmer and Perseus (already fast; the wrong domain), STOKE and Souper, FAdo and Vcsn, queueing solvers (stochastic averages, not bounds), and Walnut.

## Usable today, with no port

Minotaur (a SIMD superoptimiser on LLVM IR), CaDiCaL with SMS, TChecker, Storm, MinMaxGD and egg/egglog.

## Status

Nothing is assigned. The two pre-port tests (network calculus on stage 1's traces; uiCA on our kernels) are the cheap next step if Louis wants them.
