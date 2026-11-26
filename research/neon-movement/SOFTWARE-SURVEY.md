# Research Software Survey: Slow Specialist Tools Whose Native Port Would Move `mibz2`

**Author:** Neon (`research/neon-movement`)  
**Date:** 26 November 2026  
**Prompt:** `docs/journal/2026-11-24-algorithm-level/briefs/antigravity-software-survey-prompt.md` (`origin/master`)  
**Scope:** Survey and ranking only; no porting and no changes to `src/`.

---

## 0. Targets, referee, and where we looked

### What "help" means against the measured targets
From `origin/master`, the project's live targets and open mathematical obligations are:

1. **Construction (`C`, Target T2)** — largest room below the knee: ideal ratio **0.579** at 1 worker (80% of log-wall variation) and **0.754** at 6 workers (46% of variation), ~45% of worker time (`ideal-parts/results/full/README.md` §§1, 3, 5; journal §§72, 89). Open mathematical questions: escaping record 119's class under the referee `(Q, D, P, S)` at `K = 2`, `s <= |Σ| + O(1)` (`essence.md`, `council/comparison.md`, `reports/2026-11-24-from-search-to-formulated-problem.md` §4); cheaper IET acquisition and induction step-order (`1.812 -> 1.279` steps/run vs floor `1.220`, `docs/site/blog/2026-11-10-rauzy-veech-reconstruction.md` §5, `2026-11-20-train-tracks.md`); and positive necessity proofs / referee-class enumeration (`leads/structure-and-function.md` lines 121–127; journal §72).
2. **The Scan (`S`, Target T4)** — largest room at 10 workers (**0.919**, 53% of variation) and co-limiting with `C` at 6–8 workers (`q_SC = -0.110` at 6w, `-0.071` at 8w; at 8w, `C` alone is `0.944`, `S` alone is `0.968`, while `C` with `S` ideal is `0.661` and `S` with `C` ideal is `0.679`; `ideal-parts/results/full/README.md` §2). Serial LUT arithmetic is near its microarchitectural ceiling (`1.41–1.51 ns/sym`, `#870` regression, journal §82); within-block parallel Huffman FSM composition was ruled out by the 50-symbol selector count (journal §72).
3. **Emission (`E`, Target T1)** and **Cross-stage enrichment (`T5`)** — `E` is **0.835** at 1w / **0.851** at 6w (net of handover ~0.62–0.63 at 1–6w, ~0.75 at 8w); open questions are output-order code acquisition, closure under composition, and deferred cross-stage riders (`tracks/cross-stage.md`, `targets.tsv`).
4. **Proved Upper Certificates (`U`) and Design-Space Enumeration** — journal §70 and `reading/hybrid-dynamical-systems/relevance/decoder-as-hybrid-system.md` separate **lower bounds** $L = \max(S, W/k, R)$, **validated predictors** $\hat T$, and **proved upper certificates** $U$. We currently have **zero** proved upper certificates, and `leads/structure-and-function.md` commissions a Ma/Khammash-style enumeration over `obligations × placements` and policies `(k, W, cap)` plus a calibrated machine-cost bridge (`P-cal` in `council/comparison.md`).
5. **Unmeasured residual** — the all-ideal floor (`0.299–0.721`) is dominated by oracle handover copies (`ideal-parts/results/full/README.md` §4); that is an engineering fix (zero-copy oracles), not a math-package port.

### The Nanda filter
Vidit Nanda's rewrites of **Perseus** and **Helmer** (`WhitneyStratifications` from Macaulay2 to C++, 38–5000× faster; `notes-for-neon.md`) show the pattern: a port pays only when **both** conditions hold:
- *(A) Genuine mismatch in speed or embeddability today:* the canonical implementation is locked in MATLAB, Java, Python/Sage, Racket, OCaml, GAP, Perl, .NET/C#, or abandoned C++, **and** no fast open-source C++/Rust equivalent already exists;
- *(B) Speed unlocks a new regime for `mibz2`:* e.g., certifying $10^4\text{–}10^5$ policy/design points, running across all 233–1,724 corpus blocks, or sitting inside a search/superoptimisation inner loop.

Where a fast native C++/Rust tool already exists (e.g., `Storm`, `TChecker`, `Minotaur`, `CaDiCaL`/`SMS`, `libsemigroups`, `egg`, `libflatsurf`, `Ripser`), porting has zero payoff — those tools should simply be *used*.

### Where we looked (16 areas: 12 from the brief + 4 added)
- **Brief's 12 areas:** (1) hybrid-system reachability; (2) interval-exchange, Rauzy-induction & train-track tooling; (3) automata & transducer toolkits; (4) superoptimisers & program synthesis; (5) max-plus, event-graph & timed Petri-net analysis; (6) probabilistic & timed model checking; (7) queueing solvers; (8) combinatorics on words; (9) permutation-group & semigroup computation; (10) computational topology; (11) tropical & polyhedral geometry; (12) SAT, SMT & exhaustive-enumeration frameworks.
- **4 added areas motivated by `mibz2`'s open records:**
  - **(13) Deterministic Network Calculus & Real-Time Calculus** — directly motivated by `measurements.md` §2 (arrival/service curves & window feedback) and §70's missing upper certificates $U$.
  - **(14) Microarchitectural basic-block analyzers & analytical cache models** — directly motivated by `council/comparison.md`'s `P-cal` bridge, the i9-10900K toy problem (`reports §5`), and `S`/`C`/`M` cycle ceilings (§82, `IDEAL-PARTS-1.md`).
  - **(15) Algebraic transform-schedule compilers** — directly motivated by `leads/structure-and-function.md` (SPIRAL, Halide, Bird–de Moor).
  - **(16) Systems-biology reaction-network & structural-identifiability toolboxes** — checked because `leads/structure-and-function.md` draws heavily on Alon, Ma et al., and Khammash.

---

## 1. Candidate Catalogue

Status labels: `[ESTABLISHED]` = known, verified fit to an open project question; `[ARGUED]` = structured technical case for fit, untested empirically; `[CONJECTURED]` = plausible upside, unproved.

---

### Area 1 — Hybrid-system reachability and verification

