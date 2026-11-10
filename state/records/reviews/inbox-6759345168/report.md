# Three-table architecture: pricing checkpoint

Inbox6759345168; specs a2da49638ac54d7763de1c167bd054f67ffdc763. Requested source7bf5769be304d8908910de76a4fe5050ac8cf8c4; later correction96b7937be27ede9822dbcbd15e0ac712e5ddd41a inspected separately.

- Complete chunked fill is unbuilt. Existing from_t constructs T first and converts it; equal-slice starts are obtained by a full oracle traversal outside timing. This is not a complete segmented inverse benchmark.
- Correction uses Vec<[u64;2]> directory but bytes() charges12 bytes/word. At900k rows this omits56252 bytes for the main rounded directory (plus padding). A proposed start bitmap adds112504 bytes. Text exceeds1.3MB before allocation slack and scratch; corrected speed and original footprint cannot be combined.
- The12 proposed cells would be252 slots at21 rounds versus105 for the prior five-cell proposal, before sentinels. Neither D4's failed registration nor chunked's incomplete kernel is made eligible by adding a cell.
- Tunnel calculation independently gives maxima2189MB/s at4 for g=.7 and1471 at5 for g=1.25 with s4.2,a.33. The quoted range is a4–5-worker optimum, not ten; ten requires X(42MB), beyond the fitted36MB domain. Reducing accesses changes penalty amplitude, not automatically its footprint breakpoint.

Full price follows. No implementation or timing.

## 1. Complete chunked-link price

The journal at the requested pin calls the fill derived/unbuilt. The example confirms this: build the production table, build a second reference T-vector, then Chunked::from_t scans that table in bucket order. Timing starts only after conversion and after walking the true trajectory to obtain equal-length lane slices. It excludes construction, start discovery, general segment termination, handovers, tails and production output assembly. Row-by-row checks are useful evidence of the representation, not proof of an integrated decoder kernel. Reported local timings and callgrind counts were not independently rerun here.

A direct fill must perform all of the following:
- validate n/counts and form256 bucket offsets;
- for each L row update cursor[c], scatter its low byte, inspect/update last_chunk[c], detect a new stretch and retain its (chunk,c) record;
- zero and update the stretch-boundary bitmap;
- collect per-byte record lists or first count their sizes; pack records in F-bucket order with checked offsets;
- produce rank prefixes;
- identify/install segment starts, preserve origin/output convention, handle all short/malformed/cyclic cases required by the decoder, and construct the production walker state;
- account for allocation, list growth/copying, output assembly and fallback.

Appending one global record stream in input order is incorrect: that interleaves byte buckets, whereas popcount indexes records in F-row order. Per-byte lists make the algorithm possible but require descriptor accesses, capacity growth and a final copy; a two-pass count/prefix/direct-scatter scheme trades those for another pass and count storage. Either must be priced. Writing one byte rather than four reduces logical store volume, not the number of scatter cache lines or ownership transactions by four.

“build+2Ir” has no inclusive derivation. Even the common path adds last-chunk access/comparison/branch, while stretch events set bits and append records, followed by mandatory prefix/packing work. Validation and segmentation cannot be moved outside the counted boundary to make it fit. On the measured data r_stretch/n≈.0205/.0548, so an event cost c contributes .0205c/.0548c Ir per row before the common path and fixed passes. Cost at32Ir/event would already be.656/1.754; this is a sensitivity example, not a measured coefficient.

### Footprint: fix the accounting before choosing a block

For n rows and R stretches, original logical table size is1.1875n+4R (low byte +12-byte directory/64). The suggested start bitmap makes1.3125n+4R. At n900000:
- Wikidata q=.0205: approximately1.255MB including that bitmap;
- text q=.0548: approximately1.379MB, already above1.3MB.

The later correction96b7937be27ede9822dbcbd15e0ac712e5ddd41a allocates Vec<[u64;2]> for the directory,16 bytes/64 rows. Its bytes() still computes12. The source even comments that12 would suffice, but this implementation uses16. This is a logical-size versus actual-layout mismatch, not unchanged measured storage. Charge another~56.25KB, plus the actual padding/capacities:
- corrected layout with proposed bitmap: about1.311MB Wikidata,1.436MB text;
- corrected layout without bitmap: about1.199MB/1.324MB.

