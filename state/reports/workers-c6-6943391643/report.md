# Worker-plan C6 review — assignment 6943391643

**PROCEED on the three frozen rules and their reproduced verdicts. Proceed toward the Coordinator's bounded admission: seven lazy workers, 28 slots, past the cores and at ten requested threads on the measured i9 profile. Keep eight and below deferred as proposed.** Neither new run is a necessary prerequisite for the admitted cells under the inspected rules. This does not admit a general 0.7-times-threads rule, another machine, or W12 combined with this plan. Exact implementation correspondence, correctness and the applicable CI gate remain integration obligations.

Researcher specification: a2da49638ac54d7763de1c167bd054f67ffdc763. Independent source/evidence review; no implementation, workstation execution, new timing or merge. The old-branch head and migration-manifest SHA-256 matched the cursor before receipt 27aab25358f0a3ce990e3d722e065569c8f151d8. Only this assignment was unfinished in the inbox. Routing audit after the preceding review returned no findings.

## Freeze and provenance

| PR | measured source commit | source commit time UTC | registration/freeze posted UTC | archived run start UTC | submission |
|---|---|---|---|---|---|
| 510 | 6bb93e2195d1fb4dfa3cc947df1dfd40d794bf9f | Nov 16 21:07:36 | 6930443845, Nov 16 21:50:23 | Nov 16 22:38:00 | 6931708646, 22:48:00 |
| 514 | c23fcb19fedeb6c44a05aa4b6ded1af91f2114bf | Nov 16 23:29:10 | 6933083894, Nov 16 23:56:01 | Nov 17 07:36:12 | 6942001675, 07:46:20 |
| 515 | a6ab8ceeee0be604a68e688f8651c05efe0f0c61 | Nov 17 07:59:55 | 6942990138, Nov 17 08:36:25 | Nov 17 08:37:01 | 6943184061, 08:45:44 |

All three rules precede their data by the archived run-directory timestamps, source commits and server comment timestamps. The freeze comments' created and updated times agree. The last margin is only 36 seconds, but positive; its source was committed earlier. These are recorded timestamps, not independent clock attestation.

I fetched the actual archives with base64 file reads and verified their full SHA-256 against the workstation submission posts:

| archive | evidence commit | SHA-256 |
|---|---|---|
| workers-window/run-20261116T223800Z.tar.gz | 96df3c05a0038e5155c790284362fce33dbdbb12 | 00da50825fa1a13eed683fa153f80c716158180ddf00e79880f2c757804d1630 |
| workers-low/run-20261117T073612Z.tar.gz | 9becbc272fad1bf829af5d86b21373c1adf57a23 | fdd6434a32f24c7280f9827d255ced9ab2b036f34914df690fd7a2aff2637628 |
| workers-at-cores/run-20261117T083701Z.tar.gz | 53fc8112d2cacfcc0e71d99d9fcf295ddf7d154a | 9144f6b4580511ca8353779a4fa2293b8190e7a855ecea02566f7ebe10cb5528 |

Each commit.txt matches the source above; all toolchains record rustc 1.99.1. The Wikidata-x4 input digest is identical across runs. Generated text inputs differ across runs, as do their output sizes/hashes; each individual run has identical outputs across its own arms. Thus the text steps are not three observations of one identical file.

**Review was late, not pre-run.** That is explicitly recorded by the assignment and freeze logs, including LOG 6931106333. This review does not rewrite the chronology, establish a standing exception to C6, or authorize another run. Its technical assessment finds no correction to the registered statistical rule that would require repeating these runs under a new rule. The reported Louis instruction concerns these runs; it is not widened here.

## Replay and independent calculation

Fetched each source's bench/two_arm.py, bench/test_two_arm.py, fixture, .just/bench.just and examples/ab.rs; read the two journal registrations and PERFORMANCE-POLICY at the measured 515 source. Replayed the exact recipe arguments on the archives. **All three stdout verdicts match archived verdict.txt byte for byte.** The pinned suites pass 57/57, 66/66 and 81/81 checks (204 total). Independent SciPy Student-t calculations and separately reconstructed Holm rejection sets agree; the checker's t CDF agrees to 1e-12 at the checked values with nine degrees of freedom.

| registration | full roster | family | result |
|---|---:|---:|---|
| 510 | 240 | nine Wikidata contrasts | 8/32 ADOPTS at all three counts; 9/72 and 9/18 NOT ADOPTED |
| 514 | 240 | six Wikidata contrasts | 7/28 ADOPTS at all three counts; 6/24 NOT ADOPTED; production 9/36 remains context |
| 515 | 200 | five Wikidata contrasts | all five reject; choice 7/28 at ten, 6/24 at eight |

No invalid roster, failed output, pinning or spinner condition occurs. Past-core records are unpinned (all 20 CPUs), without a spinner. At ten, records use CPUs 0–9, one per physical core, without a spinner. At eight, they use CPUs 0–7, with the registered spinner on a disjoint core. lscpu records i9-10900K, ten physical cores, twenty CPUs, 20 MiB L3; env records no quota.

The selected arms have no SLOWER primary cell and no RSS ratio above 1.10 against their registered references. The exact main ratios are 0.9509/0.9547/0.9514 for 510; 0.9705/0.9726/0.9715 for 514; 0.8850 [0.8815, 0.8886] at ten and 0.9285 [0.9218, 0.9353] at eight for 515. The text screen detects resolved losses; INCONCLUSIVE does not mean equivalent. For example, 514 seven-versus-eight text at sixteen has an upper interval endpoint about 1.040. This is the disclosed frozen screen and the standing policy's no-resolved-primary-loss criterion, not a newly imposed non-inferiority test.

