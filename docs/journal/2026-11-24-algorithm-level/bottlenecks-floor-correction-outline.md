# Outline: the measured floor beside the published plumbing forecast

*Blog card `bottlenecks-floor-correction`, stage `outline` (26 November 2026). This is an edit plan for
two published posts, not a new post: the card asks for the floor run's measurement to sit beside the
design analysis's forecast "without rewriting the posts". Kept outside `docs/site/` so nothing
publishes before the `draft` stage.*

**Primary sources read for this outline** (every figure below is traced to one of them in §7):
- the floor results, `ideal-parts/results/floor/README.md` and `report.txt` (evidence cc8ba137,
  SHA-256 10df0d89…5e34; 648 decodes on the i9-10900K; README at 311923ef);
- #952's cold-store reread, `docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md` (at 2aaded3a);
- #966's delivery characterisation, `delivery/README.md` §1 (traced medians from evidence c71c4a56);
- the two posts: `2026-11-26-complementary-bottlenecks.md` (§6, §7) and
  `2026-11-26-from-bottlenecks-to-better-algorithms.md` (§2, §3);
- the post it must agree with, `2026-11-26-dropping-the-coordinator.md` (its results table and the
  paragraph after it).

## 1. Target claim, and what the edit is

**Target claim (one sentence):** the zero-copy floor run measured the pipeline's own cost at
5.2–5.7 ms at every worker count, about a third of the 12–19 ms the design analysis forecast, so the
plumbing is negligible and the published forecast stands corrected beside its measurement.

**The form.** Each post keeps its text, title, dek, address and social card. The forecast stays where
it is, because it was a forecast and the post said so; a dated note follows it with the measurement,
its source and what it changes. No sentence of the original is deleted or reworded, so a reader can
still see what was predicted. The front matter, the index card and `cards.json` are untouched, so
`check_site.py` has nothing new to match.

**Takeaways the notes must deliver, in this order:**
1. *Residual B, the pipeline alone* (queues, locks, hand-offs, start-up and draining, with every part
   supplied and delivery discarding its bytes): forecast ~14, ~12–15, ~14–17 and ~15–19 ms at 1, 6, 8
   and 10 workers; measured 5.7, 5.3, 5.2 and 5.4 ms. The run's frozen rule scored the forecast a
   MISS: the plumbing is smaller than predicted, 2.7% of the shipped 195 ms at 8 workers, and closed
   as a target.
2. *Residual A, the word expansion plus the pipeline:* forecast ~273, ~60–62, ~51–55 and ~50–54 ms;
   measured 303.9, 54.0, 42.4 and 40.5 ms. Reported beside the forecast, unscored (the registration
   scored only Residual B). At 1 worker the measurement is *above* the forecast, but #952 found that
   this arm reads the oracles' stores cold from DRAM, and that about 55–65 ms of the 298 ms spelling
   figure is that traffic, not the decoder.
3. *The rising all-ideal wall (112 → 142 ms, 8 → 10 workers)* was the old oracles' copying, as the
   companion argued: with zero-copy oracles Residual A doesn't rise (42.4 → 42.9 → 40.5 ms at 8, 9, 10).
   Memory as a third station, the main post's "natural repair", is not supported by this run.
4. *What the floor run newly measures* goes in one sentence only, linked, and in the corrected figures
   (see §5): the serial scan alone is 179.1 ms at 8 workers, 92% of the wall. Delivery and spelling
   are mentioned only if a sentence needs them, and then never as "48 ms flat" or "298 ms of
   decoder work".

## 2. Reader

