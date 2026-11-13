# Selective materialisation: improvement demonstrated, integration not qualified

One frozen ordinary-CI attempt, Benchmark43451114427/job131973205406, PR380 exacta8a23c11607864cb431f87fff38169e10c41685f. Artifact14053580305,47607bytes, verified ZIP SHA256964ca8ef02cb429da09c6cb00b90428e08237c4f446a47c9351afc871ea6be22. All210rows present (3arms ×7families ×10rounds), successful, pinned to logical CPU1, expected decoded length/hash identical. Native byte-oracle tests separately compare complete output with libbzip2 and Segmented; identical route counts and compressed hashes verified before timings. No discarded measurements or rerun.

AMD EPYC9V74, Ubuntu26/7.0.0-1019-azure,4vCPUs/2recorded physical cores; Rust1.100.0(0ef1bc386), release fatLTO/onecodegenunit. Both Cells binaries share common final tree/harness/compiler/flags/input, with ONLY src/cells.rs restored from correctedd4f0e8fb3ac68fa649e1810dff58dc06e9c545c2 for original. Actual ELF rustc identities agree. Generator/text57f51bbc0ca62ac3ee1cc148a1010842eea6222a,scale4, hashes in inputs.json/cells-inputs.json. Segmented is the same-head ordinary BzReader build. No hooks, phase instrumentation, new estimator, i9 measurement or warm comparison.

## Paired complete-reader result

Ratios<1 are faster/lower. Geometric mean of ten round-matched ratios and95% log-t intervals from unchanged bench/report.py::ratio_ci. CPU includes user+system. RSS is native child getrusage, not the old Python-parent proxy. The table's RSS ratio compares selective with original Cells.

| Corpus | Selective/original wall (95% CI) | Selective/original CPU (95% CI) | Selective/Segmented wall (95% CI) | Selective/original RSS (95% CI) |
|---|---:|---:|---:|---:|
| wikidata | 0.9763 [0.9734, 0.9792] | 0.9764 [0.9735, 0.9794] | 0.9787 [0.9749, 0.9826] | 1.0007 [0.9997, 1.0016] |
| onebyte | 0.9726 [0.9626, 0.9827] | 0.9762 [0.9680, 0.9845] | 1.0766 [1.0676, 1.0857] | 1.0002 [0.9986, 1.0019] |
| pattern | 0.7746 [0.7727, 0.7765] | 0.7806 [0.7787, 0.7825] | 1.7589 [1.7264, 1.7919] | 1.0013 [0.9992, 1.0034] |
| random | 0.9971 [0.9924, 1.0017] | 0.9973 [0.9927, 1.0019] | 0.9930 [0.9858, 1.0003] | 1.0005 [0.9983, 1.0027] |
| records | 1.0064 [0.9964, 1.0164] | 1.0062 [0.9962, 1.0162] | 0.9989 [0.9919, 1.0059] | 0.9988 [0.9955, 1.0021] |
| runs | 1.0052 [0.9942, 1.0163] | 1.0038 [0.9932, 1.0144] | 1.0027 [0.9846, 1.0210] | 1.0002 [0.9962, 1.0041] |
| text | 1.0004 [0.9968, 1.0039] | 1.0003 [0.9966, 1.0041] | 1.0023 [0.9975, 1.0070] | 0.9977 [0.9947, 1.0007] |

Wikidata improves2.37% [2.08,2.66] wall and2.36% CPU against original Cells; pattern improves22.54% [22.35,22.73] wall and21.94% CPU; onebyte improves2.74% wall. Random/records/runs/text are inconclusive (intervals contain1); no primary loss or material secondary loss against original. RSS is essentially unchanged, consistent with unchanged allocations. This passes the candidate's registered *relative-to-original* useful-gain test. It does NOT qualify a default or merge.

Onebyte still budget-refuses: this is a partial-induction/fallback path result, not an admitted Cells inversion win. Successful-cell counters omit the refused attempt's work. Random and records are column fallback; runs and text density-refuse. Do not attribute their noise to saved grammar writes. Wikidata196/431blocks admit,185density/2budget/48column; pattern22/22admit with one short cycle. All counts identical old/new. These are block counts, not an84% byte-fraction remeasurement.

## Absolute resource ledger

Medians, not ratios of the medians used as the estimator. Wall/CPU milliseconds; native peak RSS MiB. Triples are Segmented / original Cells / selective Cells. Per-sample extrema and pairedRSS intervals remain in paired-analysis.json.

