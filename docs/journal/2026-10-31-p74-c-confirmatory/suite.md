# P74-C adversarial suite: design and audited handoffs (untimed)

Frozen before any timing. Generator: `examples/p74_suite.rs`, base seed
`0x74c0202610310100`; file i (1-based, in the order below) uses seed + i, and edge
files draw their pairs from (seed + i) XOR `0x5a5a5a5a5a5a5a5a`. Regenerate with
`cargo +1.90.0 run --locked --release --example p74_suite -- DIR`. Each file is one
bzip2 stream of ONE block, compressed by the repository's encoder (`common::compress`,
bundled libbzip2 1.0.8), level 9 except `edge-l1-*` (level 1). Total 2,590,825 bytes:
the files are not committed; their SHA-256 are in `suite.json` and `frozen.json`.

**Constructions [D].** L's region for first-column byte c holds the bytes preceding c.
- *pair:* iid letters a..h (low ranks), then pairs (r, 0xFF), r uniform over 0x00..0xFE.
  Region 0xFF, the last of L, holds exactly the r's: a high-rank suffix in a 256-symbol
  alphabet, every rank class present.
- *keyed:* letters, then units (c_j, 0xFF, 4 key digits), key = j; region 0xFF holds
  c_0, c_1, ... in j order, an exact script: lows alternate 0x40/0x41, events cycle a
  pool of H otherwise-unused values (rank H − 1 in a burst, H + 1 between lows).
- Boundary designs solve for S_0, F_0 (symbols and high events before the suffix) by a
  deterministic fixed-point iteration on the actual compressed stream, with pre-declared
  alternates (letter count 20000 + a, a = 0..7, first that converges): `hug-eq-p32-extra`
  and `hug-eq-p32-early` converged at a = 3, the others at a = 0. Edge files search the
  letter count for the exact S (pre-declared alternates: pairs + 0..7): `edge-l1-s50000`
  and `edge-l9-s450000` used pairs + 1. Every alternate is chosen by the generator's
  fixed rule, not by hand, and before freezing.
- Text has no run of 4 equal bytes, so RLE1 is the identity: suffix output start =
  text length − (number of 0xFF).

**Families.** split q ∈ {5, 25, 50, 75, 90}% of ~400k symbols; boundary (`hug-*`):
burst to the boundary, then period 32 at equality (evades; the draft rule would switch),
+ one extra event at S_1000 + 17, the event at S_1000 one early, period 33 (under), and
period 31 from 4 events of slack (over); rank classes with steady rank 32, 63, 64, 127,
128, 191, 192, 224, 252, 255; edges S = 16383/16384/16385 (S₀), 49999/50000 (level 1,
2S ≥ N_max) and 449999/450000 (level 9).

**Audit [M].** `audit_manifest_corpora` (the production scanner, the actual kernel's
handoff hook, both CP = 0 and CP = 16384) agrees with the independent oracle at every
block; `bench/p74_experiment.py suite` checks the expectation: 22 handoffs, 5 non-
switching (2 by design, 3 bypassed). **No block failed to hand off as designed; none was
regenerated or dropped.** `--check` (all five arms, libbzip2, independent inverse BWT)
and `bzip2 -t` pass on all 27. Host: Intel Xeon @ 2.80 GHz, AVX2, rustc 1.97.0
(the pinned 1.90.0 run repeats all of this before timing).

