# Three performance-analysis courses: what they contain, and what each gives mibz2

*The Coordinator, 24 November 2026, answering Louis's question to ChatGPT Pro: "tell me all about the content of these". Louis's instruction: read the full material, with no summarising subagents. The full question and ChatGPT Pro's first answer are in `../external/chatgpt-performance-resources.md`.*

## What was read

Everything below was read directly with curl and pdftotext:
- **The three pages Louis linked,** in full.
- **For Jain:** all 30 module pages and their topic lists. Six slide decks were read in full: common mistakes and the systematic approach (k_02), experimental design (k_16), fractional factorial designs (k_19), simulation analysis (k_25), operational laws (k_33), and mean-value analysis (k_34).
- **For Harchol-Balter:**
  - the book's full table of contents (*Performance Modeling and Design of Computer Systems*, Cambridge, 2013);
  - the Fall 2025 class page with its schedule, homework list and readings;
  - the theory course's topic list and book list.

**Not read:** the lecture videos, the books' text (beyond contents and slides), and the theory course's 2003 notes page, whose link returns 404.

---

## 1. Raj Jain: *Computer Systems Performance Analysis: Techniques for Experimental Design, Measurement, Simulation, and Modeling*

**What it is.** A five-day course for faculty, given in July 2010 at Guna, India, in an Indo-US engineering-education workshop. It is built on Jain's book *The Art of Computer Systems Performance Analysis* (Wiley, 1991), and states that "the techniques of the course can be used to analyze and compare any type of systems including algorithms". Each module page has slides (PDF), slides with audio, and an MP4 lecture.

### Day 1: method, metrics, workloads (chapters 2–6)

**Common mistakes (ch. 2).** Twenty-two mistakes, among them: no goals, biased goals, an unsystematic approach, the wrong metrics, an unrepresentative workload, the wrong technique, overlooked parameters, ignoring significant factors, inappropriate experimental design or level of detail, no sensitivity analysis, ignoring variability, outliers mistreated, assumptions omitted. A 23-item checklist mirrors them. Jain's **systematic approach** has ten steps, repeated:
1. state goals and define the system;
2. list services and outcomes;
3. select metrics;
4. list parameters;
5. select factors to study;
6. select the evaluation technique;
7. select the workload;
8. design experiments;
9. analyse and interpret;
10. present results.

**Techniques and metrics (ch. 3).** Choosing between measurement, simulation and analytic modelling. "Three rules of validation": don't trust one technique until another confirms it. Metric classes: rate, time and resource per service. Capacity and utility.

**Workloads (ch. 4–6).**
- Types of test workload: instruction mixes, kernels, synthetic programs, application benchmarks.
- Selection: services exercised, level of detail, representativeness, timeliness.
- **Characterisation:** averaging; single- and multi-parameter histograms; **principal component analysis**; Markov models of transitions; **clustering** (sampling, parameter selection, transformation, outliers, scaling, distance, MST clustering and dendrograms, nearest centroid).

### Day 2: data and statistics (chapters 10–15)

- **Data presentation and ratio games (ch. 10–11):** Gantt charts and Kiviat graphs. "Ratio games" are the ways a ratio metric or a percentage can be made to favour either side, and the guidelines against them.
- **Summarising data (ch. 12):**
  - when to use the mean, median or mode;
  - the **geometric mean of ratios**, and the harmonic and weighted-harmonic means;
  - the mean of a ratio;
  - indices of dispersion;
  - quantile–quantile plots.
- **Comparing systems (ch. 13):** confidence intervals, **paired versus unpaired** comparisons, the sample size needed for a mean, and why intervals are preferred to hypothesis tests.
- **Regression (ch. 14–15):**
  - simple and multiple linear regression, with confidence intervals for parameters and predictions;
  - visual tests of the four assumptions: linearity, independent errors, normal errors, constant variance;
  - ANOVA and the F-test, multicollinearity, categorical predictors;
  - curvilinear regression and **Box–Cox transformations**;
  - outliers, and common mistakes in regression.

### Day 3: experimental design (chapters 16–23)