| Corpus | Wall ms S/O/N | CPU ms S/O/N | Peak RSS MiB S/O/N |
|---|---:|---:|---:|
| wikidata | 431.50 / 432.20 / 422.05 | 433.20 / 433.85 / 423.70 | 26.09 / 34.23 / 34.25 |
| onebyte | 17.45 / 19.40 / 18.80 | 19.15 / 21.10 / 20.50 | 33.48 / 35.99 / 35.97 |
| pattern | 26.00 / 59.50 / 46.05 | 27.70 / 61.15 / 47.75 | 16.20 / 18.58 / 18.58 |
| random | 108.85 / 108.65 / 108.55 | 110.50 / 110.35 / 110.25 | 22.80 / 22.96 / 22.94 |
| records | 117.55 / 116.85 / 117.15 | 119.25 / 118.50 / 118.85 | 23.03 / 23.20 / 23.19 |
| runs | 11.50 / 11.50 / 11.50 | 13.15 / 13.20 / 13.15 | 23.11 / 23.23 / 23.22 |
| text | 57.40 / 57.50 / 57.60 | 59.10 / 59.20 / 59.35 | 19.19 / 19.36 / 19.32 |

RSS apparatus controls pass prospectively frozen checks: exact child PIDs; self-getrusage1/64MiB peaks2440/66952KiB, wait42740/67252, procVmHWM2652/67164. Each method rises64512KiB; within-control disagreement<=300KiB, below2MiB bound. Native getrusage is responsive and agrees with independent process measurements on these controls. This is not an independent wait4 audit of every decoder sample. No allocation reduction is claimed; decoded output, retained buffers and position table remain unchanged.

## Why the work reduction is real, and what is not established

CI actual optimized induction has904bytes original vs827selective, matching local matched1.100 result. Both binaries retain the expected function. Original emits both vector copies and scalar rule stores unconditionally, with cmov selection; new code branches before them. Thus LLVM had NOT already eliminated the discarded stores. Selected short path additionally needs no la.min(16) clamp. Register allocation/control dependence and complete binary layout differ; all are included in the measured whole-reader result. A disassembly/code-size observation cannot isolate their separate cycle effects.32U+16S avoided addressed store bytes and32U avoided source-read bytes are source-operation identities, not measured DRAM traffic. No kernel-only timing or instrumented fraction is inferred. Original/selected binary SHA256s are recorded in ci-a8a23c1/selective-codegen/manifest.json.

This AMD runner's originalCells/Segmented Wikidata ratio1.0025[0.9985,1.0065] is neutral; selective/Segmented0.9787[0.9749,0.9826] is faster. The earlier i9 .881643 result came from another source/toolchain/host. Do not multiply these ratios, claim an extra i9 improvement, or substitute this run for workstation qualification.

## Integration blockers preserved

1. Separate ordinary default head/base arms flag4-thread wall regressions: random1.0279[1.0048,1.0516], text1.0387[1.0067,1.0716], Wikidata1.0230[1.0037,1.0428]. Pattern4t is faster0.9534[0.9114,0.9973]; the other17intervals include1. Default head/base .text digests DIFFER; no claim of byte-identical default code. This comparison includes the inherited365 slice against master3e7e294, not just selective versus original. It does not identify causality. Existing slice-one gate6775049636 requires every default arm level; it FAILS here despite green workflow exit. No secondary rescue run or gate waiver.
2. Whole selective Cells remains pattern1.7589[1.7264,1.7919] versus Segmented (+75.9%), onebyte1.0766[1.0676,1.0857]; Wikidata RSS1.3128 (+31.3%), patternRSS1.1482 (+14.8%). These are existing Cells-route adoption costs, not new-versus-original regressions. They prevent treating this as a qualified default and are not owner-approved exceptions.
3. Independent B5 is now CLOSED for the changed native path by original Researcher REVIEW6857443323/report e1500ba4539924e0e8407f11a90f3d93c4217318, explicitly carrying to a8a23c1 after H1 correction. Full CI Miri40/40 also passes, including all six Cells selections. All29check runs are success or legitimate release/deploy skips, all7workflows success, hosted legacy success. The PR test-merge tree equals exact head tree b380b0003877bbe99764b787195f90f955b0689f. These clear safety/check dependencies, NOT the performance blockers above. Independent review did not certify the whole harness or integration.

**Decision: retain this beneficial experimental candidate; no integration/default/release clearance.** Original Cells remains the comparator. The negative-candidate long-factor successor is NOT activated, because this candidate demonstrated a gain. Coordinator's next decision is the disposition of this positive local optimisation alongside the failed default-arm gate and unresolved broader Cells adoption costs. Do not start a second writer, dispatcher or measurement programme implicitly. Independent B5 closure and green checks do not erase the performance blockers. The single next bounded engineering question for Coordinator routing is default-arm attribution: compare the inherited365/common-base default path against pinned master3e7e294 and the a8a23c1 default path before selecting any corrective code or separately registered measurement. Do not reopen this successful native optimisation or commission a second writer by implication.

Reproduce analysis without measurements: python3 analyse.py ci-a8a23c1 /path/to/repository-at-a8a23c1. The analyser rejects missing/duplicate/error rows, wrong length/input identities, mismatched affinities and incomplete pair coverage before calling the standing estimator. The unchanged default gate's adverse results remain in result.json/comment.md/runs.jsonl. Raw210 samples, input/source/compiler/binary identities, controls and all oracle/codegen logs are retained with the verified archive.
