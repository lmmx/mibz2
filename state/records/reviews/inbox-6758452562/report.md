# Inbox 6758452562 — #324 rescue, then two-state tunnel pricing

Spec pin: a2da49638ac54d7763de1c167bd054f67ffdc763. Proposal/audit only: no constructor, candidate build, decoder execution, timing or merge.

## A. Rescue #324 first

**Disposition: high-priority untimed causal isolation, then at most a prospectively registered corrected candidate under D082. #324 remains FAIL.** Three target-data gains make this a better immediate opportunity than an uncounted tunnel representation. Off-path harm is real evidence about the process, not proof of a slower new algorithm and not proof of mismeasurement.

Pins: #324 e88c063155c3e734e748e99ccda025393302720f; CI table base 5e7b995471591eacdd5ae32cebfb8bbb578cbff5; the local profile base is separately 3e1a7ec361ee6c8ea6b540264aa69189d4823d9e. Guardian D082: 605d3a22013016cd1feb7a1e21d2eb6e47c8b053, REVIEW6758456856. Source and four archived Callgrind profiles independently read; registered wall-time figures below are reported results, not a new raw-paired replay.

### A1. What the evidence actually isolates

Reported target 1t wall changes: Wikidata −12.8/−14.5/−15.8%, text −23.3/−24.5/−26.2%. Random 1t +3.4/+5.3%, records +1.5/+2.1% in runs2/3, on EPYC7763/9V45 rather than the first Intel host. Architecture and run are confounded; neither AMD causality nor runner noise is established. Records2t/4t stay unresolved.

The selector is once per block: !cooperative && reads_runs && max_len<2^24 && 2*syms.len<=max_len. Runs has two 1024-byte arrays plus three Vec headers, about2120 bytes with 64-bit three-word Vecs. Arrays zero at default construction; the Vec backing allocations do not exist until needed. A source field appended to a Rust-layout struct does not establish a stable field offset, caller frame, or alignment. Production callers must be inspected, including worker closure storage, not just decode_block's argument.

Independent archived replay (inclusive drv::measured, one fresh-Scratch block):

| corpus | base | head | delta | MTF inclusive, both | room walk inclusive, both |
|---|---:|---:|---:|---:|---:|
| random b0 | 84,474,095 | 84,474,156 | +61 | 60,252,556 | 9,253,915 |
| records b0 | 95,404,355 | 95,404,416 | +61 | 71,102,362 | 9,392,629 |

**New attribution: +61 is not a measured selector cost.** Both profiles decompose exactly into +6 caller self-Ir, +7 caller memset Ir (149→156 across two calls), and +48 in a newly outlined mibz2::finish. Base calls unrle_avx2/CRC directly; head calls finish, whose self cost is48 and whose two callees retain their exact counts. MTF self/callee counts and room self/callee counts match, including reserve/malloc calls. This is a concrete changed call boundary, not speculation about all code being unchanged. It is evidence in this diagnostic binary, not proof of the production binary's inlining or hot addresses.

61/base is only0.0000722% random and0.0000639% records. This rejects ordinary proportional added instruction volume as the explanation of percentage harms; it is not a cycle bound. A new branch, alignment, dependency or cache effect can change cycles without changing Ir. The driver's Scratch::default is outside measured; so are scanner/Huffman work and process setup. No inference that the full process differs by only61.

### A2. Ranked mechanisms and cheapest no-timing falsifiers

Ranking is investigation priority, not posterior probability. Static evidence can reject necessary premises, not measure a hardware penalty.