Holm controls each registration's gain family. It does not confer a campaign-wide 5% error guarantee over adaptively chosen later registrations. Student-t inference also retains its paired-log sampling assumptions. There is no result-driven rerun or threshold relaxation in this assessment.

## The chain and the direct evidence

The cross-run products of the exact point estimates are **0.922859, 0.928578, 0.924248** at 12/16/20. They are descriptive chained estimates, not one direct paired test, and not uniformly 0.918. Multiplying endpoints is not a justified confidence interval. Nor does passing two no-loss screens logically imply the endpoint has passed a direct no-loss screen.

But 514 also ran production 9/36 alongside 7/28. Reconstructing matched rounds directly gives:

| requested threads | same-run seven/nine Wikidata ratio [95% t interval] | text ratio [95%] | Wikidata RSS ratio |
|---|---|---|---:|
| 12 | 0.917298 [0.910491, 0.924156] | 0.985765 [0.968482, 1.003357] | 0.7693 |
| 16 | 0.917167 [0.913270, 0.921081] | 0.999885 [0.960017, 1.041410] | 0.7839 |
| 20 | 0.919770 [0.912771, 0.926822] | 0.971218 [0.945178, 0.997976] | 0.7835 |

CPU ratios for Wikidata are 0.7362/0.7443/0.7449. This independently supports the approximate 8% time reduction and removes dependence on cross-run stationarity for that descriptive comparison. **It stays an exploratory/context contrast.** I have not promoted it into 514's Holm family, rewritten its verdict, or treated its post-selection intervals as a newly registered confirmation. The admission basis remains the registered sequential improvements, with the same-run context as corroboration.

## Admission scope and the next runs

1. **Past cores and ten requested threads:** proceed on the measured profile. These are useful, substantial gains, not merely unresolved point improvements. The policy does not require held-out corpora, proof of a global optimum, or complete causal identification before moving a measured gain toward integration. The proposed held-out direct-seven-versus-nine run is valuable for generalisation and for limiting selection optimism; it need not block this bounded change. Preserve its prospective rule and report it even if unfavorable.
2. **Eight and below:** retain the Coordinator's explicit deferral until the sweep. Eight's historical 6/24 ADOPTS remains true. The sweep may choose a better tested arm there; it is required before extending to the unmeasured four/six-thread decisions. This deferral is a separate admission constraint, not a rewrite of 515 or a claim that its eight-thread result failed.
3. **No universal 0.7 × T formula:** nine requested threads, low counts, other hardware and other kernels are not supplied by these three registrations. Past the cores, the candidate is constant seven, not 0.7 times an arbitrarily large request. Keep other routes unchanged and verify deterministic nonactivation plus the applicable neutral CI bench off-target.
4. **Preserve the actual intervention:** seven *lazy* workers and 28 slots. At the inspected source, Plan::Workers(7) alone selects eager Cells because LAZY_FROM_THREADS is eight. Occupancy arms explicitly call parallel_workers(..., true). A production patch that only reduces the integer and falls through planned() is a different intervention. Check the built decoder's kernel, count and window, on the exact final head.
5. **Affinity is part of the evidence:** ten's direct result is under the registered physical-core spread mask, and eight includes the external spinner. They do not measure every scheduler/affinity state. Route correspondence must be established separately: the occupancy arms force a decoder; they do not test activation of a future hardware-gated production route. No new timing is demanded here merely to erase that disclosed limitation.
6. **Keep W12 separate:** these runs do not certify the combination with W12 or a later kernel/layout change. No exact-head CI, malformed-input equivalence or new B5 evidence was independently reproduced in this statistical review.

## Scientific corrections; not extra adoption gates

- Lowering workers while retaining four slots per worker also lowers total slots. These experiments qualify the *coupled plan*, not an isolated worker-count mechanism. The window optimum at nine workers need not be the optimum at seven.
- Seven is the best tested past-core count in this design; it is not a proven global optimum. The tested six arm's lack of gain over eight does not prove equality.
- Falling aggregate LLC misses/cycles supports a shared-resource explanation, but does not isolate L3 capacity from scheduling, queue/window footprint or contention. A smooth observed sequence is not proof of a unique smooth-capacity causal law.
- LOG 6943291920's whole-process busy-share fit does not identify the workers' own utilisation or establish 1.7 cores of non-worker demand. When effective frequency is flat, defining u = CPU/(wall × W) makes wall proportional to cycles/(W × u) nearly an accounting identity. Values above one are already evidence that u is not literal worker utilisation.
- The archived *judged* ten-thread records contain per-thread CPU samples. Summing only mibz2-N workers gives median worker CPU/(W × wall) **0.763, 0.857, 0.931, 0.925, 0.931** for W=10,9,8,7,6. Summing the other listed threads gives about **1.077, 1.093, 1.067, 1.004, 0.910** CPU-seconds per wall-second. These samples are rounded and not the diagnostic's separate perf runs; they cannot close all accounting. They do show why the aggregate ratio's match is not independent causal validation. The measured wall gain stands without that story.

Expected: an occupancy turn near seven/eight. Observed: seven wins among tested past-core and ten-thread arms; six wins among eight-thread candidates. Roughly 7–8% past-core and 11.5% ten-thread time is removed relative to the relevant production references; remaining time is not thereby classified as inherent. The next decision with highest immediate value is the low-count policy boundary and held-out generalisation, not another attempt to make these already-passing comparisons pass.

## Reproduction

Companion files: replay.py (exact checker invocations), independent.py and independent.json (independent arithmetic). Fetch the three archives from the evidence commits above and unpack under window/, low/, cores/. Fetch each measured source's bench/two_arm.py, bench/test_two_arm.py and its fixture under source-window/, source-low/, source-cores/ respectively. Run python replay.py and python independent.py. They read existing measurements only.