- **Terminology (ch. 16):** response variable, factors, levels, primary and secondary factors, replication, design, **interaction**. Types: simple (one factor at a time), full factorial, fractional factorial. Common mistakes include "interactions ignored" and "one factor at a time".
- **2^k designs (ch. 17):** the sign-table method for effects, and **allocation of variation**: how much of the variance each factor and interaction explains.
- **2^k·r designs, with replication (ch. 18):** experimental error, confidence intervals for effects and contrasts. **Multiplicative models** analysed on logarithms, which suits run-time ratios exactly.
- **2^(k−p) fractional factorials (ch. 19):** a worked 2^(7−4) design, sign tables, **confounding and its algebra, and design resolution**. Case studies include a scheduler design.
- **Other designs (ch. 20–23):**
  - one-factor experiments with ANOVA;
  - two-factor full factorials without and with replication; one case study is cache design alternatives;
  - general k-factor full factorials. Jain's case study 23.1, a paging process, simplifies the model to its significant effects.

### Day 4: queueing models (chapters 30–34)

- **Queueing basics (ch. 30–31):**
  - Kendall notation A/S/m/B/K/SD;
  - Little's law and its proof;
  - stochastic processes: Markov, birth–death and Poisson;
  - M/M/1.
- **Queueing networks (ch. 32):**
  - open, closed and mixed networks;
  - product form, **closed product-form networks**, BCMP networks;
  - the machine-repairman and **central-server models**;
  - types of service centre: fixed-capacity, delay, and load-dependent.
- **Operational laws (ch. 33):**
  - These are laws "that do not require any assumptions about the distribution of service times or inter-arrival times" (Buzen 1976; Denning and Buzen 1978). Their assumptions are *operationally testable*: job flow balance, for example, is checked by comparing arrivals with completions.
  - **The laws:** the utilisation law U = X·S; the **forced-flow law** Xᵢ = X·Vᵢ, with visit ratio Vᵢ; Little's law; the general and interactive response-time laws.
  - **Bottleneck analysis:** the device with the largest total demand Dᵢ = Vᵢ·Sᵢ is the bottleneck, and the asymptotic bounds follow. Throughput X ≤ min(1/D_max, N/(D + Z)). The knee is at N* = (D + Z)/D_max: beyond N* jobs there is certainly queueing.
  - The homework asks which device becomes the bottleneck when one component is made twice as fast or twice as slow. That is modification analysis in miniature.
- **Mean-value analysis (ch. 34):**
  - open networks, with Rᵢ = Sᵢ(1 + Qᵢ) and Qᵢ = Uᵢ/(1 − Uᵢ);
  - **exact MVA for closed networks,** the recursion over population N;
  - approximate MVA;
  - **balanced job bounds,** tighter than the asymptotic bounds.

### Day 5: simulation (chapters 24–28)

- **Introduction (ch. 24):**
  - common mistakes and a checklist;
  - open against closed models, and stable against unstable;
  - simulation types: Monte Carlo, **trace-driven** (its advantages and disadvantages), and **discrete-event**: its components and event-set algorithms.
- **Analysing simulation results (ch. 25):**
  - **verification against validation;**
  - verification techniques: top-down modular design, traces, continuity tests;
  - validation by expert intuition, real-system measurements and theoretical results;
  - **transient removal:** truncation, initial-data deletion, moving averages of independent replications, batch means;
  - terminating simulations and stopping criteria;
  - variance estimation by independent replications, batch means, or regeneration;
  - variance reduction.
- **Random numbers and variates (ch. 26–28).** Generators: LCGs, Tausworthe and GFSR, combined generators, seed selection. Tests: chi-square, Kolmogorov–Smirnov, serial and spectral. Variate generation: inverse transform, rejection, composition, convolution.

### What Jain gives mibz2

1. **A checklist to audit our own method against.** We have already hit several of the 22 mistakes:
   - "one factor at a time" and "interactions ignored": the four-arm test was itself an interaction;
   - "ignoring variability": noise of 4–9% at 10 workers;
   - "biased goals" and "ratio games": Neon's headlines.
