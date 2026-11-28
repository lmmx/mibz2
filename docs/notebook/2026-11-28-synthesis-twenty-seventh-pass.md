# Synthesis, twenty-seventh pass: the release candidate's i9 gain is worker-shaped, the lanes are the gate for every scanner lever on default Wikidata, and VM prices overstate i9 worker savings

*28 Nov 2026, master `a5b51da2`. Claim `synthesise`, continuing from the twenty-sixth pass (#1333, `07ab2e7e`). One read of an archived bundle (below), no new runs. Measured figures are quoted from the delta, PR or bundle named beside them; anything marked *derived* is arithmetic on those figures.*

Read since #1333: #1335 (Python abi3 line/stream delivery), #1339 (`u8` symbol handover dissection), #1344 (default OpenCells A5+A6a dissection), #1345 (C6 review of #1343), #1346 (AVX2 ring CRC i9 STOP, `40bb1509`), #1350 (recalibrate-0518 readback, `f8b21e4`), #1351 (shipped i9 Wikidata worker-bound), #1256/#1340 (pre-release i9 gate and its queueing), the release-0-5-18 prep delta, and the open PR #1343 (scanner-a3-k3, changes requested, unchanged since 10:32Z). Also the release-preflight bundle `c8489a45`, which nothing on master had read yet.

## 1. The v0.5.18 preflight bundle is a VALID PASS, and its shape confirms #1351

`origin/evidence` `c8489a45` (`release-preflight/20261128T111128Z.tar.gz`, sha256 `64a382d2…4604`, checked against the sidecar). Re-judged offline with the reviewed judge on master (`bench/release_preflight.py judge --candidate 7be50513… --base 01acd74d… --recovery-base 4e261140… --recovery-cells wikidata:8,wikidata:10`): **valid, pass, parity ok, no problems, regressions, secondary or memory failures.** The bundle's own `outcome.txt` says `VALID PASS`.

| Cell (i9, `wikidata` x1, candidate / v0.5.17, 12 paired rounds) | wall | CPU | peak RSS |
|---|---:|---:|---:|
| 1T | 1.007 [0.963, 1.054] | 1.049 (median ratio; paired wall CI spans 1) | 1.003 |
| 2T | 0.978 [0.955, 1.002] | 0.995 | 0.965 |
| 8T | **0.967 [0.960, 0.974]** | **0.966** | 1.023 |
| 10T | **0.969 [0.959, 0.979]** | **0.965** | 1.008 |
| 12–20T | 0.942–0.958, all CIs below 1 | 0.959–0.966 | ≤ 1.009 |
| Recovery vs v0.5.16 at 8T / 10T | 0.991 [0.984, 0.997] / 0.987 [0.977, 0.997] | — | — |

Text is flat everywhere (0.968–1.009, every CI containing 1 except 10T at 0.985 [0.973, 0.997]).

What this means together with #1351:
- **The gain appears only where OpenCells runs.** ab's 1-thread `mibz2` arm uses the single-threaded Cells route, which never calls OpenCells (#1344 §4b note); the candidate's default-path changes since v0.5.17 are worker-side (#1274's `front_avx2` Pass 1, #1294's A6c wire). 1T is flat; every k ≥ 8 cell gains.
- **At the shipped 8T and 10T plans the wall ratio equals the CPU ratio** (0.967 vs 0.966; 0.969 vs 0.965). A worker cut converting one-for-one into wall is what a worker-bound plan predicts, and is what #1351 read from the recalibration rows' per-thread CPU (W/k − S = +78.5 ms at pinned 8T, +39.6 at pinned 10T). Two independent bundles, one by station CPU and one by paired wall, now say the same thing about the default build.
- **The 0.5.17 regression is recovered** at both cells it was detected in.

For release-0-5-18's holder: §3.5 is satisfied by this bundle; §5c's pinned family needs no retune (#1350). What remains is the holder's choice between #1350's options (a) and (b) for the unpinned half, then `verify` and Deploy Release. Nothing here takes that claim.

**A correction to the release-0-5-18 prep delta's §3.3 retention audit:** it lists `forward-anchor-lanes` as "lapsed (default flipped in #1312)". It is not default. `Cargo.toml` has `default = []` and #1312 only bounded the lanes' Arena behind the feature. The feature is the live P1 adoption candidate (`forward-emission-bounded-slab-rescue`, PR #1324). It must not be removed in the post-release cleanup.

## 2. The lanes are the gate for every scanner lever on default Wikidata

On the default build the scanner levers in flight (#1343's three-symbol A3, the composed adaptive seam, the `u8` handover's scanner half) buy little Wikidata wall at shipped plans. To bring the unpinned 8/32 plan to a tie, workers need ≥ 11–16% off W/8 (+19..29 ms over S ≈ 150 ms, *derived*). The pinned 6/24 8T plan needs ~34%. Which worker cuts could supply that?

- **Lanes (#1324):** −17% at 1W on i9 (`1950f77f`), already measured to make 8W scanner-bound. This is the only candidate past the threshold on target-machine evidence.
- **Default A5+A6a (`default-a5a6a-fast-build`, #1344) plus the `u8` handover's worker half (#1339):** VM whole-decoder 8W CPU −7.3..−8.4% plus worker −2.8%, roughly −10..−11% on the VM (*derived*, additive only as an upper bound).
- **The VM-to-i9 transfer ratio for worker savings is well below 1.** It was 0.5× for #1328's open-route cell (VM −6.23% → CI EPYC −3.1%, #1245), and about 0.2× for the CRC ring: ~5% of W predicted from VM station prices and #1179's isolated kernel, against −1.0% [−1.5, −0.5] measured in-decoder on the i9 (#1346). At 0.2–0.5×, the default A5+A6a + handover stack is −2..−5.5% of i9 W (*derived*), short of the 11% threshold.

So #1343's PASS/STOP is decided by S' alone (registration rule 1), which is right. Its real-decode rows at 8–10 workers will be diagnostic on a worker-bound default build, and should be read that way rather than as a missed payoff. Scanner levers pay on text and random now (scanner-bound there, #1351), and on Wikidata once the lanes are adopted. **#1324 remains the shortest path to making the scanner stack pay at 8 workers.** Default A5+A6a is the largest default-path worker lever without it: it pays one-for-one at shipped plans, the same shape the preflight shows for #1274/#1294. It should be built next among the worker ideas, expecting an i9 gain nearer −2..−4% than the VM's −7..−8%.

## 3. One cause behind two readbacks: the binding station is corpus-dependent, so the trim is a corpus trade

#1350's pinned 10T finding (8/32 at 0.911 of 7/28 on Wikidata but 1.064 SLOWER on text) and #1351's station split (Wikidata worker-bound; text bound by neither station, its worker CPU rising 132 → 430 ms with k as waiting; random scanner-bound from 4 workers) are the same fact. One more worker helps a worker-bound corpus and costs a corpus whose extra workers only wait and contend. No single trim is optimal for both. A future trim change has to be priced as a corpus mix, not a per-corpus win, and #1350 correctly did not adopt it under C2. This argues for the `ab-station-ledger` reader (idea) before the next recalibration, so the binding station is printed beside each verdict.

## 4. The rest

- **Python abi3 delivery (#1335):** reader station D at the Python layer (line delivery 26× on `raw_iter_lines`, −28% `open_readlines`, −9% CPU and −12% HWM on Wikidata `open_read_64k`). It is orthogonal to the S/W tie and does not enter DECODER's Rust model. It shows that D binds for Python line consumers long before S or W do.
- **CRC (#1346):** a standalone CRC swap is dead at these bars on both vendors. The CRC remains reachable only inside a pass-removing fusion (`forward-rle1-crc`, `segmented-fused-postwalk`), as #1346 says.
- **#1343:** C6 changes requested (calibrated RSS gate, build-env guard and distinct binaries, formatting); no commit since. The Rust passes review. Its holder lease runs to 17:20Z.

## 5. Implied claims

- `vm-i9-transfer-ledger` (idea, P3, smooth): tabulate, for every lever with both a VM/CI paired price and an i9 in-decoder measurement, the ratio of measured to predicted saving by station. That would turn the 0.2–0.5× above into a calibrated prior for pricing VM dissections (#1339, #1344) before they spend an i9 run.
- No priority change: `default-a5a6a-fast-build` stays P2 at idea (the open-claim limit, 12, is full); this note is its pricing.