Someone who has read either 26 November post, or arrives at §6 or §3 from a search. They know what
an oracle is in this experiment (a part's answer supplied from a recording) and what the all-ideal
configuration is, because the surrounding section has just told them. They don't know the floor
run's arm letters, Residual A and B as measured quantities, or the cold-store finding. So the note
names the arm by what it removes ("every part supplied and the words not expanded"), not as
`SFCMRD`, and puts any letters in link targets only.

## 3. Where each note goes (the argument path)

Each insertion answers the question the sentence before it raises: "was the forecast right?"

**Main post, `2026-11-26-complementary-bottlenecks.md`:**
- **§6, straight after the paragraph ending "…leaves the scanner–construction result untouched."**
  (it states the 12–19 ms forecast). One paragraph, measured figures first: the run has been made;
  the pipeline alone took 5.2–5.7 ms at every worker count, about a third of the forecast; the
  expansion plus the pipeline took 42.4 ms at 8 workers and did not rise towards 10, so the rise to
  142 ms was the oracles' copying. Link the floor results (pinned) and the companion's new row.
- **§5, after "Memory as a third station is the natural repair, still a hypothesis, …"**: one
  sentence: the zero-copy run found no sign of it between 8 and 10 workers, with memory traffic at
  13–17 GB/s, far under the machine's bandwidth. Link §6's note.
- **§7, after "The decoder's true floor is still unknown, because …"**: one sentence pointing to
  §6's note: the floor has since been measured, and what binds at 8 workers is the serial scan
  (179.1 ms of 194.6 ms). The plan table's first row ("What remains once the oracles' copying…") is
  left as written; the note answers it.

**Companion, `2026-11-26-from-bottlenecks-to-better-algorithms.md`:**
- **§3, after the paragraph ending "…are measured rather than inferred ([the floor run](…))."**
  A lead-in sentence, then a small table, forecast beside measurement:

  | Per decode (ms) | 1 worker | 6 workers | 8 workers | 10 workers |
  |---|---:|---:|---:|---:|
  | Residual A, forecast | ~273 | ~60–62 | ~51–55 | ~50–54 |
  | Residual A, measured | 303.9 | 54.0 | 42.4 | 40.5 |
  | Residual B, forecast | ~14 | ~12–15 | ~14–17 | ~15–19 |
  | Residual B, measured | 5.7 | 5.3 | 5.2 | 5.4 |

  Then two to three sentences: Residual B was the run's scored prediction and missed low (the rule
  in the registration, frozen before the data); Residual A at 1 worker includes about 55–65 ms of
  cold oracle-store reads (#952), so its excess over the forecast is the oracle's, not the decoder's;
  and the table's last forecast row (copying and lock contention) is not re-derived, because the
  matrix and the floor run are different runs and a difference across them would mix their noise.
  Intervals go in the table's caption line: 95% over the blocks, one run, one machine, one workload.
- **§2, after "That is a hypothesis." (the three-station paragraph):** one sentence, the same as the
  main post's §5 note, linking §3's table.

Notes stay plain prose (no ornament: an ornament belongs to a word, and a correction is not one).
Each opens with its date in the sentence ("Measured on 26 November, …") rather than a bold staged
lead-in, and keeps the forecast's own words intact above it.

## 4. Figures

None new. The table in the companion is the one display; a figure would restate four numbers
against four ranges. The companion's "Lower bound / oracle / predictor" axis figure (§5) stays as
published: it is labelled as the matrix's measurements and forecasts, and the floor post
(`floor-and-pooled-scan`) draws the new floor.

## 5. Consistency with the published and pending posts

- **`dropping-the-coordinator`** already states: plumbing 5.2–5.7 ms (2.7% of the 8-worker wall);
  `E_consume` 298 ms at 1 worker "of which ~55–65 ms is oracle-store traffic", 37 ms at 8; delivery
  29.1 ms at 1 worker and 34.0 ms at 8, 44–78 ms at 10; "the apparently flat 48 ms floor was the
  oracle store streaming from DRAM". The notes use these figures and these qualifications verbatim
  where they mention them.
- **Delivery.** If a note mentions delivery at all, it gives the real reader's copy-out (29.1 ms at 1
  worker, 34.0 at 8, from c71c4a56's traced medians via #966) and never the floor run's 47.3–49.3 ms
  difference, which #952 showed was the R oracle's 209.7 MiB store read cold.
- **Spelling.** The published posts' 259.4 ms "expanding the words into bytes" (warm construction,
  the matrix's arm E) is consistent with #952's estimate of ~230–245 ms in situ; the notes don't
  replace it with 298 ms.
- **The pending `floor-and-pooled-scan` draft** (stage review) still says "delivery's serial copy-out
  takes 48 ms flat" (draft lines 58, 88–89, 205–216, including a figure title). Not this card's to
  edit; flagged in the delta for whoever takes its `revise`.
- **A labelling discrepancy to state, not hide:** the floor README and report score the forecast as
  "Residual B at 1 worker, 14–19 ms", while the companion's table gives ~14 at 1 worker and the main
  post "12 to 19 ms at six to ten workers". The notes quote the posts' own per-count forecasts and
  give the measurement per count, which is right under either reading (a MISS at every count).

## 6. Goal questions (for the review stage's simulated reader)

1. *(summary)* What did the design analysis forecast for the pipeline's own cost, and what was
   measured? — forecast 12–19 ms; measured 5.2–5.7 ms at every worker count; scored MISS (low).
2. *(status)* Which numbers in §3's table are measured, and which argued? — the measured rows are from
   the floor run; the forecasts, and the copying-and-contention row, are argument.
3. *(justification)* Why does Residual A at 1 worker exceed its forecast when B falls short? — the
   arm reads the oracles' stores cold; ~55–65 ms of the spelling figure is that traffic.
4. *(motivation)* Why keep the forecast in the post at all? — it was registered before the data;
   prediction beside outcome is the house rule for reporting measurements.
5. *(transfer)* The companion forecasts 172–175 ms for a pooled scan. What would a reader check before
   trusting a published forecast of this kind? — whether an oracle run has since measured it, and
   whether that run's oracles carried their own cost (copying, cold stores).

**Held out:** (a) why the rise from 112 to 142 ms is no longer evidence for a memory station;
(b) what binds at 8 workers once the plumbing is negligible.

## 7. Fact ledger

| Figure | Value | Source |
|---|---|---|
| Residual B (pipeline alone, `SFCMRD`) | 5.7 [5.5, 5.8], 5.3, 5.2 [5.0, 5.5], 5.4 ms at 1, 6, 8, 10 | report.txt "The floor and its parts" |
| range across 1, 6–10 workers | 5.2–5.7 ms | same; README §1 |
| share of shipped wall at 8 | 2.7% of 194.6 ms | README §1, report k 8 `none` |
| Residual B forecast | ~14, ~12–15, ~14–17, ~15–19 ms | companion §3 table; main post §6 "12 to 19 ms" |
| scored verdict | MISS (smaller than predicted) | report "The prediction, scored"; README §1 |
| Residual A (`SECMRD`) | 303.9, 54.0, 42.4 [40.3, 44.4], 42.9 (9), 40.5 [37.6, 43.5] ms | report.txt |
| Residual A forecast | ~273, ~60–62, ~51–55, ~50–54 ms; unscored | companion §3; report line 72 |
| cold-store share of `E_consume` | ~55–65 ms of 298 ms | #952 §2; dropping-the-coordinator table |
| warm-C spelling (matrix arm E) | 259.4 ms at 1 worker | #952 §2; companion §3 |
| memory traffic, all-ideal arms | 13–17 GB/s | README §4 |
| scan alone at 8 (`FCMRD`) | 179.1 ms, 92% of 194.6 | README §2 |
| delivery copy-out, real | 29.1 ms at 1, 34.0 ms at 8, 43.7 at 10 (`none`) | #966 §1 table (c71c4a56 traced medians) |
| delivery "48 ms flat" | artifact of the R oracle's 209.7 MiB cold store | #952 §1; #966 §1 |

## 8. Links (pinned at draft time)

- Floor results: `ideal-parts/results/floor/README.md` at 311923ef (or the draft's master head).
- #952's reread at 2aaded3a; #966's `delivery/README.md`.
- The floor run's registration, `registration-floor.md` on #917, for "frozen before the data".
- Each post's note links to the other post's note, so the two corrections can be read together.

## 9. Checks for the draft stage

`check_prose.py` (no dek change, but the body's flagged share is reported), the strict site build,
`check_render.py` (no wrapped line starting with `#`: write "PR #952"), and `check_site.py`. No agent
or model names: the design analysis is "the design analysis", as the posts already call it.
