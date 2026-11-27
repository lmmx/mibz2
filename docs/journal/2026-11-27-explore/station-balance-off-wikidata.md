# Where is the knee off wikidata? Scanner and worker demand on eleven corpora

The `explore` claim, 27 November 2026. `frontier/DECODER.md`'s system picture is that the
serial scanner (S) and the k workers (W/k) co-limit at k = 8 on the i9, and the ninth
synthesis pass derives that the knee now ties or flips to worker-bound. Every number behind
that comes from one input, `corpora/wikidata.bz2`, which compresses 17.7×. Scanner work
scales with symbols and worker work with output bytes, so the balance point should move with
the compression ratio. Nobody had measured where it sits on other inputs. The route census
(`route-census.md`) priced the inverse off wikidata, but not the two stations against each other.

The decision rule was frozen and pushed before any output was read
([`station-balance/registration.md`](station-balance/registration.md), commit 46a50f1).

## Method

`examples/station_balance.rs` (research only, no `src/` edit) decodes a file on one thread
in order, as `decode_with` does. It times each block's `Scanner::next` (headers, selectors,
Huffman: S) and its `decode_block::<OpenCells>` (MTF/RLE2, construction, emission, RLE1,
CRC: W) separately. For each block and station it keeps the fastest of 5 passes. The balance
point is **k\* = W / S**, the worker count at which W/k = S. It measures demand, not a
wall-clock prediction: max(S, W/k, D) is a lower bound, not an equality.

The corpora are wikidata (the dump's own lbzip2-shaped blocks), wikidata re-encoded by
`bzip2 -9` as a control, and nine built in the container with `bzip2 -9`. Those are prose
(text docs and package metadata, 8.2 MB), pysrc (Python 3.12 stdlib), rustsrc (the cargo
registry's `.rs`), json (this repository's `.json`/`.jsonl`), mdjournal (this repository's
`docs/**/*.md`), logs (`/var/log` and dpkg state), elf (`/usr/bin`), dna (uniform ACGT) and
random. pysrc, rustsrc and dna hash to the same bytes as the route census's (SHA-256
prefixes are in the delta). The runs were three interleaved rounds on a 4-vCPU Xeon @ 2.1 GHz
VM with no PMU, 2 MiB L2 per core and 260 MiB L3, and medians of the rounds are reported. The
rounds agree within 1–5% (`station-balance/rounds.tsv`).

## Result

| corpus | ratio | syms / out byte | S ns / sym | W ns / out byte | k\* (VM) | r = k\*/k\*_wiki | i9 k\* (derived) |
|---|---:|---:|---:|---:|---:|---:|---:|
| wikidata | 17.7 | 0.119 | 2.68 | 2.03 | 6.33 | 1.00 | 8 |
| wikidata, bzip2 -9 | 17.7 | 0.119 | 2.62 | 2.02 | 6.47 | 1.02 | 8.2 |
| prose | 6.8 | 0.325 | 2.83 | 3.82 | 4.16 | 0.66 | 5.3 |
| pysrc | 5.5 | 0.393 | 2.78 | 4.00 | 3.67 | 0.58 | 4.6 |
| rustsrc | 10.4 | 0.230 | 2.73 | 3.40 | 5.39 | 0.85 | 6.8 |
| json | 8.1 | 0.289 | 2.32 | 3.07 | 4.56 | 0.72 | 5.8 |
| mdjournal | 3.9 | 0.552 | 2.68 | 5.79 | 3.91 | 0.62 | 4.9 |
| logs | 6.5 | 0.330 | 2.64 | 3.52 | 4.03 | 0.64 | 5.1 |
| elf | 2.8 | 0.552 | 3.74 | 7.52 | 3.65 | 0.58 | 4.6 |
| dna | 3.7 | 0.923 | 1.88 | 7.57 | 4.37 | 0.69 | 5.5 |
| random | 1.0 | 1.000 | 3.63 | 12.41 | 3.42 | 0.54 | 4.3 |