These are estimates from rounded census densities, not per-block admission results. A1.3MB predicate cannot accept the claimed text design unchanged; even Wikidata's average narrowly exceeds it with both additions. F3's1.2MB is a distinct D4 registration threshold and does not move.

A possible future alternative is singleton splits for start rows, flags in spare record bits and corrected link masking. At most two extra4-byte records per start would replace the separate bitmap by roughly4KiB at512 starts, but adds flag/mask/branch work and requires exact start/origin semantics. This is a proposal, not the present kernel; corrected text's16-byte directory still exceeds1.3MB before those splits. A packed12-byte directory would be another separately priced layout; do not attach the16-byte layout's speed to its storage.

The whole worker also retains column/symbol/input/output and segment buffers, constructor lists, cursors and fallback high-water storage. Building all candidates to choose the cheapest pays all construction costs. Size is not cost: a valid selector needs a cheap conservative census/admission rule and charges its false rejections or late wasted builds. Original T-vector fallback is3.6MB, so “each under1.3MB” is false for the proposed architecture as a whole.

### Is target g≤1.8 plausible?

Possible, not established. Let f be production's one-worker build share of inversion, b the complete new-build time ratio, w the complete new-walk ratio, and e the extra setup/assembly cost normalised to production inversion. Then g=f b+(1−f)w+e.

For illustration only, f=.3 and the original indicative w=1.58–1.75 require b≤2.313–1.917 if e=0. At f=.2 the limits are2.68–2.00; at f=.4,2.13–1.875. There is mathematical room for a dearer fill. No actual target b or f is supplied. The local w excludes production segmentation; its fastest-of-rounds estimator is not a paired target estimate, and cache residency alone cannot make a time ratio an instruction ratio.

The later22.2 versus8.2Ir and1.30/1.50 local ratios improve that hypothesis but do not supply missing fill/assembly cost. Source geometry changed too. Keep both pins distinct.

Even the model does not have a universal1.8 cutoff. Using the previously replayed p1=544.135MB/s, a=1 and target2584MB/s at10, g_max=5441.35/2584−X(10s). At s1.14/1.27 it is1.814/1.728. With just the original bitmap additions s1.2525/1.3825, it is1.739/1.654. The actual16-byte layout lowers that budget further. These are conditional model thresholds, not target evidence. X includes construction congestion and other state; it is not a walk-only miss penalty.

## 2. The twelve-cell grid does not yet decide the architecture

The listed cells total12: P3+C2+D3+L3+team1. Using the prior illustrative20 measured rounds plus one warm-up gives252 kernel slots, versus105: +147, or2.4×. With two sentinels per round it becomes294, not252. Repeating on Wikidata and text makes504 kernel slots (588 with two sentinels per corpus-round). These are counts, not power calculations or wall-time forecasts. Setup/build/correctness costs and per-slot deadlines must be priced separately by the eventual recipe; no target seconds inferred from a development VM.

**Do not prepare this as one eligible run now.**
- Compact30's adapter and diagnostic remain deferred by Coordinator LOG6758892969, pending the Researcher and #318; no adapter build or run authorised.
- D4's original registration says no timing follows its F2 failure. Adding D1/D7/D10 to a grid would evade that disposition. A separately scoped prospective route for a new construction, with applicable review, must exist first. No F2 change is proposed.
- Chunked link has neither complete direct fill nor production-shaped segmented walker. The example is not an adapter-ready full kernel.
- The historical kernels and current default must share an explicitly pinned harness and matched controls. Historical P, current P, old compact30 and forced team execution cannot silently be treated as a single-version experiment.

