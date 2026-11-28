# Synthesis, twentieth pass: after the lanes, the knee needs all three lines at once

*28 Nov 2026, master `805285a`. Smooth zone, no run made. All new figures here are derived from
recorded measurements; each source is named. The nineteenth pass (#1250) read only the
forward-lanes bundle; this pass reads what landed beside it.*

**Read:** `2026-11-27-scanner-a2-fast-i9-verdict` and `2026-11-28-scanner-a2-fast-i9-run-review`
(#1252, bundle `9dacc085`); `2026-11-27-explore-composed-scanner-stack` (#1235);
`2026-11-27-explore-two-lane-scan` (#1251); `2026-11-27-mtf-fold-batch-measure` (#1234, STOP);
`2026-11-27-open-cap-prescan-cost` (#1246); `2026-11-27-crc32-avx2-ring-barrett-build-2` (#1249,
STOP); `2026-11-27-explore-slow-mode-settle` (#1240); `2026-11-28-release-0-5-18` (#1253);
`2026-11-27-ci-pattern-4t-arm-bias`; dead-ends rows for #1034, #1053, #1119, #1179;
`src/open.rs` (`cap`, `Cap::dense`, `fold`); PERFORMANCE-POLICY §4.

## 1. Pricing the three lines together at 8 workers (Wikidata x4, i9)

Start from the nineteenth pass's traced lanes cell: scanner 179.4 ms (contended), workers
169.6 ms/w, wall 184.8 ms; wall ≈ max(S, W) + ~5.4 ms in all four traced cells.

| step (cumulative, derived) | S (contended) | W/w | binds | wall |
|---|---:|---:|---|---:|
| lanes (measured) | 179.4 | 169.6 | S by 9.8 | 184.8 |
| + A2 fast (#1117, −4.52% S′, i9 PASS) | ~171.3 | 169.6 | tie (S by ~2) | ~176.7 |
| + rest of the composed scanner stack (#1235: −9.8..−10.3% S, VM) | ~161.5 | 169.6 | W by ~8 | ~175 |
| + mtf-fold A5+A6a, portable build (#1234: −24 ms of 771 ms 1W wikidata; ×4/8 ≈ −12 ms/w) | ~161.5 | ~157.6 | S by ~4 | ~167 |

Reading:
- **A2 alone takes the post-lanes knee to a tie.** Everything after it on the scanner side is masked at
  8W on Wikidata unless a worker cut lands too, and vice versa. #1252's k = 8 ratio of 0.9999 on the
  non-lane base is the mirror image: a scanner cut on a worker-bound base.
- **The next ~10% of 8W wall needs all three lines** — forward-lanes adoption, the scanner stack and a
  worker-side A5+A6a cut — and the only worker candidate with a measured, exact gain on the stations
  the lanes leave untouched (A5+A6a; the lanes cut only A6bc+A7) is mtf-fold. Its STOP is on guards,
  not on Wikidata. That makes the mtf-fold guard rescue the binding worker lever once lanes and A2 land.
- Use the **portable-build** figure (−3.38% worker CPU) for i9 pricing, not native (−7.58%) or
  JCC-32B (−7.76%). #1119's i9 run showed Comet Lake-S stepping 5 carries the silicon fix and JCC
  alignment is +3.05% *slower* on i9 workers; on the Emerald Rapids VM (no erratum) a JCC-32B arm
  measures layout perturbation only. The same caveat applies to #1235's JCC k* (8.85 → 9.78).
- Caveats: the scanner inflates +10..15% under 8 contending workers (nineteenth pass), and I applied
  #1235's and #1234's ratios to the contended figures as if inflation were proportional. #1235 is VM
  evidence; A2 is i9. The fold's ms saving is assumed additive with the lanes because the stations are
  disjoint, which no run has tested.

## 2. mtf-fold's text failure is the population the shipped path already learned to refuse

#1234's `text` guard (+36..+47%) fails because all 28 full blocks run the whole fold (Pass 1 + 2,
OutQ16, edges) and are refused only at the end by `f.m > cap.m`, then replay the shipped path. These
are dense-band blocks (0.10 ≤ r/n < 0.21) that refuse on m: exactly the population #1040 kept the
`within_cap` pre-scan for in [0.15, 0.21), and that #1246 (route-identical, −2.9% 8T CPU) let
`try_run` refuse inside Pass 1 in [0.10, 0.15). The fold path has no equivalent: its only early
refusal, `fold::refused`, is on literals.

A sound streaming rule exists because `n ≤ bound` (`b.max_len`) and both counters are monotone
(`f.m += w` only; records only accumulate), while `Cap::dense(n).m = min(256 000, 0.27 n)` is
non-decreasing in n:
- records-so-far ≥ ⌈0.21 · bound⌉ ⇒ r/n ≥ 0.21 ⇒ `cap` is `None` ⇒ NotOpen;
- records-so-far ≥ ⌈0.10 · bound⌉ and f.m > `Cap::dense(bound).m` (or ns > `DENSE_STRIPS`) ⇒
  `cap` is `None` or `dense(n)` with `dense(n).m ≤ dense(bound).m < f.m` ⇒ NotOpen.

Neither ever refuses a block the shipped path opens, so routes are unchanged by construction. How
early it fires on `text` (and whether n ≈ bound on full blocks, which makes it tight) is measurable
without the i9. Filed as `mtf-fold-streaming-admission` (idea). `runs`'s A9 staging regression is a
separate fix and stays in that claim's second step.

## 3. Three recent STOPs and one rule turn on peak RSS bounds that sit below their own noise

- #1179 (CRC ring, zero heap): STOP on a ≤1.005x point bound; the same run moved cells −2.6..+2.2%.
- #1119 (JCC flags): one of its failures is 8W RSS 1.0227x against ≤1.020x for a .text change.
- `ci-pattern-4t-arm-bias`: on 2-core CI runners `pattern` 4T is bimodal in RSS (36–38 vs ≥39 MB,
  +4–8 MB of queued output), and round 0 (head always first, no warm-up) shows RSS 1.054x on strict
  nulls. #1179's failing cells are `text`/`pattern` at 1–4T: the same regime is the obvious suspect.
- Against that, the one real RSS effect, #1053's +4.19 MiB/8W past the 20 MiB L3, was about hot
  per-worker working set, and the lanes' +6.33 MiB (1.061x) still win (nineteenth pass).

PERFORMANCE-POLICY §4 allows ≤10% growth; registrations have been freezing 0.5–2% point bounds with
no A/A calibration. The common object is the RSS noise band per corpus × thread × runner. Existing
bundles and the strict-null CI runs already hold the A/A data to derive it without a new run. Filed
as `rss-bound-calibration` (idea, smooth): a census and a table registrations can cite, which
`crc32-avx2-ring-rescue` needs first.

## 4. Smaller relations

- **The A3 register budget is the shared object of three scanner lines.** The two-lane probe falls
  from 0.844x to 0.923x when lane state spills; #1235 found #1117's slice adds one `(%rsp)` reload to
  S3's inner loop (records S +2.2..+2.5%); k3-pipe keeps it at one. The two-lane build should start on
  the merged #1117 + k3-pipe loop and its gate should count inner-loop stack loads. And per §1 it is
  masked on Wikidata 8W after A2; its value is on text and off-Wikidata corpora, as its claim says.
- **Release 0.5.18** (#1253): the candidate holds #1234 and #1246 (worker path) and maybe #1117
  (scanner). On the shipped non-lane base, worker-bound at 8W, #1246 should show in the i9 gate's
  8/10T cells (its VM 8T wall was −3.1%; 0.5.17's loss was 0.974/0.985), and #1117 should not.
  That is the prediction to read the pre-release bundle against.
- **Settle and the pre-release gate.** #1240 ties i9 slow mode to sub-second settle gaps. I did not
  check the settle in #1117's harness; its gated cell is the serial scan phase, not an unpinned wall.
  The pre-release gate judges unpinned 8/10T walls, so it should keep ab's default 2 s settle.