| File | Family | S | Bypass | Suffix starts (output) | Switch: consumed / output / F | Draft rule | Suffix rank mode | High events in suffix by class 1..7 | Verified |
|---|---|--:|---|--:|---|--:|--:|---|---|
| `split-q05` | split | 399842 | no | 400000 | 20705 / 400859 / 770 | 20704 | 116 | 47757 47798 47899 47491 47361 47816 46246 | yes |
| `split-q25` | split | 398792 | no | 400000 | 102610 / 403816 / 3329 | 102610 | 76 | 37854 37721 37821 37573 37433 37688 36276 | yes |
| `split-q50` | split | 397252 | no | 400000 | 204712 / 407457 / 6520 | 204712 | 64 | 25075 25108 25011 25013 24995 25266 24382 | yes |
| `split-q75` | split | 395920 | no | 400000 | 307120 / 411199 / 9720 | 307120 | 188 | 12394 12535 12635 12555 12505 12484 12135 | yes |
| `split-q90` | split | 394991 | no | 400000 | 368274 / 413283 / 11631 | 368274 | 193 | 5023 4907 5061 4983 5040 5039 4952 | yes |
| `hug-eq-p32` | boundary | 309415 | no | 320000 | none | 257568 | 63 | 8062 1647 32 10 0 0 0 | yes |
| `hug-eq-p32-extra` | boundary | 309670 | no | 320003 | 289841 / 360174 / 9180 | 257824 | 63 | 8070 1648 32 10 0 0 0 | yes |
| `hug-eq-p32-early` | boundary | 309409 | no | 320003 | 289567 / 360161 / 9171 | 257568 | 63 | 8062 1647 32 10 0 0 0 | yes |
| `hug-under-p33` | boundary | 310798 | no | 320000 | none | 258976 | 63 | 8106 1597 32 10 0 0 0 | yes |
| `hug-over-p31` | boundary | 310678 | no | 320000 | 262847 / 332169 / 8336 | 262816 | 63 | 8098 1698 32 10 0 0 0 | yes |
| `rank-032` | rank-class | 59546 | no | 60010 | 53303 / 61769 / 1788 | 53303 | 32 | 7972 30 0 0 0 0 0 | yes |
| `rank-063` | rank-class | 52016 | no | 60010 | 45527 / 61523 / 1545 | 45527 | 63 | 7941 32 29 0 0 0 0 | yes |
| `rank-064` | rank-class | 59446 | no | 60010 | 53194 / 61760 / 1785 | 53194 | 64 | 5 7967 30 0 0 0 0 | yes |
| `rank-127` | rank-class | 52330 | no | 60010 | 45854 / 61536 / 1555 | 45854 | 127 | 4 32 7904 32 30 0 0 | yes |
| `rank-128` | rank-class | 59533 | no | 60010 | 53289 / 61768 / 1788 | 53289 | 128 | 3 32 32 7903 32 0 0 | yes |
| `rank-191` | rank-class | 52713 | no | 60010 | 46238 / 61537 / 1567 | 46238 | 191 | 2 0 12 32 7843 72 41 | yes |
| `rank-192` | rank-class | 59569 | no | 60010 | 53288 / 61731 / 1788 | 53288 | 192 | 2 0 11 32 34 7879 44 | yes |
| `rank-224` | rank-class | 59560 | no | 60010 | 53272 / 61724 / 1787 | 53272 | 224 | 0 2 0 31 32 62 7875 | yes |
| `rank-252` | rank-class | 59592 | no | 60010 | 53322 / 61742 / 1789 | 53322 | 252 | 0 2 0 31 32 62 7875 | yes |
| `rank-255` | rank-class | 59559 | no | 59995 | 55116 / 63569 / 1845 | 55116 | 255 | 0 2 0 31 32 66 3875 | yes |
| `edge-l9-s16383` | edge | 16383 | yes | 16498 | none | none | 198 | 380 371 355 409 372 354 372 | yes |
| `edge-l9-s16384` | edge | 16384 | no | 16515 | 14007 / 17138 / 560 | 14007 | 139 | 375 347 355 412 384 350 379 | yes |
| `edge-l9-s16385` | edge | 16385 | no | 16504 | 14015 / 17134 / 560 | 14015 | 98 | 360 371 394 398 388 359 370 | yes |
| `edge-l1-s49999` | edge | 49999 | no | 50214 | 20882 / 51097 / 775 | 20882 | 77 | 3736 3878 3657 3750 3804 3734 3629 | yes |
| `edge-l1-s50000` | edge | 50000 | yes | 50207 | none | none | 190 | 3773 3762 3750 3613 3858 3825 3629 | yes |
| `edge-l9-s449999` | edge | 449999 | no | 453418 | 259391 / 462810 / 8228 | 259391 | 170 | 25092 24970 25144 24935 25035 25121 24425 | yes |
| `edge-l9-s450000` | edge | 450000 | yes | 453432 | none | none | 218 | 24860 24783 25232 25214 25000 25116 24480 | yes |

`F` is high events processed before the switch; "Draft rule" is the consumed position
at which the S_before rule would switch (for `hug-eq-p32`, at the first boundary: the
counterexample in a real stream). Keyed files have 29–73 high events in the prefix
(region transitions), recorded as measured.