**The rule's verdict:** over the seven named corpora (prose, pysrc, rustsrc, json, mdjournal,
logs, elf) the median r is **0.64**, with a range of 0.58–0.85. That is in the frozen
"≤ 0.75: corpus-dependent" branch. The wall check applied to no corpus, because no VM k\* was
≤ 2.5.

## What it means

- **Wikidata sits at the worker-heavy extreme.** On every other input the workers finish a
  block in 0.54–0.85 of the scanner-relative time they need on wikidata. The cause is the one
  the question guessed. The scanner's cost a symbol barely moves (2.3–2.8 ns on text). Symbols
  per output byte rise 1.9–4.6× on text. Worker cost per output byte rises too, by 1.5–2.9×,
  partly because off wikidata Open is a minority route (route census). It rises less, so the
  balance tips toward the scanner.
- **The encoder isn't the cause.** The bzip2 -9 control's k\* is within 2% of the dump's,
  consistent with `encoder-dependence.md`. Runts dilute the window, but they don't change
  demand.
- **Derived for the i9 (a prediction, not evidence):** scaling by r from the shipped
  wikidata knee of ≈ 8 puts typical text's balance at **k\* ≈ 4.6–6.8 (median ≈ 5.1)**. At
  8 workers, on such input the shipped decoder would be scanner-bound, with roughly a third
  of worker capacity idle. Worker-side levers (A7+A9, OutQ16, A6b) that the ninth pass prices
  at the k = 8 knee would be masked there. Scanner levers (#1034, `scanner-a2-fast-build`,
  #1031) and the slack-only hybrid scan (`pooled-scan-hybrid`) are the ones that pay. The
  hybrid's condition S′ − W′/8 > ~5.5 ms becomes S′(1 − k\*/8), about 0.15–0.43 S′, rather than
  the ≤ 0–4 ms gap the ninth pass derives for wikidata. The ninth pass's "hybrid value moves
  to k = 9–10" is a wikidata statement.
- **Worker gains made off wikidata are masked.** `dense-cap-precheck` cuts W on dense-band
  text. Even so, at k = 8 it lowers that text's k\* further and mostly frees worker slack.
  Its value there is CPU and energy, plus wall at k ≲ 5, not k = 8 wall.

## Supplementary: walls on this VM (not under the rule)

`bzcat` at a window of 4 × workers, 7 interleaved rounds, median (min) in ms. bzcat runs a
stdin thread, the scanner, k workers and the writer. So at k = 3 it has six threads on 4
vCPUs, and the k = 3 figures are oversubscribed.

| corpus | k = 1 | k = 2 | k = 3 | k3/k2 |
|---|---:|---:|---:|---:|
| wikidata | 535 (480) | 294 (266) | 240 (219) | 0.82 |
| mdjournal | 119 (99) | 76 (68) | 68 (65) | 0.90 |
| elf | 296 (279) | 188 (158) | 186 (167) | 0.99 |

The direction agrees: elf (k\* 3.65) stops improving from 2 to 3 workers, while wikidata
(k\* 6.3) still gains 18%. The plateau sits well above max(S, W/k), though: elf has S = 62 ms
and W/3 = 75 ms but a wall of ~186 ms. So something besides the two stations binds on this
small, oversubscribed machine. This is a direction check, not a confirmation of k\*.

## Not done, and caveats

- No i9 run. The i9 column assumes that r transfers from a 1-thread VM measurement with
  2 MiB of L2 to eight contending workers on a 256 KiB L2 and 20 MiB L3. On wikidata, L3
  contention inflates W on the i9 (worker-l3-footprint), and that inflation may differ on
  text. Claim `text-knee-i9` (idea) measures this.
- The corpora are small: 9–33 blocks each, against wikidata's 233 full blocks. They are
  ad hoc and not checked in.
- D (the reader's copy) isn't priced here. It scales with output bytes, as W does.
