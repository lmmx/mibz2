# The i9 "slow mode" (b) tracks the settle gap, not pinning

One line: of six same-day i9 bundles, only the one run at `--settle 0.2` shows mode (b). Every bundle at ab's default 2 s settle is unimodal fast, unpinned rows included, so mode (b) is a short-gap harness state and not the shipped default's majority state.

- **Claim:** `explore` (recurring). Cold readback of archived bundles; nothing re-run.
- **Evidence (`origin/evidence`, each SHA-256 checked against its archived `.sha256`):** `5e3f28cc` text-knee 17:27Z (`786084e2…`); `d6bc8ca2` scanner-adaptive-seam 19:26Z (`c14e53b4…`); `51183982` crossover 17:12Z (`be8305bd…`); `0ffba60b` crossover 21:44Z (`facc37f5…`); `5182d546` release-scaling 21:33Z (`3b3af8b1…`).
- **Classifier:** a run is slow if its CPU exceeds 1.2x (1.25x for the load test) the fastest run in its (corpus, arm, threads, cpus) cell. Mode (b) is a ~1.4-1.6x uniform CPU scale, so the threshold is not delicate.

## Counts

| bundle | settle | unpinned rows | slow / runs |
|---|---|---|---|
| text-knee `5e3f28cc` | 0.2 s (`--settle 0.2`, one fresh `ab run` a sample) | k = 4..10 | 119 / 252 (wikidata x4: k=6 7/9, k=8 8/9, k=9 6/9) |
| adaptive seam `d6bc8ca2` | 2 s (default) | 6, 8, 9, 10, 12 threads, wikidata x4 | 0 / 160 |
| crossover `51183982`, `0ffba60b` | 2 s | 12, 16, 20 | 0 / 400 and 0 / 400 (the 17:12 one ran 15 min before text-knee) |
| release-scaling `5182d546` | 2 s | 12, 16, 20 | 2 / 240 (both 16T, ~1.2x, not mode (b)'s 1.45x) |

- **Same machine, corpus and cells.** Text-knee's wikidata is x4 (838,800,784 bytes out), as is the seam's. At 8 threads unpinned, text-knee's fast run is 0.196 s, CPU 1.78; its slow runs are 0.285-0.320 s, CPU 2.46-2.69. All 32 of the seam's 8T runs fall between 0.189 and 0.212 s, CPU 1.70-1.85. The seam's perf counters put every one of its 160 runs at 4.871-4.897 GHz effective.
- **It is uniform.** In a slow text-knee run every thread scales by the same ~1.5-1.6x, the scanner included (8T: scan 0.16 -> 0.26 s, each worker 0.17 -> 0.27 s), at unchanged RSS. That points to the clock (the ~1.43 turbo/base ratio the clock-modes registration cites), not extra work.
- **Not SMT packing.** Post-run per-CPU frequencies (siblings are i and i+10) show slow runs with fewer recently busy CPUs and no more busy sibling pairs than fast runs.
- **Short timescale.** In text-knee, slow runs follow more CPU in the preceding 1 s (wikidata median 3.45 vs 2.74 CPU-s; docs_md 3.79 vs 2.71) but not in the preceding 10 or 30 s (within 1-3%). So it is not a long power-budget window (PL1/Tau). Slow and fast runs interleave from the second run of the sweep, with no clustering in time. A preceding slow run doesn't predict a slow one (0.23 vs 0.31).
- **Duty cycle.** Text-knee kept 2.30 cores busy on average over 188 s, with a 0.31 s median gap between runs. The seam kept 0.76 cores busy over 372 s, with a 2.11 s gap.

## What it changes

- **DECODER.md's clock-mode sentence** ("the unpinned mode is the shipped default's majority state … recorded i9 medians are mostly slow-mode") generalises from text-knee alone. Its 69% slow figure is a property of a 0.2 s settle. Corrected there.
- **Text-knee's verdict stands.** It was judged within mode. Its pooled medians are short-gap medians, though, and mixed-mode.
- **`i9-clock-modes` (#1201, unrun):** its frozen design varies settle over {0, 0.2} s only, both on the short side of the split above. As registered, it can't separate "harness artefact" from "met in production", which its own question asks. Its premise that mode (b) is "unpinned only" is also contradicted: the seam ran unpinned at 6-12T with none. A flag holds the run until a production-like gap (ab's default 2 s settle, or the gap a real caller leaves) is a level, or the question is narrowed (`frontier/flags/2026-11-27-clock-modes-settle-levels.md`).
- **#1119's 6T mode (b) (12/12 in pair position 2)** is consistent with this: a back-to-back second run is the shortest gap there is. I did not check #1119's settle.

## Caveats

Observational, across recipes: settle is confounded with the harness (text-knee spawns a fresh `ab run` per sample; the others run every round in one process), with the time of day, and with the corpus mix. That bundles run on either side of text-knee, within two hours, show no slow runs argues against a machine episode, but only a run that varies settle alone, with everything else fixed, separates the two causes. The mechanism (turbo denied for a while after a heavy run? HWP?) is untested.