2. **Operational laws as the completeness checks,** valid without distributional assumptions and operationally testable. Our decoder is a **closed network**: the window holds a fixed population of blocks in flight. The asymptotic bounds and the knee N* are the analytic counterpart of our crossover k\*, with N playing the role of the population and the scanner of D_max.
3. **Multiplicative models with 2^k·r and fractional factorial designs** for intervention experiments, with **allocation of variation** reported alongside the effects.
4. **The verification and validation discipline for the discrete-event model** (ch. 25), including transient removal, which covers our start-up and drain.
5. **Paired comparisons and geometric means of ratios** for every A/B we run. That is what the CI bench and the four-arm test already do.

---

## 2. Mor Harchol-Balter: 15-857 *Analytical Performance Modeling & Design of Computer Systems* (CMU)

**What it is.** A graduate course, cross-listed with Tepper, that counts towards the CS PhD theory core. Fall 2025: three sessions a week, the Friday one a TA-led recitation. It follows her textbook *Performance Modeling and Design of Computer Systems: Queueing Theory in Action* (Cambridge, 2013) closely.

**Aim.** "Analytic stochastic modeling" to answer design questions *before* building, such as: "One fast disk, or two slow ones? … The best choices are often counter-intuitive."

**Listed topics:**
- **operational laws:** Little's law, the response-time law, asymptotic bounds, **modification analysis**;
- discrete- and continuous-time Markov chains, renewal theory, time-reversibility;
- the Poisson process: memorylessness, Bernoulli splitting, uniformity, **PASTA**;
- queueing: open and closed systems, M/M/1, M/M/k, M/M/k/k, M/G/1 in full, M/G/k, G/G/1, Laplace and z-transforms;
- **networks of queues:** Jackson and classed Jackson;
- simulation: time against ensemble averages, generating random variables, the **inspection paradox**;
- empirical workloads: heavy tails, Pareto, variability and tail behaviour;
- server farms: capacity provisioning, power management, routing;
- **scheduling:** FCFS, priorities, PS, LCFS, FB, SJF, PSJF, SRPT, and the current research (SOAP, Gittins, Whittle);
- data centres: multiserver scheduling, multi-dimensional jobs, the c-μ rule, **parallel jobs with different speedup functions**.

**Assessment.** Weekly homework (40%) and two midterms (25% each), with no final. **"ONE 30-minute individual or pair meeting with Mor where you present an example of queueing in your research or your life"** (5%). LLMs are not allowed for homework.

**The Fall 2025 schedule,** by chapter:
1. motivation; 2. notation; 4. simulating random variables; 5. convergence, and time against ensemble averages;
2. 25 (z-transforms); 6. operational laws and Little's law; 7. **modification analysis**, including "a real-world caching application";
3. 8–9. Markov chains and ergodicity; 11. the exponential and Poisson; 12–13. the conversion to M/M/1, and M/M/1; 14. M/M/k (midterm 1 falls here);
4. 15. capacity provisioning; 16. Burke's theorem; 17. Jackson networks; 18. classed networks; 20. Pareto; 21. phase-type distributions;
5. 23. renewal-reward and M/G/1; 24. task assignment; 26. M/G/1 transforms; 27. busy periods; 28–33. scheduling (midterm 2).

Optional readings include:
- "Closed versus open system models" (NSDI 2006): how the effect of scheduling differs between closed and open systems;
- AutoScale (TOCS 2012);
- optimal power allocation in server farms (SIGMETRICS 2009, "many turtles versus one fast rabbit");
- task-assignment surprises (SIGMETRICS 2009);
- the SOAP scheduling paper.

**The textbook,** 33 chapters in 7 parts:
- **I. Introduction to queueing,** ch. 1–2. Closed networks come in two kinds, interactive and **batch**, with throughput in a closed system and the differences between closed and open.
- **II. Probability background,** ch. 3–5. Generating random variables; sample paths, and time against ensemble averages.
- **III. "The Predictive Power of Simple Operational Laws: What-If Questions and Answers",** ch. 6–7. Little's law for open and closed systems, with proofs; the generalised Little's law; the forced-flow law; **device demands**; **asymptotic bounds and modification analysis for closed systems**.
- **IV. Markov chains to simple queues,** ch. 8–13. DTMCs, ergodicity, PageRank and Aloha, CTMCs, M/M/1 and PASTA.
- **V. Server farms and networks,** ch. 14–19:
  - M/M/k and M/M/k/k, and three server organisations compared;
  - capacity provisioning and square-root staffing;
  - Burke's theorem, tandem servers;
  - Jackson product form;
  - classed networks, including "CPU-bound and I/O-bound jobs";
  - **closed networks of queues, with mean value analysis via the Arrival Theorem.**