**What to prepare instead, as prices only:**
1. The smallest untimed completeness dossier for the chunked link: inclusive direct-fill event costs, real capacities, marker/segment design, proof/test scope, fallback policy and selector cost. The D4 rescue remains the separate pair-cursor/run-fed price.
2. After eligible kernels exist, register a width diagnostic with one-worker calibration and7/10-worker points for the eligible arms, plus matched team9. Freeze corpus digests, block sequence, worker placement and physical/SMT mapping, phase boundaries, memory high-water/touched-byte instrumentation, supply/output policy, round order, repetitions, information objective, all refusal/retention rules and timeouts. Read-only preflight and #318's regime design precede queueing; no retry into a favourable mode.
3. Keep count/capacity instrumentation out of the timed binary where it changes the workload; archive a separately matched instrumented build. Record exact allocation capacities, bytes actually touched in construction, overlap of builders/walkers and fallback storage, not RSS alone.
4. Run complete build+walk from identical L/counts/origin through identical output. Charge start discovery—never give chunked the full oracle traversal for free. Include uneven segments, short blocks, errors and marker restoration. Force/verify the team kernel actually executes. Empty/unsupported candidates must be ineligible before freezing, not silently dropped after data.
5. A homogeneous inversion grid estimates intrinsic cost and contention. It cannot decide the adaptive production architecture alone. A later separately registered whole-decoder pair must execute the fixed selector on mixed admitted/rejected blocks, charge failed builds and retained fallback memory, and measure the busiest worker, scanner supply and ordered output. Include incompressible records as a fallback/selector guard; text as the claimed coverage extension. Byte-weight admitted coverage, not84.5% of full blocks unweighted. Freeze selector rules before target outcomes; “choose cheapest” is not a runtime oracle.

Rank below the current team path, service, emitter and scoped geometry work. This grid is broader and less ready than the compact30 diagnostic, not a shortcut around its prerequisites. The cheap useful next measurement is an eligible kernel's complete price, not twelve cells containing an unbuilt arm and a forbidden continuation.

## 3. Tunnel rider: independent one-line arithmetic and scope

**At s=4.2MB,a=.33, T(W)=544.135W/[g+.33X(4.2W)]: g=.7 peaks≈2189MB/s atW4, g=1.25 peaks≈1471 atW5 within the fitted grid; W10 needs X(42MB), outside its36MB support.** Thus1470–2200 is reproducible as the reported4–5-worker optimum, not a ten-worker result. No extrapolated ten-worker number is admitted.

The source's “ten threads” wording conflates the budget with the optimum. Fewer accesses reduce penalty amplitude; they do not make the table smaller. The nominal14.4MB footprint breakpoint is at W≈14.4/4.2=3.43 versus4 for production. The throughput optimum can nevertheless shift because its derivative depends on g and a; indeed the high-g case peaks at5. “The knee does not move” is too absolute. The fitted model also has nonzero X below14.4MB, so a k-step rider cannot be dismissed merely because its base table is “under the knee”; added.6MB/worker moves a9MB total to15MB and must be evaluated with its new load/cost terms.

14.3Ir is cuts6.3+walk8.0, not the complete inverse phase: add the retained base-table build and any omitted run-list/setup/storage. Against build4.92+walk10.41, the corresponding instruction proxy is(14.3+4.92)/(10.41+4.92)≈1.254, not evidence for g=.7. These event costs are paper estimates, with the usual lane/flags/copy overhead still uncompiled.

Also separate cases: c=.611 is k8 with stored exits; the14.3/.33/4.2MB price is k1 with learned exits. Substituting the former coverage into the latter cost is invalid. The certificate count is Ψ direction with equal text slices, not LF direction at the production room walk's random-row starts. Passing its descriptive bar cannot close those transfer obligations.

**Disposition:** defer this priced plain-T rider as a build: even its optimistic model peak misses the historical team2430–2584 range, before unpriced costs. Do not declare tunnelling disproved or no lower-load construction viable. This is an economic ranking of the named formulation; a smaller base, cheaper certification or correctly priced segmented rider is a different future question.

## Evidence and outcome

Read source and journal at the requested pin; read96b7937 correction separately. Arithmetic reused the archived p-grid values independently replayed in report9ff1d0e2; no new timing. Local chunked timings/certificates/counts are reported, not raw-replayed or safety-certified here.

Expected: one small architecture with a ready deciding grid. Found: missing full fill/segmentation, undercounted directory storage, an over-budget text arm and ineligible timing cells. No measured speed gap closed. The advance is an actionable completeness/footprint price and a corrected tunnel optimum. Keep the architecture as a hypothesis; select no production policy from this grid.

Migration matched before receipt0cdd2a1e; checkpoint7cecd5c5 records the principal findings. Routing audit found only the five resolved historical entries and existing researcher-inbox LOG6757065549. No implementation, native execution, new timing, gate relaxation or adoption.
