# Focused round 4: Modelling the cache to rightsize workers

**PROCEED for the article corrections at fbc0a3ee0656893830e3a229e9f822a06b8c6efc.** All five requested points are resolved. The earlier live-publication mismatch is also resolved in this client's current browser observation. No chain-level problem introduced by the corrections warrants another full review.

Assignment: [6956224553](https://github.com/lmmx/mibz2/pull/162#issuecomment-6956224553). Prior review: [6953772428](https://github.com/lmmx/mibz2/pull/145#issuecomment-6953772428), [round3 report](https://github.com/lmmx/mibz2/blob/f3dba9877e11167c0b8d0ec02973e84213e929c6/state/reports/blog-r3-6953475922/report.md).
PR549 was read back as merged, head38613e6c1e4ef5b4b8d49806cd06669586e31697, mergefbc0a3ee0656893830e3a229e9f822a06b8c6efc. Its diff changes only the main article and notes. Scope is those corrections and their consequences, not a new audit of all unchanged exposition or a production admission.

## Findings

| Requested check | Evidence and disposition |
|---|---|
| 1. Causal wording in takeaways, main §5 and notes §3 | **Resolved.** The text now distinguishes failure to transfer the low-count pinned gains from proof that cache effects are absent. CPU competition is implicated; those points cannot calibrate cache contention alone. Ten-thread gain survives full affinity, with exact mechanism unresolved. The closing chain and sweep paragraph agree. |
| 2. Failed eager7 prediction and five-of-eleven count | **Resolved.** Notes §3 now records eager7/lazy7 at ten: predicted0.98 [0.95,1.02], measured1.0906 [1.0821,1.0992], refuted. The ledger says five of eleven values outside the bands and explicitly identifies the original heading's error. |
| 3. Actual0.5.4 admission predicate | **Resolved.** Takeaway5 enumerates pinned4/6/8/10 with exactly one allowed logical CPU per distinct physical core, or the entire machine at10 and above. Other masks retain the old plan. Main §5's ten-CPU/five-core counterexample is correct. |
| 4. Sparse sweep, initial gain and Qureshi citation | **Resolved.** “Selected worker counts” removes exhaustive-sweep language.11.5% is identified as the initial pinned result;12.7% is the later unpinned result. The MLP statement now cites the four-author ISCA2006 paper and §1.1; UCP remains attached to cache utility. |
| 5. Transfer-test pointer | **Resolved.** Main §5 now explicitly links to the notes' three prospective refuters. The live link was clicked and landed at the actual topology-and-transfer section. |

### Causal check

The replacement counterexample is explicitly hypothetical:0.96×1.0417=1.000032, which rounds to1.000. It demonstrates non-identifiability of the two contributions from a net ratio; it does not pretend to estimate either contribution in mibz2. The important correction is not merely a softer adjective: the text now retains the possibility of a cache benefit offset by lost parallelism. Excluding those confounded points from a cache-only fit remains justified.

The revised conclusion does not claim that persistence at ten identifies a unique cache, bandwidth, scheduling or overlap mechanism. This closes the material objection in round3. No new experiment is needed to settle this editorial point.

### Ledger check against primary record49

Read [record49 at c7ea4a5c](https://github.com/lmmx/mibz2/blob/c7ea4a5c5e8447fbdfd407c6f0ae07b6669bd87e/docs/journal/monodromy/2026-11-17-worker-rule.md), not just its heading. Its eleven rows contain five failures:

- eight threads:5/8 measured.997 above.99;
- eight threads:5/6 measured1.078 above1.06;
- six threads:lazy5/lazy6 measured.925 below.94;
- four threads:lazy2/lazy4 measured1.396 above1.35;
- four threads:lazy2/production measured1.533 above1.47.

The new annotation corrects the count without rewriting the frozen source or its other verdicts. Record49 freezes the eager7 band; the [earlier independent eager-trim replay](https://github.com/lmmx/mibz2/blob/56034daf7439de030f8957e3bfbc8a4abce9f4f9/state/reports/eager-trim-6947285652/report.md) supplies the observed1.0906 interval. The added row preserves the failed kernel prediction alongside successful eager5/eager3 predictions. Observed eager5 .937852 and eager3 .959393 lie inside their respective [.86,.95] and [.95,1.03] prediction bands. No raw experiment was rerun in this focused round.

### Admission check against the release

Fetched src/parallel.rs both at the reviewed merge and at v0.5.4: contents are identical, blob **f9ab5e0b8f14b91d88e747f642fda8742be5f47e**.

The operative chain is:
- trim_cell selects the measured4/6/8 cells or the ten-worker reference cell at requested10 and above;
- trimmed also requires measured_i9 and affinity_admits;
- affinity_admits = pinned_one_a_core(T,m) OR (whole_machine(m) AND T>=10);
- pinned_one_a_core requires target physical cores=T and target logical CPUs=T;
- whole_machine requires a present target topology equal to machine topology.

Thus a ten-logical-CPU mask on five physical cores fails the pinned condition and is not the whole machine. The revised prose handles the prior counterexample. Machine recognition additionally requires the exact CPU/topology/cache profile, kernels, platform and unlimited quota; the short article is a description of admitted placement classes on its test machine, not a replacement specification of every guard. It does not newly authorise arbitrary masks or other hardware. The later paragraph explicitly locates0.873 at ten threads; it is not an observed ratio for every higher count.

### Citation check

Read the [author-hosted primary paper](https://users.ece.cmu.edu/~omutlu/pub/qureshi_isca06.pdf), title and authors plus §1.1. Qureshi, Lynch, Mutlu and Patt distinguish isolated misses from concurrent misses and demonstrate that lower miss count need not mean fewer stall cycles. This supports the revised local claim. The visible notes citation and Further reading both use “A Case for MLP-Aware Cache Replacement,” ISCA2006, DOI10.1109/ISCA.2006.5. No novelty claim for that general principle follows.

## Live verification

On2026-11-17, before the17:53:34UTC completion check:
1. Opened the [main article](https://lmmx.github.io/mibz2/blog/2026-11-17-modelling-the-cache-to-rightsize-workers/) in the browser and read its current accessible page text. It contains the14-minute split article, the11.5%/12.7% qualification, selected-count wording, corrected takeaways, net-ratio counterexample, exact mask counterexample and unresolved-mechanism closing chain.
2. Clicked the main §5 “notes” link. It navigated to [notes §5](https://lmmx.github.io/mibz2/blog/2026-11-17-modelling-the-cache-to-rightsize-workers-notes/#5-topology-and-transfer). The section contains the three frozen refuters: three beating four by more than2%; two within3% of four; any lazy plan beating the reference by more than2%.
3. The notes page's current accessible text contains the five-of-eleven correction, failed eager7 row and corrected four-author MLP citation.

The direct HTTP attempt was unavailable and the search reader could not access these URLs; browser navigation supplied direct live-page evidence. These are observed content/link checks, not a new desktop/phone/theme screenshot certification, a full build, or proof that every byte equals the source commit. Unlike round3, the corrected main text and working notes are present.

## Scope and disposition

Expected: targeted repairs to five prior objections, without another full rewrite. Observed: all five land, with source and live checks aligned. The editorial gap identified in that review is closed. The scientific uncertainty remains correctly open; this verdict does not qualify the general model, prove a unique mechanism, re-admit production or certify new machines.

No further article revision is required by this focused review. Next value lies in the already scoped research tests, not another review cycle of these repaired paragraphs. The separate knee-article findings remain separate.

Receipt4731da2773728ac2e3d1b0f7b97869cc10f5d12c preceded review work. Pinned specs a2da49638ac54d7763de1c167bd054f67ffdc763 read; migration matched old head1283b969e9e295e268257cd1b6be3ad7da29e6dd and manifest SHA-2569b6bfe5b8e2193a68d8e788873da11947197c3457b6ad6e454b580a05f7b8fd0. Inbox listed oldest first; only this item unfinished. Pinned coord.py audit since6955971254 found no misroutes; REVIEW routes to145. No implementation, merge, new timing, authority change or automation-setting change.
