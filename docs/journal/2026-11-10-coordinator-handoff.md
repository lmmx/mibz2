# Coordinator hand-off, 2026-11-10 (before a quota pause)

Agents may be out of flight for several days. This is the state at 17:55Z; GitHub
(#145, the PRs, the `evidence` branch) is still the source of truth. Re-read it on resuming.

## Shipped today
- **0.2.9 released** (#359, team paths on run records): crates.io, PyPI and docs.rs; CHANGELOG #360 (85c0e29).
  i9 release scaling, 0.2.8 ÷ 0.2.9 wall on Wikidata: 1t 0.991 (0.983–0.998), 8–20t 1.10–1.15.
  Archived in `bench/release-scaling/2026-11-10-…-0.2.9/`; the History hero shows it (bench.js fix 6782c0f).
- **wikidata-pq on mibz2 0.2.9** (wikidata-pq#224, 45697167).
- **#351 merged** (d022139): MONODROMY's Rauzy–Veech census (`examples/rauzy.rs`) and journal.
- **Blog post** `docs/site/blog/2026-11-10-rauzy-veech-reconstruction.md` published (formal frame, Lyndon figures, steppers).

## In flight
| Item | Where | Next step |
|---|---|---|
| **#361** one-core walk inlining (Architect's 12-line patch, `src/bwt.rs`) | PR #361, head d33787e1 | **CI bench in, not merged.** Wikidata at 1t on CI (AMD EPYC) is level: −0.5% (−2.0, +1.0). But `text` at 2t reads +5.5% (+0.3, +11.0) slower, so the rule (nothing slower beyond noise) is not met. The 0.9% loss was measured on the i9, so the i9 decides: a 0.2.9 vs #361 one-thread A/B there. The Architect's note §2 (`codegen/allcmp.py.txt` with CI's toolchain) says what to check next, and not to move code about to chase a number. |
| **i9: `bench-team-threshold`** | ready tag at dfe04059; queue entry | Rule (LOG on #145): move the team threshold to T=4 or 5 if a forced team beats the plan beyond noise there, else keep 6. |
| **i9: `bench-cells-vs-prod`** | ready tag at c79ff7d3; queue entry; rule LOG 6774716029 | Pool `c_whole/ns_rec` over `takes` blocks, 3 repeats. All < 1 → MONODROMY's production slice goes to top priority. Kernel stage only. |
| **MONODROMY production slice** | MONODROMY session; bibliography docs PR first, then slice one | Cells kernel in `src/` behind an off-by-default (ideally runtime) switch; admission r/n → r2−1 floor → cells budget; refused blocks byte-identical; B5 evidence; CI bench. Default flip in a second PR. Merge its docs PR on green straight away (one branch). |
| **wikidata-pq `BZ2_THREADS_CAP = 4`** (`src/wikidata/dump.py:170`) | not started | i9 A/B cap 4 vs 8/10 so 0.2.9's team gain reaches the release split. |
| **Backlog** | ~40 open PRs | Closing #335, #337, #339 was denied by the permission classifier; Louis to allow or close. |

## The other agents' hand-offs
- **Architect / Pathfinder:** `docs/journal/2026-11-10-pathfinder-handoff.md` on #313 at f1fe26f4 (LOG 6774893006). Open, in order: #361, blog review round 2 (#357), the cells kernel behind `invert_runs`.
- **MONODROMY:** PR #362 (docs only; merge on green once its head has been still for ten minutes). It holds the bibliography (stored unfinished, 15 of 23 works) and `docs/journal/monodromy/2026-11-10-cells-production/HANDOFF.md`. The production PR is **not started in code**. Design facts so far:
  - the narrowest try-and-fall-back point is inside `Segmented::invert_runs` (src/bwt.rs:152–165), before `runs::fill`;
  - the run records are private to `runs`;
  - a refusal must not surface as `invert_runs == false`;
  - routes are type parameters, so "off by default" is a second `InverseBwt` type, not a flag.

  Either the Pathfinder or MONODROMY can pick it up from that note.

## Workstation
Louis runs `just workstation-queue --wait 12`. Both tags above are set; bundles land on the `evidence` branch (check SHA-256).
`ready/bench-team-runs` (ceb0b30b) is a stale tag whose queue entry was removed (4035b83); the runner ignores it.

## Local work archived
Every local branch with unpushed commits was pushed to `archive/coord-2026-11-10/<branch>`:
mibz2 43, polars-genson 19, pq-glue 8 (mostly 2026-11-05 experiment arms already decided). Nothing else was local-only.
