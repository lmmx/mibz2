# Review of the `floor-and-pooled-scan` draft: a simulated reader, and every number against its source

*26 November 2026. A cold review of [`floor-and-pooled-scan-draft.md`](floor-and-pooled-scan-draft.md) (at `9564f80`, #953), by an agent other than its drafter, under the board's `review` stage (`frontier/README.md`, "The blog board"). Each number was checked against the primary file it rests on (`ideal-parts/results/floor/report.txt`, `pooled-scan/results/report.txt`, `pooled-scan/registration.md`, `ideal-parts/results/full/README.md`), not against the outline's fact ledger. The pooled-scan bundle was pulled from `origin/evidence` (`pooled-scan/run-20261126T133034Z.tar.gz`, SHA-256 `97e397ad…8867`, matching) and its `runs.jsonl` traces re-read to check #952's figures independently.*

**Verdict: revise, then ready.** The structure is right and answers every question the outline set. Nearly all of its figures match their sources. But three findings are **factual errors that the newer results introduced** (F1–F3, all from #952, `docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md`), and one is **a caveat attached to the wrong row** (F4). Together they change two of the five takeaways and the "what comes next" list. Two more are internal inconsistencies in §5's queueing argument (F5, F6). The rest are small.

## Findings, in order of weight

**F1. Delivery's "flat 48 ms" is a cold-store artifact. Must fix.**
- *Where:* the opening ("delivery's serial copy-out takes 48 ms flat at every worker count"), takeaway 3, §2's table (`SFCMR` 52.3 ms) and "Third, delivery's copy-out is a flat 48 ms serial floor" paragraph, both figures (`taring` panel B's delivery bar; `reservoir`'s 47–49 ms "ledge" and its "spelling drops below delivery's 48 ms" callout), and §6's last bullet.
- *Why:* in the floor run, `SFCMR`'s reader copies from the `R` oracle's 209.7 MiB store, ten times the 20 MiB L3, so every byte comes from DRAM: 5.1 M LLC load misses and 16.9–17.5 GB/s at every count (`report.txt`, "Beside them"). The 17 GB/s is that cold stream's rate. On real blocks the reader's `copy_out` is far smaller. Re-read from the pooled-scan bundle's traces (n = 3 a cell, ledgers on, medians), I get exactly #952's numbers: **29.1 ms at 1 worker, 27.6 at 6, 30.6 at 7, 34.0 at 8, 43.2 at 9, 43.7 at 10** in `none`. And it is not flat: it **grows as the workers get faster**, to 40.0/39.5 ms at 8 and **67.4/77.9 ms at 10** in `PC`/`SC`, as up to 12–16 MiB of ready blocks (`READY_BYTES` = 12 MiB) crowd the L3. `frontier/surface.tsv`'s D row already reads this way.
- *Fix:* replace the 48 ms with the trace figures, say plainly that the floor run's 48 ms was the oracle store's DRAM rate and was corrected the same day, and give the real story, which is stronger: delivery is cheap while the workers are slow and gets dearer exactly as they speed up. Say that these are reader activity times from traced decodes, not wall differences, so they are not directly comparable to the floor table's columns. §6's remedy list should note that zero-copy block lending already exists (`StreamDecoder::read_block`, `src/parallel.rs:880`, `#[doc(hidden)]`), so the open question is the L3 working set and exposing `read_block`, not building lending. Both figures need their D series redrawn or removed.

**F2. Word spelling's 298 ms carries a cold-store share. Should qualify.**
- *Where:* the opening, takeaway 3, §2's second finding ("22% of the 1,342.7 ms decode"), and §6 ("emission's 298 ms word spelling").
- *Why:* `SECMRD` spells from two cold oracle stores (`C`'s 120.6 MiB and 425.1 MiB of recorded `u32` codes): at 1 worker it reads 504.3 MiB from DRAM against 151.1 MiB for the entire shipped decode (#952 §2). In the matrix's arm `E`, with real (warm) construction, `consume_ns` is already 41.5 ms lower. #952 estimates in-situ spelling at **~230–245 ms** (about 17–18% of the 1-worker wall). That is an estimate built across runs, not a measurement.
- *Fix:* keep 298.2 ms as what the arm measured (it is right as a number), and add a clause: an upper bound inflated by cold oracle reads, with in-situ spelling estimated at roughly 230–245 ms. The spelling census (#951) and the Rauzy output census (#958) have since measured its parts; one sentence in §6 pointing to them would date the post correctly.

**F3. K1's "structural, not unlucky" needs #952's cold-store bias beside it. Must fix.**
- *Where:* §5's last paragraph ("explains the K1 miss to the millisecond … capping full pooling near 50% capture … K1's narrow miss was structural, not unlucky") and §6's opening.
- *Why:* the zero-copy `C` oracle makes the walk (`A7`) read construction's store cold in every `C` arm. I re-derived #952's table from the traces and it reproduces exactly: `A7` rises by 220.4 ms (`PC − P`) and 196.2 ms (`SC − S`) at 8 workers, 293.9 and 257.7 ms at 9. `PC` and `SC` are worker-bound (`job_wait` 3.6 and 0.2 ms a worker at 8), so their walls pay that bloat. `C` alone is scanner-bound (`job_wait` 50.6 ms a worker at 8, 91.8 at 9), so its wall absorbs it. The bias narrows the `C − SC` gap that the capture divides by. Subtracting the traced bloat moves capture at 8 from 0.52 to about 0.73 (#952 §3). The `PC ≈ SC + s/k` arithmetic still holds (both arms carry the bloat). What doesn't hold is the conclusion that pooling is *capped* near 50%.
- *Fix:* keep the verdict (C2: it is frozen, and `P/none` is still 1.052 at 8 workers, so the prototype loses where mibz2 ships whatever K1 says). But say in §5 that the ~50% figure is partly an artifact of the cold-store baseline, give the ~0.73 as a diagnostic bound and not a re-grade, and add the methodological lesson to §6: a paired-oracle test must not set a worker-bound candidate against a scanner-bound cold-store baseline (the `warm-paired-oracle-k1` idea claim). That lesson is a better closing for the "keeping the verdict" section than the current "structural, not unlucky".

**F4. The cold-store caveat sits on construction's row; the data put it on emission's. Must fix.**
- *Where:* §2's table ("Construction (cold-store caveat)") and the paragraph after it ("Construction's isolated row (`SFMRD`) therefore overstates its standalone cost"), `taring` panel B's asterisk and footnote ("overstating C"), and held-out question 6 in the outline.
- *Why:* arm names list the *ideal* parts, so `SFMRD` has construction real and emission (with its spelling) free. The walk never runs there: `report.txt` gives `SFMRD`'s `A7` as 0.0 at every count. The cold reads happen in `SCMRD`, where emission is real and construction is the borrowed oracle: `A7` 98.9 ms a worker against 72.4 shipped at 8, 697.9 against 545.2 at 1. So it is **emission's** 98.1 ms (and 694.2 ms at 1 worker) that is overstated, by roughly the warm-cache benefit that real construction gives the walk. The floor's own README (§2 and §5) makes the same slip, which is probably where the draft took it from.
- *Fix:* move the caveat and the asterisk to the emission row, and rewrite the paragraph: the walk reads construction's store cold when construction is an oracle, so emission's isolated cost is overstated; the scan's row has no cross-stage cache effect. A separate small claim should correct the floor README's §2 label and §5 bullet. The outline's held-out question 6 then asks about the wrong row; the reviser should answer it as "emission's row, not construction's".

**F5. §5 puts 8 workers on both sides of the knee.**
- *Where:* "Below the knee ($w/k \ge s$), pooling must lose … slowed 8 workers by 5.2%. At 8 workers, adding $s/8$ onto the workers' 185 ms load gives about 207 ms", then "With mibz2's measured knee $w/s \approx 7.67$ … cannot break even until $k \approx 8.7$".
- *Why:* with a knee at 7.67, 8 workers is *above* it, where the scanner binds. The 185 ms is the `S` arm (ideal scan, real workers) at 8 (184.9 ms), which says the workers bind at 8 in *this* run: $w/8 \approx 185 > s \approx 178$, so $w/s \approx 8.3$ and the break-even $1 + w/s \approx 9.3$. The 7.67 comes from the queueing post's run of 25 November on other code. Both readings predict a loss at 8 and roughly break-even at 9, so the conclusion survives; the derivation contradicts itself.
- *Fix:* derive $w/s$ from this run's `S` arm (name it as the source of the 185 ms), cite the 7.67 as the earlier measurement, and say the two agree on the break-even to within a worker.

**F6. "Every measurement in Section 4 drops straight out of those two equations" overstates a first-order model.**
- *Why:* the model holds $w$ constant, but the run's total CPU in `none` rises from 1.714 s at 8 workers to 2.007 s at 10, and the `S` arm's wall implies $w \approx 1{,}480$ ms at 8 and $\approx 1{,}700$ ms at 10. Part of that is the RLE1/CRC stage under L3 pressure: its traced `A9` in `none` rises 89.5 → 104.0 → 133.0 ms at 6, 8 and 10 workers (reproduced from the bundle). With constant $w$, the model predicts a larger win at 10 than the 6.4% measured.
- *Fix:* "the model's first-order arithmetic accounts for the direction and the crossover", plus a sentence that $w$ itself grows with $k$. The `SC + s/k` checks (157.5 vs 158.1 ms; 160.5 vs 159.9 ms) are good and should stay.

**F7. The memory takeaway is right about DRAM bandwidth and silent about L3 capacity.**
- *Where:* takeaway 2, §1's second result, §6's "Closed targets".
- *Why:* the arithmetic is correct (Residual A flat at 42.4, 42.9, 40.5 ms; 15.2–16.6 GB/s), and dead-end row 8 stands for DRAM bandwidth. But F1 and F6 show the 20 MiB L3 binding from 8 to 10 workers: delivery and RLE1/CRC both slow down there, and `SECMD`'s memory traffic climbs 16.5 → 21.6 → 25.4 GB/s at 8, 9, 10 (`report.txt`), so its wall rises slightly (55.0 → 55.2 → 56.6 ms) while `SECMRD`'s stays flat (40.8, 42.4, 39.7 ms). The synthesis (#954) records this refinement.
- *Fix:* "DRAM bandwidth did not bind" in the takeaway, and one sentence saying the L3's capacity does, for delivery and RLE1/CRC, which are the targets the post should hand on.

**F8. RLE1/CRC's "~14 ms" is a wall difference and hides a worker cost.**
- *Where:* §2's table row and `taring` panel B.
- *Why:* 55.0 − 40.8 = 14.2 ms at 8 and 369.9 − 304.1 = 65.8 ms at 1 are correct as wall differences. But the traced `A9` (RLE1/CRC CPU summed over workers) is 104.0 ms at 8 in `none`, 7% of worker CPU, and its 4-pass `unrle_in_place` is the mechanism behind F6's growth (#952 §4; surface row R; `single-pass-rle1-crc`). §6 lists three worker-side targets and omits this one.
- *Fix:* label the row as a wall difference, and add RLE1/CRC to §6's list of worker-side targets.

**F9. Small exactness fixes.**
- *"it accounts for 87 to 93% of `SECMRD`":* it is 98% at 1 worker (298.2 / 303.9) and 87–90% from 6 to 10. Say "87–98%" or "nearly all of it at 1 worker, 87–90% from 6 workers up". (The floor README's "87–93%" has the same slip.)
- *"median wall time dropped by 6.4% from 197.6 ms to 187.2 ms":* those medians give 5.3%. The 6.4% is the per-block geometric mean `P/none` = 0.9357. Use one or the other with its own pair of numbers. The same applies in takeaway 5.
- *"worker idle share fell from 6.5–20.2% in `none` down to 1.2–2.5% in `P`":* that range is 8–10 workers; at 6 and 7 it is 1.4 → 0.9% and 1.9 → 1.2% (the §5 table has it right). Say "at 8 to 10 workers".
- *"a 16-minute floor diagnostic in PR #917":* #917 is the recipe and #921 the oracles; the results merged in #931. Cite #931 for the result, as the Sources should too.
- *#741, "buffering the full input and holding $4 \times$ threads blocks in flight inflated peak memory":* the registration lists these as the causes *named*, "hypotheses, unmeasured". Say "probably".
- *`reservoir`'s caption, "shared-cache contention slightly raises single-thread scan time from 169.7 ms … to 179.1 ms":* `FCMRD` has every worker part free, so there is little to contend with; the clock falls from 5.06 to 4.96 GHz between those cells (`report.txt`). Say "partly the lower all-core turbo; the rest unmeasured".
- *"the oracles copied about 4.9 GB":* the source is the companion post's §3, not a results file. Fine, but cite it there.
- *Sources:* repin from `daf02245` to a commit that includes #952 and #954 (e.g. `f88024a`), so that the D and R corrections' sources are the ones cited. `daf02245` predates #953 and #952 both.
- *Slug and date:* the card's `post` path carries `2026-11-26`, and the meta line says 26 November. If it publishes later, settle the dated slug before the first publication (no redirects).

## Checked and correct

- **The floor run:** 648 decodes (9 arms × 6 counts × 8 blocks, plus 216 traces) in 16 minutes; counts 1 and 6–10; scanner pinned to CPU 6, 216 of 216; removed parts' busy time ≤ 0.15%; no spelling calls in `F` arms.
- **The floor table:** every Residual A, Residual B, `E_consume` and `SFCMR − SFCMRD` value and interval at all six counts, as in `report.txt` (the latter correct as measured; F1 is about what it means).
- **Plumbing:** 5.2–5.7 ms; 5.2 mean and 5.3 median at 8; 2.7% of 194.6 ms; handovers 0.96 ms a thread at 1 and 0.15 at 8, 0.3–0.4% of Residual A; the 14–19 ms prediction scored MISS.
- **Memory:** `SFCMRD` 11.9–13.1 GB/s; `SECMRD` 15.2–16.6 GB/s at 8–10; shipped 15.1 GB/s at 10; `SFCMR` 5.1 M LLC load misses against 0.3 M.
- **Single-stage walls:** 194.6 / 1,342.7; 179.1 / 169.7; 98.1 / 694.2; 83.6 / 609.5; 52.3 / 54.7; 20.1 / 141.1; 5.3 / 5.8; `FCMRD` 177.0–179.1 ms at 6–10. `A7` 98.9 vs 72.4 and 697.9 vs 545.2. Trace ledger `E_consume` 299.7 ms at 1 and 39.8 ms a thread at 8.
- **The first matrix:** all-ideal 391, 94, 112, 142 ms; `S` alone −3.2% at 8 (0.968); `S` with `C` 195 → 125 ms.
- **The earlier attempts** (registration §2): 9.3 GB/s and 0.19%; 1.615, 1.780, 0.981; 0.935, 0.883, 1.11–1.13; RSS 1.28 and 1.99; #770's 1.605 at 8 and 54–61% from 8 to 20, lead 0.18–0.20 s, helpers 0.07–0.08 s each, base 0.17 s, workers ~40% idle.
- **The kill test:** 480 decodes (384 timed, 96 traced) on 12 blocks, the first 3 traced unpinned; exactness 10 of 10; 0 dropped, 0 fallbacks, 0 degraded. The K1/K2 table at all five counts, every ratio, interval, bar and capture; ceilings 0.8758 and 0.9058 against bars 0.8670 and 0.8636 (misses 0.009 and 0.042). K3 0.988–1.014. K4 4.82 ms [4.81, 5.00], 9.842 GB/s, CPU 4, 1,724 + 48 markers, 11 passes. `C`, `PC`, `SC` at 8 (183.2, 158.1, 135.3) and 9 (184.2, 159.9, 140.2); 10 workers capture 0.627. `P` at 8 and 9: 207.5 [206.5, 208.6] and 195.2 [192.3, 198.1] against the 172–175 ms forecast.
- **The redistribution:** the scan-CPU table (all five rows and intervals); idle shares; `pool_copy` 4.32–7.24 ms for 45.2 MiB; `pool_find` 4.97–7.35 ms; 190.6 MiB symbol queue. The $s/k$ checks: 22.2 → 157.5 vs 158.1 ms; 20.3 → 160.5 vs 159.9 ms; $s/k$ ≈ 46% of the 47.9 ms `C − SC` gap at 8.
- **#952's figures**, re-derived from the bundle's traces: `copy_out` in all six arms at every count, `A9` in all arms, `ΔA7` in all three pairs and `C`/`PC`/`SC` `job_wait` a worker, all matching its tables to 0.1 ms.
- **The site's rules:** no agent or model names in the draft or the figure script; equations in KaTeX; mechanism names throughout; no redirects. `check_prose.py` runs; its flagged sentences are long-number lists, as the drafter's delta says.

## Simulated reader: the outline's questions

| question | answered by | verdict |
|---|---|---|
| 1, why run a zero-copy floor after the matrix | §1, the two contaminants and what they hid | yes |
| 2, the two hidden costs and how they scale | takeaway 3 and §2 | yes as written, but the answer is now wrong for delivery (F1) and too large for spelling (F2). After revision the answer is better: one cost shrinks with workers, the other grows. |
| 3, why not a standalone 7% test, and why the paired test still failed | §3 (masked stage, 3.2%) and §§4–5 | yes, but the reader leaves believing the ~50% cap is structural; F3 changes that |
| 4, why slower at 6–8, even at 9, faster at 10 | §5 | mostly. A careful reader trips on the knee (F5): 8 is called below a knee said to be at 7.7. |
| 5 (transfer), $s = 100$, $w = 400$, $k = 4, 5, 6$ | §6's general rule $k^* = 1 + w/s$ and the two remedies | yes: break-even at 5, loss at 4 (125 vs 100 ms), win at 6 (83 vs 100 ms), and the slack-only hybrid or a smaller $w$ for both |
| 6 (held out), construction's row vs the scan's | §2's caveat paragraph | the reader answers as the draft says, but the draft has the wrong row (F4). After revision: emission's row is the inflated one. |
| 7 (held out), why stop rather than ship behind an 8-worker flag | §6's opening and gloss (frozen rule) | partly. The gloss gives the discipline; the decisive engineering fact is that `P/none` is 1.052 at 8 workers, so a flag at 8 would ship a 5% slowdown. §6 should say that in one sentence; it is already in §4. |

**Register:** clear, well linked, and cold-enterable. It is long (18 min) and number-dense: §4's and §5's tables plus the inline lists make 30-odd sentences that `check_prose.py` flags for load. The revise pass will lengthen §§2, 5 and 6 slightly; it can pay for that by moving the earlier-attempts list in §3 to two sentences and a link to the registration.

## What the revise stage should do

1. Apply F1–F4 (they change takeaways 2, 3 and 4, §§2, 5 and 6, and two figures), then F5–F9.
2. Regenerate `taring` and `reservoir` from `docs/scripts/figures_floor_and_pooled_scan.py` with the corrected delivery series and the asterisk on emission's bar.
3. Rerun `check_prose.py` and `link_audit.py`, and repin the Sources.
4. Separately (not in the post's PR): correct the floor README's §2 label, §3 "87–93%" and §5 bullet, which carry F4's and F9's slips.
