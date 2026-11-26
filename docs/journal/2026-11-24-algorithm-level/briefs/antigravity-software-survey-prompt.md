# Antigravity (Neon): which slow, famous research software would change what we can do, if made fast?

Given to Louis to paste, 26 November 2026. It is independent of the ideal-parts passes and can run beside them.

---

Work in the lmmx/mibz2 repository on your branch `research/neon-movement` (fetch `origin/master` to read from; don't push to it). Don't touch `src/`. Commit your answer as `research/neon-movement/SOFTWARE-SURVEY.md` with Louis's identity (`git config user.name "Louis Maddox"`, `git config user.email louismmx@gmail.com`), with no AI attribution, and push it. Then post a short `FROM: neon (movement) / TO: coordinator / TYPE: REVIEW` comment on #145 linking it at its commit. Never put a PR or issue number straight after fix, close or resolve.

**Why this survey.** Vidit Nanda (Oxford) has just remade two research packages as fast native code with Python bindings, both correct and well known but slow:
- **Perseus,** his persistent-homology package, "entirely rewritten in Nov 2026 to be very fast and memory efficient";
- **Helmer,** a C++ port of Martin Helmer's Macaulay2 package WhitneyStratifications, "between 38 and 5000 times faster, with the same strata" (people.maths.ox.ac.uk/nanda/software.html).

The pattern is general: specialist mathematical software that is correct and respected but slow, or locked in Macaulay2, Sage, GAP, Magma, Singular, MATLAB, Java or abandoned C++, can now be remade fast. Louis's question is ours, not Nanda's: **which such packages, made fast (ported to Rust or C++), would materially help this project?**

**What "help" means here.** Read first, on `origin/master`, under `docs/journal/2026-11-24-algorithm-level/`:
1. `programme.md`, `essence.md`: the problem and the referee (Q, D, P, S);
2. `ideal-parts/results/full/README.md`: the measured room per part. Construction (T2) is largest below the knee, the scan (T4) above it, and the two co-limit at 6–8 workers. The decoder's own residual is unmeasured.
3. `reading/hybrid-dynamical-systems/relevance/levers.md` and the journal's §70: lower bounds against predictors against *proved upper certificates*, the last of which we have none of;
4. `leads/structure-and-function.md`: design for this decoder's purpose; design-space enumeration (Ma et al.; Khammash);
5. the Rauzy–Veech line: the blog post `docs/site/blog/2026-11-10-rauzy-veech-reconstruction.md` and its technical companion;
6. `ideal-parts/antigravity/notes-for-neon.md`: the Coordinator's first reading of Helmer.

**The task:**
1. **Enumerate candidates widely.** Look beyond the obvious, and say where you looked. The areas below are a starting list; add any we haven't thought of.
   - hybrid-system reachability and verification (for example CORA, SpaceEx, Flow\*, JuliaReach);
   - interval-exchange and Rauzy-induction tooling (for example surface_dynamics, flatsurf);
   - automata and transducer toolkits (for example FAdo, Vcsn, AWALI, symbolic automata);
   - superoptimisers and program synthesis (for example STOKE, Souper);
   - max-plus, event-graph and timed Petri-net analysis;
   - probabilistic and timed model checking (for example PRISM, Storm, UPPAAL);
   - queueing solvers (for example LINE, JMT, LQNS);
   - combinatorics on words (for example Sage's words, Walnut);
   - permutation-group and cycle-structure computation (GAP);
   - computational topology (Perseus, discrete Morse theory);
   - tropical and polyhedral geometry (polymake, gfan);
   - SAT, SMT and exhaustive-enumeration frameworks.
2. **For each candidate:**
   - what it computes;
   - its language, its speed today and its licence;
   - **the specific question in this project it would answer,** naming the file or journal section where that question is open;
   - what "fast" would unlock that "slow" cannot, for example enumeration over every block of a corpus, an exhaustive design space, or certificates per input class;
   - a rough porting cost, and whether a fast equivalent already exists;
   - labels: `[ESTABLISHED]` for a known fit, `[ARGUED]`, or `[CONJECTURED]`.
3. **Rank the top five** by (value to the measured targets: construction, the scan, the certificates) × (likelihood the port pays). For the top two, give:
   - the smallest experiment that would show the fit is real before anyone ports anything;
   - what porting would involve: core algorithms, dependencies, a test oracle against the original.
4. **The ones that look relevant but aren't,** and why, briefly.

**Rules:**
- every claim about a package is traced to its documentation or paper, with a link;
- no porting yet. This pass is a survey and a ranking only.
