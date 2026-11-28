# Synthesis, twenty-fifth pass: working set is scanner time, the L6a premise moved, and a VM-to-i9 transfer ratio

*28 Nov 2026, master `c11df034`. Claim `synthesise`, continuing from the twenty-fourth pass (`#1307`, `5c762ed1`). No new runs. Every figure is quoted from the delta or bundle named beside it; anything marked *derived* is arithmetic on those figures, not a measurement.*

Landed since `#1307`: `#1312` (`forward-emission-bounded-slab-rescue`, bounded 80 KiB lanes Arena, behind `--features forward-anchor-lanes`), `#1274` (`runs::front_avx2`, i9 C6 PASS `95edea2a`, default path), `#1304`/`#1308`/`#1309`/`c7fa3284`/`#1315` (scanner PMU recipe, queue, CONFIRMED verdict `a7ac01d8`), `#1313` (fold A5+A6a dissection, idea `fold-a5a6a-fast-pass1-pass2`), `#1314`/`#1319` (`forward-lanes-mlp-and-prefetch` admitted, then scoped C2 STOP), `#1316` (ledger), `#1318` (A6b dissection, idea `a6b-zero-rss-endpoint-build`), and the blog post on clock hysteresis and peak-RSS noise (`b28dd818`).

## 1. Worker working set is paid in scanner cycles at 8 workers

`a7ac01d8` separates the two effects that made the 8-worker scanner slower: the turbo bin (1W to 4W, `cycles/ref-cycles` 1.43 to 1.33, flat after) and L3 eviction (4W to 8W at constant clock). Scanner instructions are invariant, so every extra cycle is stall.

- `base`, 4W to 8W: `1.0230×`, `+17.43M` cycles, 88.4% of them L3-miss stalls.
- `lanes` (unbounded `#1217` Arena), 4W to 8W: `1.0647×`, `+49.28M` cycles, 76.4% L3-miss stalls, with `+6.53 MiB` peak RSS over `base` at 8W (110.88 vs 104.35 MiB).