| rank | mechanism | cheapest untimed discriminator / rejection condition |
|---|---|---|
| 1 | Caller/data geometry changed by inline Runs; hot literal count RMW versus MTF stack accesses | Exact production binaries: archive actual counts base, accessed counts slots, MTF spill/local addresses and caller stack depth at the same call sites, residues mod1024 and4096; include heap backing addresses. Compare master, failed head and one boxed proposal. Identical relevant address relationships reject the proposed *shift* for that site. A changed Rust field offset alone neither establishes nor rejects it. If boxing fails its predeclared restoration test, reject that remedy's rationale before timing. |
| 2 | Changed shared caller inlining and instruction/code placement | First inspect the observed finish boundary, then all production MTF/Huffman/room callers: normalized instructions, calls, spills, loop entry offsets, text addresses modulo cache line/page and hot text footprint. Identical bodies reject changed-instruction/inlining explanations at those sites; identical hot placement rejects simple alignment displacement. Different hashes/positions are leads, not evidence of I-cache stalls. Diagnostic finish outlining makes this rank concrete. |
| 3 | Stack footprint, page touches and translations, distinct from false dependencies | Record actual worker object placement, frame sizes, stack probes, zeroed byte ranges, touched pages and allocation lifetimes once, untimed, through the real entry. No extra touched pages rejects extra-page-touch at that boundary; no changed translation/address stream rejects this specific TLB mechanism. A ~2KiB size delta alone does not imply an extra page per block. Construction/zeroing outside measured must be charged. |
| 4 | Branch predictor aliasing, including code relocation and new selector history | Compare branch PCs/targets and dynamic outcome sequences through the same process entry; if both history and relevant placement are unchanged, a new-branch-history/PC-displacement account fails there. Unknown proprietary predictor indexing prevents a complete static aliasing falsifier. Count mispredict events only in a separately authorized diagnostic; identical Ir cannot settle it. Do not present a guessed predictor index as proof. |
| 5 | Runner/order/frequency variance | Reanalyse existing paired samples, order, CPU model, governor/frequency/affinity/steal and confidence intervals, with no new timed runs. A transcription/order error can be rejected or found this way; variance itself has no universal no-timing falsifier. Repeated signed harm on two hosts merits investigation; one host per run cannot separate architecture from run conditions. Preserve the frozen verdict. |
| 6 | Direct selector bookkeeping | Source plus executed call graph/counts already reject a per-row selector in these two profiles. It executes at block granularity and the entire delta is61. Check full production entry before extending this bound to all inputs. A selector branch's indirect predictor/code-placement effects belong to ranks2/4, not a million-fold unexplained Ir surcharge. |
| 7 | Runs backing allocations contaminating fallback-only inputs | Check complete eligibility and per-process allocation trace, fresh vs reused Scratch, with no timing. Empty Vecs allocate no run buffers in default; on a process containing only nonqualifying blocks this proposed run-buffer mechanism is absent. Confirm process boundaries in the registered runner before extending that conclusion to CI. Mixed blocks in one worker retain capacities and can affect later fallback. +1–10% RSS on qualifying data does not demonstrate allocations in another corpus/process. |

The old #277 codegen journal §6 explicitly withdraws its mod4KiB discriminator. The1024-byte counts array covers *every* residue mod1KiB. Which slots are accessed and their frequencies matter; a clean4KiB window is not a clean1KiB control. Old ~50 cycles/exposure was a fit, not a measured latency. That journal's two hypotheses remain live. Do not inherit an uncorrected §2–5 claim into the rescue.

### A3. Which correction is worth a falsifier?

1. **One boxed-Runs diagnostic is a sensible first narrow change**, because it removes ~2KiB of inline arrays without changing the selected algorithm. It is not a verified correction. Eager Box adds allocation, zeroing, pointer indirection and drop even to fallback; heap placement itself changes. Observe actual production frames/addresses and the common callers before selecting it for a fresh registration.
2. **Lazy separately owned per-worker Runs** is the better fallback-only cost contract if the API/lifetime permits it: allocate on the first qualifying block; reuse thereafter; charge retention on mixed inputs. Option<Box<Runs>> and an external per-worker sidecar are both separate allocations, not distinct causal theories. The external sidecar can preserve more of the original Scratch shape, but caller state/closure layout and code placement can still change. Extra allocation/zero/drop costs and pointer loads remain on qualifying data; optional tests remain. Do not silently combine this with outlining as an unidentifiable multi-factor rescue.
3. **Stable padding/alignment** is a diagnostic control only until actual constraints justify a layout contract. repr/alignment can enlarge every worker and cross pages; field alignment does not align Vec backing memory. Padding search until pass is forbidden. counts spanning1KiB precludes a universal modulo1KiB non-overlap promise.
4. **Outlined selected body** addresses code footprint if rank2 evidence wins; the cheap selector should remain at the dispatch boundary. An always-executed outlined selector adds a call to fallback. Marking a frequently taken path cold on Wikidata/text is unjustified and can damage the very gain being rescued. Outlining cannot by itself restore data/frame geometry.

Economic next step: read the pending three-way *production* geometry and code comparison. If the one boxed change restores the specified geometry without new fallback work of consequence, proceed to Researcher assessment and D082's fresh frozen sequence. If geometry does not discriminate, redirect to the observed shared call/code-placement change rather than explaining the failure away.