#### 1.1 CORA (COntinuous Reachability Analyzer)
- **What it computes:** Set-based reachability (zonotopes, polytopes, Taylor models, probabilistic zonotopes) for linear, nonlinear, and hybrid automata with uncertain inputs/parameters ([GitHub](https://github.com/TUMcps/CORA); [Manual v2026](https://tumcps.github.io/CORA/manual/index.html); Althoff 2015).
- **Language, speed today, licence:** MATLAB (OO toolbox; slow interpreter loop on hybrid guard intersections, though a C++ subset `CORA/SX` gave up to 3× speedup — [JuliaReach paper](https://arxiv.org/html/1901.10736v2)); **GPL-3.0**.
- **Project question:** Worst-case completion time $U$ and safety of $\mathcal{H} = (C, F, D, G)$ under contention intervals (`decoder-as-hybrid-system.md`, `levers.md`, journal §70).
- **What "fast" would unlock:** Sweeping continuous-flow hybrid models over `(k, W, cap)` without MATLAB licensing.
- **Porting cost & existing fast equivalent:** Very high (~50k+ LOC MATLAB). **Fast native equivalents already exist:** `JuliaReach` / `ReachabilityAnalysis.jl` (compiled Julia, MIT, matching/beating SpaceEx — [GitHub](https://github.com/JuliaReach/ReachabilityAnalysis.jl); [HSCC 2019](https://arxiv.org/html/1901.10736v2)), `SpaceEx` (C++), `HyPro` (C++). More fundamentally, our decoder has trivial countdown flows ($\dot r_i \in [-\overline{\rho}_i, -\underline{\rho}_i]$) and huge discrete state (`W = 4k`, ready set $R$), which is a poor match for continuous ODE flowpipe tools.
- **Label:** `[ARGUED]` (mismatch: wrong hybrid sub-class; see §4).

#### 1.2 SpaceEx, Flow*, JuliaReach (`ReachabilityAnalysis.jl`)
- **What they compute:** Support-function reachability for piecewise-affine hybrid automata (`SpaceEx`, Frehse et al. 2011, [RWTH evaluation](https://publications.rwth-aachen.de/record/962039/files/962039.pdf)); Taylor-model flowpipes for nonlinear ODEs (`Flow*`, Chen et al. CAV 2013, [GitHub](https://github.com/chenxin415/flowstar)); lazy/concrete convex-set reachability (`ReachabilityAnalysis.jl` + `LazySets.jl`, [GitHub](https://github.com/JuliaReach/ReachabilityAnalysis.jl)).
- **Language, speed today, licence:** `SpaceEx`: C++ (GPL-3.0, Java GUI, older unmaintained codebase); `Flow*`: C++ (GPL); `ReachabilityAnalysis.jl`: 100% Julia (**MIT**, already near-C++ speed).
- **Project question:** Same as CORA (`decoder-as-hybrid-system.md`, §70).
- **What "fast" would unlock:** Little — `ReachabilityAnalysis.jl` and `SpaceEx`/`Flow*` are already compiled; bottleneck on our pipeline is discrete location explosion, not ODE integration speed.
- **Porting cost & fast equivalent:** High; `ReachabilityAnalysis.jl` (MIT) and `HyPro` (C++) already cover this niche.
- **Label:** `[ARGUED]` (already native / wrong sub-class).

---

### Area 2 — Interval-exchange, Rauzy-induction, and train-track tooling

#### 2.1 `surface_dynamics` (Delecroix, Rüth et al.) — **RANK #3**
- **What it computes:** Interval exchange transformations (IETs), Rauzy classes and Rauzy diagrams, Rauzy–Veech and Zorich induction, Kontsevich–Zorich cocycle & Lyapunov exponents, square-tiled surfaces (origamis), cylinder decompositions, and Abelian/quadratic strata ([GitHub](https://github.com/flatsurf/surface-dynamics); [PyPI](https://pypi.org/project/surface-dynamics/)).
- **Language, speed today, licence:** **85.6% Python, 9.1% Cython, 5.2% C** locked inside **SageMath** (requiring GMP, PARI/GP, GAP, PPL, LattE); **GPL-2.0+**. Designed for small symbolic alphabets ($d \le 20$); completely unusable on real bzip2 blocks where $d = r \in [10^4, 10^5]$.
- **Project question:**
  - **T2 (Construction) & T1 (Emission):** Step-order optimisation (`1.812 -> 1.279` steps/run vs floor `1.220` in `docs/site/blog/2026-11-10-rauzy-veech-reconstruction.md` §5; `2026-11-12-rauzy-technical-companion.md` §2);
  - Origami lift & cylinder/switch decomposition (`2026-11-20-train-tracks.md` §4);
  - Acquisition cost (`0.10–0.34 h_1`, `reports §4` item 1) and characterising fragmentation on high-entropy blocks (`reports §4` item 2, record 119).
- **What "fast" would unlock:** Specialising the discrete-IET / origami / generalised-induction / Zorich-acceleration subset to `u32` integer weights in Rust unlocks **corpus-wide structural censuses and induction-schedule search across all 233 Wikidata blocks + prose + hex** ($r \sim 60{,}000$), testing non-endpoint cuts, interior/two-sided splitting, and cylinder-first reductions at machine speed.
- **Porting cost & fast equivalent:** Moderate for the discrete integer-IET / origami / Rauzy-schedule core (~4–6k LOC Rust, 3–5 weeks; drops PARI/PPL/LattE/number-field dependencies). `libflatsurf` ([GitHub](https://github.com/flatsurf/flatsurf), C++ 88.6%, GPL-3.0) is a fast C++ sibling for *flat triangulations and saddle connections over real algebraic number fields*, **not** $r \sim 10^5$ discrete integer IETs with word-grammar tracking. `mibz2` already has fast Rust right-end induction (`examples/rauzy.rs`, `src/open.rs`), so the port specifically targets *exploratory induction orders, origami/cylinder decompositions, and generalised splits*.
- **Label:** `[ESTABLISHED]` for structural correspondence (`2026-11-10-rauzy-veech-reconstruction.md`, `2026-11-20-train-tracks.md`); `[CONJECTURED]` for beating Open's `A6b` construction cost in production.

#### 2.2 `libflatsurf` / `sage-flatsurf`
- **What it computes:** Exact flat triangulations, Delaunay decompositions, saddle connections, and flow decompositions on translation surfaces over exact number fields (`e-antic`) ([GitHub](https://github.com/flatsurf/flatsurf)).
- **Language, speed today, licence:** **C++ (88.6%)** + Python/cppyy wrapper; **GPL-3.0+**.
- **Project question:** Origami-lift geometry (`2026-11-20-train-tracks.md` §4).
- **What "fast" would unlock:** Already fast C++; its exact algebraic-number-field triangulation overhead is unneeded for integer-grid BWT origamis.
- **Porting cost & fast equivalent:** Already native C++.
- **Label:** `[CONJECTURED]` (already native; over-generalised arithmetic for integer BWTs).

#### 2.3 `curver` and `flipper` (Mark C. Bell)
- **What they compute:** Curve-complex geodesics, train-track splitting/folding, Nielsen–Thurston classification, and mapping-class word problems via ideal triangulation coordinates and straight-line programs (SLPs) ([curver GitHub](https://github.com/MarkCBell/curver); [curver docs](https://curver.readthedocs.io/); [Bell software](https://markcbell.co.uk/software.html)).
- **Language, speed today, licence:** **Pure Python**; **MIT** (`curver`). README explicitly notes the polynomial-time Bell–Webb algorithm's constants *"currently make this implementation impractical."*
- **Project question:** Train-track splitting/folding equivalences and SLP-encoded curve actions (`2026-11-20-train-tracks.md` §§2–3, 5).
- **What "fast" would unlock:** Testing train-track split sequences and Bestvina–Handel graph maps on medium-genus punctured spheres / BWT run-networks ($r \sim 10^2\text{–}10^3$ reduced core switches).
- **Porting cost & fast equivalent:** Moderate (~5–8k LOC Rust). No fast Rust equivalent, though `curver`'s mapping-class machinery is built for *homeomorphisms of surfaces*, whereas BWT blocks are single weighted tracks (where `surface_dynamics`'s discrete IETs are far closer to LF).
- **Label:** `[CONJECTURED]` (interesting train-track SLP connection, indirect fit to T2).

---

### Area 3 — Automata and transducer toolkits

#### 3.1 `FAdo` (Moreira, Reis et al.)
- **What it computes:** Symbolic manipulation of DFAs, NFAs, regular expressions, transducers, bisimulation quotients, and random DFA/RE generation ([PyPI](https://pypi.org/project/FAdo/); [CIAA 2009](https://www.researchgate.net/publication/221568175_FAdo_and_GUItar_Tools_for_Automata_Manipulation_and_Visualization); [arXiv:1009.3599](https://arxiv.org/html/1009.3599v1)).
- **Language, speed today, licence:** **Pure Python**; slow on large state/alphabet automata; **GPL-3.0+**.
- **Project question:** Huffman decoder FSM synchronisation length (`leads/structure-and-function.md` §"The concrete lead" & item 5) and finite-state transducer composition across scanner/MTF (`tracks/cross-stage.md`).
- **What "fast" would unlock:** Sweeping transducer minimisation and synchronisation across all 6×1,724 canonical Huffman tables.
- **Porting cost & fast equivalent:** Moderate, **but fast C++ equivalents already exist:** `Awali` (C++, GPL-3.0, [GitLab](https://gitlab.in2p3.fr/vaucanson/awali)), `Mata` (C++, MIT, [GitHub](https://github.com/verifit/mata); [paper](https://www.researchgate.net/publication/379594086_Mata_A_Fast_and_Simple_Finite_Automata_Library)), and `OpenFST` (C++, Apache-2.0). Moreover, journal §72 ruled out within-block parallel Huffman FSM composition due to the 50-symbol selector count.
- **Label:** `[ARGUED]` (redundant: fast C++ equivalents exist; target lead already narrowed).

#### 3.2 `Vcsn` (Vaucanson 2) and `Awali`
- **What they compute:** Weighted finite automata and transducers over arbitrary semirings (Boolean, $\mathbb{Z}$, $\mathbb{Q}$, tropical $(\min,+)$, $(\max,+)$) with determinisation, minimisation, and rational expressions ([Vcsn GitLab](https://gitlab.lre.epita.fr/vcsn/vcsn); [Awali GitLab](https://gitlab.in2p3.fr/vaucanson/awali)).
- **Language, speed today, licence:** Both C++ + Python (`awalipy`), **GPL-3.0+**. `Vcsn` is effectively frozen (~2018, GCC 5/6, Boost + JIT clang plugins); **`Awali`** is its actively maintained C++ fork (updated Oct 2025).
- **Project question:** Weighted/semiring enrichment transducers (`essence.md` §"enrichment along composition"; `tracks/cross-stage.md`) and tropical-weighted automata for cost bounds.
- **What "fast" would unlock:** `Awali` is already native C++ (plus `QuAK`, C++17 MIT, for nested quantitative automata — [arXiv:2605.12418](https://arxiv.org/pdf/2605.12418)); no speed port needed.
- **Porting cost & fast equivalent:** High; `Awali`, `OpenFST`, and `QuAK` already exist in native C++.
- **Label:** `[CONJECTURED]` (already native in `Awali`/`QuAK`).

#### 3.3 `SVPAlib` (`symbolicautomata`, D'Antoni et al.)
- **What it computes:** Symbolic finite automata (SFAs), symbolic finite transducers (SFTs), symbolic streaming string transducers (SSTs), and transducer inversion/equivalence over interval Boolean algebras ([GitHub](https://github.com/lorisdanto/symbolicautomata); Hu & D'Antoni PLDI 2017 [paper](https://heping.me/papers/pldi17.pdf)).
- **Language, speed today, licence:** **Java (99.6%)**, Maven/JVM; **Apache-2.0**.
- **Project question:** Symbolic transducer composition & inversion over 256-byte / 16-bit symbol alphabets (`tracks/cross-stage.md`: Huffman → MTF → RLE2 boundary; `essence.md` enrichment/fusion).
- **What "fast" would unlock:** Composing symbolic interval-predicate transducers over byte ranges without alphabet blow-up. However, inverse MTF has $256!$ states (non-regular over full bytes) and Neon's T5 theorems (`T5-CROSS-STAGE-HUFFMAN-ENRICHMENT.md`) already showed $\Delta Q = 0$ pre-MTF, leaving only RLE2/small-alphabet synthesis.
- **Porting cost & fast equivalent:** Moderate (~10k LOC Rust). No Rust equivalent for symbolic SSTs; `Microsoft.Automata` is C#/.NET.
- **Label:** `[CONJECTURED]` (narrow scope: MTF is non-regular over 256 bytes).

---

### Area 4 — Superoptimisers and program synthesis

#### 4.1 `STOKE` (StanfordPL)
- **What it computes:** MCMC stochastic superoptimiser and SMT-verified equivalence checker for x86-64 assembly kernels ([GitHub](https://github.com/StanfordPL/stoke); Schkufza et al. ASPLOS 2013; [arXiv:2109.13498](https://arxiv.org/html/2109.13498v2)).
- **Language, speed today, licence:** **Abandoned C++11** (README explicitly states *"nobody is actively working on this code base"*, requires GCC 4.9 / Ubuntu 14.04–16.04, incomplete AVX2/BMI2 validator semantics, hours per loop-free kernel); **Apache-2.0**.
- **Project question:** Finding optimal x86-64/AVX2 instruction sequences for hot inner loops: `src/huff.rs` (`1.41–1.51 ns/sym`, §82), `src/runs.rs` (MTF/RLE2), `src/open.rs` (`A6b` IET-step), and AVX2 `copy_if` left-pack (`reports §4` item 6).
- **What "fast" would unlock:** Modern AVX2/BMI2 x86-64 stochastic search with a fast microarchitectural cost model (`uiCA`, Rank #2).
- **Porting cost & fast equivalent:** Very high (~40–60k LOC + full x86-64/AVX2 SMT semantics). **Partial native equivalent already exists:** `Minotaur` (C++, MIT, actively maintained LLVM SIMD superoptimiser with Alive2+Z3 — [GitHub](https://github.com/minotaur-toolkit/minotaur)), which can run directly on `mibz2`'s `rustc --emit=llvm-bc`! What is missing is a fast x86-64 microarchitectural cost oracle (`uiCA`, Rank #2), not a full STOKE rewrite.
- **Label:** `[ARGUED]` (superseded at IR level by `Minotaur`; assembly-level scoring bottleneck is `uiCA`).

#### 4.2 `Souper` and `Minotaur` (LLVM superoptimisers)
- **What they compute:** Counterexample-guided SMT superoptimisation of LLVM IR integer peepholes (`Souper`, [GitHub](https://github.com/google/souper), Sasnauskas et al. [arXiv:1711.04422](https://arxiv.org/html/1711.04422v2)) and **SIMD/vector + memory LLVM IR** (`Minotaur`, Liu et al. [arXiv:2306.00229](https://arxiv.org/abs/2306.00229), [GitHub](https://github.com/minotaur-toolkit/minotaur), backed by Alive2 + Z3 + `llvm-mca`).
- **Language, speed today, licence:** Both C++. `Souper` (**Apache-2.0**) was **archived on 30 Oct 2025** and lacks memory/SIMD support. `Minotaur` (**MIT**) is active, parallel offline (`cache-infer`), and handles AVX2/SIMD LLVM IR directly.
- **Project question:** Peephole and SIMD optimisation of `src/huff.rs`, `src/open.rs`, `src/runs.rs`, and AVX2 compaction (`reports §4` item 6).
- **What "fast" would unlock:** Nothing via porting — `Minotaur` is already native C++ and can consume `rustc --emit=llvm-bc` directly.
- **Porting cost & fast equivalent:** Already native (`Minotaur`).
- **Label:** `[ESTABLISHED]` as a tool to **use**, not port.

#### 4.3 `Rosette` (Torlak & Bodik) / `Grisette` (Lu & Bodik) / `Sketch` (Solar-Lezama) — **RANK #5 (with `SMS`)**
- **What they compute:** Solver-aided symbolic evaluation and **CEGIS (`∃ program ∀ input`) program synthesis** for custom domain-specific virtual machines (`Rosette`, [GitHub](https://github.com/emina/rosette), Torlak & Bodik 2013/2014; `Grisette`, [POPL 2023](https://lsrcz.github.io/files/POPL23.pdf), [GitHub](https://github.com/lsrcz/grisette); `Sketch`, Solar-Lezama 2008; also `SuFu` for intermediate-structure fusion, [PLDI 2024](https://xiongyingfei.github.io/papers/PLDI24.pdf)).
- **Language, speed today, licence:** `Rosette`: **Racket** (BSD-3) — notoriously slow symbolic evaluation and large formula blowup (Grisette POPL '23 reports 91.2% smaller formulas than Rosette 4); `Grisette`: Haskell (BSD-3); `Sketch`: Java frontend + C++ backend (MIT). None integrates dynamic symmetry breaking over rule tables.
- **Project question:** **T2 (Construction) & T1 (Emission) — the Khammash / Ma / Bundala "three-node network" enumeration & positive necessity proof** (`leads/structure-and-function.md` lines 121–127; journal §72: *"A SAT encoding could settle a positive necessity statement in a bounded class: 'any inversion within $K$ sweeps and $s$ cursors makes at least $h_1 - c$ addressed accesses'. Either answer is new."*).
- **What "fast" would unlock:** A native Rust symbolic-DSL CEGIS + `SMS`-style dynamic-symmetry-breaking engine (wrapping native `CaDiCaL`/`Kissat`) eliminates Racket/JVM symbolic-term overhead and symmetry explosion, making exhaustive search / UNSAT necessity proofs feasible for $(K, s)$-cursor BWT machines up to $n = 8\text{–}12, |\Sigma| = 2\text{–}3$.
- **Porting cost & fast equivalent:** Moderate (~4–6k LOC Rust for an ORG-style symbolic evaluator + CEGIS + custom symmetry propagator over `CaDiCaL` IPASIR-UP). `Grisette` (Haskell) reduces formula size 5× over `Rosette`, and `SMS` (C++, MIT) provides dynamic symmetry breaking for graphs, but no native Rust/C++ tool combines both for custom cursor-machine transition tables.
- **Label:** `[ARGUED]` for the referee-class necessity problem (journal §72); `[CONJECTURED]` for scaling $n$ high enough to separate asymptotic behaviour from small-$n$ lookup artefacts.

#### 4.4 `egg` / `egglog`
- **What they compute:** E-graphs and equality saturation for algebraic rewriting, fusion, and cost-guided extraction ([GitHub](https://github.com/egraphs-good/egg); POPL 2021).
- **Language, speed today, licence:** **Rust** (already native, high-performance); **MIT**.
- **Project question:** Bird–de Moor algebraic fusion/tupling rewrites (`leads/structure-and-function.md` lines 26–31) and bit-level expression simplification.
- **What "fast" would unlock:** Already fast Rust; ready to use directly (and pair with fast `uiCA`, Rank #2, as its extraction cost model).
- **Porting cost & fast equivalent:** Zero (already Rust).
- **Label:** `[ESTABLISHED]` as a tool to **use**, not port.

---

### Area 5 — Max-plus, event-graph, and timed Petri-net analysis

#### 5.1 `MinMaxGD` / `PyMinMaxGD` and `ETVO` (Cottenceau, Hardouin, Lhommeau)
- **What they compute:** Exact arithmetic on ultimately periodic formal power series in the dioid $\mathcal{M}_{in}^{ax}\llbracket \gamma, \delta \rrbracket$ (Baccelli, Cohen, Olsder & Quadrat 1992) for **Timed Event Graphs (TEGs)** (`MinMaxGD`, [GitHub](https://github.com/GitUALH/MinMaxGD); [PyMinMaxGD GitLab](https://gitlab.univ-nantes.fr/dioids/python-toolbox)) and **Weighted TEGs / periodic holding times / partial synchronisations** (`ETVO`, [LARIS 2019](https://www.researchgate.net/publication/335992292_A_C_toolbox_to_handle_series_for_event-varianttime-variant_max_systems); [arXiv:2204.01479](https://arxiv.org/pdf/2204.01479)).
- **Language, speed today, licence:** **C++** (~5k LOC for `MinMaxGD`, GPL-3.0; SWIG Python wrapper `PyMinMaxGD`, GPL-3.0); `ETVO` is C++ built on `MinMaxGD`.
- **Project question:** Exact max-plus algebraic model of the scanner–workers–reader pipeline with window tokens $W$ and in-order resequencing (`leads/structure-and-function.md` line 76; journal §72: max-plus reproduces stage 1 within 0.2% up to 7w and 1.3% above).
- **What "fast" would unlock:** Already native C++ (and small enough to compile and call via `PyMinMaxGD` or Rust FFI). Its main limitation is mathematical rather than speed: TEGs model *deterministic* synchronisation (fixed/periodic holding times) rather than uncertain variable block-service envelopes — which **Deterministic Network Calculus** (Area 13, Rank #1) generalises directly via min-plus/max-plus curve envelopes.
- **Porting cost & fast equivalent:** Low (~5k LOC if wrapped/rewritten in Rust to fix raw-pointer/integer-overflow ergonomics), but already C++.
- **Label:** `[ESTABLISHED]` for fixed-rate TEG analysis (`leads/structure-and-function.md` line 76); dominated for variable-block certificates by Area 13.

#### 5.2 ScicosLab MaxPlus toolbox
- **What it computes:** Max-plus linear algebra, spectral radius / maximum cycle mean (Howard's policy iteration), and linear systems over $\mathbb{R}_{\max}$ (INRIA Maxplus group, [INRIA 2012 report](https://radar.inria.fr/report/2012/maxplus/uid29.html); [ScicosLab history](https://www.rocq.inria.fr/scicos/history.html)).
- **Language, speed today, licence:** Scilab/ScicosLab + C extensions; legacy ScicosLab environment is largely abandoned.
- **Project question:** Maximum cycle mean / throughput of deterministic pipeline event graphs (`leads/structure-and-function.md` line 76).
- **What "fast" would unlock:** Trivial — Karp's algorithm and Howard's policy iteration are ~100 lines of Rust; no library port needed.
- **Porting cost & fast equivalent:** Trivial (~100–300 LOC).
- **Label:** `[ESTABLISHED]` conceptually, trivial to implement directly.

#### 5.3 `TINA` (Time Petri Net Analyzer, LAAS-CNRS) and `LoLA`
- **What they compute:** State-class graphs, structural place/transition invariants (P/T-semiflows), deadlock/livelock checks, and worst/best-case end-to-end latency on Time Petri Nets (`TINA`, [LAAS homepage](https://projects.laas.fr/tina/options.php); [arXiv:1509.06506](https://arxiv.org/html/1509.06506v1); `LoLA`, [MCC survey](https://homes.cs.aau.dk/~srba/files/MCC:24.pdf)).
- **Language, speed today, licence:** `TINA`: compiled **OCaml/C** binaries (freeware/academic binary distribution; auxiliary `mcc` is CeCILL-B); `LoLA`: C++ (AGPL). Both already handle $10^7\text{–}10^8$ states fast (2019 MCC winner).
- **Project question:** Deadlock/livelock freedom and buffer-ownership invariants (lever 4b in `levers.md`; conservation laws in `decoder-as-hybrid-system.md`).
- **What "fast" would unlock:** Already fast native binaries; however, Time Petri Nets lack continuous multi-rate parametric synthesis (which `IMITATOR`, Rank #4, provides).
- **Porting cost & fast equivalent:** High; already fast native tools (`TINA`, `LoLA`, `PetriSpot`).
- **Label:** `[ARGUED]` (already fast; useful off-the-shelf for structural deadlock checks).

---

### Area 6 — Probabilistic and timed model checking

#### 6.1 `PRISM` and `Storm`
- **What they compute:** Exact and parametric verification of DTMCs, CTMCs, MDPs, Markov automata, and probabilistic timed automata (`PRISM`, [GitHub](https://github.com/prismmodelchecker/prism), Java 51.6% + C 30% + C++ 8.7%, **GPL-2.0**; `Storm`, [GitHub](https://github.com/stormchecker/storm), **C++ 98.7%** + `stormpy`, **GPL-3.0**, [paper](https://www.researchgate.net/publication/353036839_The_probabilistic_model_checker_Storm)).
- **Project question:** Stochastic pipeline scheduling policies (MDPs over dispatch choices $G(x)$ in `decoder-as-hybrid-system.md` / `levers.md` lever 4) and exact rational reward/completion-time verification (even Isabelle-certified via Storm).
- **What "fast" would unlock:** `PRISM`'s Java explicit engine is slower than C++, **but `Storm` (C++20, actively maintained v2.0.0, supports PRISM/JANI/GSPN input languages and `stormpy`) is ALREADY the fast native C++ remake of PRISM!**
- **Porting cost & fast equivalent:** Zero porting needed — use `Storm` directly.
- **Label:** `[ARGUED]` as a tool to **use** for stochastic policy comparison, not port.

#### 6.2 `UPPAAL` and `TChecker`
- **What they compute:** Zone-based (DBM) reachability, deadlock-freedom, and worst-case execution time for timed automata and priced/statistical timed automata (`UPPAAL`, C++, **proprietary / free for non-commercial academic use**; `TChecker`, C++, **MIT**, simulation-based zone subsumption — [ProVerB](https://slebok.github.io/proverb/tchecker.html); [zone survey arXiv:2207.07479](https://arxiv.org/html/2207.07479v1)).
- **Project question:** Bounded-horizon deadlock freedom and worst-case makespan on concrete timing bounds (`decoder-as-hybrid-system.md`).
- **What "fast" would unlock:** `TChecker` is already fast open-source MIT C++, though it lacks *parametric* synthesis and *multi-rate stopwatches* (needed when contention slows worker clocks $\dot r_i \in [\underline{\rho}, \overline{\rho}]$).
- **Porting cost & fast equivalent:** Already native (`TChecker` MIT for non-parametric timed automata).
- **Label:** `[ARGUED]` (use `TChecker` for fixed-rate checks; need `IMITATOR` below for parametric/multi-rate).

#### 6.3 `IMITATOR` (Étienne André) — **RANK #4**
- **What it computes:** **Parametric Timed Automata (PTA) and Multi-Rate Stopwatch Automata** — computes **Behavioral Cartography** (exact convex polyhedral partition of parameter space into certified safety/deadlock-free/deadline-meeting regimes) and **Minimal/Maximal-Time Parameter Synthesis** ([GitHub](https://github.com/imitator-model-checker/imitator/); [website](https://www.imitator.fr/); André, CAV 2021).
- **Language, speed today, licence:** **OCaml** calling Parma Polyhedra Library (PPL) via C stubs, single-threaded; **GPL**. Slow on concurrent multi-worker models with $k \ge 4$ and bounded admission windows $W \ge 8$ due to OCaml runtime/GC overhead, sequential exploration, and lack of worker-permutation symmetry reduction.
- **Project question:**
  - **Proved Upper Certificates ($U$)** and **certified regime maps** over uncertain contention rate parameters $(r_s, r_w, r_r, W, k)$ (`decoder-as-hybrid-system.md`, `levers.md` levers 2/3/4/4b, journal §70);
  - Directly fills the exact role the Coordinator identified for Nanda's **Helmer** in `notes-for-neon.md` — **a certified polyhedral regime stratification of our piecewise-linear/timed-event pipeline** (where Helmer only applies to smooth polynomial models!).
- **What "fast" would unlock:** A parallel Rust/C++ port combining parametric zone/polyhedra arithmetic with **symmetry reduction across interchangeable workers** (collapsing $k!$ worker permutations!) unlocks exact worst-case makespan certificates $U(r_s, r_w, W, k)$ and deadlock/buffer-ownership proofs for levers 2–4b (including future adaptive hysteresis policies in `levers.md`) up to $k = 8, W = 16$.
- **Porting cost & fast equivalent:** Medium–High (~12–18k LOC Rust/C++, 6–10 weeks; depends on exact rational polyhedra / `PPL` or `cddlib`/LP dual-simplex backend). No fast parallel open-source equivalent exists for parametric multi-rate stopwatch cartography.
- **Label:** `[ESTABLISHED]` for exact match to multi-rate stopwatch + integer-guard semantics of `decoder-as-hybrid-system.md`; `[ARGUED]` for scaling to $k=8$ under symmetry reduction.

---

### Area 7 — Queueing solvers

#### 7.1 `LINE` (Casale et al., Imperial College) and `JMT` (Politecnico di Milano)
- **What they compute:** Analytical (MVA, NC, fluid, matrix-analytic, LQN, cache-queueing) and discrete-event simulation solvers for queueing networks (`LINE`: MATLAB / Java / Python wrapper around JMT & LQNS, **BSD-3**, [homepage](https://line-solver.sourceforge.net/); `JMT`: **Java**, **GPL-2.0**, [GitHub](https://github.com/Java-Modelling-Tools/jmt)).
- **Project question:** Surrogate queueing models (`programme.md` step 6, `briefs/character.md`).
- **What "fast" would unlock:** Very little. `programme.md` (lines 44–49, citing `briefs/character-review-decisions.md`) explicitly notes that exact product-form MVA is *not justified* for the actual window-bounded finite pipeline and is restricted to a documented surrogate. Moreover, `LINE`/`JMT` already solve a 3-station surrogate in milliseconds–seconds, and they compute **stochastic averages** ($\hat T$), not **proved upper certificates** ($U$).
- **Porting cost & fast equivalent:** High (~50k+ LOC); `LQNS` (C++, [awesome-queueing](https://github.com/A-Herzog/awesome-queueing-theory-software), commercial/academic) is already C++, and a custom Rust trace-driven simulator (`briefs/character.md`) is ~300 LOC and matches stage 1's exact event semantics far better.
- **Label:** `[ARGUED]` (low value to port; see §4).

---

### Area 8 — Combinatorics on words

#### 8.1 `sage.combinat.words` (SageMath)
- **What it computes:** Lyndon words and factorisations, Christoffel/Sturmian/episturmian words, BWT and bijective BWT, Rauzy graphs, return words, factor complexity ([SageMath spkg/docs](https://doc.sagemath.org/pdf/en/reference/spkg/spkg.pdf)).
- **Language, speed today, licence:** **Python/Cython** inside SageMath; **GPL-2.0+**. Orders of magnitude too slow for $n = 900{,}000$ blocks.
- **Project question:** Lyndon trees, return words, and factor statistics of BWT blocks (`2026-11-10-rauzy-veech-reconstruction.md` §4, `research/neon-movement/S1-TEXT-AXIS.md`).
- **What "fast" would unlock:** Little via porting: Duval's linear-time Lyndon factorisation, SA-IS/`libsais`, and suffix-tree/factor-complexity algorithms are already well-known $O(n)$ C/Rust routines (and `mibz2` already writes its own custom Rust censuses).
- **Porting cost & fast equivalent:** Fast $O(n)$ C/Rust primitives already exist (`libsais`, `divsufsort`, `mibz2` kernels).
- **Label:** `[ARGUED]` (no general library port needed; individual algorithms are trivial in Rust).

#### 8.2 `Walnut` (Mousavi & Shallit)
- **What it computes:** Decides first-order logic over $\langle \mathbb{N}, + \rangle$ extended by $k$-automatic and Pisot/Ostrowski-automatic infinite words via deterministic finite automata ([GitHub](https://github.com/Walnut-Theorem-Prover/Walnut); [arXiv:1603.06017](https://arxiv.org/pdf/1603.06017); Shallit CUP 2022).
- **Language, speed today, licence:** **Java** (v7.1 / v8.0-alpha); **GPL-3.0**.
- **Project question:** Possibly checking word-combinatorial conjectures on Sturmian/Fibonacci/Pell witness families (`essence.md` line 25, `S3-PELL-RECURRENCE.md`).
- **What "fast" would unlock:** Nothing for real corpora — Wikidata JSON, prose, and high-entropy hex blocks are **finite non-automatic strings** that `Walnut` cannot represent; for toy automatic witnesses, stock Java `Walnut` already runs in seconds ([arXiv:2610.04842](https://arxiv.org/html/2610.04842)).
- **Porting cost & fast equivalent:** Moderate, zero payoff for `mibz2` (see §4).
- **Label:** `[CONJECTURED]` (wrong domain for real blocks; see §4).

---

### Area 9 — Permutation-group and semigroup computation

#### 9.1 `GAP` and `Magma`
- **What they compute:** Finite permutation groups (Schreier–Sims, partition backtrack, conjugacy classes, character tables), finitely presented groups, and discrete algebra (`GAP`, [GitHub](https://github.com/gap-system/gap), GAP language + C kernel, **GPL-2.0+**; `Magma`, proprietary C).
- **Project question:** Cycle structure of LF (`council/comparison.md` §1: single-cycle $\iff$ primitive word), MTF transition monoid (`E1-CROSS-STAGE-MTF-MONOID.md`), and symmetry groups of small cursor-machine rule tables (`leads/structure-and-function.md`).
- **What "fast" would unlock:** Nothing. A single $n = 900{,}000$ LF permutation needs a 10-line $O(n)$ cycle scan, not Schreier–Sims. For finite transformation monoids / congruences / Knuth–Bendix, **`libsemigroups`** ([GitHub](https://github.com/libsemigroups/libsemigroups), C++17 + AVX `HPCombi`, **GPL-3.0**, used *inside* GAP!) is already blazing fast and exposes Python bindings.
- **Porting cost & fast equivalent:** Immense; `libsemigroups` + `nauty`/`Traces` already cover the native high-speed pieces.
- **Label:** `[ARGUED]` (no port needed; see §4).

---

### Area 10 — Computational topology

#### 10.1 `Perseus` (Nanda), `Ripser` (Bauer), `GUDHI`, Discrete Morse Theory
- **What they compute:** Persistent homology barcodes of filtered simplicial/cubical complexes (`Perseus`, C++, discrete Morse reduction, recently rewritten fast Nov 2026 — [Nanda software](https://people.maths.ox.ac.uk/nanda/software.html); `Ripser`, single-file C++, **MIT**, apparent-pair implicit coboundary reduction — [GitHub](https://github.com/Ripser/ripser), [Bauer 2021](https://arxiv.org/pdf/1908.02518); `GUDHI`, C++17 + Python, **MIT** — [GitHub](https://github.com/GUDHI/gudhi-devel)).
- **Project question:** None of `C`, `S`, `E`, or `U` reduces to simplicial persistent homology. (Bauer's *apparent-pair* local cancellation in `Ripser` is an instructive conceptual cousin of Rauzy–Veech contraction — avoiding materialising the full boundary matrix by local pairing — not a library to import.)
- **What "fast" would unlock:** Nothing — `Perseus`, `Ripser`, and `GUDHI` are all already fast C++.
- **Porting cost & fast equivalent:** Already native.
- **Label:** `[CONJECTURED]` (not applicable to `mibz2`; see §4).

---

### Area 11 — Tropical and polyhedral geometry

#### 11.1 `TPLib` (Tropical Polyhedra Library, Xavier Allamigeon)
- **What it computes:** Tropical (max-plus) polyhedra in $\mathbb{R}_{\max}^d$, extreme points and rays via tropical double description, minimal half-space representations, tropical polar cones, and **tropical Fourier–Motzkin projection** for disjunctive max-plus invariants in static analysis and real-time verification ([OPAM](https://opam.ocaml.org/packages/tplib/); [Allamigeon thesis](http://www.cmap.polytechnique.fr/~allamigeon/publications/thesis.pdf); [HAL 2014](https://inria.hal.science/hal-01087367v1/document)).
- **Language, speed today, licence:** **OCaml** (`ocaml >= 3.12`, `num`/`zarith`/`mlgmp`), single-threaded, legacy build (`ocamlbuild`); LGPL/open-source academic package.
- **Project question:** Synthesising **disjunctive max-plus invariants** and proving upper bounds $U$ for the max-plus pipeline recurrence (`leads/structure-and-function.md` line 76; journal §§70, 72) and polyhedral regime boundaries where $\max(S, W/k, R)$ switches binding term (`measurements.md` §6).
- **What "fast" would unlock:** A Rust port using Allamigeon's almost-linear directed-hypergraph terminal-SCC double-description algorithm ([arXiv:1112.1444](https://arxiv.org/pdf/1112.1444)) unlocks fast max-plus invariant synthesis over 10–30 pipeline variables (worker completion dates, window release dates, head-of-line ages). **Honourable mention (#6):** directly complementary to Network Calculus (#1), though narrower because it operates on static finite-dimensional max-plus polyhedra rather than arbitrary-length streaming arrival/service curves.
- **Porting cost & fast equivalent:** Low–Moderate (~3–5k LOC Rust, 2–4 weeks). No fast Rust/C++ equivalent specifically for tropical double description + Fourier–Motzkin (`polymake` has heavier Perl-wrapped tropical polytopes).
- **Label:** `[ARGUED]` (honourable mention #6).

#### 11.2 `polymake` and `gfan` (and `Helmer`)
- **What they compute:** Convex polytopes, polyhedral fans, tropical hypersurfaces/intersection theory (`polymake`, [GitHub](https://github.com/polymake/polymake), C++ 74.2% + Perl 18.3%, **GPL-2.0+**, [arXiv:1612.02581](https://arxiv.org/html/1612.02581v1)); Gröbner fans and tropical varieties of polynomial ideals (`gfan`, [homepage](https://users-math.au.dk/~jensen/software/gfan/gfan.html), C++, **GPL-2/3**); Whitney stratifications of algebraic varieties (`Helmer`, C++ + msolve/FLINT, **GPL-3.0**, `notes-for-neon.md`).
- **Project question:** `polymake`'s tropical polytopes overlap weakly with max-plus geometry; `gfan` and `Helmer` apply only if a smooth/polynomial performance surrogate is introduced (`notes-for-neon.md`).
- **What "fast" would unlock:** Nothing competitive with `TPLib` or Network Calculus (#1); `polymake`'s ~500k LOC Perl/C++ stack is massive overkill for `mibz2`, and `gfan`/`Helmer` are already C++.
- **Porting cost & fast equivalent:** Prohibitive (`polymake`); already C++ (`gfan`, `Helmer`).
- **Label:** `[CONJECTURED]` (see §4).

---

### Area 12 — SAT, SMT, and exhaustive-enumeration frameworks

#### 12.1 `CaDiCaL` / `Kissat`, `Z3` / `cvc5`, `SMS` (SAT Modulo Symmetries)
- **What they compute:** CDCL SAT (`CaDiCaL`, [GitHub](https://github.com/arminbiere/cadical), C++, **MIT**); SMT (`Z3`, C++, **MIT**; `cvc5`, C++, **BSD-3**); isomorph-free combinatorial generation and alternating $\exists\forall$ search via dynamic symmetry breaking + Co-Certificate Learning inside CaDiCaL (`SMS`, Kirchweger & Szeider, [ACM TOCL 2024](https://d-nb.info/1367527805/34), [arXiv:2306.10427](https://arxiv.org/html/2306.10427v2), [GitHub](https://github.com/markirch/sat-modulo-symmetries), C++, **MIT**).
- **Project question:** Positive necessity proof in the referee's class (`leads/structure-and-function.md` lines 121–127; journal §72: *"any inversion within $K$ sweeps and $s$ cursors makes at least $h_1 - c$ addressed accesses"*).
- **What "fast" would unlock:** `CaDiCaL`, `Z3`, `cvc5`, and `SMS` are **already state-of-the-art native C++**! The gap is that `SMS`'s native C++ propagator hard-codes graph/matroid adjacency canonisation — extending its IPASIR-UP dynamic symmetry breaking + CEGIS to $(K, s)$ streaming cursor machines is the **Rank #5** synthesis opportunity (paired with `Rosette`/`Alloy` below).
- **Porting cost & fast equivalent:** Already native C++ (use directly; only the custom cursor-machine symmetry propagator / CEGIS encoder is missing).
- **Label:** `[ESTABLISHED]` for SAT necessity proofs (journal §72).

#### 12.2 `Alloy` / `Kodkod`
- **What they compute:** Bounded relational model finding with automatic symmetry breaking compiled to SAT ([GitHub](https://github.com/AlloyTools/org.alloytools.alloy), Java 90.1% + C++ 5%, **MIT**).
- **Project question:** Specifying and exhaustively enumerating small streaming cursor machines (`leads/structure-and-function.md` lines 121–127) or finite-state buffer-ownership protocols (`levers.md` lever 4b).
- **What "fast" would unlock:** Replacing Java/Kodkod's slow relational-to-CNF lowering and static symmetry breaking with a Rust CEGIS + dynamic-symmetry-breaking engine (Rank #5) speeds up encoding/enumeration by $10^2\text{–}10^3\times$ and enables $\exists\forall$ quantifier alternation.
- **Porting cost & fast equivalent:** Moderate for the core relational/DSL-to-SAT CEGIS engine (Rank #5).
- **Label:** `[ARGUED]` (folded into Rank #5 alongside `Rosette`).

---

### Area 13 (Added) — Deterministic Network Calculus & Real-Time Calculus

#### 13.1 `Nancy` (Zippo & Stea) + `Panco` (Bouillard) + `NCorg DNC` / `DiscoDNC` (Bondorf et al.) + `RTC Toolbox` (Wandeler & Thiele) — **RANK #1**
- **What they compute:**
  - Exact $(\min, +)$ and $(\max, +)$ convolution, deconvolution, sub-additive closure, pseudo-inverse, composition, backlog bounds ($\sup_t [\alpha(t) - \beta(t)]$), delay bounds (horizontal deviation), and **flow-controlled (window-constrained) feedback-loop fixed-point bounds** on ultimately pseudo-periodic (UPP) piecewise-affine arrival/service curves (`Nancy`, [GitHub](https://github.com/rzippo/nancy), SoftwareX 2022, Zippo & Stea 2023 *Computationally efficient worst-case analysis of flow-controlled networks with Network Calculus*, [ECRTS 2026](https://drops.dagstuhl.de/storage/00lipics/lipics-vol375-ecrts2026/LIPIcs.ECRTS.2026.5/LIPIcs.ECRTS.2026.5.pdf); `RTC Toolbox`, [ETH MPA](https://www.mpa.ethz.ch/));
  - Tight **FIFO and cyclic-dependency** deterministic network bounds via linear programming (`Panco`, Bouillard, *Performance Evaluation* 2022, [GitHub](https://github.com/anne-bou/panco), [arXiv:2605.08944](https://arxiv.org/html/2605.08944));
  - Automated multi-hop server-graph bounds (TFA, SFA, PMOO, TMA, FIFO-LUDB) (`NCorg DNC` / `DiscoDNC`, [GitHub](https://github.com/NetCal/DNC), [ECRTS 2026](https://drops.dagstuhl.de/storage/00lipics/lipics-vol375-ecrts2026/LIPIcs.ECRTS.2026.6/LIPIcs.ECRTS.2026.6.pdf)).
- **Language, speed today, licence:**
  - `RTC Toolbox`: **closed-source MATLAB + Java** (abandoned/legacy MATLAB R14SP3+);
  - `NCorg DNC` / `DiscoDNC`: **Java** (**LGPL-2.1**), JVM overhead, optional commercial CPLEX for LUDB;
  - `Panco`: **Pure Python** shelling out to external `lp_solve` binaries (**BSD-3-Clause**), slow for sweeps;
  - `Nancy`: **C# 12 / .NET 8** (**MIT**), fast within .NET using exact `BigInteger` rationals, but locked in the .NET runtime and lacking `Panco`'s cyclic/FIFO LP relaxation.
- **Project question:**
  - **Proved Upper Certificates ($U$)** (`decoder-as-hybrid-system.md`, journal §70 — where we have **zero** certificates today);
  - **`measurements.md` §2 explicitly points here:** *"in network calculus, the backlog bound is the vertical gap between the arrival and service curves. Admission itself shapes the observed issue process, so a frontier fitted under the current window does not size a different window."* Flow-controlled DNC models `issued - taken < W` and `ready_bytes < cap` directly as a min-plus window-feedback loop, and strict rate-latency service curves $\beta_{R,T}$ handle arbitrary contention rate intervals $\dot x \in [\underline{\rho}, \overline{\rho}]$!
  - **Ma/Khammash design-space enumeration** (`leads/structure-and-function.md` lines 70–75): scoring every `(obligations × placements × k × W × cap)` candidate by a *proved* upper certificate $U$ alongside lower bound $L$.
- **What "fast" would unlock:** A native Rust crate (`nc-cert`, with `pyo3` bindings) combining `Nancy`'s exact UPP $(\min,+)$/$(\max,+)$ + window-feedback closure with `Panco`'s FIFO/cyclic PLP builder (calling native `HiGHS` in-process instead of Python → `lp_solve` subprocesses) runs **100×–1000× faster than `Panco`/Java/MATLAB** with zero JVM/.NET dependency, unlocking **certified sweeps over $10^4\text{–}10^5$ pipeline designs and per-input-class worst-case certificates $U$ across all 1,724 blocks**!
- **Porting cost & fast equivalent:** Low–Moderate (~3–5k LOC Rust, 2–4 weeks; exact differential test oracles in `Nancy`, `NCorg DNC`, and `Panco`). **No native Rust or C++ open-source equivalent exists** (C# `Nancy` and Java `NCorg DNC` are the only modern open-source engines).
- **Label:** `[ESTABLISHED]` for certificates & admission-window bounds (`measurements.md` §2, §70); `[ARGUED]` for design-space ranking tightness on FIFO in-order resequencing.

---

### Area 14 (Added) — Microarchitectural basic-block analyzers & analytical cache models

#### 14.1 `uiCA` (uops.info Code Analyzer, Abel & Reineke) — **RANK #2**
- **What it computes:** Cycle-accurate steady-state basic-block throughput and cycle-by-cycle pipeline execution traces (MITE/DSB/LSD front-end, micro/macro-fusion, rename, ROB, RS, port-mapping scheduler, loop-carried dependency chains) calibrated from empirical `uops.info` microbenchmarks across Intel Core microarchitectures from Sandy Bridge through **Skylake / Coffee Lake / Comet Lake / Rocket Lake** — achieving **~1% prediction error**, an order of magnitude more accurate than `llvm-mca` or `OSACA` ([GitHub](https://github.com/andreas-abel/uiCA); [ICS 2022 paper](https://publikationen.sulb.uni-saarland.de/bitstream/20.500.11880/33280/1/3524059.3532396.pdf)).
- **Language, speed today, licence:** **Pure Python** (`uiCA.py`, `facile.py`, `x64_lib.py`) using Intel XED; **AGPL-3.0** (`uops.info` XML data is separately public). Takes ~0.2–2 s per basic block in Python — **500×–2000× too slow** for inner-loop superoptimisation or sweeping thousands of instruction schedules.
- **Project question:**
  - **T4 (Scan, `src/huff.rs`), T2 (Construction arithmetic `A6a`/`A6b`, `src/open.rs`), T3 (MTF/RLE2, `src/runs.rs`), and AVX2 `copy_if` compaction (`reports §4` item 6):** our fixed toy machine is an Intel **i9-10900K (Comet Lake / Skylake-derived)** (`reports §5`). `uiCA` directly models the exact front-end/port/dependency bottleneck of `S` (`1.41–1.51 ns/sym`, `#870`, §82) and `A6b` (~103 cyc/IET-step, `IDEAL-PARTS-1.md`).
  - **`council/comparison.md` `P-cal` bridge:** turns abstract instruction/dependency proposals into exact i9-10900K cycle counts offline, before burning hours on `just submit` workstation runs.
- **What "fast" would unlock:** At ~10–50 µs/block in Rust (clean-room implementation of the ICS '22 parametric pipeline model + `iced-x86` + compiled `uops.info` tables), `uiCA` becomes an **embeddable microarchitectural cost function inside an x86-64/AVX2 superoptimiser, `egg` cost extractor, or automated kernel-schedule search** over Huffman bit-extraction, MTF SIMD, AVX2 left-pack, and `A6b` hash/IET loops! *(Scope caveat: assumes L1 cache hits, so L3/DRAM miss stalls in `A6c`/`E` remain priced by the separate memory/MLP latency term `L* = (ℓ+c)/c` from §72.)*
- **Porting cost & fast equivalent:** Low–Moderate (~3–4k LOC Rust, 2–3 weeks; clean-room from ICS '22 paper avoids AGPL-3.0). `llvm-mca` (C++, Apache-2.0) is fast but has ~10× higher error on Intel Skylake/Comet Lake front-end and port-assignment quirks (Abel & Reineke ICS '22 §6); `OSACA` ([GitHub](https://github.com/RRZE-HPC/OSACA)) is also pure Python.
- **Label:** `[ESTABLISHED]` for i9-10900K `S`/`C`/`M` cycle modelling & `P-cal` bridge; `[ARGUED]` as inner cost oracle for automated AVX2/bit-loop search.

#### 14.2 `HayStack` (Gysi et al., analytical cache-miss modelling)
- **What it computes:** Exact analytical L1/L2/L3 cache-miss counts for affine loop nests (fully associative LRU) using polyhedral Presburger counting (`ISL` / `Barvinok`) ([GitHub](https://github.com/spcl/haystack); Gysi et al. PLDI 2019, [arXiv:2001.01653](https://arxiv.org/html/2001.01653v1)).
- **Language, speed today, licence:** C++ (built on LLVM/Clang + ISL/Barvinok + GMP/NTL).
- **Project question:** Cache-miss prediction for `C` (`A6c`) and `E`.
- **What "fast" would unlock:** Nothing — already C++, and `A6c`/`E`'s data-dependent random gathers (`LF[i]`) are **non-affine**, outside polyhedral Presburger counting!
- **Porting cost & fast equivalent:** Already C++; wrong access class (affine vs data-dependent gathers).
- **Label:** `[CONJECTURED]` (non-fit for indirect gathers).

---

### Area 15 (Added) — Algebraic transform-schedule compilers

#### 15.1 `SPIRAL` (CMU — Püschel, Moura, Franchetti et al.)
- **What it computes:** Algebraic representation of linear signal transforms (SPL / $\Sigma$-SPL Kronecker factorisations), rewrite search over schedules, and platform-tuned SIMD/multicore C code generation ([GitHub](https://github.com/spiral-software/spiral-software); [Proc. IEEE 2005, 2018](https://www.spiral.net/); [bibliography](https://spiral-software.github.io/spiral-software/bibliography.html)).
- **Language, speed today, licence:** **Embedded GAP interpreter** (`gap/bin/gap`) + Python profiler; **Simplified BSD**.
- **Project question:** Cited in `leads/structure-and-function.md` (line 97) as an exemplar of algebraic algorithm/schedule enumeration.
- **What "fast" would unlock:** Very little for `mibz2` — SPIRAL's SPL algebra targets structured *linear* transforms (DFT, NTT, SpMV), not nonlinear variable-length Huffman bitstreams or data-dependent BWT interval induction. Porting SPIRAL out of GAP would take person-years for an algebra that does not express `src/huff.rs` or `src/open.rs`.
- **Porting cost & fast equivalent:** Very high (person-years); for general algebraic rewriting in Rust, `egg`/`egglog` already exists.
- **Label:** `[CONJECTURED]` (methodological inspiration only; see §4).

---

### Area 16 (Added) — Systems-biology structural / identifiability toolboxes

#### 16.1 `CRNT Toolbox`, `STRIKE-GOLDD` (MATLAB), `StructuralIdentifiability.jl` (Julia)
- **What they compute:** Deficiency theorems for mass-action chemical reaction networks (`CRNT`), Lie-derivative / differential-algebraic structural identifiability of nonlinear/rational ODEs (`STRIKE-GOLDD`, MATLAB, GPL-3.0, [GitHub](https://github.com/afvillaverde/strike-goldd), [arXiv:2207.07346](https://arxiv.org/html/2207.07346v1); `StructuralIdentifiability.jl`, Julia, [GitHub](https://github.com/SciML/StructuralIdentifiability.jl), [tutorial arXiv:2505.10517](https://arxiv.org/html/2505.10517v4)).
- **Project question:** Checked because `leads/structure-and-function.md` uses Alon (Part III), Ma et al. (2009), Araujo–Liotta (2018), and Khammash (2021).
- **What "fast" would unlock:** Nothing — Khammash and Ma transfer *methodologically* (necessary conditions + finite design-space enumeration), not via mass-action ODE kinetics.
- **Porting cost & fast equivalent:** High; irrelevant mathematical class (see §4).
- **Label:** `[CONJECTURED]` (see §4).

---

## 2. Top-Five Ranking

Ranked by **`(value to measured targets: C [T2], S [T4], U [certificates]) × (likelihood the port pays)`**:

| Rank | Package(s) to port | Target(s) moved | Current blocker | What a fast Rust port unlocks | Port cost | Fit label |
|---:|---|---|---|---|---|---|
| **1** | **Deterministic Network Calculus (`Nancy` [C#/.NET, MIT] + `Panco` [Python/lp_solve, BSD-3] + `NCorg DNC` [Java, LGPL-2.1])** | **Certificates ($U$)** (`§70`, `measurements.md` §2) + **Design-space enumeration** (`leads/structure-and-function.md`, `S × C` trade-offs) | Split across .NET, Java, closed MATLAB (`RTC`), and slow Python+`lp_solve`; zero native Rust/C++ UPP+window+LP engine | **100×–1000× faster certification:** sweeps $10^4\text{–}10^5$ `(obligations × placements × k × W × cap)` policies and produces **proved upper certificates $U$** per input class with window admission feedback | Low–Med (2–4 wks, ~3–5k LOC) | `[ESTABLISHED]` / `[ARGUED]` |
| **2** | **`uiCA` (uops.info Code Analyzer — pure Python, AGPL-3.0; clean-room from ICS '22 paper + `uops.info` XML)** | **Scan (`S`, T4)**, **Construction arithmetic (`C`, T2 `A6a`/`A6b`)**, **MTF (`M`, T3)**, **AVX2 `copy_if`**, **`P-cal` bridge** (`council/comparison.md`) | Pure Python cycle simulator (~0.2–2 s/block); `llvm-mca` has ~10× higher error on Intel Skylake/Comet Lake (our i9-10900K) | **500×–2000× faster (~10–50 µs/block):** turns `uiCA` into the live cost oracle inside `egg`/stochastic x86-64/AVX2 kernel search and offline `P-cal` pricing without workstation runs | Low–Med (2–3 wks, ~3–4k LOC) | `[ESTABLISHED]` / `[ARGUED]` |
| **3** | **`surface_dynamics` (SageMath / Python 85.6%, GPL-2.0+) — integer-IET / Rauzy-schedule / Origami subset** | **Construction (`C`, T2)** (`0.579` at 1w, `0.754` at 6w) & **Emission (`E`, T1)** (`2026-11-10`, `2026-11-20` blogs) | Locked in Sage + GMP + PARI + GAP + PPL; only practical for $d \le 20$ symbolic IETs | Corpus-wide search over generalised induction orders (`1.812 -> 1.220` floor), Zorich acceleration, and origami/cylinder reductions across all 233–1,724 blocks ($r \sim 60\text{k}$) | Med (3–5 wks, ~4–6k LOC) | `[ESTABLISHED]` / `[CONJECTURED]` |
| **4** | **`IMITATOR` (OCaml + PPL, GPL) — parametric multi-rate stopwatch automata** | **Certificates ($U$)** & **Certified polyhedral regime maps** (`decoder-as-hybrid-system.md`, `levers.md`, `notes-for-neon.md`) | Single-threaded OCaml + PPL; lacks $k!$ worker-permutation symmetry reduction; chokes at $k \ge 4$ | Piecewise-linear analogue of Nanda's **Helmer**: exact polyhedral regime partitions of $(r_s, r_w, W, k)$ certifying deadlock-freedom, lever 4b ownership, and worst-case $U$ | Med–High (6–10 wks, ~12–18k LOC) | `[ESTABLISHED]` / `[ARGUED]` |
| **5** | **`Rosette` (Racket, BSD-3) / `Alloy`+`Kodkod` (Java, MIT) fused with `SMS`-style dynamic symmetry breaking** | **Construction (`C`, T2) & Emission (`E`, T1)** — Khammash/Ma/Bundala referee-class enumeration & SAT positive necessity proof (`leads/structure-and-function.md`, §72) | Racket/JVM symbolic-term blowup; `SMS` C++ hard-codes graph adjacency matrices, not cursor-machine rule tables | Native Rust ORG symbolic evaluator + CEGIS + custom IPASIR-UP symmetry propagator over `CaDiCaL` for $(K, s)$-cursor BWT machines up to $n \approx 10\text{–}12$ | Med (4–6 wks, ~4–6k LOC) | `[ARGUED]` / `[CONJECTURED]` |

*Honourable mention (#6):* **`TPLib`** (Xavier Allamigeon, OCaml — tropical polyhedra & tropical Fourier–Motzkin elimination), which cleanly synthesises static disjunctive max-plus invariants (~3–5k LOC Rust) but is narrower than #1 (streaming UPP arrival/service curves + window feedback + FIFO LP).

### Why the ranking orders them this way
- **#1 beats #4 (`IMITATOR`)** because Network Calculus handles all $J = 1{,}724$ blocks without discrete state-space explosion, directly models our admission window (`issued - taken < W`) and `measurements.md` §2's arrival/service curves, costs a third as much to port, and has three independent exact oracles.
- **#2 (`uiCA`) beats #3 (`surface_dynamics`)** because #2 hits **both** co-limiting bottlenecks (`S` and `C`'s `A6a`/`A6b` arithmetic) plus `M` and AVX2 compaction on our exact i9-10900K machine with near-certain engineering/pricing payoff, whereas #3's mathematical upside on `C` is bounded by record 119's negative closure theorem inside rigid-interval sweeps.
- **#5 (`Rosette`/`Alloy` + `SMS`)** has the highest *theoretical breakthrough* ceiling (a machine-checked positive necessity theorem or a novel small-$n$ cursor gadget outside record 119's class), but sits at #5 because the $10^{50}$ rule-table scaling barrier (§72) lowers the probability that $n \le 12$ separates asymptotic high-entropy behaviour from small-$n$ lookup artefacts.

---

## 3. Deep Dives for the Top Two

### Deep Dive #1 — Deterministic Network Calculus Engine (`Nancy` + `Panco` → Rust)

#### Smallest experiment before porting anything
Run stock **`Nancy`** (via `.NET 8` / `nancy-playground`, [GitHub](https://github.com/rzippo/nancy-playground-artifact)) and stock **`Panco`** (Python + `lp_solve`, [GitHub](https://github.com/anne-bou/panco)) on **stage 1's actual measured traces** (`character/run-20261125T001135Z`, 1,724 blocks at $k \in \{1, 6, 8, 10\}$):
1. Extract empirical block-arrival / burst envelopes $\alpha(t)$ (from `measurements.md` §2, already scripted in `math-targets/scripts/stage1_analysis.py`) and strict rate-latency service curves $\beta_S(t), \beta_{W,k}(t), \beta_D(t)$ from the measured min/max per-block service times and contention spreads at $k \in \{1, 6, 8, 10\}$.
2. Compute in `Nancy`:
   - Tandem min-plus convolution $\beta_{\text{pipe}} = \beta_S \otimes \beta_{W,k} \otimes \beta_D$;
   - Window-flow-control feedback closure for `issued - taken < W = 4k` (Zippo & Stea 2023 algorithm);
   - Deterministic upper certificate $U_{\text{NC}}(k)$ for total completion time of 1,724 blocks and max ready-queue backlog.
3. Compute in `Panco` (PLP mode): FIFO-constrained delay/backlog bound capturing in-order reader resequencing (`D`).
4. **Pass / kill gate (1–2 days of work, zero Rust written):**
   - **PASS (proceed to port):** $U_{\text{NC}}(k) / T_{\text{wall}}(k) \le 1.5$ across $k \in \{1, 6, 8, 10\}$, $U_{\text{NC}}$ reproduces the knee near $k = 7\text{–}8$, and $U_{\text{NC}}$ strictly separates at least two candidate policies (e.g., `S`+`C` co-improvement vs `C`-only at 8w; `W = 4k` vs `W = 2k`).
   - **KILL (do not port):** In-order head-of-line heterogeneity makes both curve and PLP bounds vacuous ($U / T_{\text{wall}} > 3\times$) or blind to the $k = 8$ crossover.

#### What porting would involve
- **Core algorithms (~3–5k LOC Rust):**
  1. **Canonical UPP piecewise-affine curve representation** (`Curve = { prefix: Vec<Segment>, period_start, period_len, period_height }`, exact rational slopes/intercepts) following Bouillard–Thierry (2008) and `Nancy` (`Sequence` / `Curve`).
  2. **Closed $(\min, +)$ and $(\max, +)$ operators:** pointwise $\min/\max$, sum/difference, $(\min,+)$ convolution (`Isospeed` isomorphism optimisation, Zippo–Nikolaus–Stea 2023), $(\min,+)$ deconvolution, non-decreasing upper/lower closures, horizontal & vertical deviation, and **sub-additive closure** $\overline{f} = \inf_{n \ge 0} f^{(n)}$ (required for window flow control).
  3. **Flow-controlled window feedback solver:** fixed-point / closed-form service curve for a $W$-credit window + byte-cap shaper (Zippo & Stea 2023).
  4. **FIFO/cyclic PLP certificate generator** (from `Panco`, Bouillard 2022): builds the sparse polynomial-size linear program directly in Rust memory and solves in-process (no temp files or `lp_solve` subprocesses).
- **Dependencies:**
  - Exact rational arithmetic: `i128` fast-path with `num-bigint`/`num-rational` fallback (or `rug`/GMP if arbitrary-precision denominators grow during repeated convolution);
  - LP solver for `Panco` PLP: `highs` (via `good_lp` or direct C-API FFI to HiGHS, MIT) or pure-Rust `clarabel`;
  - Parallelism & bindings: `rayon` for sweeping $10^4\text{–}10^5$ design points; `pyo3` for Python notebooks alongside `stage1_analysis.py`.
- **Test oracle against the originals:**
  - Differential fuzzing + corpus regression suite checking **exact rational bit-for-bit equality** of every curve operator (convolution, deconvolution, sub-additive closure, window-feedback service curve, horizontal/vertical deviation) against **`Nancy`** (`.NET 8`, MIT) on 10,000 randomised UPP curves plus `Nancy`'s own unit test suite;
  - Exact bound comparison against **`NCorg DNC`** (`DNC_func_tests` submodule) on feedforward topologies and against **`Panco`** on FIFO/cyclic topologies (tolerance $\le 10^{-9}$ for floating-point LP solvers, plus LP dual-certificate verification for soundness!).

---

### Deep Dive #2 — Clean-Room Rust `uiCA` (Intel Skylake / Comet Lake Microarchitectural Simulator)

#### Smallest experiment before porting anything
Run stock Python **`uiCA.py -arch CFL`** ([GitHub](https://github.com/andreas-abel/uiCA); Coffee Lake shares Comet Lake / i9-10900K's exact Skylake-derived core pipeline) on 4–6 compiled basic blocks extracted via `objdump`/`cargo asm` from `mibz2`:
1. `decode_group_win` in `src/huff.rs` (shipped 2-symbol LUT loop, measured `1.41–1.51 ns/sym`, ~`7.4 cyc/sym`, `13.7 cyc/lookup` in §82 / `IDEAL-PARTS-1.md`);
2. The 3-symbol LUT loop from `#870` (measured regression in journal §82);
3. The `A6b` induction inner step in `src/open.rs` (~`103 cyc/step` total, split into compute vs cache-miss residual);
4. The MTF / RLE2 hot loop in `src/runs.rs` and a candidate AVX2 `copy_if` left-pack kernel (`reports §4` item 6).
5. Compare `uiCA` and `llvm-mca -mcpu=skylake` against hardware-counter cycles/iteration on the i9-10900K.
6. **Pass / kill gate (1 day of work, zero Rust written):**
   - **PASS (proceed to port):** `uiCA` predicts the L1-resident Huffman loop (`src/huff.rs`) within **$\le 5\%$** of measured cycles/lookup, correctly predicts the direction and magnitude of `#870`'s 3-symbol regression, and materially beats `llvm-mca` (e.g., `llvm-mca` error $> 15\%$ or wrong sign on `#870`).
   - **KILL (do not port):** `llvm-mca` already matches `uiCA` within $3\%$ on all `mibz2` hot loops (in which case we just embed C++ `llvm-mca` directly), OR branch mispredictions / L2–L3 misses dominate even the scanner loop so badly that L1-hit basic-block throughput has no predictive power.

#### What porting would involve
- **Licence hygiene:** Stock `uiCA` is **AGPL-3.0**. Because Abel & Reineke's ICS '22 paper ([PDF](https://publikationen.sulb.uni-saarland.de/bitstream/20.500.11880/33280/1/3524059.3532396.pdf)) publishes the full parametric pipeline model (§4) and `uops.info`'s measured instruction XML tables are separately published, write a **clean-room Rust implementation** (MIT/Apache-2.0) of the ICS '22 model consuming `uops.info` XML — using stock `uiCA.py` strictly as an external black-box differential test oracle!
- **Core algorithms (~3–4k LOC Rust):**
  1. **Offline table compiler (`build.rs` or generator):** parses `uops.info` `instructions.xml` once into compact static Rust/binary tables mapping `(iclass, operand_pattern) -> (uops, port_masks, latencies, macro/micro-fusion flags)` for `SKL`/`CFL`/`RKL`.
  2. **Fast x86-64 decoder:** `iced-x86` (pure Rust, MIT, zero Intel XED build dependency!) decodes raw machine-code slices in nanoseconds and maps instructions to table entries.
  3. **Cycle-by-cycle pipeline simulator** (ICS '22 §4.3): flat fixed-size ring buffers (no heap allocations per cycle!) modelling 16B fetch / MITE / DSB / LSD front-end, IDQ, 4-wide issue/rename/allocate with zero-idiom & move-elimination, 224-entry ROB, 97-entry RS, port-pressure tie-breaking scheduler (8 ports on Skylake/Comet Lake), and steady-state loop convergence detection.
  4. **Batch/search API:** zero-copy slice evaluation (`eval_throughput(&[u8]) -> f64`) + `rayon` parallel batch scoring, directly callable as the fitness function for an `egg` extractor or AVX2/bit-manipulation stochastic search.
- **Dependencies:** `iced-x86` (MIT), `quick-xml` (MIT, offline table gen only), `rayon` (MIT/Apache-2.0), optional `pyo3` bindings.
- **Test oracle against the original:**
  - Black-box differential test against stock `uiCA.py -arch CFL -TPonly` across **1,000+ basic blocks from BHive** (the benchmark suite in ICS '22 §5) plus every hot loop compiled from `mibz2` (`src/huff.rs`, `src/open.rs`, `src/runs.rs`, AVX2 kernels), asserting $\le 0.5\%$ throughput agreement (exact match on non-stochastic port tie-breaks) and verifying $\ge 500\times$ wall-time speedup.

---

## 4. Packages That Look Relevant — But Aren't (and Why)

| Package(s) | Why it looks relevant | Why NOT to port it for `mibz2` |
|---|---|---|
| **CORA, SpaceEx, Flow\*, JuliaReach** | We just modelled the decoder as $\mathcal{H} = (C, F, D, G)$ from Goebel–Sanfelice–Teel (`decoder-as-hybrid-system.md`). | Built for continuous ODE flowpipes with few discrete modes; our decoder has trivial countdown flows and huge discrete state (`W = 4k`, ready set $R$). Also `ReachabilityAnalysis.jl` (MIT) and `SpaceEx`/`Flow*` are already fast/native. Use DNC (#1) or `IMITATOR` (#4) instead. |
| **Helmer (`WhitneyStratifications`)** | Direct inspiration for this survey (`notes-for-neon.md`). | Already fast C++ (Nanda), and stratifies *smooth algebraic varieties*, whereas `mibz2`'s arithmetic is discrete and its performance model is piecewise-linear/max-plus (`notes-for-neon.md`). |
| **Perseus, Ripser, GUDHI (TDA / discrete Morse)** | Direct inspiration for this survey; apparent pairs resemble local contraction. | All three are already fast C++, and none of `C`, `S`, `E`, or `U` reduces to simplicial persistent homology. |
| **LINE, JMT, LQNS** | Jain's course and `programme.md` step 6 mention queueing networks and MVA. | `briefs/character-review-decisions.md` (`programme.md` lines 44–49) ruled out exact product-form MVA for the real window-bounded finite pipeline; these give stochastic averages ($\hat T$), not deterministic certificates ($U$), and already solve surrogates in ms–s. |
| **Walnut** | Famous Java theorem prover for combinatorics on words. | Decides first-order logic over *$k$-automatic / Pisot infinite sequences*; real Wikidata/prose/hex blocks are arbitrary finite high-entropy strings outside automaticity. |
| **GAP / Magma** | BWT inversion is permutation inversion; MTF generates a transformation monoid. | A single $n = 900{,}000$ LF permutation needs a 10-line $O(n)$ scan, not Schreier–Sims; finite transformation monoids already have fast AVX-accelerated C++ **`libsemigroups`**. |
| **SPIRAL (GAP)** | Cited in `leads/structure-and-function.md` for algebraic algorithm/schedule search. | Its SPL/Kronecker algebra is specialised for *linear* transforms (DFT/NTT/SpMV), not variable-length Huffman bitstreams or indirect BWT interval induction; person-years to port. |
| **polymake / gfan** | Tropical geometry = max-plus geometry. | `gfan` computes algebraic Gröbner fans; `polymake` is a ~500k LOC Perl/C++ polyhedral suite whose overhead dwarfs `mibz2`'s needs — `TPLib` (#6) or DNC (#1) hits our exact max-plus/min-plus objects in ~3–5k LOC. |
| **STOKE / Souper** | Classic superoptimisers for hot loops. | `STOKE` is abandoned GCC-4.9 C++11; `Souper` was archived Oct 2025 without SIMD/memory. Native C++ **`Minotaur`** already superoptimises SIMD LLVM IR (`rustc --emit=llvm-bc`); the missing piece to port is `uiCA` (#2). |
| **FAdo / Vcsn / SVPAlib** | Huffman is an FSM; cross-stage fusion looks like transducer composition. | Within-block parallel Huffman FSM composition was ruled out in §72 (50-symbol selector count); MTF has $256!$ states (non-regular over bytes); `Awali`, `OpenFST`, and `Mata` are already fast C++. |
| **CRNT Toolbox / STRIKE-GOLDD / StructuralIdentifiability.jl** | `leads/structure-and-function.md` adopts Khammash, Ma et al., and Alon. | Those tools check continuous mass-action ODE kinetics; Khammash/Ma transfer to `mibz2` *methodologically* (discrete rule-table necessity & enumeration -> Rank #5), not via ODE reaction kinetics. |

---

## 5. Already-Fast Native Tools Worth Using Directly (Zero Porting Cost)

Before porting anything, five already-fast open-source C++/Rust tools can be applied directly to `mibz2`'s open tasks:

1. **`Minotaur`** (C++, MIT, [GitHub](https://github.com/minotaur-toolkit/minotaur)): run offline on `rustc --emit=llvm-bc` of `src/huff.rs`, `src/open.rs`, `src/runs.rs` to test for missed LLVM AVX2/scalar peepholes.
2. **`CaDiCaL`** (C++, MIT) + **`SMS`** (C++, MIT, [GitHub](https://github.com/markirch/sat-modulo-symmetries)): direct SAT backend for MONODROMY's positive necessity proof in the referee's class (journal §72).
3. **`TChecker`** (C++, MIT, [ProVerB](https://slebok.github.io/proverb/tchecker.html)) & **`Storm`** (C++20, GPL-3.0, [GitHub](https://github.com/stormchecker/storm)): fixed-rate timed deadlock checks and MDP dispatch-policy comparison (`levers.md` levers 3, 4, 4b).
4. **`MinMaxGD` / `PyMinMaxGD`** (C++, GPL-3.0, [GitHub](https://github.com/GitUALH/MinMaxGD)): exact fixed-rate TEG transfer-series sanity check on the pipeline max-plus recurrence (`leads/structure-and-function.md` line 76).
5. **`egg` / `egglog`** (Rust, MIT, [GitHub](https://github.com/egraphs-good/egg)): Bird–de Moor algebraic fusion/tupling exploration (`leads/structure-and-function.md` lines 26–31).
