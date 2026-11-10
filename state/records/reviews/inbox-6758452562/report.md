# Inbox 6758452562 — #324 rescue, then two-state tunnel pricing

Spec pin: a2da49638ac54d7763de1c167bd054f67ffdc763. Proposal/audit only: no constructor, candidate build, decoder execution, timing or merge.

## A. Rescue #324 first

**Disposition: high-priority untimed causal isolation, then at most a prospectively registered corrected candidate under D082. #324 remains FAIL.** Three target-data gains make this a better immediate opportunity than an uncounted tunnel representation. Off-path harm is real evidence about the process, not proof of a slower new algorithm and not proof of mismeasurement.

Pins: #324 e88c063155c3e734e748e99ccda025393302720f; registered base 5e7b995471591eacdd5ae32cebfb8bbb578cbff5. Guardian D082: 605d3a22013016cd1feb7a1e21d2eb6e47c8b053, REVIEW6758456856. Source and four archived Callgrind profiles independently read; registered wall-time figures below are reported results, not a new raw-paired replay.

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

## B. Two-state tunnel lane

Pending pricing at this checkpoint.