D082 remains binding: preserve exact failed head and all three outcomes; no fourth #324 attempt. Exact new head/base, one substantive delta, full costs and falsifier, all corpora/thread cells and original thresholds/rounds/estimator. New sequence runs1/2 plus conditional3, maximum3, not a single run that cannot satisfy the other-run condition. B5 mutation and independent adversarial obligations remain before production admission. No runner shopping or post-hoc padding sweep.

Expected versus observed: large target gain, off-path harms failed admission. Gap closed here: the +61 replay and its call-boundary decomposition, plus rejection of the stale4KiB discriminator. Cause and corrected whole-decoder gain remain unknown. Next experiment's value is high: isolate collateral cost while retaining the demonstrated target opportunity.

## B. Two-state tunnel lane: certificate first, then price

Pin f666ea9f3e611e114211c58172d5d2482ce15010: tunnels journal blob feac6abe1e1e4aeff3a865538ae95d7eb91eee57; phrase_census.rs blob 0818b651180416bcfd04536ed9107ded4a55197a. This is source/algebra review, not a newly executed corpus census. The supplied D4-delta walk count is still in flight: its ~18 inversion figure is a paper estimate, not a reproduced counted baseline.

### B1. The census has an off-by-one certificate

**REVISE before using its heights as shared words or entry-to-exit translations.** The source defines K[q]=0 at a cell start, otherwise1+K[Psi(q)]. For adjacent rows inside a cell, K is the number of certified equal-emission/adjacent-transition steps *before* a cell boundary. Therefore a multi-row cell [a,b) has safe height **min K[q], a<q<b**, not1+min K. The pinned source uses the latter in both word and raw. Its fixed-stride start-union construction, separately, does agree with K>=k.

Minimal valid BWT counterexample, algebra replay attached:
- text aab; sorted rotations aab,aba,baa; L=baa, F=aab, Psi=[1,2,0].
- Cell[0,2) has interior K[1]=1. The census assigns height2.
- Row0 emits aa and exits at2; row1 emits ab and exits at0.
- Thus neither a shared two-byte word nor a single affine two-step exit exists. Height1 is valid.

TUNNEL mode advances on the actual trajectory and increments lengths; it does not validate a stored common word and affine exit for every row of a cell. A counted trajectory advance is not a safe table certificate. The assertion near the census start checks the ordinary Psi trajectory against bwt::walk, not these tunnel words. Correct the height convention and verify both word and exit identity before repricing. This correction can change steps, bytes/step, word storage and construction cost; don't just subtract one from a published aggregate.

A second independent issue: **48.1% of bytes in phrases>=33** comes from the earlier phrase census, not certified whole-cell words encountered by the segmented fixed-stride walk. All-row coverage, whole-cell greedy-walk coverage, phrase coverage and stride-k landing coverage are different measures. Sampling only the origin's residue class is not the production walk with512 segment origins. 0.0736 lookups/row is conditional arithmetic, not a proved upper bound. Likewise short-step counts and an uncounted loop do not disprove *every* whole-tunnel construction; they make the particular plain constructor/variable-word loop economically weak. No line-level absent-advantage conclusion follows.

### B2. Construction that preserves the room

Proposed bounded variant: same fixed-k output quantum in both states; **never a variable-length bulk copy inside the ordinary pass**.

- Ordinary state: existing k-step record, including its short-step/marker exception. A known tall-entry tag may enter copy state after paying the ordinary directory/entry access. Recognition must not require a second rank on every ordinary pass. Charge any sidecar lookup.
- Certify safe h, round down to a multiple of k, and store the word plus an affine exit for that exact rounded length. Precompute that exit during construction. A certificate for h bytes does not license using the unrounded exit after floor(h/k)*k bytes.
- On entry cache word pointer, remaining quanta and actual exit row computed from the entering row plus the certified delta. The first quantum still paid the ordinary lookup. Subsequent copy passes load k contiguous word bytes, store k output/tile bytes, advance pointer, decrement remaining, and branch on exhaustion. Reinstall the cached exit row on completion. No table rank during those passes.
- Both states advance the same shared output offset by k. Room bookkeeping survives. Changing a lane from copy to ordinary does not mean changing its segment or independently advancing its output offset.
- Words are prebuilt immutable data. Copying from another lane's partially produced output would be a different algorithm with dependency and ownership costs; it is not this price.

