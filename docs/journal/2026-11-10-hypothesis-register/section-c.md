### (c) Work that is not the walk

Slice: P70 to P89 of the 30 October register (20 IDs; the alias table has no P69). Read: the
register entries, sections 3.2 to 3.3 of the companion, every later entry the task names, the
source at master where an entry's claim about the code could be checked, and the `par-scan`
bundle on the `evidence` branch (its summaries, printed with `git show`, nothing extracted).

Short names for sources. All are under `docs/journal/` unless a path is given.

| short | file |
|---|---|
| REG, MAIN | scratchpad `portfolio.md`, `portfolio-main.md` |
| RS | `2026-10-27-research-synthesis.md` |
| OS | `2026-10-27-output-side.md` |
| WS | `2026-10-28-workstation-scaling.md` (section 9) |
| OPB | `2026-10-29-one-pass-build.md` |
| P87I, P87V | `2026-11-02-p87-production-integration.md`, `2026-11-05-p87-production-validation.md` |
| HR | `2026-11-05-huffman-register.md` |
| SR, BL, TL | `2026-11-05-mtf-short-runs.md`, `-mtf-byte-list.md`, `-mtf-two-level.md` |
| BRV, FP | `2026-11-05-build-run-vectors.md`, `2026-11-05-fused-postwalk.md` |
| BC, BC/ss, BC/nogo | `2026-11-05-binding-census.md` and its `serial-stages.md`, `nogo.md` |
| ISA, WR | `2026-11-05-isa-probe.md`, `2026-11-05-walk-round.md` |
| KM, TPR | `2026-11-07-knee-mechanism.md`, `2026-11-07-team-pipeline-result.md` |
| RF8 | `2026-11-08-run-fed-tvector-and-psi2-census.md` |
| RFS, RFD, RFL | `2026-11-09-run-fed-stage-result.md`, `-run-fed-direct-result.md`, `-run-fed-leader-result.md` |
| RFW1, RFW2 | `2026-11-09-run-fed-leader-workstation-result.md`, `-run-fed-leader-second-attempt-result.md` |
| C29R5 | `2026-11-09-c29-r5-result.md` |
| BW1, BW2 | `2026-11-09-build-window-runs.md`, `2026-11-09-build-window-runs-v2.md` |
| SVC | `2026-11-09-service-2x3-result.md` |
| PS, DC, BT, SN | `2026-11-10-parallel-scan.md`, `-by-data-class.md`, `-borrowed-tricks.md`, `-share-nothing.md` |
| BWT, BK | `2026-11-10-bandwidth-titration.md`, `2026-11-10-backward-table.md` |
| DER, DEG | scratchpad `reg2/design-e-result.md`, `reg2/design-e-registration.md` |
| PSB | `evidence` branch, `par-scan/20261110T030608Z.tar.gz`: `summary-wikidata.md`, `summary-wikidata-l1.md`, `summary-records.md`, `outcome.txt`, `check-records.txt`, `inputs.json` |
| CL | `/home/user/mibz2/CHANGELOG.md` |
| src | `/home/user/mibz2/src/mtf.rs`, `src/coop.rs`, `src/bwt.rs`, `src/block.rs` at the working tree |

