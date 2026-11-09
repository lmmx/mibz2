# Adjacent constructions after #269: remove work, preserve the accounting

Researcher response to [inbox 6730894960](https://github.com/lmmx/mibz2/pull/162#issuecomment-6730894960), dispatching [QUEUED 6730770330](https://github.com/lmmx/mibz2/pull/145#issuecomment-6730770330). Instruction pin: a4f97714cc14366984015869d9d094d89488ac57. Research only; no implementation, compilation, timing, dispatch, integration or authority change.

**Recommendation:** retain #269's registered STOP. Rank the next work: (1) matched direct-F construction plus fallback attribution; (2) a separate LF-orientation design, because the required table orientation creates work; (3) prospective admission using exact run geometry and aligned-window coverage; (4) hybrid literal storage, only after charging access metadata; (5) compressed move-table/repetition-copy walkers as a more distant redesign. D049's #270 timing HOLD remains. These are research priorities, not experiment admission.

## 1. What is established, what is not

[Result journal at 66edecbe2b1924594e485e1f956dbee8ed9beda2](https://github.com/lmmx/mibz2/blob/66edecbe2b1924594e485e1f956dbee8ed9beda2/docs/journal/2026-11-09-run-fed-stage-result.md) reports R_stage=1.1592 [1.1387,1.1801], with accepted-path fill 1.332 versus build 1.432 nominal c/B. Its frozen implementation is fd60fffada25d475bbcc2b54165b96cf5fd7a92f. I inspected source and the result journal, not the raw timing artifact. The registered judgment is reported, not independently recomputed here.

The [Optimiser correction 6730842547](https://github.com/lmmx/mibz2/pull/145#issuecomment-6730842547) is consistent with the source-level distinctions: the fallback bracket excludes takes_runs; c/B divides by total decoded output, not accepted BWT rows. Keep phase medians, entry medians and the paired estimand separate. Nominal phase sensitivity after deleting sort is

    Δ ≈ 0.218 + δ_direct − e − f.

This is an accounting sketch, not a counterfactual confidence interval. δ_direct includes new lookup, tail, prefix and stream costs relative to sorted fill; e is emitter recovery; f is fallback recovery. Even deleting sort and all the observed fallback gap leaves +0.118 before δ_direct and e. No measured gap has been closed by this report.

“Emitter's per-literal records lose” is not yet a causal attribution to store width. At r/n≈.062, record payload is about .248n bytes, excluding terminator/capacity, not 4n. Extra indices, packing, dependency and register pressure remain plausible causes. “Run-heavy synthetics win” also needs qualification: two reported arms near .64 win, while the r/n≈.18 synthetic loses at about1.546. R_floor=.321 and R_warm=1.010 weaken a mandatory sequential-store/cold-destination explanation; they do not uniquely identify the production bottleneck.

## 2. Primary sources and their exact reach

Page numbers below are PDF page numbers unless an article page is written as 9:7 or12:2. Sources were read, not merely found in bibliographies.

| Source | Verified claim and location | Applicability boundary |
|---|---|---|
| [Kärkkäinen–Kempa–Puglisi, Slashing the Time for BWT Inversion, DCC2012](https://www.cs.helsinki.fi/u/tpkarkka/publications/dcc2012.pdf) | PDFpp3–4, Theorem1/Fig2: LF uses cumulative counts and occurrence rank; its dense output can be written in L order. pp4–5: pair symbols and independent traversal starts reduce walk misses/exposed latency. pp5–7: run continuity supports Precopy; its representation encodes repetition metadata within LF. | These are different constructions and walkers, not evidence that #269's emitter is cheap. The text assumes a unique sentinel. Multi-start metadata and preprocessing cost are real. Historical speedups do not transfer to current flagged multi-lane walking. |
| [Kärkkäinen–Puglisi, Cache-Friendly Burrows-Wheeler Inversion, CCP2011 preliminary](https://www.cs.helsinki.fi/juha.karkkainen/publications/ccp2011.pdf) | PDFpp2–4, Figs2–3 and §IV: alternative inversion directions, packed symbol/link layouts, and copying already recovered repetitions. | Repetition-copy is a walker change. It does not supply a record-free MTF emitter. Reverse reconstruction and output retention must fit this decoder. |
| [Brown–Langmead, Bounding the Average Move Structure Query…, SEA2026](https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/LIPIcs.SEA.2026.9/LIPIcs.SEA.2026.9.pdf) | pp9:5–9:7, Theorem2/Corollary3/Lemma6: length capping bounds average work over distinct positions; full traversal amortization assumes a single cycle. LF/FL run structure permits O(r) construction using character-ordered intervals. | Strong adjacent support for run-prefix placement, not a promise of zero ordering/metadata work. The present counting sort is already linear. Its cyclic-input premise needs checking before using the traversal bound. |
| [Brown–Gagie–Rossi, RLBWT Tricks, SEA2022](https://drops.dagstuhl.de/storage/00lipics/lipics-vol233-sea2022/LIPIcs.SEA.2022.16/LIPIcs.SEA.2022.16.pdf) | pp16:6–16:8, §§3.1–3.2: run/offset tables, per-character monotone mappings, block encoding, sampled differences and entrance/exit samples. | Sampling buys compressed random access at a cost; this is not plain-literal output without metadata. Genomic alphabet/query results cannot price a 256-symbol construction stage. |
| [KKP, Hybrid Compression of Bitvectors for the FM-index, DCC2014](https://www.cs.helsinki.fi/group/pads/hybrid_bitvector.html); [Chiu–Kempa, Fast Select Queries Using Hybrid Bitvectors, SEA2026](https://drops.dagstuhl.de/storage/00lipics/lipics-vol371-sea2026/LIPIcs.SEA.2026.12/LIPIcs.SEA.2026.12.pdf) | The authors'2014 implementation page verifies per-block encoding selection; its paper spans302–311, but its full text was not obtained. The2026 paper pp12:2–12:4 describes plain/minority/run encodings with block, superblock and hyperblock metadata. | An applicable representation principle, not a proven inverse-BWT speedup. The2026 paper is the page-level primary source for the encoding details used here. |
| [libsais source, c284ba843dd16c159acb976f17bc19382793ea75](https://github.com/IlyaGrebnov/libsais/blob/c284ba843dd16c159acb976f17bc19382793ea75/src/libsais.c#L10899) | calculate_biPSI, lines10899–10946: ordinary byte input, bucket counters, uniform16-symbol special cases, and unrolled ordinary cases. The fast pair path also checks the corresponding second symbol window and primary-index crossing. | Plain input plus run-aware work batching, not an RLBWT/literal hybrid. mibz2 already has a related16-row optimization; simply citing RUN_LANES is not a new candidate. |

The source search found no proof that arbitrary F-output ordering becomes free. Nor a validated universal predictor of construction benefit from length entropy. Those absences are search limits, not impossibility claims.

## 3. Direct F placement: delete a representation, not “the sort” by name

Own derivation. For maximal L run i=(p_i,ℓ_i,c_i), define

    a_i = C[c_i] + Σ(j<i, c_j=c_i) ℓ_j.
    LF(p_i+t)=a_i+t; Ψ(a_i+t)=p_i+t, 0≤t<ℓ_i.

The production packed table is **T[q]=L[q] | (Ψ(q)<<8)**. Consequently, assigning q=a_i+t still needs L[q], not c_i. Writing c_i in the low byte is wrong in general.

Three different mechanisms must not be conflated:

1. **No comparison sort:** [checked_sort](https://github.com/lmmx/mibz2/blob/fd60fffada25d475bbcc2b54165b96cf5fd7a92f/src/run_stage.rs) already counts runs per byte, prefix-sums slots, and scatters records stably. It is O(r+256), not O(r log r). The2026 linear-construction theorem does not remove this implementation's asymptotic term.
2. **No F-record array:** scan L runs, compute a_i, and fill each destination interval directly. Recover L[q] by predecessor at interval entrance, then advance over its L-run boundaries. This is the relevant C31 analogue. Per-run independent binary searches cost O(r log r) probes, but they are unnecessary: the stronger resumed-cursor construction below is linear.
3. **Sequential destination order:** retain per-symbol run lists or indexed links, then iterate symbols and their runs. This makes writes monotone but reintroduces grouping, links, allocation or replay. Calling it “no sort” cannot make those costs disappear.

**Stronger construction: seed once per nonempty symbol, then resume.** After counts are final, merge sorted bucket starts C[c] with the L-run starts once, recording each bucket's initial L predecessor j[c]. Keep j[c] alongside its advancing destination a[c]. Every later run of c begins exactly where its preceding F interval ended. Its cursor therefore only advances within c's disjoint F bucket. Summed across symbols, subsequent predecessor advances cross at most r−1 L boundaries, not256r. Seeding costs O(r+256), cursor state O(256), and filling visits the same union-partition cells in a different order. The oracle checks table equality and the crossing bound for all9,840 columns. This removes repeated predecessor searches without restoring an F-record array. It is a derivation, not a claim that existing C31 lacks it or that its256 live cursors are cheap.

**Tail obligation:** sorted fill can round a cell store up to8 because a later, higher destination repairs the excess. Direct L-order emission cannot blindly inherit that proof. For L=bba, emitting the bb F interval first and the a interval last lets the final rounded store overwrite already-correct higher destinations. The oracle records wrong links3,4 in a three-row table. This is a counterexample to an adaptation, not a finding against the already reviewed C31 implementation. Use exact/masked tails or prove destination order/ownership; parallel shares also require disjoint footprints. A narrower optimization is possible: runs of each symbol arrive in increasing F order, so an overstore confined to that symbol's bucket can be repaired by later work in the same bucket. Only its final boundary needs exact protection. Require w+8≤bucket_end for every rounded vector store, and an exact suffix otherwise. Across helpers, substitute the helper-owned bucket subinterval for the whole bucket. This may avoid a scalar tail per run; it requires a new footprint proof and cannot inherit sorted-fill safety by assertion.

**Falsifiable CI prediction:** a source-matched direct-F arm eliminates the F-array writes/reads, but changes predecessor probes, tails and stream scheduling. Measure all of these and build+following-walk together. If removal of F materialization fails to pay for those changes, or stage still loses after emitter/fallback controls, this candidate fails despite a faster fill. Keep #270's helper max/barrier critical path separate from the serial phase budget.

## 4. The orientation alternative: why require Ψ construction?

Own application of the classical LF formula. A run can instead fill

    U[p_i+t] = c_i | ((a_i+t)<<8).

These are sequential L-order stores. The symbol is constant within each run. No L[q] destination load, no intersection of L and F partitions, no F-record array, and only one per-symbol count advance per run. The table remains4n bytes; the representation/walker contract changes.

Recover the BWT-domain text backwards: start at the primary row; emit U[row]'s byte into output[n−1−k], then row=U[row]>>8. The independent oracle checked this on every ternary text of lengths1–8, **9,840 texts**, including periodic texts. It separately checked direct-F table equality for9,840 arbitrary columns. This is finite mathematical checking, not bzip2 decoding validation.

**Where the cost moves:** current forward Ψ walking, flagged lanes, segment discovery, output assembly, RLE1 decoding, CRC ordering and primary-index convention all need re-derivation. A conservative reverse intermediate costs n byte stores and a forward read into RLE1; direct reverse RLE1 is not granted. Existing parallel readiness and memory lifetimes can change. Malformed-input obligations remain. No production edit here.

**Falsifiable CI prediction:** this orientation removes the constructor's destination-byte load chain and replaces union-partition cells by L runs. Constructor operation counts should reflect that exactly. It merits implementation only if projected savings plausibly exceed reverse-output/RLE1 and walker costs; its eventual comparison must include those costs and the whole decoder. A faster U builder alone is insufficient. This is the highest-impact new adjacent design, but ranks behind matched C31 in near-term test readiness.

**A second transfer trap:** periodic cyclic BWTs need not produce one LF cycle. Text abab has L=bbaa, LF=[2,3,0,1]; four steps from0 visit0,2,0,2. Thus the2026 paper's full-single-cycle amortization cannot be quoted unchanged for every bzip2 block. The example refutes the premise's universality, not the claimed theorem or all possible adaptations. A compressed walker needs a separate periodic-block bound or a worst-case-safe fallback/balancing argument.

## 5. Literals without records: separate the cheap stream from its index

The direct lesson from hybrid bitvectors is **local representation choice with access metadata charged**. From libsais: batching run work while keeping byte input is a valid competing arrangement. Neither source establishes that a hybrid stream wins here.

For a prospective representation with l explicit literal bytes, h extension descriptors of k bytes each and metadata M, payload is l+kh+M. Compare with4r+4 actual record bytes, not4n. For the current valid MTF stream, r equals the number of nonzero-rank literals plus one if it begins with a RUNA/RUNB group. A literal followed by zeros forms one run; “number of extension groups” is not “number of runs”.

Two bounded forms:
- **Plain literal regions + compressed long-run regions**, with per-region decoded-start/payload-offset checkpoints. This avoids a position word for each ordinary literal. Random L[q] access needs region location and local decoding/rank; direct-F construction pays for that access.
- **Byte L + run hints**, preserving O(1) L[q] but retaining n-byte emission. This targets table work, not the #269 emitter loss; it competes against an already run-aware production baseline.

Longer bit-packed starts are not automatically cheaper. A20-bit row position plus8-bit byte needs28bits, so a three-byte record cannot carry it unchanged. Sampled variable-size records trade payload for reconstruction, alignment and branches.

**Falsifiable CI prediction:** freeze a concrete format and index first. Compare whole MTF+construction+walk and peak scratch, with descriptor/header bytes and lookup operations counted. Reject if reduced emitter time is lost to destination reconstruction. For byte-L+hint, expect no claim of eliminating L emission; success must come from less table work. Do not introduce Huffman/MTF fusion merely to erase a local store if it serializes the scanner.

## 6. Admission: run lengths help, but the histogram is insufficient

The present selector uses symbols.len/max_len, not measured r/n. [mtf_runs_scalar/avx2](https://github.com/lmmx/mibz2/blob/fd60fffada25d475bbcc2b54165b96cf5fd7a92f/src/mtf.rs) makes a stronger observation possible: nonzero MTF ranks always change the front symbol; a following RUNA/RUNB group extends that front. Thus valid-stream run starts and lengths can be recovered without knowing byte identities, with the initial group/EOB and malformed arithmetic handled explicitly.

C41 can gather n,r, extension groups, max length, bins and row-weighted mass in runs≥8 or≥16 during existing symbol production. Charge finalization/counter work. For baseline comparison, add a more specific feature: **coverage of aligned16-row windows wholly inside a run**. For run[p,p+ℓ), the number is

    max(0, floor((p+ℓ)/16) − ceil(p/16)).

This counts the windows actually eligible for build_one_pass's fixed16-row fast path, unlike a long-run fraction. Columns a^16 b a b^16 and a b^16 a^16 b share n, byte counts and the run-length multiset, but have respectively1 and0 eligible windows. This is a constructor-domain witness; no claim that both are valid cyclic transforms is needed for that identity.

For run fill, let D be the sorted union of L-run boundaries and F-interval boundaries, including0,n. Cell lengths d_j are adjacent differences:

    m=|D|−1 ≤2r−1; W8=8 Σ_j ceil(d_j/8).

An exact, untimed witness **with valid cyclic transforms**:
- text aabca → L=caaab, cells[1,2,1,1], W8=32;
- text aacab → L=bcaaa, cells[1,1,1,1,1], W8=40.

Both have n=5, counts(a,b,c)=(3,1,1), r=3, run lengths{1,1,3}. Therefore r/n, longest run, run-length entropy, every histogram bin and row share≥8 are identical, but store work differs. Entropy is not a sufficient statistic. No before-MTF predictor based only on that histogram can determine exact F/L overlap.

**Falsifiable CI prediction:** first test a prospective model using cheap n,r, extension and aligned16 coverage; then assess whether residuals track measured m/W8 and per-symbol stream structure available later. Freeze coefficients/admission before the evaluation evidence; preserve registered workloads and estimand. An oracle that picks the faster arm after seeing each block's timings is only a ceiling, never a deployable selector. Failure to reduce weighted misselection cost including instrumentation rejects the predictor. A low run density alone cannot qualify it.

## 7. What can bind build_one_pass?

The inspected scalar path in [src/bwt.rs](https://github.com/lmmx/mibz2/blob/fd60fffada25d475bbcc2b54165b96cf5fd7a92f/src/bwt.rs) has this address/data chain:

    c=L[i] → s=ftab[c] → bounds check and L[s] load → packed word → T[s] store
                        ↘ ftab[c]=s+1 feeds the next occurrence of c

Across different symbols, work can overlap. Repeated symbols serialize a shared cursor unless the16-row fast path batches them. Destination streams are per-symbol monotone, not arbitrary independent random stores. Candidate bottlenecks include counter recurrence, dependent L[s] latency, bounds/control work, front-end throughput, load/store ports, and live-stream pressure. Native listings and counters have not been inspected in this report; no one of these is established as binding.

The sequential store reference removes address generation, L[s] dependency, checks and stream competition along with other work. Warming T changes initial residency but not those dependencies, and does not independently warm/test every source load. Neither ratio excludes all store-system effects.

**Falsifiable CI predictions, diagnostic only:**
- Precompute destination indices in a separate control to break the counter-address chain: reduced construction time supports that chain's importance, but the added array traffic and preprocessing exclude direct adoption.
- Exact run batching should reduce counter updates and dependent address operations in proportion to covered rows; no time response weakens recurrence as the binding term.
- Match destination/input residency and buffer identities, and inspect generated instructions, before interpreting cache/port counters.
- LF-oriented construction should remove L[s] reads structurally. If only the builder improves, reverse-walk/output overhead is the remaining problem; if the builder does not, arithmetic/dispatch/store supply may dominate.
- For fallback attribution, matched calls with precomputed decisions, equal capacities and swapped persistent buffers separate dispatch from history. Restore production histories for the final comparison.

The papers on cache-friendly inversion primarily accelerate traversal; their cache-miss arguments do not diagnose this constructor. “Scatter-free LF” is a real classical orientation, whereas “scatter-free packed Ψ in current layout” still owes stable grouping or interval placement. Name the representation and phase.

## 8. Result and next value

Expected: literature could bypass the measured emitter/sort losses. Found: close linear run-prefix analogues and a stronger orientation alternative, but no free conversion to the present packed Ψ contract. Measured gap closed: **none**. Mechanism learned: destination-symbol recovery and partition geometry are representation costs; histogram-only admission loses relevant geometry; current data do not isolate a unique hardware bottleneck.

Next value: matched C31 plus fallback attribution is ready to answer the smallest open cost question. In parallel as design work, price sequential LF construction against reverse output/RLE1 and the existing walker before committing an experiment. Hybrid literal formats and compressed walkers remain conditional on that accounting. No rule, minimum, workload or HOLD is relaxed.

Artifacts: [oracle.py](oracle.py), [oracle.json](oracle.json), [routing.json](routing.json). The oracle is an untimed mathematical checker, not production code. Pinned coord.py audit reports only five previously resolved historical misroutes; route selects #145. Its post-publication verify hard-codes FROM coordinator; exact body/#145 and a role-aware check will be recorded separately without disguising that limitation.

