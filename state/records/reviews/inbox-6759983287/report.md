# Share-nothing: load-law transfer fails; capacity remains a hypothesis
Brief [6759983287](https://github.com/lmmx/mibz2/pull/160#issuecomment-6759983287). Specs a2da49638ac54d7763de1c167bd054f67ffdc763. Result record 8f74ab81; submitted run head2ed6aa7e372c34bf5ec553becab999f1f3f4bb1d, frozen482df588e2a97b99c6797c7bc2a24098015968dd.

## Evidence and outcome
Downloaded the original binary archive because the GitHub text connector rejected non-UTF8 content. SHA-256 independently matches **1e347594515a2215e63fb186c5681f329046844a62ff71f1617763d2f68136ad** at evidence commit6b84f37b67286afffcd3ca49ad6ca05edc469443. Extracted data only; did not execute its binary. Read slots.jsonl, runs.jsonl, corpora.json and judgement.json. Checked732 corresponding slot/run identities and zero slot statuses; excluded round0 using **slots**, not runs: each separately spawned child's runs.round is0. Retained60 chronological rounds,600 grid observations and120 sentinels. Recomputed paired geometric CPU ratios exactly: G2=1.1833821656, G9=3.2612397811, R=0.3628626673. No new interval calculation or full admission-judge replay claimed; archived intervals remain those of the registered judge.

**No identifiable percentage of R belongs to table bytes.** This result supports pursuing the compact-state hypothesis; it does not establish a table-only existence proof. At least two causes move together: storage geometry and the rate/shape of work supplied to it. The old law's numerical transfer fails precisely where the decision matters.

## 1. Cell-by-cell out-of-sample check
Use the previously replayed production inversion grid (report9ff1d0e2e4d32921f2400376378fb9efec2f6436): X at aggregate table MB3.6,7.2,14.4,18,21.6,28.8,36 equals0,.0153572,.489932,1.092901,1.963254,5.247926,8.627249. Linear interpolation only inside this range. The strongest literal table-only transfer, with no new fitted coefficient, predicts CPU growth versus4 as **[1+X(sW)]/[1+X(4s)]**. Here s=.8 or3.6 nominal full-block MB, not a measured live-residency census.

| level | W | CPU seconds median | measured CPU growth/4 | predicted | measured/predicted |
|---|---:|---:|---:|---:|---:|
| -2 |2|.65470|.96155|1.00000*|.96155*|
| -2 |4|.68245|1|1*|1*|
| -2 |6|.69225|1.02274|1.00512*|1.01753*|
| -2 |8|.73035|1.07102|1.01194*|1.05838*|
| -2 |10|.79290|1.18338|1.06809*|1.10794*|
| -9 |2|.60430|.76486|.68148|1.12235|
| -9 |4|.79175|1|1|1|
| -9 |6|1.26185|1.61578|1.98885|.81242|
| -9 |8|2.11855|2.63116|4.19343|.62745|
| -9 |10|2.57560|3.26124|6.46154|.50472|

*All -2 normalized predictions require an explicit **flat X=0 extension below3.6MB**, since the4-worker reference is3.2MB. Thus the strict fitted-domain answer for those ratios is **not available**; the starred values are a disclosed scenario, not validated out-of-domain predictions. W4 is normalization, never independent validation.

The table-only law predicts R≈.1653, against.3629 measured: observed/predicted≈2.20. It overpredicts -9 inflation by almost2× at10 while underpredicting -2 growth. This is not evidence that capacity contributes zero; it rejects using this particular transferred table-only model as a quantitative decomposition.

X was inferred from an older **build+walk wall-throughput** grid, not today's whole-process CPU grid. With fully occupied workers CPU can track inverse throughput; scanner-limited workers break that mapping. The u24 fit already included layout/instruction/load-count assumptions and showed up to~13% error at10 in the earlier replay. Re-estimating coefficients on this run and calling that out-of-sample would be false.

A mixed model C_l(W)=H_l(W)+K_l[1+a_l X(L_l(W))] has unknown H, K, a and actual L. H contains scanner, Huffman/MTF, output and per-block work. There are no matched one-worker phase counts or instantaneous live-table observations here. Even a constant additive H dilutes the predicted ratio, while busy-worker occupancy changes its argument. These terms cannot be inferred uniquely from10 aggregate cells. No nontrivial causal fraction follows without assumptions; interactions mean a simple additive percentage need not even lie between0 and100.

## 2. What changed with level
- **Blocks:**1831 versus421 by archived magic census, ratio4.349. Approximately1410 extra block headers, Huffman-table setups, dispatches, buffer lifetimes and completion/CRC bookkeeping at-2. Blocks_by_magic is the archive's census, not a fresh exact BWT-row census. Full-block .8/3.6MB overstates partial blocks and says nothing about simultaneous active ones.
- **Input bytes:**15,837,840 versus11,855,421, +33.59% compressed data for identical209,700,196 decoded bytes and matching decoded SHA-256. Scanner input work therefore is not held constant.
- **Huffman/MTF:** boundaries, coding tables, selector structure, run distribution and symbol sequence change. More table initializations per decoded byte can hurt; shorter live symbols/output/table buffers can help. Neither sign can be isolated by R.
- **Feed and occupancy:** sampled scanner CPU medians are approximately.09s at every-2 cell, versus.05/.06/.06/.07/.08s at-9. These are coarse sampled per-thread figures, **not additive exact attribution**. At-2 W8 and10, exact process CPU/wall has medians6.45 and6.94; at-9 it is7.91 and8.61. These include scanner/main/reader, so they are not active-worker counts. They nevertheless disprove equating requested10 with ten continuously busy walking tables. Scanner backpressure can reduce contention and produce a low G2.
- AtW2, -2 total CPU is already.0504s higher than-9, despite smaller blocks/tables. AtW10 the difference reverses by1.7827s. This is a large interaction; neither difference identifies fixed-block cost alone.
- Static allocated capacity is not hot footprint. Retained scratch can be large but cold; a fresh zero/fill pass can make it hot. Charge touched pages/lines by phase and overlap, not just table.bytes() or RSS.

Do not assign scanner .09s/whole CPU as a causal wall fraction. Critical supply, overlap and idle workers matter. The scanner-fed ceiling Q and a finder-fed candidate are different systems.

## 3. Reprice against observed whole-decoder curves
Observed-2 throughput is1861.53MB/s at8 and1819.13 at10; -9 is784.22 and704.28. These are whole-decoder **workload observations**, not a prospective candidate's throughput bound. The original-Wikidata default sentinels have pooled median~1499.47MB/s in this attempt; that is useful context but a different block stream and mode mixture, not a registered candidate/default comparison. Never compare the teams'2430–2584 **inversion-only** figures directly with these whole-decoder figures.

A useful explicit sensitivity surface is:
**T_c(W) ≤ min(Q_c, A_W / ([1−f+f g] h_c))**, conditional on A_W being a valid small-state, large-block proxy.
Use observed-2 A8=1861.53,A10=1819.13 only as a **scenario anchor**; f is unmeasured matched one-worker build+walk CPU share; g candidate complete CPU ratio; h additional concurrency penalty. In general A_c need not equal A_-2 in either direction.

At illustrative f=.7 and h=1:
| candidate | illustrative g (not measured native CPU) | W8 MB/s | W10 MB/s |
|---|---:|---:|---:|
| compact30 |1.18|1653|1616|
| D4 failed build+walk |22.17/15.38=1.4415|1422|1390|
| chunked link |1.8|1193|1166|

These are **not forecasts** or qualification budgets. Compact30's g is an old cross-machine inferred estimate; D4 substitutes an instruction ratio for CPU; chunked1.8 is an unsupported complete-fill hypothesis. Any h>1 reduces them, supply caps them, and no candidate yet establishes the anchor. The theoretical gross advantage of the-2 proxy over-9 is2.374×/2.583×, leaving room for costs, but it does not show the candidate has that room.

The footprint target is materially stronger than merely “smaller than T”:
- **D4:**~.857MB used on counted blocks is close to.8, but scratch previously reached2.59–4.40MB/worker and record reservation~2.05MB. F2 build14.45>7.35 remains FAIL; only197/233 prior full blocks meet1.2MB, a block-count statistic not byte-weighted admission. No failed-candidate timing. Pair-word cursors/run-fed acquisition remain conditional rescue constructions; their initialization, validation and emitter costs are charged.
- **Chunked:** corrected16-byte directories plus proposed start bitmap give~1.31MB Wikidata/~1.44MB text at900k, before scratch/capacity. That is1.64–1.80× the target table, plus900k-row buffers. Existing conversion needs T and oracle starts; its local walker speed excludes those. Complete fill, bucket-ordered records, rank/start/refill metadata, segment termination, assembly and fallback must replace them. No free fill+2Ir.
- **Compact30:**~1.95MB used is2.44× target; at8–10 nominal aggregate15.6–19.5MB versus-2's6.4–8.0MB, before other state. A claim that it inherits-2's inflation is particularly unsupported. It is a different locality point, not the desired counterfactual.

The new data withdraws any use of old2628–3578 or~2350 inversion projections as a whole-decoder ranking. It does **not** loosen D4 filters or end the compact-state line.

For an adaptive mix with fallback byte fraction q, a serial-service approximation is 1/Tmix=(1−q)/Tgood+q/Tfallback+selector/wasted-build seconds per MB. Thus a modest slow tail is expensive; weighted block counts and arithmetic means of rates misprice it. At concurrency, critical-worker imbalance and hot overlap can make this worse; the actual mixed decoder must be paired against default on the same input.

## 4. Economic ranking versus team ports
**Port2 (whole build_one_pass loop) then port3/C29 MTF remain ahead of building a three-table comparison**, on implementation risk and direct relevance to the existing team path, not a claimed measured wall win. The residual source/profile record proposes roughly1.5 and.6 Ir/D cuts. Those are component opportunities, not independent gains that can simply be added to wall throughput. For a hypothetical28Ir/D process they are~5.4%/~2.1% instruction shares; critical helper build and leader MTF can have different wall leverage.

Account for D085: condition3 of#335 remains FAIL-MEASUREMENT, the initial numbers were withdrawn, and Coordinator6759906409 recounts unique gz profiles at89d589ba after detecting raw/gzip duplication. Its restored figures are a reported recount, not independently re-audited in this task. B5 and scoped holds still bind. Port ranking is prospective, not permission to bypass those holds.

A **cheap width intervention** is worth inserting before expensive new compact constructors: it tests the premise common to all three. The full adaptive whole-decoder comparison of6759415547 stays the eventual discriminator once complete eligible kernels exist; the homogeneous12-cell inversion grid still cannot choose the adaptive decoder or beat a production default by itself.

## 5. Cheapest separating measurement: fixed input, expanded addressed layout
No build authorized or performed. Proposed first diagnostic: use the **same-2 compressed file**, unchanged scanner/Huffman/MTF/block sequence, with the same32-bit link semantics in records of4 versus16 bytes. Store/load at a fixed aligned stride, touching the actual addressed records; preserve output/segment semantics. At200k rows the footprint becomes.8 versus3.2MB. A20-byte aligned stride brackets3.6MB from above at4.0MB if needed later, but is not the cheapest initial contrast. Mere oversized allocation or untouched padding is NOT a live-footprint intervention.

This is cheaper than inventing a capped-9 mixed-width representation: reducing900k links to .8MB requires compression, directories/exception handling and construction, changing exactly the mechanisms being priced. Expanded-2 retains logical links and block count. Charge larger allocation/zeroing/write-allocate traffic, address generation, and memory budget. It tests **addressed-layout/cache-traffic impact**, not pure LLC capacity independent of traffic; instruction differences and per-phase counts must be disclosed. Native throughput alone cannot separate capacity, bandwidth and dependency latency.

Minimal diagnostic grid: two widths × W4/W10, paired/interleaved with the same unmodified default sentinels, fixed order/random seed, input and binary pins, output checks, total and phase CPU, scanner/worker occupancy. This directly estimates width×worker interaction at fixed block count. AddW8 only if the information objective includes the knee. Retain a low-W count/control to expose intrinsic expanded-build cost; no one should label the whole contrast capacity-only.

For an **illustrative**20 measured rounds+one warmup,4cells+2sentinels =126 child slots; addingW8 makes168. From this archive,732 slots span394.54s with median slot duration.533s: naive unchanged-slot pricing gives~68/91s drive time, **not a deadline or guarantee**—the inflated arm is intended to be slower. Price sum of each arm's expected decode+fixed gap/telemetry, plus compilation, setup/preflight and checks; a120s per-slot timeout gives a loose operational bound, not expected runtime. No sample size/power/threshold is fixed here. Researcher/Guardian must review a frozen information objective and invalidity policy before an eligible run; a workstation run remains Louis's C5 submission.

Falsifier: if the expanded layout preserves near-2's low4→10 inflation despite actually touched3.2MB/worker and comparable occupancy, nominal T size alone is insufficient. If inflation rises strongly, the fixed-block experiment supports the layout/traffic channel, without identifying what portion of the original R it explains. A later factorial same-input representation contrast and phase/occupancy analysis would be needed for that attribution.

## Scope and persistence
No decoder executed, candidate implemented, compiled or timed; analysis used existing submitted records and algebra. No new acceptance criterion, kernel budget or old verdict changed. Migration verified. Routing audit: existing unresolved researcher-inbox LOG6757065549 unchanged; no new finding. Current brief completes after REVIEW readback; persistent inbox task remains enabled.