*Derived:* the `lanes` excess over `base` is `+31.85M` scanner cycles for `+6.53 MiB`, about **4.9M cycles, or 0.63% of the 8W scanner, per MiB of 8-worker working set**. This is one linear slope across a threshold (the i9's 20 MiB L3), so it is an order of magnitude, not a law. It still matters because on forward lanes, Wikidata at 8 workers is scanner-bound (surface row S, `e98948b6`): there, a worker-side byte saved is scanner time saved, and worker-CPU savings are masked.

Consequences:
- **Prediction for `#1312` (falsifiable, free to test):** VM RSS shows `#1312` removes 96% of the 8W excess on `base` flags (`+0.30` vs `+7.69 MiB`) and 82% on `jcc` (`+1.29` vs `+7.22 MiB`). If the slope holds, the `lanes` 8W/4W scanner ratio falls from `1.0647` to about `1.025–1.03`, near `base`'s `1.0230`. The frozen `bench-scanner-contention` recipe already measures this; rerunning it with the bounded build is the cheapest i9 discriminator of whether RSS was the whole cause. Opened as idea `scanner-contention-bounded-recheck`.
- **The base residual (`+2.3%`, 88% L3 stalls) is the general tax.** RSS-neutral or RSS-negative worker levers are worth more at 8W than their 8W CPU figure says. `a6b-zero-rss-endpoint-build` (VM `0.980×` 8W HWM, about `-1 MiB`) is worth roughly `0.6%` of 8W scanner cycles on this slope (*derived*), on top of its CPU.

## 2. L6a's premise has moved: RSS is mostly spent, the stitch is what's left

The twenty-fourth pass ordered the lanes stack as bounded slab, then `L6a` (`forward-rle1-crc-vectored-build`) "reducing 8W RSS", then `mtf-fold-streaming-admission`. Three deltas since then re-price `L6a`:

1. `#1312` removed most of the 8W RSS that `L6a` was to remove (above). Its RSS case now rests on the residual `+1.29 MiB` (`jcc`, 8W), not on `+6.7 MiB`.
2. `forward-lanes-mlp-and-prefetch`'s STOP root cause: `#1312`'s 4 KiB chunk permutation plus tail stitch is about 20% of A7. A zero-stitch streaming emission removes that cost and is the condition under which the delta says K=8 + `edges[next]` prefetch (`-6.2..-18.5%` A7) is unclosed.
3. The fold dissection (`#1313`) finds a trap on `L6a`'s path: if `L6a` swaps a cleared `out` into `s.ll` (`len == 0`), `buf.resize` pays `21.5–22.3 ms` of `memset(0)`, 13.4% of A5+A6a. Preserving `s.ll.len()` costs `0.25–0.28 ms`.

So `L6a`'s value is now mainly *removing the stitch* (about a fifth of A7, and the gate on K=8 prefetch), with RSS secondary, and its implementation must keep `s.ll.len()`. I added the three premises to `forward-rle1-crc-vectored-build`; its question and stop test are unchanged.

## 3. A first VM-to-i9 transfer ratio for worker-CPU levers

`#1274` is the first worker kernel with both a VM prediction and an i9 C6 result: registration predicted `0.97`, the C6 review revised to 1.7–2.3% at 1T, and the i9 gave `0.9892` at 6T (significant), `0.9939` at 1T (not significant), masked at 8–10T. *Derived:* the i9 delivered roughly a third to a half of the VM prediction. One sample, one kernel, but it is the only calibration we have, and it bears on two idea claims whose frozen i9 gates sit near their VM effect:

- `a6b-zero-rss-endpoint-build` gates `wikidata/1` at `<= 0.985×` with CI below 1. Its VM whole-decoder 1W effect is `-1.64%` (`base`) / `-1.82%` (`jcc`). At the `#1274` ratio that is about `0.6–0.9%`, below the gate; even full transfer barely clears it. Its stronger VM number is `workers-lazy` 1W `base` `-6.23%`, but the gate binds on both. Before i9 time is spent, its registration review should decide whether the 1W gate is reachable or whether the claim's case is the 8W RSS/scanner coupling of section 1, which the current gate does not measure (C2: this is a pre-data registration question, not a relaxation).
- `fold-a5a6a-fast-pass1-pass2` claims `-3.7..-4.0%` of B4 worker CPU L3-cold on the VM; at the same ratio, about 1.5–2% on the i9 at 1–6T, masked on Wikidata at 8–10T. Its i9 gate (`wikidata/1` `ci_hi <= 0.975`) is in the same danger. It also inherits `#1274`'s Pass 1 directly, so its codegen risk is lower than its timing risk.

## 4. Release 0.5.18: the candidate is no longer what its claim describes

`release-0-5-18`'s question lists one commit (`#1234`) and argues §5c does not apply because only native instruction selection changed. Since then six more `src/` commits are on master: `#1246` (skips a pre-scan, routes unchanged), `#1117` (scanner A2), `#1268` (feature notes), `#1294` (wire), `#1274` (`runs::front`), `#1312` (feature-gated lanes). RELEASING §5c names "an extra pre-scan even when the final routes are identical" and work-per-block changes explicitly, and says unchanged admission counts are not evidence of unchanged cost. `#1246` and `#1274` both change a worker's work per block on the default path, so §5c's "when" condition is now plausibly met and must be reassessed by whoever takes the release; I appended that to the claim's question. The block itself (`pre-release-i9-gate`, `#1256`) is unchanged.

## Claims touched

- New idea: `scanner-contention-bounded-recheck`.
- Premises added (no other change): `forward-rle1-crc-vectored-build`.
- Question appended (candidate list and §5c note; status, stop test unchanged): `release-0-5-18`.
- `synthesise` stays open (recurring).