- **VI. Real-world workloads,** ch. 20–27:
  - heavy tails and Pareto;
  - **phase-type distributions and matrix-analytic methods,** including time-varying load;
  - BCMP with processor-sharing servers;
  - M/G/1 and the inspection paradox;
  - task assignment;
  - transform analysis;
  - power optimisation.
- **VII. Smart scheduling,** ch. 28–33.

### What 15-857 gives mibz2

1. **Modification analysis for closed systems** (ch. 7) is the formal version of the question we keep asking: *what if this stage were twice as fast?* It works on device demands and the closed-network bounds, without building anything. It is how a mathematical device should be priced before it is built: as a change to one Dᵢ.
2. **Closed against open models** (ch. 2 and the NSDI 2006 paper). Our decoder is closed, with a fixed window. ChatGPT Pro's admission-feedback finding is the closed-network effect: the population is held by whichever block the consumer is waiting for.
3. **MVA through the Arrival Theorem** (ch. 19) is the exact analytic model for the closed product-form approximation of our network, beside the discrete-event simulation.
4. **Classed networks** (ch. 18), for the routed graph. Blocks taking the Open route and blocks taking the column or fallback route are different job classes, with different demands at each activity.
5. **Variability and tails** (ch. 20–21, 23): the inspection paradox and heavy tails. A long-tail block dominates residence times; that is the "slow early block stalls the scanner" mechanism. Phase-type fits of the per-block service times are how to carry that into an analytic model.
6. **"Parallel jobs with different speedup functions"** (the data-centre unit) is the team or helper split within a worker team that MONODROMY's record 125 added.
7. **The research meeting,** "an example of queueing in your research": casting your own system as a queueing model is the exercise we have just done.

---

## 3. Mor Harchol-Balter: 15-849b *Theory of Performance Modeling* (CMU, Fall 2003)

**What it is.** The "extra-theoretical version" of the course, offered "only every 2 or 3 years", for PhD students only (no auditors). It used her own notes, the predecessor of the book, and she has called queueing theory "an old area of mathematics which has recently become very hot". Seven homeworks (60%), two midterms (18% each), one grading meeting (4%).

**Techniques named.** Stochastic analysis, discrete- and continuous-time Markov chains, renewal theory, **product form**, Laplace transforms, **supplementary random variables**, **fluid theory**, scheduling theory, **matrix-geometric techniques**. The emphasis is on realistic, especially **heavy-tailed**, workloads.

**Part I: Operational laws ("laws that hold independent of any assumptions"), back-of-the-envelope bounds, modification analysis.**
- Terminology, and open against closed networks, the closed ones being batch or terminal-driven.
- **Little's law** for open and closed systems, with full proofs.
- Time against ensemble averages, applied to simulation.
- The response-time law, the **forced-flow law**, the **bottleneck law**, combining laws.
- **Asymptotic bounds and modification analysis for closed systems.**

**Part II: Traditional queueing theory,** with exponential service, Poisson arrivals and FCFS:
- DTMCs: stationary equations, ergodicity, time-reversibility, and PageRank as a DTMC;
- the exponential and Poisson process;
- CTMCs and M/M/1; PASTA; finite buffers; M/M/m/m and M/M/m;
- design questions: many hosts or one, balance or not, migrate or not;
- Burke's theorem, tandem queues;
- **Jackson networks by local balance;**
- classed and **closed networks;**
- **BCMP theorems with proof;**
- "what queueing theory can't model";
- **matrix-geometric techniques.**

