# Seven-family coverage and early-dispatch delta

Inbox [6850660172](https://github.com/lmmx/mibz2/pull/160#issuecomment-6850660172); parent [6850318070](https://github.com/lmmx/mibz2/pull/160#issuecomment-6850318070), completed by REVIEW6850572409/report247ef9a2f4d53795364c7fc697a7859c06864e82; amendment [6850583026](https://github.com/lmmx/mibz2/pull/145#issuecomment-6850583026). Policy a2da49638ac54d7763de1c167bd054f67ffdc763. Saved-data/static planning only; no decoder, generator, compiler, census or timing executed.

## Decision and one next preparation

Select **one CI-first seven-family identity/correctness/route preparation**, with the conditional all-small-spine writer eligibility question folded into its untimed Cells checks. This is preparation for the finite two-arm programme below, not an admitted job. Keep the writer as the best credible implementation falsifier pending eligibility and MONODROMY's assessment; do not select an early dispatcher as a third arm yet.

A cheap descriptor threshold separates the observed structures, but there are no per-block counterfactual speed labels and no current live-route observations for four families. Its general misclassification loss is unbounded by these data. Do not turn that separation into a shipping rule. Preserve the measured Wikidata hooks-off whole-reader ratio 0.881643 [0.856982,0.907014] on the pinned i9 workload. Pattern's loss is a reason to repair or discriminate the path, not to close the Cells line.

This delta supersedes the parent's **unadmitted 60+9 three-corpus proposal as the next coverage preparation**. It neither admits that experiment nor changes the completed 120+9 registration. No new broad writer ranking. MONODROMY queue6850491677 retains mathematical ownership; Pathfinder retains later buffer/API design.

## Evidence layers

- Historical matched seven-family CI: run [42901406328](https://github.com/lmmx/mibz2/actions/runs/42901406328), source **57f51bbc0ca62ac3ee1cc148a1010842eea6222a**, artifact13802703606 SHA256 **c36061ac6a7e8ac2991acbd82e2935fd20f9a3d075e102bd42c67b46cb575141**. The parent verified this archive. Its actual Cells slice contains 140 rows, ten pairs per family, one thread, EPYC9V74/Rust1.99.1. Arms are mibz2@bin/ab and mibz2-cells. This is matched evidence, unlike the Alternatives charts. It is a different source/build/harness boundary and host from the current live-reader experiment; no numerical transfer between them.
- Current live reader: corrected source **d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2**, recipe **cb73bf909397e2fff8c90f0a00437061ad06478e**, submitted evidence01f5bc23ea2242969b084ebff7bf3b178814f3a2, archive SHA256 **23c8712e1d1f2a64bbbb2f10232db7f21a8a8499826ac8f9ac42a2bd27374414**. Parent replay retained unchanged; only saved-row inventory/CPU summaries and descriptor screening added here.
- [coverage.json](coverage.json) contains full hashes, lengths, historical RSS ranges, paired wall/CPU summaries and descriptor counts; [seven-delta.py](seven-delta.py) performs saved-JSON arithmetic only. Historical intervals below are the original ten-pair log-t convention (df9, critical2.262), not the proposed future simultaneous rule.

## Exact seven input identities

Synthetic generation is pinned to **57f51bbc0ca62ac3ee1cc148a1010842eea6222a**, tests/common/mod.rs::corpora and examples/ab.rs gen, explicitly **scale4**, bundled libbzip2 **1.0.8 level9**. The command's omitted-scale default is8, so omission is wrong. Use the archived compressed objects if available, otherwise reproduce only in the later authorised preparation and require these exact hashes.

| Family | Compressed bytes | Decoded bytes | Compressed SHA256 |
|---|---:|---:|---|
| wikidata | 11854933 | 209700196 | `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915` |
| onebyte | 51 | 20000000 | `22924f5b3187645f97653ae4923df14d2597816c2e57f2c7228bc844119c4a9e` |
| pattern | 1116 | 19600000 | `0f07e2a274b9189677c0f22fd3fc8d15f17324b1f49e651a321239321d4e1407` |
| random | 7097921 | 8000000 | `10d444e6204289af739688e54a348940d7915957172ea65035b08b72dfdc3f18` |
| records | 9587840 | 9600000 | `ff8b5f6ed3dd44459908a01a588f6979b2544c6a444c9520741a2aa96c391f8c` |
| runs | 37995 | 10000235 | `af3be4d130faf312fe087baee037ca0d802dcf9cd6aefeefe9d9373b25a92fb8` |
| text | 2625903 | 13238014 | `f0031809969382478252695d4b219eee69da60f8ef45e845c247362bb4710d75` |

Generator details matter:
- onebyte: a repeated20,000,000 times. pattern: 0123456 repeated2,800,000 times.
- runs: append x/y/z runs of length1..600 to at least10,000,000 bytes; last run overshoots to10,000,235. Seed0xfeed_beef_dead_cafe, u64 xorshift13/7/17; draw length first, then symbol for each run. **Generate runs before random using the same RNG state.** random then takes8,000,000 low bytes; independent fresh seeding changes the identity.
- records:2,400,000 little-endian u32 words, ((i wrapping-multiply2654435761)>>12) OR (i<<20).
- **text:** clean source tree57f51bbc, sorted immediate regular files in src, then tests, then examples, followed by README.md. No recursive walk. Repeat this concatenation with the generator's numbered copy separators until at least12,000,000 bytes; actual13,238,014. A moving master checkout, extra untracked immediate file or changed source produces a different corpus. Future preparation must retain an ordered path/hash manifest of every included file and the exact compressor/library identity. The matching generator function at d4f0 does not make that tree's text bytes equivalent.
- wikidata: bench-data pin **a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a**, bench/wikidata.json pins wikidata-20261019-all.json.bz2, whole-block extraction from twelve1MiB windows at odd/24 offsets, twelve concatenated streams. Preserve compressed block bits; no recompression or synthetic scale.

Decoded SHA256 already recorded:
- wikidata: `3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf`.
- onebyte: `aded0ea9b4d06589b13d00bab483faf479d61ed5de21f1760aa7018a28e330e5`.
- pattern: `b23397674d9cede1cca8b4654870946900a1a364b9f1779d6f64df4a88d61946`.
- random: **NOT RECORDED** in the inspected archive.
- records: **NOT RECORDED** in the inspected archive.
- runs: **NOT RECORDED** in the inspected archive.
- text: **NOT RECORDED** in the inspected archive.

Historical CI: every family has equal Segmented/Cells rolling ab64 hashes and decoded lengths in all ten pairs, with empty error fields. This is recorded decoder agreement, not a cryptographic independent-oracle proof. The seven ab64 values are retained in coverage.json. Current live three-family saved prechecks have matching full-output hashes/lengths against the oracle; three large Wikidata output files were omitted from the archive, so their bytes cannot be independently re-compared here. No current live correctness result is claimed for the other four families.

Prospective missing-data acquisition: seven inputs, one untimed Segmented output, one untimed Cells output, one independent oracle output each; retain SHA256, length, CRC/error status and byte equality, including every concatenated stream. Obtain the four missing decoded SHA256 values from these checks, compare synthetic oracle bytes to generated bytes, freeze the manifest before any measured process. Missing/different compressed hashes stop preparation; do not silently bless a new text corpus.

## Coverage matrix

Routes come from the **current live instrumented saved trace**, not from the historical CI arm. A/D/B/C means admitted/density/budget/column. Floor and invalid counts are zero in the three captured traces; unknown elsewhere. Column blocks have no run-front record descriptor. Correct decoded bytes do not imply successful Cells induction.

| Family | Current live A/D/B/C | Historical CI wall C/S [95%] | Historical CI CPU C/S [95%] | Historical CI RSS median S/C KiB |
|---|---|---|---|---|
| wikidata | 196/185/2/48 | 1.000093 [0.989560,1.010738] | 0.999980 [0.989466,1.010606] | 26764/35080 |
| onebyte | 0/0/1/0 | 1.109001 [1.095113,1.123065] | 1.096976 [1.081640,1.112529] | 34226/36818 |
| pattern | 22/0/0/0 | 2.231663 [2.216660,2.246768] | 2.151648 [2.138493,2.164885] | 16590/19034 |
| random | NOT MEASURED | 1.006498 [0.999085,1.013966] | 1.006395 [0.999136,1.013707] | 23404/23496 |
| records | NOT MEASURED | 0.996952 [0.986273,1.007746] | 0.996422 [0.986329,1.006619] | 23658/23772 |
| runs | NOT MEASURED | 1.004255 [0.983362,1.025592] | 1.005238 [0.984041,1.026892] | 23662/23784 |
| text | NOT MEASURED | 1.012133 [0.979513,1.045838] | 1.011804 [0.980045,1.044592] | 19672/19806 |

| Family | Current live hooks-off wall C/S [95%] | Current live hooks-off CPU C/S [95%] | Current live RSS |
|---|---|---|---|
| wikidata | 0.881643 [0.856982,0.907014] | 0.881507 [0.856732,0.906999] | uniform41168KiB; parity NOT established |
| onebyte | 0.535508 [0.331856,0.864135] | 0.527262 [0.323403,0.859624] | uniform41168KiB; parity NOT established |
| pattern | 1.530338 [1.154570,2.028403] | 1.536183 [1.156459,2.040588] | uniform41168KiB; parity NOT established |
| random | NOT MEASURED | NOT MEASURED | NOT MEASURED |
| records | NOT MEASURED | NOT MEASURED | NOT MEASURED |
| runs | NOT MEASURED | NOT MEASURED | NOT MEASURED |
| text | NOT MEASURED | NOT MEASURED | NOT MEASURED |

Historical RSS is recorded per-child data with ranges in coverage.json, not a current-memory qualification. Current onebyte has one392,160-row BWT block and20M decoded bytes: all attempts budget-refuse. Its fast whole-process observation cannot establish an induction benefit. Wikidata's431 blocks contain209,706,557 BWT rows versus209,700,196 decoded bytes; admitted rows176,398,965. Do not mix the denominators.

## Early dispatch: availability, price and falsifier

At the existing run-front handoff, n, orig_ptr and record-vector length r are available; B=floor(4n/25) is computed by Cells. This r counts capped run records, **not the coalesced LF run count**. Existing density rejection10r>=n is already early. True coalesced run/letter counts, floor feasibility and grammar-spine shape require work later inside Cells; they are not free early predictors. Existing bounded symbol sampling in takes_runs is already charged, but its transient sample counts are not a reusable stored scalar. Reducing256 counts for entropy/letter diversity adds work and does not satisfy the zero-new-scan premise.

Illustrative saved-data screen: bypass Cells if **64r<n**, after existing length/origin bounds, before order/allocation/induction. It changes no observed Wikidata decision:0/196 admitted,0/185 density,0/2 budget records bypass;48 column blocks stay outside it. It bypasses22/22 pattern admissions and1/1 onebyte budget attempt. Admitted Wikidata r/n spans0.059039..0.094072; pattern0.00787551..0.00788017; onebyte0.00788199. These are structural labels only. The threshold is post hoc, with no held-out four-family validation. A known F01 long-writer witness n550,r8 also passes64r<n, illustrating that low density is not a proof of a small-only writer.

Conditional cost ledger:
- Extraction: reuse n and vector.len(), no additional traversal, heap allocation or persistent array; B supplies no independent information beyond n.
- Execution: one scalar shift/multiply, compare and branch on every eligible block, plus bound handling; generated instruction cost and branch prediction remain unmeasured. Existing n<2^24/r<=n bounds keep64r safe in the intended integer range.
- Charge run-front construction already paid before this point. Bypassed blocks still pay Segmented inversion and all common decoding/CRC/output. Keep error/origin/overflow validation and stable fallback semantics; a shortcut must not turn malformed input into success.
- Bypassing saves only the attempt and writer work that would otherwise occur. Admitted beneficial blocks misclassified lose their Cells advantage; allowed adverse blocks retain their regression plus dispatch overhead. No saved per-block counterfactual timing pairs price either error. No conversion of r/n separation into expected ns/row, probability of win or general coverage.
- No capacity saving is guaranteed: buffers can retain high water from earlier admitted blocks.

By comparison the **post-induction all-small-spine writer guard** costs a predicate over the marked spine; it is not early dispatch. If every unmarked factor is short (<=16), its proposed fast path can avoid4R NIL initialization,2R position scans and the third spine walk while retaining actual output copies/repetition. Parent pattern ceilings remain10,800,036 logical initialized bytes and5,400,018 scan visits; actual eligibility is unknown. Zero long-copy bytes does not prove zero long seeds. Failed eligibility gives zero savings plus guard work; early dispatch would save more upstream work if accurate, but currently has the weaker evidence for preserving benefit across families.

Thus no robust cheap speed discriminator is established. Retain the writer falsifier, do not build both routes in parallel. Fold the eligibility proof into existing MONODROMY assessment. In the prospective Cells precheck, record small/long seed counts, maximum unmarked factor length and reachable expansion depth without per-copy clocks; if any required factor is long, the all-small guard is false for that block. Use those post-induction fields only to assess the writer, never as early-dispatch inputs.

## One prospective programme: seven-current-readers-v1 (NOT ADMITTED)

Purpose: close current-build seven-family whole-reader and memory coverage, with explicit per-corpus outcomes. This is a diagnostic, not production/default/dispatch qualification.

1. **Freeze before measurements.** Seven identities above; current SegmentedBzReader and unchanged CellsReader at d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2, recipe derived prospectively from cb73bf909397e2fff8c90f0a00437061ad06478e. Commit the exact proposed wrapper/source diff, Cargo.lock, Rust1.99.1/toolchain/flags, fat-LTO hooks-off binary hashes, input hashes and decoded oracle manifest. Both arms use identical4MiB drain/hash/error handling. No writer change, dispatcher, extra gate, phase clock or spinner. The new recipe commit does not exist yet: preparation is incomplete until Coordinator freezes it and normal review/admission succeeds.
2. **First use CI for untimed preparation.** Exactly21 corpus checks (7×Segmented/Cells/oracle) and2 memory positive controls. Complete byte checks before measurement, with one counters-only Cells check per family logging n,r,p,B and admitted/density/floor/budget/invalid/column outcome per block, plus the bounded writer-eligibility fields above. No timings from these instrumented checks enter inference. Synthetic malformed/periodic contract tests and B5 evidence remain separate requirements, not conferred by seven valid corpora.
3. **Two measured arms only.** No dispatcher selected, so no third arm budget. Exactly10 paired rounds×7families×2arms = **140 fresh measured processes**, one allowed logical CPU on one CI host, sequential complete-reader wall and child user+system CPU. Record CPU model, topology, affinity, governor, kernel and runner conditions. No automatic i9 escalation; an i9 version would need its own frozen identity/admission and Louis's execution under C5.
4. **Fixed balanced schedule.** C=[wikidata,onebyte,pattern,random,records,runs,text]. For round j=0..9 visit C[(j+k)%7], k=0..6. Within every pair, Segmented first for j in{0,3,4,7,8}, Cells first otherwise (five each). Fixed0.25s settle before each measured process; no added warm-up, warm20+20, rerun or condition-selected samples. Pair by round and family. Both arms see exactly the same file identity.
5. **Finite resource bound.** **163 total programme executions** (140 measured+21 checks+2 memory controls), excluding builds and input generation. Each execution has a30s kill deadline; preparation/builds collectively capped30min, entire CI job capped90min. Abort and preserve partial evidence on deadline, crash, digest mismatch or invalid memory controls; no replacement samples or retry-to-pass. All source/input generation is before the measurement phase, never concurrent with it. Budget is an upper bound, not a predicted runtime.
6. **Intervals and regression rule.** For each corpus and wall/CPU separately, take ten paired log(C/S) values. Report geometric ratio and ordinary95% log-t interval, plus simultaneous intervals exp(mean ± t[9,1-.05/(2×14)]×sd/sqrt10) across14 endpoints. Freeze the quantile/evaluation implementation before the run. A corpus is diagnostically non-regressing only if **both simultaneous upper bounds <=1.02**. A lower bound>1.02 is detected material regression; otherwise failure to establish non-regression is INCONCLUSIVE, not PASS. No pooling, winsorising, best-of, dropping slow onebyte pairs or extending ten pairs. Report Wikidata gain preservation separately: simultaneous upper<1 and point<=0.95 is the proposed information target. It cannot compensate for another corpus's failure. These are new prospective diagnostic rules, not retroactive gates for the old run or changes to standing acceptance.
7. **Credible RSS.** Record exact-PID wait4 ru_maxrss and native child's post-exec VmHWM after draining, in KiB, along with start/end VmRSS and PIDs. Do not use cumulative RUSAGE_CHILDREN maxima. Keep the launcher small: do not generate/retain corpus byte arrays before spawning it. Cross-check the two channels and preserve discrepancies; uniform41168KiB remains unexplained, not a baseline to subtract. Two untimed fresh native controls touch1MiB and64MiB; the larger must produce at least32MiB more VmHWM. Failure makes the memory channel invalid and stops measurement admission. Report all ten matched absolute peaks, range, median and C/S ratios per corpus. For diagnostic memory containment require each matched C/S VmHWM ratio<=1.10, with valid controls and no unexplained wait4 discrepancy; otherwise memory containment is not established. No sampled-RSS average replaces a peak or claims hardware traffic.
8. **Retain everything.** Commands, schedule, per-process result/resource/error rows, partial attempts, full correctness byte files or an explicitly documented retention limit, SHA256 manifest, route tables, environment and original analysis. No production qualification from valid-input checks or one-core data; CI's adoption grid and independent safety obligations remain outstanding.

Expected information: complete the four missing current-build rows, establish credible memory accounting, identify whether a writer-only preparation covers the failing blocks, and expose any harmful mixed-family routing. Historical evidence already predicts a serious pattern concern, not a surprise to explain away. If the untimed guard screen rejects the writer premise, stop that implementation preparation and take the existing MONODROMY result; do not silently substitute a dispatcher or third arm. The diagnostic's seven-family question remains separate from a future candidate-vs-current-Cells comparison.

## Economic disposition and preserved outcomes

The prior expectation was that live coverage explained a stage-to-reader gap. The finished experiment instead showed an actual11.8% Wikidata off-build gain, a pattern regression and instrument perturbation; no quantitative Wikidata/onebyte phase attribution survived. This delta closes the **inventory/design** gap, not a runtime gap: four current-family measurements, their routes and decoded SHA256s remain missing. The older seven-family CI slice is useful evidence, but cannot fill those current-build cells.

The next preparation has lower information cost than a new general writer architecture or immediate i9 experiment: reuse exact inputs, seven untimed checks per decoder/oracle, and one explicit memory validity test. No off-build writer saving or classifier speedup has been measured. Do not rank broad replacement construction above the existing bounded writer falsifier on structure alone; #366 one-core work stays deferred absent its own evidence. Preserve365 DEFER, Wikidata/onebyte attribution STOP, all original samples/verdicts, B5 independence, and warm20+20 unadmitted. No implementation, timing, ready tag, queue edit, cap/default/gate change, merge or release is authorised by this report.

## Routing and reconciliation

Coordinator LOG6850687056 confirms this bounded delta is the outstanding seven-family requirement, with no new ready job and the mathematical question folded into existing MONODROMY queue6850491677. Routing helper audit on connector snapshots found five already-resolved historical misroutes and the existing Researcher inbox LOG6757065549; no new assignment or repair is made here. REVIEW routes to145. Its verify helper hard-codes FROM coordinator even for role publications; exact post readback will be recorded separately rather than misstate a helper PASS.