`2026-11-05-tail-passes.md` (#173) is about the walk's tail. It bears on this slice only as a
third instruction cut that was closed on the bench's power, beside #169 and #183.

| ID | idea, one line | the advantage it claims, as a quantity that could be false | status 2026-11-10 | closed by: construction / advantage / n.a., with the figure | whole-loop price: counted figure, or "derived: …", or "unknown" | cheapest falsifier now: kind and what | bearing on share-nothing | sources | premise the verdict rests on | premise still true? (yes / no / moved: how) |
|---|---|---|---|---|---|---|---|---|---|---|
| P70 | Two Huffman symbols a lookup (#24). Extended on 5 November by keeping the bit reader's state in registers (#204). | Scanner CPU falls: 0.16 → 0.10 s at 4 threads on CI for #24 (REG). For #204, a scanner stage cut of at least 10% on an Intel runner (HR). | shipped | n.a. #204 merged as 264ae5bb: scanner stage −12.9% (−13.9, −11.9) on Xeon 8370C, −5.0% (−6.3, −3.6) on EPYC 9V74, one thread, CI. #204 was never timed on the i9. | Counted: 3.47 Ir a decoded byte for the whole scanner, 144.8 Ir a 4-lookup iteration (Wikidata, callgrind, HR). Nominal 1.35, 3.17 and 9.29 c/B on Wikidata, text and records (DC, one container). | None needed for what shipped. For more: a counting probe of `read_item` on records; DC counted it on text only (270 M instructions a pass). | cuts CPU a byte. With the parallel scan the scanner's work runs on every worker, so its cycles stay: "cores, not cycles" (PS). | REG; HR; BC/ss; DC §1, §4b; PS | "83% of lookups already decode a pair" (BC/ss); "the scanner's own CPU fell 0.16 → 0.10 s" (REG). | yes for the mechanism. moved for the share: the scanner is 11%, 16% and 27% of one thread on Wikidata, text and records (DC), not 5.4%. |
| P71 | A smaller Huffman lookup table (fewer bits, or built only to the block's longest code). | Scanner time falls. No figure was claimed; the register bounds the gain "under 5%" of a decode. | conditional | n.a. Never built. Rejected by arithmetic alone: "no experiment: the whole scanner is 5.4%" (RS), VM, one thread, Wikidata. | unknown for the variant. The scanner today: 3.47 Ir a decoded byte (HR). | counting probe: callgrind of `read_item` and its table construction on level-1 Wikidata (2,098 blocks, PS) and on records, at two table widths. | cuts CPU a byte on every worker once the scan is parallel. Small. | REG; RS; BC/ss; ISA; DC §1, §4b; PS §4 | "the whole scanner is 5.4% of a decode"; the table is "L1-resident". | moved: the scanner is 11%, 16% and 27% by class (DC) and 20.6% on level-1 input (PS §4). The recorded condition, "the scanner becoming a larger share", holds. Against the idea: `read_item` runs at 2.7 instructions a cycle or more on text (DC §4b), so table misses are not shown to cost. |
| P72 | Workers read their own blocks from speculative markers. A finder thread only hints; the chain alone accepts (#10). | At ten workers on the i9, find over scan is at least 1.15 on inputs with a high scanner share (PS §5). | validated | n.a. Diagnostic harness, i9, 1 to 10 plain workers, five rounds: at ten, find over scan is 1.615 (rounds 1.597–1.958) on level 1, 1.780 (1.454–1.987) on records, 0.981 on level 9 (PSB). Not built in the library; no streaming input. | Finder: 0.19% of one thread's decode (local, PS). CPU-seconds at ten on level 1: 0.818 scan, 0.840 find. At one worker find is 0.833 of scan on level 1, 0.764 on records (PSB). | i9 recipe: the library version with a streaming input (PS §6), same grid. Before it, plain workers on `lbzip2 -2` input (SN §4.2). | removes cooperation: it removes the scanner thread, the cap on ten independent workers. | REG; MAIN §3.3; WS §9; PS; SN §3b; PSB | "binds only from about 10 workers" (RS); "consistent with the scanner bound #10 describes. It isn't shown" (WS §9). | no. Shown: one scanner holds level 1 at about 1,600 MB/s from 6 to 10 workers on the i9 (PSB). The condition "a workstation curve that bends before its physical core count … confirmed by a paired experiment" is met. |
| P73 | The MTF list's first entries held in registers (#25). | The MTF stage is faster on Wikidata with no loss on high-index data. | superseded | construction. #25: Wikidata −1.6% at 1 thread, random +11.1%, CI, on bench 1.0.0, whose arm order favoured head by about 2% (BC/nogo). #207, the same idea on a two-level list, gave random −13.3% and records −26.4% at 1 thread on EPYC 7763 (TL). | Counted by llvm-mca: the chain is 3.0 cycles (znver4) and 4.0 (skylake) a move below index 32, against 11.0 before (TL). Whole loop on text: 53 instructions a move (DC §4a). | None needed on x86-64. Open: CI A/B on an Intel runner (none drew one, TL); aarch64 runs the scalar kernel (src). | cuts CPU a byte, on the one-worker kernel that independent workers run. | REG; TL; BC/nogo; DC | "high-index symbols cost more" (class III); "Cond.: none recorded for x86-64". | no: with #207 the high-index corpora gained (TL). Tested at 1 to 4 threads on CI only; never at ten on the i9. |
| P74 | Hold the list's first 32 bytes in registers, so no move waits on a failed store-to-load forward. | MTF and RLE2 fall 26–32%, which is 4.6–5.8% of a one-core decode (MAIN §3.2e). | shipped | n.a. #207 (bc899400): stage −23.5% on EPYC 7763, −18.5% (−21.8, −15.1) on EPYC 9V45; Wikidata 1-thread wall −5.2% (−5.9, −4.5) and −2.1% (−3.8, −0.4). Only `mtf_avx2` changed. The checkpointed kernel that teams run keeps the chain (src; TL "Unchanged"). | As P73. Records: 79 instructions a move, every index past the registers' 32 (DC §5). | For the team kernel (C29): an untimed transfer audit (C29R5), then an i9 recipe. CI timing is closed at 0.9890 [0.9732, 1.0050], 9 threads. | cuts CPU a byte. Independent workers already have it; the teams do not. | REG; MAIN §3.2; TL; BC/nogo; RFL; C29R5; DC; src | "`mtf_cp2p`/`mtf_cp2` 1.0582 ±0.0093" on the i9; "MTF is 24% of a 3x3 worker's time". | moved: that i9 figure was taken on the checkpointed kernel, the one #207 left unchanged. On text a move is now bound by instructions and mispredictions, "nothing is left for a chain to explain" (DC §4a). |
| P75 | libbzip2's two-level sliding list, for moves at high indices. | Moves at index 16 and above cost less than a scalar shift. It counts only where they are common: 95.5% of moves on records (REG). | conditional | n.a. Never tested as posed: "No experiment now", from a census on Wikidata. #207 adopted the two-level list (TL). Its far moves got slower a symbol (index 64: 17.3 → 22.6 cycles) while `records` gained 26.4% at 1 thread. | Records: 79 instructions a move in 17 nominal cycles (DC §4a). A sliding second level: unknown. | counting probe: callgrind of `mtf_avx2` on records by index class, against a sliding second level. | cuts CPU a byte on incompressible data only. None on Wikidata. | REG; RS §4; TL; DC §1, §5 | "Amdahl-bounded on Wikidata"; "in case a records-heavy workload appears". | moved: MTF is 50% of a records decode and 27% of a text one (DC §1). No records-heavy workload is named in the record. |
| P76 | MTF records the byte counts every 16 KiB, so a team's threads can build disjoint row ranges. | The team build runs at 1.25× its rate or more, for a checkpoint cost under 2% of MTF (REG). | shipped | n.a. T-vector phase, i9, research kernel. #78's own pass rule was never reported against (K50); it still is not. | REG: 0.9% of the round. Now: the checkpointed front is 1.640 against plain MTF's 1.290 nominal c/B (CI Xeon 6973P-C, one pinned copy, RFS). MTF is 4.4 instructions a decoded byte in the default plan against 3.9 on one thread (SN §1). | counting probe: callgrind of `mtf_checkpoints_avx2` against `mtf_avx2` on the same blocks. | removes cooperation: independent workers never call it, and a run-fed build needs no checkpoints (RF8 §2.3, BK §2). | REG; RFS; RFL; SN; TL; src | "checkpoints cost 0.9% of the round". | no: #170, #172 and #207 changed only the plain kernel (src). The twin now costs about 1.27× it (inferred: 1.640 ÷ 1.290). |
| P77 | A team's leader runs the next block's MTF, and the last block's RLE1 and CRC, while its helpers invert (pipelined teams). Later form: service threads. | It recovers the helpers' idle share: at most 0.19 of a 3×3 team's core-time (REG). | shipped | n.a. 0.2.6 (#247, #250), i9, 10 threads, level-9 Wikidata: 1.072 (1.054–1.089) over 0.2.5's plan (CL). Service form: svc2@10 is 1.045 [1.017, 1.073] over that default with 12.7% less CPU (SVC); it is not the default. | Counted: the default plan runs 38.0 instructions a decoded byte against one thread's 26.7 (SN §1). The pipeline alone spends 8.9% more CPU than plain teams (CL). | read of an archived bundle: `plan-levels/20261108T094232Z` beside PSB, for the default's rate on level-1 input. | removes cooperation: this is the layer a small table makes unnecessary (SN §3). | REG; KM; TPR; CL 0.2.6; SVC; SN | "helpers are idle 0.19 of a 3x3 team's core-time"; "an upper bound, not a throughput gain". | yes on level 9. On levels 1 and 2 the default is 0.763 and 0.776 of 0.2.4 at 10 threads (CL), a regime the register never read. |
| P78 | MTF writes each byte straight into the T-vector's words, with no last column; the front end fused further (#8 step 3). | Peak RSS falls about 2.5 MB a slot and worker; speed about neutral (REG, not measured). | open | n.a. Never built: "planned, not built" (BWT §5). | unknown. Derived: MTF would write 4 bytes a row in place of 1 (RS §3). The column is 12.8% of the default plan's cold lines written (simulated, BWT §2). | counting probe: BWT's cache simulator on a scratch MTF that writes into `tt`; then the CI stage harness. | shrinks other live memory: the column, 0.9 MB a front. | REG; RS §3; P87V; RF8 §2.4; BWT; BK; DC §2 | "peak RSS down by roughly 2.5 MB per slot and worker". | moved: #164 made the column the walk's output buffer on the one-worker kernel and dropped `rle` (71 → 66 MB at 4 threads, P87V). Removing the column there now frees nothing (RF8 §2.4). In teams the column is still separate. |
| P79 | Sixteen equal last-column bytes take one cursor step and one 64-byte store (#32). Since 9 November, mixed windows of up to 6 runs go run by run (#312). | The build stage falls: 5.70 → 2.55 c/B for #32 (VM); −12% to −28% predicted for #312 (BW2). | shipped | n.a. #312 (fce9149d): build stage −28.4% on EPYC 7763, −8.7% and −9.2% on Xeon 8573C; Wikidata 4-thread CPU −3.0%, −1.6%, −3.1% (BW2). #169's SSE2 stores alone were "closed as not shown" (BRV). | Counted: 4.92 instructions a row on Wikidata and 11.04 on text for `bwt::build` (DEG §4). | counting probe: #312's window path ported to `coop::FusedBuild::share`, which has only the 16-row path (src). | cuts CPU a byte on the one-worker build. The team build has not got #312. | REG; BRV; BW1; BW2; RF8; DEG; src | "81.9% of rows via run stores". | yes on Wikidata (83.4% of rows, BW1). no on records and random: no one-run windows, 95.36% and 88.97% of windows hold sixteen runs (BW2). |
| P80 | Build the table in four interleaved quarters, to overlap the cursor's store-to-load chain (#23). | The forwarding wait it removes exceeds the cost of four write streams. | superseded | construction. #23: Wikidata +12.1% (+9.9, +14.4) at 1 thread, CI EPYC 7763, before #32. The chain was later shown to cost: #312 removed it inside windows for −8.7% to −28.4% of the build stage (BW2). | unknown for quarters on today's build. The incumbent is 4.92 instructions a row (DEG). | census: equal-neighbour rows left in windows of 7 to 16 runs (6.56% of Wikidata's windows, BW2); then a counting probe. | cuts CPU a byte. Small. | REG; BC/ss; BC/nogo; BW1; BW2; RFS | "splitting the scatter into four write streams costs more than the forwarding it removes". | moved: true of four streams as built, but the forwarding was not small. Never re-tested on top of #32 or #312, and never on the i9. |
| P81 | The one-core build writes each entry whole in one pass, with no init pass (#114). | The build stage is faster with its interval above 1, and the decoder is not slower. | shipped | n.a. CI build −6.0% (−8.5, −3.5), decoder +0.8% at 1 thread; i9 1.048 at 1 thread and 1.035 at 4 (OPB). | Counted: 4.92 instructions a row on Wikidata, 11.04 on text, #312 included (DEG §4). | None needed for the change. For the open sub-question (why CI and the i9 disagree): the sibling-pair matrix of BC, the build beside a walk. | cuts CPU a byte: this is the build ten independent workers would run. | REG; OPB; BC; BC/ss; RFS; BW2 | "L loads compete with the SMT sibling on CI"; "Not established: why." | moved: not settled. The build's four-copy inflation is ×2.25 on Xeon 8370C, ×1.43 on EPYC 7763, ×1.76 on EPYC 9V74 (BC). The stage is "not store-bound" (R_floor 0.321, RFS). |
| P82 | A team's threads build disjoint row ranges of one table in one pass, from checkpoints (#104). | The team's build is at least 1.15× the given-counts build: 1.313 ±0.033 at 3×3 (REG). | shipped | n.a. Measured against q, a research baseline, not production (K20, K26). That is still so. | Counted: the build is 9.2 instructions a decoded byte in the default plan against one thread's 6.2 (callgrind, SN §1, before #312). `FusedBuild::share` writes 35.5% of the plan's cold lines (simulated, BWT §2). | counting probe: callgrind of the default plan with #312's window path in the share. | removes cooperation: a table under about 1.2 MB makes shared builds unnecessary (SN §3). | REG; SN §1, §2; BWT; WS §9; src | "four tables fill the L3" (SN), the reason teams exist. | yes for level-9 Wikidata with a 4-byte table. no for level 1 to 3 input, where plain workers' CPU grows ×1.10 to ×1.44 from 4 to 10 (WS §9). |
| P83 | A write-only build scatter that takes each byte from the first column, not from L (A′). | The build drops its read of L at each slot. | superseded | construction: bz2ngrs's build was 1.67× (VM) and 1.91× (CI) slower than ours, since it lacks run stores. Ours without the read was never built; BC/nogo prices it at 0 to −1.5% of 4-thread wall on CI. | unknown for A′. Its successor, the LF-direction fill from runs: 0.293 against `bwt::build`'s 0.458 ns a row (local, BK §2); 0.78 of the build on text (DC §2). | CI A/B at the stage harness: the fill beside the build, front plus build at most 0.75 of today's (BK §2). | cuts CPU a byte; the LF fill also removes the column. | REG; BC/nogo; BK §1, §2; DC §2, §4c | "Hypothesis 1 … loses its premise on both machines". | moved: #114 removed the init pass; what is left is "the read of ll at each destination slot" (BC/nogo). The condition (a workstation build row) was never run. |
| P84 | Build a compact table straight from L or its runs in one pass, with no T-vector first. | Construction costs no more than the T-vector build, for a table well under 4 bytes a row. | conditional | construction: design E's direct build ran 1.827× (1.775–1.881) the scatter's time at 0.82× its instructions (CI EPYC 9V74, one thread, Wikidata, DER). The builds of C and D are not built. | E: build 2.59 against 1.42 c/B; walk 48.2 against 10.4 instructions a row (DER). D: derived, 6 to 10 instructions a row for the doubling build against 4.9 (SN §3a). | counting probe: the doubling pass's instructions a piece, by listing; over 20 closes k = 8 (BK §4a). | shrinks the table: 1.44 MB at k = 4 for D, 0.75 MB for C (SN §3). | REG; RF8 §3; BK; DER; SN §3a | "writing fewer bytes need not be less work" (REG). | yes, and now measured by E. BK's "the table's bytes are the floor, whoever writes them" is contradicted by DER. |
| P85 | RLE1 reads the walk's segments in chain order with the CRC folded in; no assembly copy (#48 step 2, built as #183). | Post-walk traffic falls from 3n + 2m to n + m bytes, and the 4-thread wall is faster beyond noise (FP). | conditional | construction's registered rule, on power: 4-thread wall −4.2% (−10.0, +1.8) and +1.5% (−3.9, +7.2), CI. 1-thread wall was faster beyond noise in both runs (−1.1%, −2.5%); fusion alone −1.5% (−2.4, −0.6) at 4 threads on Xeon 8370C (FP). Never run on the i9. | Counted: −116,646 instructions on Wikidata block 1 for the fusion, +41,319 on `runs`; simulated L2 misses after the walk −57% (FP). | counting probe: BWT's simulator on #183's head with ten plain workers. Then a CI A/B registered on 1-thread wall, with 4 threads pooled over three runs (BC/nogo). | shrinks other live memory: segment assembly is 15.2% of the default plan's cold writes (SN §3). #183 acts on the one-worker path only (FP). | REG; OS §6, §7; FP; BC; BC/nogo; BWT §1, §5; SN | "1.4% (0.14 of 9.94 c/B)", under the 5% gate. | moved: still small in time alone, but assembly inflates ×2.97 and ×3.53 at four copies on AMD (BC). The condition "over 5% of a decode" is not met in time. "CI could not have shown what it is worth on the i9" (BWT §1). |
| P86 | Non-temporal stores for buffers written once on the output path. | It saves the read-for-ownership and keeps output out of the caches, where a buffer is not read again soon. | conditional | n.a. A code trace, no timing: "Every buffer mibz2 writes is read again almost at once" (OS §5). One construction since, non-temporal copies in the walk's flush: "No time resolved", neutral (WR). | unknown. Model: at most 6% of the default plan's traffic, for a block's decoded bytes (BWT §5). | i9 recipe: `just bench-bw-titrate`, registered, no data yet. NOT BOUND closes it. | shrinks other live memory: it keeps decoded bytes out of the L3. None on the table. | REG; OS §5; WR; KM H4; BWT; BT §3 | "every buffer is re-read almost at once"; "the one write-once buffer is `decode_parallel`'s". | moved: at ten threads the reader's copy is 17.7% of all data writes and 52% of cold reads (BT §3), and each buffer misses on 72–84% of its lines (simulated, BWT §2). No streaming boundary exists yet; `BufRead` would remove the copy instead. |
| P87 | The walk writes its output over L's buffer; the worker never allocates `rle`. | About 0.9 MB less resident memory a block. The register's pass bar: the occupancy curve at 6–10 workers at least 1.03× today's. | shipped | n.a. #164 (e26e3d9c): Wikidata 4-thread wall −2.2% (−4.0, −0.5) on Xeon 8573C, −1.1% (−2.3, +0.2) on Xeon 6975P-C; peak RSS 71 → 66 MB in both. i9 at 4 workers: +1.35% [+0.29, +2.41], mechanism INCONCLUSIVE (P87V). The 6–10 worker bar was never run. | Counted: 400 to 542 fewer instructions a full block (P87V). | i9 recipe: plain workers at 6 to 10 with and without the in-place path, on level-1 input, where the table does not bind. | shrinks other live memory, for independent workers only: `Plain` and `Cooperative` return false and keep a separate buffer (P87I §3). | REG; P87I; P87V; KM E5 | "does not shrink the walk's hot working set" (K46). | yes. The teams the i9 runs by default do not take the in-place path. |
| P88 | Decode randomised blocks (bzip2 0.9.0 and earlier). | Completeness. No speed claim. | open | n.a. Still refused with `ErrorKind::Randomised` (src). The parallel-scan harness returns the same error at the same block (PS §2, case 4). | unknown. "About 30 lines" in pbz2 (REG). | none needed. | none. | REG; PS §2; src | "completeness, not speed". | yes. |
| P89 | aarch64 kernels and timing. | Not stated as a quantity. | open | n.a. Correctness only. CI was green on aarch64 for #183's head (FP); #312's scalar paths pass Miri on an aarch64 target (BW2). No ARM hardware has been timed. | unknown. | CI A/B on an aarch64 runner; the record has none. | none: the objective is the i9. | REG; FP; BW2; PS §2, §3; src | "No ARM hardware timed; no aarch64 bench runner". | yes. The gap has widened: #207's register list is x86-only (src), and the parallel scan's finder falls back to scalar, 1.08 against 9.25 GB/s (local, PS §3). |

### Mined

Ranked by what each could be worth. Every "(inferred)" is this section's own arithmetic on
the figures named.

1. **P72, the parallel scan.** The verdict was "conditional", with the premise "it binds only
   at high worker counts or on incompressible data" and the condition "a workstation curve that
   bends before its physical core count, with rising idle while the scanner reads", confirmed
   by a paired experiment. WS §9 had said of the level-1 plateau: "consistent with the scanner
   bound #10 describes. It isn't shown." PSB now shows it on the i9. On level 1 the scanner arm
   reads 1,524, 1,628, 1,600 and 1,598 MB/s at 4, 6, 8 and 10 workers; the finder arm reads
   2,563 at 10, a paired 1.615. On records it is 254 against 426 MB/s, a paired 1.780. On
   level 9 it is 0.981: the knee binds first. MAIN §3.3 called #10 "a prerequisite for any
   result above the headroom, not a standalone candidate"; for level-1 and incompressible
   input it is now a standalone gain. What it makes possible: ten independent workers, with
   CPU nearly flat on level 1 (0.662 → 0.840 CPU-seconds from 1 to 10). Cheapest falsifier:
   the library version with a streaming input, the same grid on the i9. Worth: 1.6× on level 1
   and 1.78× on records at ten workers, measured in a harness. On level 9 nothing until the
   table shrinks, then the model's 2.6 GB/s at an inversion 1.2× production's against about 1.5
   (SN §3b). A caution on the records cell: the input is 19.2 MB in 22 blocks, so ten workers
   hold about two blocks each.

2. **P84, building a compact table directly.** The register's note was "writing fewer bytes
   need not be less work". BK §2 then said "the table's bytes are the floor, whoever writes
   them", and BK §4 priced C's constructor at "build 2.4 → about 0.2 c/B". DER contradicts
   both: E's sequential build, "at 0.82× the instructions, took 1.83× the time" of the scatter.
   SN §3a prices D's doubling build on paper at 6 to 10 instructions a row and says it has "the
   shape of E's build". What that makes possible is a harder price for D before it is built.
   Apply E's time penalty to D's build (1.827 ÷ 0.82 = 2.23 times the time an instruction,
   counted as that many production instructions): k = 4 becomes
   6 × 2.23 + 12 = 25.4 against production's 15.3, so g = 1.66, and 3.0 ÷ (0.3 + 0.7 × 1.66) =
   2.05 GB/s; k = 8 becomes 10 × 2.23 + 6 = 28.3, g = 1.85, 1.88 GB/s (inferred). If the walk's
   paper count is also low by half, as E's was, k = 4 becomes 13.4 + 24 = 37.4, g = 2.44, which
   is the break-even of 2.5 (inferred). So D at k = 4 survives one of E's two errors and not
   both. Cheapest falsifier: the listing count of the doubling pass, then a callgrind count of
   the complete build, before any walk is written. Worth: 1.9 to 2.05 GB/s against about 1.5
   at ten on the i9 if only the build is under-priced (inferred, SN §3b's model); nothing if
   both are.

3. **P76 and P74, the team's MTF kernel.** The register prices P76 at "checkpoints cost 0.9%
   of the round" and files P74 as open with an i9 figure, "`mtf_cp2p`/`mtf_cp2` 1.0582". TL
   ships P74 but says "Unchanged: … `mtf_scalar` and `mtf_checkpoints`". The source agrees:
   `mtf_checkpoints_avx2` still stores the list to memory each move, still calls `write_bytes`
   for every run, and still maps through `seq_to_unseq`. So none of #170, #172 or #207 reaches
   the kernel that every team plan runs, the i9's default among them. RFS measures the gap:
   front 1.290 nominal c/B for the plain MTF, 1.640 with checkpoints (CI, Xeon 6973P-C). SN §1
   counts MTF at 4.4 instructions a decoded byte in the default plan against 3.9 alone. The
   construction exists (C29, the register-list checkpoint kernel): 0.9739 [0.9682, 0.9797] of
   the control's wall at 9 threads on CI (RFL), then 0.9890 [0.9732, 1.0050], INCONCLUSIVE
   (C29R5). On the i9 both attempts were INCONCLUSIVE with control fidelity FAILED (RFW1,
   RFW2). The entries disagree on whether it can pay. RF8 §2.3 calls the leader's serial stage
   "the team's critical path". C29R5 says "cutting front work F can't buy elapsed time while
   the helpers bind". DC §4a says "No plan measured so far has MTF as the stage a team waits
   on", yet SVC's trace found teams idle 0.165 of the time waiting for a prepared block. What
   it makes possible: under share-nothing the question disappears, since workers
   run `mtf_avx2`. Cheapest falsifier: a callgrind count of the two kernels on the same blocks,
   then C29R5's untimed transfer audit. Worth: at most 2.6% of wall at 9 threads on CI
   (1 − 0.9739); on the i9 it cannot be bounded from the record.

4. **P82 and P79, the team's build.** OPB says of #114: "From 6 physical cores the default
   plans teams. Their fused build is untouched." BW1 then finds that "22.6M rows (9.6% of all
   rows …) equal their predecessor inside a mixed window. Each is one link of a store-forward
   chain today", and #312 removes that chain from `build_one_pass` for −8.7% to −28.4% of the
   build stage. The source shows `coop::FusedBuild::share` with the one-row step and the
   16-row path only. So the chain is still in the build the i9's default runs, where the build
   is 9.2 of 38.0 instructions a decoded byte (SN §1) and 35.5% of the cold lines written
   (BWT §2). What it makes possible: the same window path in the share, whose ranges are
   already disjoint. Cheapest falsifier: a callgrind count of the default plan with the path
   ported. Worth: #312 cut the one-pass build's instructions by 13.4% to 16.2% (BW2); the same
   share of 9.2 is 1.2 to 1.5 instructions a byte, 3.2% to 3.9% of the plan's 38.0 (inferred).
   In time it cannot be bounded: the gain was three times larger on EPYC 7763 than on Xeon
   8573C, and the i9 is Intel.

5. **P82 and P77, "four tables fill the L3", by data class.** SN's premise for the whole
   cooperative layer is that "a level-9 block's table is 3.6 MB and four of them fill the
   i9's cache", and SN §3b reads level-9 Wikidata as "the knee, as before … its CPU grows
   6.4×". PSB's records input is also made of full-size blocks (19.2 MB in 22 blocks). There
   the finder arm's CPU grows from 0.221 to 0.344 CPU-seconds from 1 to 10 workers, ×1.56
   (inferred), and throughput rises to 426 MB/s. DC §1 gives the reason: the walk is 13% of a
   records decode and 55% of a Wikidata one. So the knee's cost follows the walk's share, and
   on incompressible input ten full tables are affordable with no small table at all. The
   default plan's rate on records or text on the i9 is not in the record; in a team the
   leader's serial MTF would be half the decode (inferred from DC §1). Cheapest falsifier: an
   i9 recipe, the default plan against plain workers with the finder on records and on text at
   ten threads. Worth: cannot be bounded from the record; the one figure is PSB's 1.78× over a
   one-scanner plain-worker arm.

6. **P78, the front end without a column, and the emitter's three prices.** The register has
   P78 open on a memory claim. The later record moved its content to a run emitter and priced
   it three ways. RFS (registered, CI Xeon 6973P-C): "the emitter is 17% dearer than plain MTF
   (R_front 1.185)". RFD (registered, CI EPYC 9V74): the same emitter is "6% cheaper than A's
   MTF (1.020 against 1.085)". DC §2 (local, a newer emitter, `run_front`): 0.173 against 0.423
   ns a row on Wikidata, 0.987 against 3.00 on text, 9.34 against 8.84 on records. The entries
   do not reconcile, and none is on the i9. DC's model for text rests on the third: MTF 5.47 →
   1.80 c/B and a decode of 20.30 → 16.07, −21%. Cheapest falsifier: the CI stage harness on
   `run_front` against `mtf::mtf`, text and Wikidata, registered; DC itself asks that "the
   emitter's production gate should read text as well as Wikidata". Worth: −21% of a text
   decode by DC's model, up to −30% with a table that costs the runs'; on records a loss of 5%
   of the MTF stage (BT §4).

7. **P85, the fused post-walk pass.** The verdict was "rejected", class III, "the ceiling is
   below the gate", on "1.4% (0.14 of 9.94 c/B)", with the condition "only if a runner or a
   change puts the assembly over 5% of a decode". It was then built (#183). FP: the 1-thread
   wall was faster beyond noise in both PR runs, and the fusion alone was −1.5% (−2.4, −0.6)
   at 4 threads on one Intel run; it closed because "the registration required the 4-thread
   wall". BC calls it "a statistical-power case, not a failed mechanism". BWT §1 adds that
   "CI's runners have no bandwidth wall at 4 threads, so CI could not have shown what it is
   worth on the i9", and SN §3 puts segment assembly at 15.2% of the default plan's cold
   writes. Two limits: #183 changes the one-worker path only ("the cooperative kernel's path
   is unchanged"), and BC says it "is not reopened without the maintainer", a restriction that
   predates the delegation of merge acceptance on 9 November (CLAUDE.md; whether it lifts the
   restriction is not in the record). Cheapest falsifier: BWT's simulator on #183's head with
   ten plain workers, then a CI A/B registered on 1-thread wall. Worth: 1.1% to 2.5% of a
   1-thread decode, measured on CI. At ten on the i9, at most 1 ÷ (1 − 0.152) = 1.18× if the
   plan is bandwidth-bound (inferred; the titration has no data).

8. **P75 and P73, moves at high indices.** P73 was rejected as class III, "high-index symbols
   cost more", on random +11.1% and records +9.7%. P75 was shelved with "the sliding block
   can't move Wikidata measurably … No experiment now". TL reverses the first: with the list's
   head in registers, `records` gained 26.4% to 28.5% and `random` 13.1% to 13.3% at 1 thread.
   TL also records that the far path itself got slower: index 64 went from 17.3 to 22.6
   cycles a symbol and index 100 from 20.3 to 23.6, with the net gain's cause "not measured".
   DC §5 counts what is left: "79 instructions a move; every index is past the two registers'
   32", with MTF at 50% of a records decode. What it makes possible: a second level built for
   far moves, which is P75's own subject. Cheapest falsifier: a callgrind count of `mtf_avx2`
   on records by index class. Worth: DC bounds a redesign at "1.4 to 1.6× at most" on the
   stage, so 0.50 × (1 − 1/1.4) = 14% to 0.50 × (1 − 1/1.6) = 19% of a records decode
   (inferred); nothing on Wikidata.

9. **P86, and the reader's copy.** The verdict was "rejected for the decode path", class I,
   because "every buffer mibz2 writes is read again almost at once", the last of them "the
   reader's copy, by the caller" (OS §5). BT §3 reads that copy on the ledger's ten-thread run:
   "209.7 M bytes written, 17.7% of all data writes, and 52% of the run's cold reads", and "it
   is the next serial stage after the scanner once the decoder passes about 5 GB/s". PSB has
   plain workers at 2,563 MB/s on level 1 with the cores full. The remedy BT gives is not a
   different store but no copy: `impl BufRead`, the block lent to the caller. That agrees with
   OS §5's own lesson, "removing a copy beats changing how it stores". Cheapest falsifier: a
   callgrind count of a `BufRead` reader against `read` on one thread, then the simulator at
   ten. Worth: 1.3 of 26.7 instructions a byte at one thread, 4.9% (inferred from BT §3); at
   ten threads a serial cap near 5 GB/s by BT's model, not yet binding.

10. **P87, the pass bar that was never run.** The register set P87's bar as "the occupancy
    curve at 6–10 workers ≥ 1.03 × today's, with DRAM write bytes per output byte". It shipped
    on CI's 4-thread rows and an i9 run at 4 workers (P87V). P87I §3 records that `Plain` and
    `Cooperative` "return false" from `invert_in_place`, so at 6 threads and above, where the
    i9 plans teams, the change does nothing. Under share-nothing it would reach ten workers
    for the first time. Cheapest falsifier: the i9 recipe named in the table. Worth: in memory,
    `rle`'s 1.35 MB of capacity a workspace (KM E5) and CI's 71 → 66 MB at 4 threads; in time
    at ten it cannot be bounded from the record.

11. **P71, the scanner's share.** The verdict was "rejected (not pursued)" because "the whole
    scanner is 5.4% of a decode (VM, stage), so the ceiling is under 5%", with the condition
    "the scanner becoming a larger share (as P72)". DC §1 has it at 11%, 16% and 27% of one
    thread by class, and PS §4 at 20.6% on level-1 input. The condition holds. The later record
    also narrows what could pay: `read_item` runs at "at least 2.7 instructions a cycle" on
    text (DC §4b), ISA says "a multi-symbol Huffman decode has to come from fewer lookups, not
    from BMI2", and BC/ss bounds a wider table at "at most about 10% more to the scanner". So
    the lever is instructions a symbol, and table size is not shown to matter. Cheapest
    falsifier: the counting probe in the table. Worth: a cut of x in the scanner is 0.206 x of
    a level-1 decode and 0.27 x of a records one (inferred); x itself cannot be bounded from
    the record.

12. **P80 and P81, what binds the build.** P80's verdict rests on "splitting the scatter into
    four write streams costs more than the forwarding it removes". BRV then inferred that "the
    build's scattered writes bound it". RFS contradicts that: "The build is not store-bound.
    R_floor is 0.321 … R_warm is 1.010"; and #312 shows the forwarding chain was worth 8.7% to
    28.4% of the stage. So #23 failed as a construction, and its target was real. What is left
    of the chain is in windows of 7 to 16 runs, 6.56% of Wikidata's windows (BW2). The size is
    machine-dependent by a factor of three between EPYC 7763 and Xeon 8573C, which is P81's
    open sub-question again. Cheapest falsifier: a census of equal-neighbour rows in those
    windows. Worth: cannot be bounded from the record.

### Changed since 30 October

- **P70:** #204, the bit reader's state in registers, merged (HR). Evidence added; status
  unchanged.
- **P71:** rejected → conditional. Its premise moved (DC §1, PS §4) and its recorded
  condition holds. Nothing was built.
- **P72:** conditional → validated. PS (the exact prototype), SN §3b and PSB (the i9 run).
- **P73:** rejected → superseded by #207 (TL). BC/nogo adds that #25's bench favoured head
  by about 2%.
- **P74:** open → shipped on the one-worker kernel as #207 (TL), after #118's failed gate
  and a NEGATIVE i9 run of P74-C (BC/nogo). The team kernel's version, C29, is INCONCLUSIVE
  (RFL, C29R5, RFW1, RFW2).
- **P75:** evidence only. The two-level list shipped in #207 (TL); far moves got slower a
  symbol; DC §5 counts records.
- **P76:** status unchanged; its price is no longer 0.9% (RFS, SN §1, src).
- **P77:** open → shipped as pipelined teams in 0.2.6 (TPR, CL). SVC measured the service
  form. CL records the loss on levels 1 and 2.
- **P78:** status unchanged; its memory premise moved with #164 (P87V, RF8 §2.4). BWT prices
  the column's cold writes. The emitter line (RFS, RFD, DC §2, BK §2) is its nearest
  construction.
- **P79:** extended by #312 (BW1, BW2). #169 closed as not shown (BRV).
- **P80:** rejected → superseded by #32 and #312 (BC/nogo, BW2).
- **P81:** status unchanged. #312 sits on top of it (BW2). RFS and BC bear on its open
  sub-question.
- **P82:** status unchanged. SN §1 counts it in the cost of cooperation; BWT §2 names it the
  largest cold writer.
- **P83:** status unchanged. BC/nogo restates what is left of it; BK and DC give the
  LF-direction fill as its successor.
- **P84:** open → conditional. RF8 §3 (census), BK (designs C and D), DER (E's build
  falsified), SN §3a (C withdrawn as a walk table).
- **P85:** rejected → conditional. Built and measured as #183 (FP); read as a power case by
  BC; priced on the i9's ledger by BWT and SN.
- **P86:** rejected → conditional on the titration (BWT). WR tried non-temporal copies in
  the walk's flush. BT found the reader's copy.
- **P87:** open → shipped as #164 (P87I, P87V).
- **P88:** no change. PS §2 checks the error is unchanged under the parallel scan.
- **P89:** no change. BW2 and FP add correctness checks; #207 and the finder widen the x86
  lead.

### Reopen candidates

Rejected or superseded IDs whose rejection was of a construction, or of no construction at
all, with what a new construction would have to cost.

- **P85 (#183).** Closed on its 4-thread rule. A new registration needs no new code: 1-thread
  wall as primary, 4 threads pooled over three runs, a stated settle-run rule. It must not add
  more than #183's +41,319 instructions a block on `runs`, where it flagged once. For the i9
  it must first show fewer cold lines in the simulator with ten plain workers.
- **P83, as the LF-direction fill.** It must bring front plus build to at most 0.75 of
  today's on Wikidata at the CI stage harness, with the backward walk within noise of the
  forward one (BK §2). The incumbent build is 4.92 instructions a row.
- **P84, as design D.** Build plus walk must stay under about 2.5× production's 15.3
  instructions a row to tie the cooperative default at ten, and near 1.2× to be worth 2.6 GB/s
  (SN §3b). Its build must be counted complete, with E's 1.83× time penalty in mind.
- **P71.** Never tested. A variant must cut `read_item`'s 144.8 Ir a 4-lookup iteration, or
  the table construction a block on level-1 input; neither cost is priced for a smaller table.
- **P80.** A new construction must beat #312's build on the windows it leaves alone (7 to 16
  runs), without the four write streams that cost #23 +12.1%.
- **P86.** It needs a boundary where a buffer is not read again soon, and the titration to
  read BOUND. At most 6% of the default plan's traffic by BWT's model.
- **P75.** A second level for far moves must cost under the 79 instructions a move counted on
  records, with no loss below index 32, where #207's chain is 3.0 to 4.0 cycles.
- **P73.** Already reopened and shipped as #207. What is left is the same change for the
  checkpointed kernel (C29) and an aarch64 twin.