**Part III: "Modern" applied queueing theory,** with heavy-tailed workloads, correlated arrivals and preemptive disciplines:
- **empirical job-size measurement (Pareto);**
- M/G/1 by the tagged-job method, with M/D/1, M/E_k/1 and M/H₂/1;
- renewal-reward and the **inspection paradox**;
- Laplace and z-transforms, and busy periods;
- the supplementary-variable method;
- **fluid approximations;**
- M/G/k and task assignment;
- confidence intervals ("why confidence intervals don't work"), likely skipped;
- measured arrival processes and **self-similarity**. This covers how to tell whether arrivals are Poisson, how to detect correlation, and estimating the α parameter. Its open research question: "what is the effect of getting the arrival process wrong?";
- scheduling, parts I–IV: PS, preemptive LCFS, priorities, SJF, SRPT, with proofs by Laplace transform.

**The book list** includes:
- Ross; Gallager; Wolff; **Kleinrock vols I–II**; Nelson; Gross and Harris; Kushner (*Heavy Traffic Analysis*);
- **Lazowska, Zahorjan, Graham and Sevcik, *Quantitative System Performance* (1984)**, the classic on operational analysis and MVA;
- Menascé, Almeida and Dowdy; Jain;
- **Law and Kelton, *Simulation Modeling and Analysis*;**
- **El-Taha and Stidham, *Sample-Path Analysis of Queueing Systems*.**

### What 15-849b gives mibz2

1. **Fluid approximations.** At high load, a queueing system behaves like a deterministic flow. A fluid model of the decoder could describe scanner rate against worker demand with the window as a reservoir. It is the right level of abstraction for the 1–20 curve away from the crossover, and it gives k\* in closed form.
2. **Matrix-geometric methods.** For a quasi-birth–death structure such as "blocks in the window × worker states", they give exact stationary distributions. They are heavier than MVA, but exact without product form.
3. **Sample-path analysis** (El-Taha and Stidham): laws that hold on one realised trajectory. Our corpus is one fixed block sequence, not a random process, and ChatGPT Pro warned against M/M/k assumptions for exactly that reason. Sample-path results (Little's law, H = λG) are the right tools for a deterministic trace.
4. **Arrival-process diagnostics** carried over to service times: whether per-block service times are independent, correlated or heavy-tailed decides which analytic model is admissible. It is a cheap check on stage 1's per-block data.
5. **"What queueing theory can't model"** is a reminder to keep the discrete-event simulation primary where the network fails product form. Our routed, feedback-coupled, ordered-consumer network likely does.

---

## How the three fit the programme (`../programme.md`)

| programme step | Jain | 15-857 (book) | 15-849b |
|---|---|---|---|
| 1. task and referee | ch. 2–3: goals, metrics, the 22 mistakes and the ten steps | ch. 1–2 | — |
| 2. toy problem and workload | ch. 4–6: workload characterisation, PCA, clustering of blocks | ch. 20–21: heavy tails, phase-type | measured job sizes |
| 3. structure | ch. 32: open, closed and central-server networks | ch. 2, 17–19: closed and classed networks | product form, BCMP, its limits |
| 4. measurement | ch. 12–13: summaries, paired comparisons, sample size | ch. 5: time against ensemble averages | sample-path analysis |
| 5. design | ch. 16–23: 2^k·r, multiplicative models, **fractional factorial and resolution**, allocation of variation | — | — |
| 6. model | ch. 24–25: discrete-event simulation, verification and validation, transient removal; ch. 33–34: **operational laws, bounds, MVA, balanced job bounds** | ch. 6–7: **operational laws, modification analysis**; ch. 19: MVA by the Arrival Theorem | **fluid**, matrix-geometric |
| 7. where mathematics fits | ch. 33: bottleneck and D_max; the homework's "twice as fast" | ch. 7: **modification analysis** prices a change to one demand Dᵢ before it is built | — |

**Three concrete additions to `briefs/character.md`:**
1. Report **device demands Dᵢ = Vᵢ·Sᵢ** per activity, with the **asymptotic and balanced-job bounds** for the closed network at each k, beside the simulation.
2. Use **modification analysis** to price every mathematical candidate before it is built: change Dᵢ for the activity it targets and recompute the bounds and MVA.
3. Analyse the intervention experiments with **multiplicative models (logs) and allocation of variation** (Jain ch. 18–19). Run a **fluid approximation** as a closed-form check on k\*.

These additions are recorded in journal §53 and passed to the Pathfinder.
