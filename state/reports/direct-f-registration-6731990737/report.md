# C31 direct-F draft: REVISE evidence admission; retain the experiment's question

For inbox [6731990737](https://github.com/lmmx/mibz2/pull/162#issuecomment-6731990737), reviewed exact head **705323290b5470dea4a31910c22522c342e9d064**. Pinned team instructions a4f97714cc14366984015869d9d094d89488ac57. Research only: no Rust execution, native timing, implementation, merge or dispatch.

**REVISE before freeze:** validate the newly consumed clock/denominator evidence, make timed D path reconciliation symmetric with B, and narrow the causal wording. The direct-F construction, diagnostic thresholds and removal of old controls are otherwise reasonable. This is not an instruction to abandon C31, and not B5 clearance. I proposed resumed cursors and bucket-confined tails; my involvement is disclosed.

## 1. Independent source assessment

Read the frozen draft, src/run_stage.rs, examples/run_fed_stage.rs, bench/run_fed_stage.py, its tests and fixture, and copies.sh. Links below use [the reviewed tree](https://github.com/lmmx/mibz2/tree/705323290b5470dea4a31910c22522c342e9d064).

For a valid L run (c,p,ℓ), its next F interval [a[c],a[c]+ℓ) is disjoint from earlier intervals of c; all intervals of c tile its bucket. On it Ψ(q)=p+q−a[c]. Recovering L[q] by the bucket's resumed predecessor yields the existing packed table, not a new consumer convention. Each bucket advances through a disjoint row range, so post-seeding cursor advances total at most r−1. There are at most σ≤256 binary-search seeds: do not describe that exact seeding implementation as O(r+256) without accounting for the searches; its bound is O(σ log r) probes plus subsequent linear advancement.

The ownership-tail argument is sound on valid tilings: an overstore is repaired by later cells/runs of the same bucket; never spill into another bucket. Current code uses w+8≤bend and exact scalar suffixes. Counts accumulate in u64 before stores, n<2^24, and a successful traversal checks rising adjacent starts from0 to n and per-bucket capacity. Under those predicates a==end is implied: total assigned length n equals total bucket capacity n and none exceeds its capacity. Removing that final equality alone is an equivalent mutant; its survival need not be a missed safety obligation. This argument is not a comprehensive audit of malformed paths or CPU-feature dispatch.

**The machine-work model needs one correction.** Same logical cells does not mean identical stores. B can overstore across a bucket boundary under its globally increasing repair order; D must use scalar tails there. For a D cell [q,q+ℓ), bucket end b, the number of vector stores is

    v = min(ceil(ℓ/8), floor((b−q)/8)),
    scalar stores = max(0,ℓ−8v).

The scalar suffix region is at most seven destination rows per nonempty bucket, but some may otherwise have been covered by overstores. A narrow bucket can be entirely scalar. D may write fewer physical slots while issuing more store instructions. The harness Tally.slots is explicitly **B's** AVX2 slot count, not D's. Keep that label; a D-specific vector/scalar/physical-slot census would test the mechanism. “256 streams” is an upper bound; nonempty buckets and their interleaving determine actual live streams.

## 2. The gate and controls

The prospective .95 gate is coherent **for this materially different D candidate and diagnostic question**, provided the frozen registration explicitly preserves #269's .90 rule and STOP on fd60fffa. It must not be applied retrospectively to B or represented as relaxing the same registered result. C1 permits an information objective; FEASIBLE here only motivates another registration. It is not production acceptance, a whole-decoder claim, or authority to change C6's bench rules.

The decision regions are disjoint: g≤.95 and hi<1 plus both screens is FEASIBLE; lo>.95 is STOP for the specified minimum; otherwise inconclusive unless a positive point/interval combination fails a screen. A result with a real 4% saving may properly stop for the 5% objective. “No saving of5% shown” is not “no speedup.” The .95 point rule with hi<1 establishes evidence of some saving, not a confidence bound guaranteeing5%. The1.02 screens are point screens, not noninferiority, as the draft already says.

Dropping C and the store reference is acceptable: neither is needed to estimate the prospective D/A entry-stage contrast. Keep A, contemporaneous B, the fallback split and CRC checks. Their old readings remain at their old head and runner; no historical control is silently substituted for a current measurement. Do not revive old store-floor conclusions as current attribution. Removing arms changes interference/history and makes historical B/A only context; contemporaneous B/A is the reference.

Freeze exact full baseline and candidate identities, not merely “master at merge base.” The final head fixes the source-generated synthetic text; edits to immediate source/test/example inputs can change it. Record that this is the final head's corpus, and keep matching D and A inputs. No separate rereview of a documentation-only pin is needed absent changed measured inputs.

## 3. Budget: arithmetic right, interpretation too strong

Using the draft's rounded historical phases:

    B−A = 4.228−3.693 = .535
    K = B−A−B_sort = .535−.317 = .218
    placement_break_even = 1.332−.218 = 1.114
    placement_at_5_percent = 1.114−.05×3.693 = .92935

Thus .929 rounded is right, conditional on the historical phase scale and zero emitter/fallback recovery. #272's different numbers are correctly excluded.

**Same source does not make e=f=0 in measured time “by construction.”** Changed table writes, retained scratch, compiler layout, caches and history can change the next emitter or fallback timing even when their code is identical. Zero is a reference-model assumption. The draft appropriately measures these terms; call them controls rather than invariants. Similarly D/B compares a whole replacement of sort+fill by place, including checks, seeding, cursor scheduling, stores and tails. It cannot isolate the cost of deleting a sort or F array alone.

There is an exact per-round phase identity, with K_i=(B_i−A_i)−B_sort,i:

    (D−A)_i = K_i + δ_i − e_i − f_i.

It does not survive replacing every term independently by its median. To test accounting, reconstruct the right side **per round**, then summarize its difference from the directly computed left side. A nonzero difference between sums of component medians is not a missing phase. Existing “sensitivity, not reconstructed paired verdict” labels are useful; preserve them.

Also separate the measured stage from the consumer: entry() times **MTF plus table construction**. walk_check(), room walking, RLE1 and CRC are outside the bracket. D/A is readable as that stage's entry-point contrast, including selector and fallback, not inverse-BWT build+walk or whole decoding. A changed table-construction cache footprint can alter the following walk without appearing in Q1. A later production comparison must charge that consequence.

## 4. Reproduced admission gaps

All **45 existing Python tests pass** after fetching their exact copies.sh dependency. New untimed mutations of the repository's valid fixture give:

| Mutation | check() | Result |
|---|---|---|
| ghz=-1 | no problems | FEASIBLE; δ changes sign to+21.44 |
| ghz=0 | no problems | FEASIBLE; clock-scaled budget terms collapse to0 |
| ghz=true | no problems | FEASIBLE; boolean treated as1 GHz |
| remove ghz | no problems | readings raises KeyError('ghz') |
| Wikidata output=1 | no problems | FEASIBLE; δ becomes about−60,032,000 c/B |
| rows_runs=1, records=cells=999,999,999 | no problems | FEASIBLE with impossible path counts |

These are mutated fixtures, **not evidence that an actual run emitted bad data**. Five cases produce readings from an admitted but contradictory record; the missing-clock case fails with an exception before VALID is printed. Q1's time ratios are invariant to the clock/output mutations, so these do not demonstrate a changed primary verdict. They do undermine the newly registered budget and the claim that admitted records have consistent evidence.

**Required repair:** check ghz as a finite positive non-boolean numeric value before readings; reject missing/invalid metadata with INVALID, no readings. Cross-check Wikidata output and input sizes against its pinned corpus spec or independently validated census, not merely positive integers. For synthetics bind output to the validated input's decoded length/census; exact frozen expected values are another option. Enforce count relations (e.g. accepted rows at least accepted blocks, records at most accepted rows, zero accepted blocks implies zero accepted rows/records/cells/slots) and the representation's applicable cell bounds, accounting for per-block totals and closing records. Preserve tolerances only where logging intentionally rounds values. Add the mutations as regression evidence.

The fixture's values are schematic, so replacing them with real pinned-corpus metadata may be needed; do not weaken the real-run check to retain a convenient fixture. Do not claim authenticity against arbitrary jointly forged records: the target is internal/provenance consistency under the documented source and raw record.

**Timed D path check is missing.** In the entry sweep (lines around588), the assertion is:

    assert!(arm != 1 || by_runs == runs, ...)

Only B is reconciled, whereas D timings are also placed into a bin using precomputed takes_runs. Initial validation checks D, but that does not enforce the advertised timed-pass obligation. A later D fallback caused by failed placement can still produce correct CRC and be billed to the accepted bin. Require the assertion for both B and D, or record/reconcile their actual paths every pass. This is a latent measurement guard, not a demonstrated valid-input placement failure. D/A totals still include all elapsed stage time; the split and explanation would be wrong.

## 5. Safety and disposition

No new UB is established here. Scalar/AVX2 Miri and native correctness claims are read from the draft, not rerun or independently certified. B5 remains outstanding: absence of a run-fed fuzz mode on this branch is a coverage gap, not an exemption; #270's separate HOLD is no reason to inherit its permission or omit C31's obligations. Independent unsafe review must state its involvement and proof scope. Disassembly would test the proposed machine mechanism, but is not evidence that safe arithmetic or measured stage ratios are automatically invalid.

Prior art for this bounded review is the already reconstructed LF/Ψ run translation and resumed-cursor construction in [the prior Researcher report](https://github.com/lmmx/mibz2/blob/40f58759e4554fa0e8e08122f4bab993f73076b0/state/reports/adjacent-run-constructions-6730894960/report.md), which identified bucket-confined tails. This review checks the implementation against that mechanism, rather than borrowing historical speed claims.

Expected: removal of F materialization may not pay for streams, cursor state and tails; the prior STOP range is a subjective forecast, not derived confidence. Observed here: formula and budget consistent, but six fixture mutations expose missing validation and the timed D path guard is asymmetric. Measured gap closed: none. Mechanism sharpened: D changes physical store work as well as ordering, while Q1 leaves the following walk outside its bracket. Next value: repair these narrow evidence guards, freeze the exact pin, complete applicable B5 work, then the bounded registered diagnostic can answer D/A. No threshold change requested and no generic line-wide STOP.

Artifacts: witnesses.py and witnesses.json; exact reviewed verifier/fixture dependencies alongside for reproduction; routing.json records five already-resolved historical items and route#145. Pinned coord.py's Coordinator-only publication sender predicate is recorded separately from the Researcher-aware exact-body check.
