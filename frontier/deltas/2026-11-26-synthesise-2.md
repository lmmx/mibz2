claim: synthesise
by: claude-code
outcome: done
evidence: docs/journal/2026-11-26-synthesis/second-pass.md; #958 (rauzy-census/results-2), #960 (mtf-fold README + registration), #957, #956, #959; open #961 (floor-and-pooled-scan review), #962 (scan-lookups-and-runts), #963; #952; src/parallel.rs:47,74,82-88,402-408; live holds hold/mtf-fold-build, hold/delivery-copyout

**What was asked:** Connect the deltas since #954 into something none says alone. Check open claims' premises for staleness. Update the surface only where the evidence changes it. No hold taken.
**What I found:**
- **The 8-worker wall is co-limited:** the workers take ≈185 ms (the `S` arm, 184.9 ms; the mtf-fold registration's 47.5 ms a worker = 25.7%) against the scan's 177–179 ms. The fold (6–13 ms a worker) plus single-lane forward emission (#958's `prod_forward_emit`, 142–157 ms at 1 worker, ≈18–20 ms a worker at 8, projected) take the workers to ≈152–161 ms. The 8-worker wall then becomes the scan's, by 17–26 ms (derived). After that:
  - more worker levers ship nothing at 8;
  - #962's multi-symbol table (17–21 ms) is fully captured, ≈10–12% of the 8-worker wall rather than its 5–7%;
  - the pooled scan's break-even moves from k ≈ 9.3 to ≈ 7.8–8.2, so `pooled-scan-hybrid`'s slack premise turns true. The KILL stands (C2).
- **The three worker builds are serial:** `mtf-fold-build` (held), `preshifted-inline-back` and `forward-emission` all own `src/open.rs`. The fold's registration also pushes into `OutQ16` (`src/open/outq.rs`), which only `forward-emission` owns. The order: fold, then forward emission (twice the pre-shifted gain, and it supersedes it). The scan-side probes can run in parallel.
- **The L3 knobs:** #962's runts (12 MiB ≈ 27 places), put beside `Shared::full`, give the following. These are derived, not measured.
  - The 4k slot window binds up to about 6 workers, and `READY_BYTES` above that.
  - At 3 slots a thread the window binds through about 8 workers.
  - The cap's floor overrides any cap under about 5–6 MiB at 10 workers.
  - So part of `delivery-l3-working-set`'s grid tests the window or the floor, not the cap.
  - The fold also frees ≈1.2–2.0 MiB of L3 at 10 workers.
- **The cold-store correction is now reproduced:** #961 re-derived #952 to 0.1 ms and moved the cold-walk caveat to emission's row (`SCMRD`). The `rauzy-output` 3% gate is unaffected, being anchored to the real decode.
- **A recurring pattern:** #958 and #962 each halved a forecast because the shipped code had already taken the first half (`LIMIT = 16`; pair-symbol entries).
**What changed:**
- Added `docs/journal/2026-11-26-synthesis/second-pass.md`.
- `surface.tsv`:
  - E-spelling `room_1w`: 298 ms is an oracle upper bound, ~230–245 ms in situ;
  - C-A6c and E-walk `state`: forward emission's single-lane gain is now measured, and the claim is queued behind the fold.
- Claims:
  - `bidi-preshifted-inline` → dropped (superseded: #958 idea 2c measured it end to end, +5.3–5.9% against its 2.5% bar);
  - `delivery-copyout` retitled away from "48 ms", with premises added (question left to the holder);
  - `forward-emission`'s why and premises: the measured single-lane point;
  - `delivery-l3-working-set` question: the binding-knob note (stop test untouched).
**What's next:**
- `mtf-fold-build`'s holder: add `src/open/outq.rs` to its `owned_paths` if the prototype edits it.
- Claim `forward-emission` next on `src/open.rs`. Admit `multi-symbol-huffman-table` when #962 merges.
- Write a `pooled-scan-hybrid` registration once the fold has i9 walls.
- An `admit` could take `single-pass-rle1-crc` and `delivery-l3-working-set` (8 of 12 open).
- `open-prs-triage` merged (#957) but still reads `review`, and its 56 closures are undone.
- The floor README needs F4/F9's small fix.
**Caveats:** Analysis only. The per-worker savings are VM ratios projected to the i9, assuming 1-worker gains divide evenly across 8, and w grows with k (#961 F6). §1–§2's figures order claims; they are not results and reopen no gate.