Markers are row-specific even when the output word is common. A simple bounded certificate: for maximum burst H, mark the union of the H−1 preimages before each segment start, along the actual navigation direction. At entry reject copying on marked rows, or use a separately proved shortened descriptor. Rejection reuses the ordinary marker-safe k-step path. At most S(H−1) visits construct this union, overlaps allowed. Existing k-step flags cover only k−1 predecessors and cannot certify an H-byte burst. A one-bit unsafe map costs n/8 bytes if separately allocated, **112,500B at n900k**; reusing a proven directory flag layout may avoid it but is not free by assertion. Charge initialization, backward navigation and row-specific test at entry. After entering an unmarked certified burst no intermediate marker checks are needed.

Tail: leave fewer than k bytes to the existing exceptional path; do not overread a word or overwrite output across a segment. Segment completion/exhaustion clears copy state before another claim. LF emits downward: pre-reverse the stored k-byte words at build time or charge the byte reversal; the Psi census alone proves neither LF word layout nor its visited heights. A copy remaining at a chunk boundary resumes with its pointer/count/exit intact. The safe word budget must account for padding if any unaligned fixed-width load reaches beyond allocated words.

### B3. Price: the saved rank is not the whole pass

Define n inversion rows; P=n/k full quanta ignoring charged tails; E tunnel entries; C total copy-designated quanta including each entry's first quantum. Let:
- L = ordinary navigation+emission Ir per quantum, excluding unchanged common skeleton;
- Q = copy-body Ir per later quantum, on the same boundary;
- s = added state dispatch Ir on *every* quantum;
- e = extra entry/exit recognition, descriptor and initialization Ir per entry;
- X = additional tails, marker handling, spill/flush and setup Ir.
Then relative to the **same k-step baseline**:
**saved walk Ir = (C−E)(L−Q) − P*s − E*e − X.**
No duplicate credit for the first quantum. Add construction delta before claiming inversion saving; use actual C,E and per-worker X.

Paper budget, not compiled counts:
- dispatch test/branch roughly2 Ir/quantum if state already loaded; more with reload/spill;
- copy body load/store, pointer update, remaining update/test/branch roughly5–8 Ir/quantum, plus state-array traffic, not2;
- entry/exit roughly8–16 additional Ir before a variable sidecar search or marker-specific repair;
- a false-to-true and true-to-false transition per burst encourages long runs, but there is no prediction of two mispredicts per entry. A common lane-loop branch interleaves states; unrolled distinct PCs, global history, mixed ordinary flags and compiler if-conversion all matter. A branchless selection that still executes rank defeats the intended saving.

At the **unverified** f=C*k/n=.48 and E/n=.0086 for k8:
- ordinary lookup groups/row = (1−f)/8+E/n = **.0736**, saving **.0514** groups/row.
- s=2,e=12 already costs **.3532 Ir/row** before X.
- Break-even L−Q > **6.8716 Ir/saved group**, before build, markers and spills.
- If L−Q=8, only **.058 Ir/row** remains; if20, **.6748**. One extra state load plus store per pass costs another.25/row at k8, enough to erase the first case.
- A directory+entry lookup is not one memory access; copy state substitutes a sequential word read, it does not eliminate all reads. The critical path changes from directory→entry→next row to pointer→word and eventual exit. Lanes can hide either dependency. No conversion from lookup counts to wall gain is justified.

This explains why “48% bytes copied” can yield little *instruction* gain. It can still reduce exposed random latency or contention; that requires matched evidence, not Ir alone. k4 may have a larger saved-group budget, but its observed f,E are unknown and cannot inherit k8/phrase statistics.

### B4. Live state and construction are decisive

For32 lanes, a straightforward addition is pointer8 + remaining4 + cached exit4 =16B/lane, **512B/worker**, over existing row state (another128B). A separate tag or wider counters can make it640–768B total. A union/packed tag may reduce this; prove representable range and price extraction. This is over32 active lanes, so register spills/state-array traffic are a real risk. No new coordination is intrinsic: share-nothing keeps one owner per block. With 256 output bytes/lane a staged tile stays8KiB at either k; it holds256/k quanta, not256 k-byte words.

Per-worker live set also includes:
- Dk directory and records at *capacity*, words and padding, sparse/dense entry descriptors, marker map, optional safe-length data;
- builder levels, word-construction traversal/memoization, validation, retained run records, output segment buffers and final output;
- existing staging tile, plus pointer/count/exit state. Multiply private costs by active workers; charge leader/build barriers if shared.

