# Synthesis, second pass: the worker levers queue on one file, and then the scan binds

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), the second pass of 26 November. It follows [the first](README.md) (#954). The pass is analytical only: no new runs, no `src/` edits, and no frozen gate reopened (C2). Every number is cited to the primary file it comes from. A number derived here is marked **(derived)**. A projection from a VM census to the i9 is marked **(projected)**.*

## What was read

- **Merged since #954:**
  - #958, the `rauzy-output` census (`docs/journal/2026-11-24-algorithm-level/rauzy-census/results-2/README.md`);
  - #960, the `mtf-fold-census` and the frozen `mtf-fold-build` registration (`…/mtf-fold/README.md`, `…/mtf-fold/registration.md`);
  - #957, the triage of the coordinator-era PRs;
  - #956, the owner's decisions;
  - #959, the published post `docs/site/blog/2026-11-26-dropping-the-coordinator.md`, with its review, revise and ready deltas.
- **Open:**
  - #961, the cold review of `floor-and-pooled-scan` (`…/floor-and-pooled-scan-review.md` on `review/floor-and-pooled-scan`);
  - #962, the second `explore` (`docs/journal/2026-11-26-explore/scan-lookups-and-runts.md` on `explore/scan-lookups-and-runts`);
  - #963, the-frontier's card set to `published`.
- **Re-read against them:**
  - #952's cold-store note (`docs/journal/2026-11-26-explore/cold-store-and-rle1-llc.md`);
  - `src/parallel.rs`: `READY_BYTES` at :74, `SLOTS_PER_THREAD` at :47, `Cap` at :82–88 and `Shared::full` at :402–408;
  - every claim file in `frontier/claims/`;
  - the live holds on origin, where `hold/mtf-fold-build` was taken at 18:09 and `hold/delivery-copyout` at 17:44.

## 1. The 8-worker wall is co-limited, and the worker levers then hand it to the scan

**The balance today.** In the pooled-scan run, the `S` arm (ideal scan, real workers) takes **184.9 ms** at 8 workers. The scan alone, with every worker part free (`FCMRD`), takes **177–179 ms**. #961's F5 draws the consequence: at 8 the workers bind, but only just, with w/8 ≈ 185 ms against s ≈ 178 ms. The `mtf-fold` registration reaches the same figure another way. A5 + A6a are 47.5 ms a worker, which it gives as 25.7% of worker busy time, so a worker is busy **≈ 185 ms** at 8 **(derived)**. The two sides of the 8-worker wall are within 4% of each other.

**The worker levers now measured.** None of the three censuses says what the levers do together:

| lever | per worker at 8 | source |
|---|---|---|
| `mtf-fold-build` (A5 + A6a fold) | 6–13 ms behind `S` ideal (projected) | #960, registration §1 |
| `forward-emission`, single lane (`prod_forward_emit`) | 142–157 ms at 1 worker; ≈ 18–20 ms a worker at 8 if it scales (projected) | #958, results-2 lines 25, 127, 142 |
| `preshifted-inline-back` | ≈ 10.4 ms a worker | #958, idea 2c |

The pre-shifted table and forward emission are alternatives, not additions: forward emission removes the shift that the pre-shifted table makes cheaper. The two that stack are the fold and forward emission, together about **24–33 ms a worker**. That takes w/8 from ≈ 185 to **≈ 152–161 ms**, below the scan's 178 **(derived)**. After those two builds, the 8-worker wall is the scan's:
- **Further worker levers ship nothing at 8 workers** until the scan moves. They still count at 1 worker, and at 9–10 through the L3 (§2).
- **The scan's levers become fully captured.** #962 sizes `multi-symbol-huffman-table` at 10–12% of the scanner, ≈ 17–21 ms. It puts that at 5–7% of the 8-worker wall, using the migration rule from `leads/course-opportunities.md`, which assumes today's balance. Once the workers run below the scan, the whole 17–21 ms reaches the 8-worker wall, about **10–12%** of it **(derived)**.
- **The pooled scan's break-even moves.** Its kill rested on there being no worker slack: k\* = 1 + w/s ≈ 9.3 by #961's F5. With w ≈ 1,216–1,288 ms of worker time at 8, k\* ≈ **7.8–8.2** **(derived)**. Pooling would then roughly break even at 8 and win at 9–10 by the first-order model: (w + s)/10 ≈ 139–147 ms against 178. `pooled-scan-hybrid`, which uses slack only, needs exactly that slack, and the worker builds create it. The KILL of the pooled scan as built stands (C2). This is the condition under which its idea claim becomes worth a registration, not a re-grade.

**One more coupling.** All three worker builds own `src/open.rs`:
- `mtf-fold-build`: `src/runs.rs`, `src/open.rs`;
- `preshifted-inline-back`: `src/open.rs`, `src/open/endpoint.rs`;
- `forward-emission`: `src/open.rs`, `src/open/outq.rs`.

`frontier.py claim` refuses a striated claim whose paths overlap a live hold, so the three are serial whatever their priority. `mtf-fold-build` holds the file now. Its registration step 3 also pushes into `OutQ16`, which lives in `src/open/outq.rs`, a path that only `forward-emission` owns. The holder should add that path to its `owned_paths` if the prototype edits it, or keep the glue in `src/open.rs`.

**The order this implies.**
1. `mtf-fold-build`, which is held now.
2. `forward-emission`. It is worth about twice the pre-shifted table and supersedes it, and #958 has already measured its single-lane form, so the claim's "unmeasured" no longer holds.
3. `preshifted-inline-back`, only as an interim if forward emission's design stalls.
4. The scan side, in parallel because nothing in it touches `src/open.rs`: `multi-symbol-huffman-table` once #962 merges, and a `pooled-scan-hybrid` registration once the worker builds have measured walls.

**Caveat.** w grows with k (#961 F6: A9 runs 89.5 → 104.0 → 133.0 ms at 6, 8 and 10 workers). The per-worker savings are VM ratios projected to the i9, and the projection assumes 1-worker savings divide evenly across 8 workers. Every number in this section is a planning estimate for ordering claims. None is a result.

## 2. The L3 budget: which knob binds depends on the corpus's runts

`Shared::full` (`src/parallel.rs:402–408`) stops the scanner in either of two cases:
- **the window is full:** issued − taken ≥ `SLOTS_PER_THREAD` × k, with 4 slots a thread;
- **the cap is reached:** ready bytes ≥ `READY_BYTES` (12 MiB) *and* at least k + 2 places are in flight.

#962 finds that 198 of the corpus's 431 blocks are runts of a few dozen bytes. They interleave with the full blocks: in 186 of 197 cases a runt completes a 900,000-byte chunk. So 12 MiB of ready blocks is at least 14 full blocks, or about 27 places with their runts. Put beside the code, this means:
- **Up to about 6 workers, the slot window binds first.** It is 4k ≤ 24 places, under the cap's ~27, so the ready set stays below 12 MiB.
- **From 7 workers up, the byte cap binds.** The #952/#954 L3 trap at 9–10 workers (ready_bytes peaking at 12.0–16.3 MiB in a 20 MiB L3) belongs to `READY_BYTES` alone **(derived)**.
- **`SLOTS_PER_THREAD` is the stronger knob on this corpus.** At 3 a thread, the window binds through about 8 workers (3 × 9 = 27); at 2, through about 13 **(derived)**. `delivery-l3-working-set` grids 2, 3 and 4 slots a thread, so its 2- and 3-slot rows test the window, not the cap.
- **The cap's floor puts a lower limit on the cap.** At 10 workers the floor holds 12 places in flight, about 6 full blocks or ~5.4 MiB on this corpus **(derived)**. A `READY_BYTES` of 2 or 4 MiB in the probe's grid is overridden by the floor there. Below about 5–6 MiB the grid measures the floor, not the cap.
- **The fold also frees L3.** `mtf-fold` removes the 119–203 KiB `runs` buffer a block (p50–p90). At 10 workers that is about 1.2–2.0 MiB of the shared L3 **(derived)**. None of the other worker levers shrinks the working set; they only raise ready_bytes by running faster.

These go into `delivery-l3-working-set`'s question as a note. Its frozen stop test is untouched.

## 3. Where the cold-store correction has and hasn't reached

#952 showed two things: the floor run's 48 ms delivery was the oracle store's DRAM rate, and spelling's 298 ms carries ~55–65 ms of cold oracle reads. #961 re-derived both from the pooled-scan bundle, independently and to 0.1 ms, which turns them from one agent's reading into reproduced figures. Its F4 adds a third correction: the cold-walk caveat belongs on **emission's** isolated row (`SCMRD`, `A7` 98.9 ms against 72.4 shipped at 8), not construction's (`SFMRD`, which has `A7` = 0).

| artifact | state after this pass |
|---|---|
| `surface.tsv` D | already corrected (#954) |
| `surface.tsv` E-spelling `room_1w` | still read "298 ms (22%)"; **now** "298 ms oracle upper bound; ~230–245 ms in situ (est.)" |
| `delivery-copyout` title "48 ms" | **retitled** here; question kept for the holder |
| `floor-and-pooled-scan` draft and card question | F1–F4 in #961; the card's question still quotes 48 ms and 298 ms and is the revise stage's to fix |
| `ideal-parts/results/floor/README.md` §2, §5 | the F4 row mislabel and "87–93%"; a small docs fix, not done here |
| `rauzy-output` 3% gate (39.18 ms of 1,306 ms) | **unaffected**: anchored to the real decode's phase times (`character-stage1`), not an oracle arm |
| the published post (#959) | corrected before it shipped (29.1 / 34.0 / 44–78 ms) |

**The lesson both censuses teach.** #958 and #962 each halved an earlier conjecture for the same reason: the shipped code had already taken the first half.
- `LIMIT = 16` was already the C + E optimum, so #951's "lower `LIMIT`" lost.
- Pair-symbol entries already ship, so record 129's "13–17%" for multi-symbol decoding became 5–7%.

Any size forecast from a latency argument should first check what ships.

## 4. Claim hygiene

- **`bidi-preshifted-inline` (idea) → dropped, superseded.** Its question was whether the pre-shifted table's gain survives inside the full bidirectional walk. #958's idea 2c measured exactly that, with the pointer chase live: full A7 −15.0 to −16.9%, `C+E` 0.928–0.935×, +5.3–5.9% of the 1-worker wall, and 100% byte-identical output. Its own bar was 2.5%. The one clause #958 didn't cover, no regression on `runs` and `text`, already sits in `preshifted-inline-back`'s stop test.
- **`forward-emission` (idea): premise annotated.** "The design analysis forecasts 280–340 ms (unmeasured)" now has a measured lower point: single-lane `prod_forward_emit`, 142–157 ms at 1 worker. Multi-lane MLP is still unmeasured.
- **`delivery-copyout` (open, held): title de-staled.** The question is the holder's. #952 and #954 already say how to reframe it: L3 eviction plus exposing `read_block`.
- **Status lag:**
  - `open-prs-triage` merged in #957 with outcome *partial* and still reads `review`.
  - `the-frontier` is fixed by #963.
  - `brief` lists both under "finish before starting".
  - The triage's 56 closures are not done; they belong to its holder or a new ops claim, not to a synthesis.
- **Not changed, and why:**
  - `surface.tsv` S: #962 rewrites the row and is open; editing it here would only conflict.
  - The admission of ideas: there are 8 open claims against a limit of 12, so `single-pass-rle1-crc` and `delivery-l3-working-set` could be admitted. That belongs to an `admit`.

## What none of the deltas says alone

The next two worker builds should take the 8-worker wall from the workers and give it to the scan, by about 17–26 ms. After that:
- the scan's lever, killed or unclaimed today, becomes the one that pays at 8;
- the pooled scan's slack premise turns true at about 8 workers;
- the 9–10-worker gains then hinge on an L3 budget whose binding knob, on this corpus, is decided by the dump's runt blocks.

The work worth doing therefore runs on two tracks with no shared file. The worker side is serial on `src/open.rs`: fold, then forward emission. The scan side can start now: multi-symbol tables, then a slack-only pooling registration timed to the worker builds' walls.
