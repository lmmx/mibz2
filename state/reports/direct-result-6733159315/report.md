# C31 result: STOP stands; locality is a hypothesis, not an isolated cause

Trigger: [6733159315](https://github.com/lmmx/mibz2/pull/162#issuecomment-6733159315), dispatching QUEUED 6733117820.
Instructions: docs/team/{SPEC,SPEC.RESEARCHER,MODE,RULES,PROTOCOL}.md at a4f97714cc14366984015869d9d094d89488ac57.
Reviewed result: c41ef38d7df2f596193c01d9adbb2f4893f1d217. Registered/run head: aa409f6f2ebce9f41dc7902789f0ab8d6958b020.

## Disposition

**Concur with #277's registered Q1 STOP, on its one pinned AMD copy. Pause this D implementation; retain the run-fed line.** Do not qualify B from this exploratory repeat or change #269's original STOP. The next worthwhile scientific step is a bounded **fallback/code-generation control**, followed by a compact F-order index candidate if its static cost survives. A blind Intel/AMD rerun is not the first repair.

I previously suggested resumed-cursor direct placement and reviewed its construction/admission. This is an involved result review, not blind review or new B5 certification. No production implementation, timing, rerun, merge, tags or queue changes were made.

## 1. Evidence and §5 replay

Downloaded Actions artifact 13651923399 from run 42567752634. ZIP SHA-256 is exactly **00b0e008bdd8a90108ccaa87e21467dd35d70df35e1b71b86e687078a64a3e94**, matching API metadata and job 128997903554's upload log. The judged `run-fed-1.json` SHA-256 is **c533cd41ca721b55088defa7eacc38960cc4efe0ef067ea4ca283cdca3574716**. The other four JSON files are contended copies; none supplies Q1.

The primary records schema 2, 15 rounds, exact head, AVX2, AMD EPYC 9V74, affinity CPU 1, correctness passed, and seven corpora. The workflow and archived transcript establish one pinned run before the four-copy phase. The job's verification step says VALID; readings JUDGED and checks the registered Wikidata file digest.

I replayed the pinned verifier against independent corpus specifications: six synthetic corpora regenerated from 34 Git-blob-verified source files/README at the frozen head, plus the pinned Wikidata specification. All record/schema/count/tail/rotation/time checks passed. **Limitation:** I did not download/decode the Wikidata compressed bytes here; their physical-file SHA/block check is the CI evidence, not a fresh local check. No Rust build, disassembly, Miri or fuzz rerun was performed.

A separate calculation from the raw entry times, using paired log ratios and t=2.145 at df=14, reproduces:

| Quantity | Geometric mean | 95% interval | Meaning |
|---|---:|---:|---|
| D/A, Wikidata | 1.05245135 | [1.05059784, 1.05430813] | Q1 STOP: lower bound exceeds .95 |
| B/A, Wikidata | .97626985 | [.97128110, .98128422] | exploratory, not B qualification |
| D/B, Wikidata | 1.07803324 | [1.07269778, 1.08339524] | D NOT CHEAPER THAN B |
| D placement / B sort+fill | 1.20451530 | [1.20235285, 1.20668164] | phase attribution only |

Synthetic screen fails: `runs` D/A=1.28260079, above 1.02. Fallback point-estimate screen passes: worst `random`=1.00608201. Passing is not a noninferiority claim; its interval is not the screen. The Q1 stop already follows from the primary interval, irrespective of screens. Walk/RLE1/CRC are outside the entry bracket, so none of this is a whole-decoder verdict.

Expected: removal of sorting could recover enough stage cost to justify a new prototype. Observed: D is 5.245% slower than A and 7.803% slower than B. Reaching the .95 point target from this observed D needs a **9.735% reduction in D's total stage time**, not merely a 5% reduction. B needs another 2.691% to reach that point target, before its own screens/registration. Understanding improved; qualification did not.

## 2. What the phase result establishes—and what it does not

Placement is the measured loser on Wikidata: 1.600 nominal c/B versus .319+1.010=1.329. This is a comparison of two whole construction paths: checks, state dependencies, seeds, cursor reads, tails, control flow and compiled store loops differ. It does **not** isolate scatter locality.

The phrase “equal stores” is inaccurate. B predicts 30,923,519 vector table stores; D predicts 30,877,916 vector plus 84,059 scalar stores = 30,961,975. D issues **38,456 more predicted table stores (+0.1244%)** while writing **0.1135% fewer slots**. Similar logical volume rules out a large increase in counted slots, not extra instructions, cache-line traffic, RFOs, dependency latency or actual compiler stores. The 50.49 nonempty buckets/block are not simultaneous hardware streams. No PMU or listings isolate those effects.

A stronger internal control is already present:

| Corpus (all blocks fallback) | B/A | D/A |
|---|---:|---:|
| text | 1.063689 | 1.001729 |
| random | 1.284228 | 1.006082 |
| records | 1.374928 | .997353 |

On these inputs no sort or placement executes. Source `build` and `build_direct` have the same selector and the same fallback calls to MTF and BWT. The harness invokes distinct wrappers and buffers. Therefore “the selector alone costs B 28–37%” is unproved; “B versus D isolates the sort/locality” is false outside the accepted path. Candidate explanations include wrapper inlining/specialization, register/spill or code-layout differences, buffer placement and cache history. None is established without listings/control. This does not invalidate Q1—the registered object is the actual entry point—but changes the next diagnosis.

Another confound is visible on `runs`: D placement is dearer than B sort+fill (ratio 1.13689), yet D's entry is cheaper than B (D/B=.92736). The phase emitter, shared at source, differs too. Cost attribution cannot be promoted to a pure treatment effect.

Budget correction: exact per-round phase identity is D−A=K+δ−e−f. Breaking even requires δ≤−K+e+f. The median right-hand side is **.413436 c/B**, not .385 unless e=f=0 is assumed. Median observed δ=.590323 and median phase gap=.178151 remain losses; medians do not distribute through sums. Do not combine these phase medians into the registered entry ratio.

## 3. The “baseline changed” explanation is too coarse

Compared fd60fffada25d475bbcc2b54165b96cf5fd7a92f and aa409f6f:

- `src/mtf.rs` is the same blob **2610fa7ed1a7ee2c7653db47402bb7b7e9e3bb73**.
- `src/bwt.rs` is the same blob **cb971c9f054492a5092264421166f4e6707fb5a8**.
- Cargo.lock is identical; the Cargo.toml change in this comparison is commentary on the research feature.
- The benchmark source changes substantially: old four arms become A/B/D, changed rotation and buffer context, new checks/admission, and a different source-generated text corpus. Other repository code also changes.

Thus different master SHAs do **not** establish a changed A algorithm. Identical source also does **not** establish identical machine code. The change from B/A=1.1592 to .9763 establishes **run-context dependence**, with CPU family one candidate factor. It does not by itself establish a strong CPU-family effect. That causal wording should be corrected beside the journal, leaving both registered outcomes untouched.

## 4. F locality from run structure: useful constructions, explicit prices

For L run j=(p_j,c_j,ℓ_j), define a_j=C(c_j)+sum of earlier same-byte lengths. It owns [a_j,a_j+ℓ_j), with Ψ(a_j+t)=p_j+t. F order is therefore exactly **stable byte grouping of L runs**. Each byte's stream is already ordered. Between streams the byte orders them completely: concatenate by byte, no comparison merge or heap is required.

But grouping has not disappeared merely because it is renamed emission. One must retain bucketed metadata, distribute it, or revisit the original stream. Original-order records cannot generally be emitted irrevocably at their final F addresses before final byte counts are available. For a first b-run, a later a changes its F offset.

**Most bounded candidate: four-byte run indices.** Retain the original packed L records. Replace B's eight-byte [F-start,L-start] record with a u32 original-run index j, stable-grouped by byte. While consuming one byte's indices, reconstruct a from C(c) and cumulative lengths, recover p_j from recs[j], and ℓ_j from **recs[j+1]−recs[j] in original order**. A global L cursor advances with increasing F row q; intersect the two partitions and fill the same cells.

- Proof: stable same-byte order gives exactly the same a_j; the intervals tile each bucket and concatenated buckets tile [0,n). Intersecting two r-interval partitions gives at most 2r−1 nonempty cells. Ψ and L labels therefore match the direct stable-sort oracle.
- Untimed independent model checks all **29,524 ternary columns of length 0–9**, 250,959 rows; exact table and cell bound pass.
- Failure witness: L=[b,a,b] gives F-sorted original starts [1,0,2]. Inferring length from the next F-sorted start yields −1 for the first run, not 1. Retain original index+next-start or encode length explicitly.
- Price: halve F metadata from 8r to 4r; combined L+F records go from approximately 12r to 8r. At this accepted-row r/n, save **.24823 bytes per accepted row per materialization/read pass**, not .248 cycles. Add dependent original-record loads and length arithmetic; F order may spoil locality of these reads. A footprint reduction is not a speedup proof.
- Global F traversal permits B's repaired sequential overstores, with its existing allocation proof; it need not retain D's 256 cursor states or per-bucket ownership-tail schedule. This is a proposed design obligation, not a validated unsafe implementation.

This candidate still uses stable distribution if produced in a second pass. A genuinely fused variant appends original indices to per-byte chunks **as runs close**, then concatenates their streams; no final counting-sort pass is required. Charge extra emitter stores, chunk allocation/slack and fullness branches, stream metadata and retained L order. Do not scatter speculative per-literal updates into this structure. Fixed tiny per-byte buffers can amortize append writes; the correct granularity is closed runs, and end-to-end entry timing must charge them.

Lower priority: a linked next-index per run also uses about 4r, but turns F traversal into dependent pointer chasing. Repeated scans by byte avoid an F array but require up to σr record inspections. Batching g bytes trades ceil(σ/g) scans for g active buckets; it does not yield one global sequential stream unless each bucket's records are buffered. These are explicit space/traffic/dependency trades, not free merge constructions.

Prior art: [Wassenberg and Sanders, Faster Radix Sort via Virtual Memory and Write-Combining, §§2–3](https://arxiv.org/pdf/1008.2849) describes buffering multi-stream writes and per-key storage to avoid an initial counting pass. That supports the family of ideas, not this candidate's speed. I do not recommend its virtual-memory reservation or non-temporal stores here: block-sized tables are immediately walked, so their cache consequences differ. The run-index reconstruction above is my application to this representation; no novelty claim.

## 5. What should be registered next

**First, cheap static discrimination, then a bounded diagnostic—not a blind runner matrix.**

1. Obtain exact A/B/D entry and fallback listings, build flags/toolchain and function hashes; inspect the all-fallback discrepancy, argument count/spills, inlining and alias assumptions. Keep the actual observed paths; do not replace an expensive wrapper and quietly call it the same experiment.
2. Freeze one new information-objective diagnostic with the original B entry and a factored/shared-fallback control, keeping the records path fixed. Include all-fallback corpora and Wikidata; preserve selector charge, independent buffers, decode-order untimed walk, CRC, fixed order rotation and separate entry/phase readings. Register whether fallback divergence collapses and whether the Wikidata advantage survives as the information questions; no production acceptance. Include per-arm offsets/alignment and assembly identities to make buffer/code effects inspectable. This is a prospective recommendation, not a registration or permission to run.
3. Only then price the compact index versus existing B; demand its total grouping+fill entry cost, including induced emitter work. A phase-only win is insufficient. C29's exposed-path work retains priority; do not delay an independent prepared target recipe behind a new portability campaign.
4. If host interaction remains decision-relevant, use identical benchmark sources, compiler/options, corpus and measured arm set across the *recorded actual hosts*. ubuntu-latest does not promise an Intel/AMD pair. A single dispatch is not randomization of CPU family. To revisit historical context, a frozen old/new **harness snapshot × host** comparison is meaningful; duplicate “old A/new A” source arms alone are not, since their MTF/BWT files are identical. No new infrastructure or reserved workstation run is implied; Louis must run any separately frozen target recipe under C5.

Next experiment's value: determine whether a large avoidable wrapper/context cost is being mispriced as algorithm or hardware cost, before spending on another placement variant. The finite index construction offers a smaller, falsifiable alternative if that control leaves a useful margin. No claim that scatter placement is intrinsically exhausted, and no retrospective relaxation of any gate.

## Artifacts

`artifact/run-fed-1.json`, `artifact/sweep-1.txt`: exact downloaded primary evidence.
`replay.json`: pinned verifier replay and corpus evidence/limitation.
`independent.py`, `independent.json`: separate paired-statistic arithmetic, store/budget checks and finite run-index proof model.
`source-blobs.json`, `identity.json`, `digests.json`: source/artifact identities.
`routing.json`: pinned routing audit; no unresolved misroutes. Migration head and original manifest digest match the saved cursor. No other state branch changed.