D4-delta's previous density model is1,089,186B/900k rows (1,064,610 table plus24,576 singleton reserve), not a measured capacity. Under its unchanged1.2MB filter only**110,814B**, or**.12313B/row**, remains even before new descriptors/marker capacity. **.2B/row words alone add180,000B and miss it**; a separate one-bit unsafe map alone costs112,500B and also exceeds that residual. At .3B/row the extra270k is larger still. Thus the advertised <.2 words target is not sufficient to preserve D4's admitted footprint.

Eight spare bits in a D4 u64 can hold a tag, not an arbitrary word offset, safe length and exit. A dense two-byte side index per piece at r4/n=.1088 costs195,840B before words; a sparse descriptor map pays lookup/bitmap/directory cost. Words for only *visited* cells are not available for free before the walk. Any lazy build must charge its discovery, state, allocations and dependency.

The plain word constructor's reported .18/.30/.49 cell-steps per row at caps16/32/64 is not an amortized constant; a corrected census and selected-cell method must reprice it. No memoized construction is specified enough to assign it zero. D8 additionally inherits the too-large table concern already recorded in REVIEW6757974415; a k8 arithmetic win over D8 does not qualify against smaller D4.

### B5. Decision and cheapest falsifier

**Can it beat D4's counted walk? Possible, not established. No supplied raw D4 count licenses a numerical yes.** Keep the registered D4 filters unchanged: walk≤15Ir/output byte, construction≤7.35Ir/row, table capacity≤1.2MB. Do not replace its walk baseline with the18-Ir paper *inversion* estimate. Compare complete same-k ordinary/hybrid loops first; a D8 hybrid must then beat D4 build+walk and capacity on common blocks/segmentation, not merely D8 lookup density.

First falsifier is now **certificate validation**, cheaper than a new kernel:
1. Fix the K convention; for each admitted cell and rounded h verify equal h-byte words and affine exit on every row against the already available trajectory/navigation oracle. Include aab, singleton/bucket boundary, exact k boundary, marker inside burst and shortened last block. This review's algebra reproduces the aab counterexample without executing decoder code.
2. Extend the existing *descriptive* D4 probe/census, outside timed/counting regions and without changing its admission filters: actual segment-start phases, direction and marker plan; C,E; full safe quanta, rejected/rounded bytes, tails; all eligible and actually visited word bytes separately; descriptor/map capacities. Height data is not free if recomputed inside the measured constructor. A pure origin-stride landing fraction cannot supply C or E after copy-state excursions.
3. Apply the formula above with explicit compiled ordinary-loop costs when the D4 probe returns. Abandon this particular sidecar/copy construction if even its optimistic instruction saving cannot repay state+entry+builder costs, or it misses the unchanged capacity bound. “Eligible fraction<one third” is not a theorem: a few very long bursts can dominate; a high hit fraction can still have bad transition/metadata cost.
4. Only a surviving bounded design earns a full-loop counting prototype: same blocks, direction, segmentation, all setup/tails/callees and exact word/exit equality. Runtime tests and CI timing belong to the Coordinator's later authorised registration, not this report.

Priority remains #324 rescue first; staged-team gap/count next; existing D4 probe continues unchanged. Corrected tunnel counters can piggyback descriptively, but a fresh hybrid kernel is economically behind those opportunities. Whole-cell plain construction is weak under the stated model; tall-tunnel advantage remains open, contingent on a valid certificate, selective word construction and a smaller metadata representation.

## Evidence and execution boundary

Four historical Callgrind files parsed as text; source and journals read at named pins; one three-row algebra counterexample and cost arithmetic executed. No native candidate, constructor, decoder, timing, CI test or unsafe certification performed. No new performance credit. Raw profile base is 3e1a7ec361ee6c8ea6b540264aa69189d4823d9e; CI table base is5e7b995471591eacdd5ae32cebfb8bbb578cbff5. They are different evidence boundaries.

Migration checked before receipt: previous branch e6eea5889432aef28f4d9e450cee87dd5bf7bacf; manifest SHA25686d7a8a6ea42c2d2ade70c5ce3f0c82f375c0fd6291dbf452979bf07b100dbde. Protocol audit: five resolved historical findings and already reported LOG6757065549 on162; no new finding. Coordinator6758470358 repeats D082's actual-geometry restoration rejection test; it remains binding. New i9 gap/no-gap discussion is independent, not grounds to discard #324's EPYC results.
