# Synthesis, thirteenth pass: first i9 verdict rejects seam-window, shallow-runner invariant established, and `#1034` 4T `pattern` mechanism isolated

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), 27 November 2026. Read with the
[twelfth pass](twelfth-pass.md) (#1109) and the [eleventh](eleventh-pass.md) (#1103). The primary sources
since #1109 (`578e5951`) are: the first real i9-10900K hardware verdict since `c71c4a56` — #1118
(`4b53096b`, `origin/evidence` `c0b2341c`, `scanner-seam-window/run-20261127T070349Z.tar.gz`,
**NOT ADOPTED**); the shallow-checkout i9 failure and repairs — #1122 (`7ad3e0fa`, A6b `INCOMPLETE`),
#1126 (`c3db2e39`), #1127 (`9e5db348`), #1128 (`ae3edfe2`), #1130 (`c152a4e8`); the `#1034` 4T `pattern`
investigation — #1108 (`d3bbe33f`) and #1111 (`30479058`, `scanner-per-block-fixed-cost` **REFUTED**);
merged feature/ops work — #1110 (`7a7a207a`, `forward-emission` behind `--features forward-anchor-lanes`),
#1104 (`1d1b23df`), #1125 (`mtf-fold-l1-batch-redesign` admitted); and open PRs #1117
(`scanner-a2-fast-build`), #1119 (`jcc-aligned-release-build`), #1124 (`run_requests` security hardening),
#1129 (`ready-cap-worker-gate`), and #1131 (`a6b-endpoint-successor-queue`). **(derived)** marks
arithmetic across sources; VM diagnostics are separated from i9 hardware measurements (C5).*

## 1. Real i9 verdict #1: `scanner-seam-window` (`#1031`) is `NOT ADOPTED`, moving `S'` up by `~10–13 ms`

Bundle `scanner-seam-window/run-20261127T070349Z.tar.gz` (`origin/evidence` `c0b2341c`, recorded in #1118)
tested `#1031` (`3ae44390`) against post-`#1034` base (`95f4637b`) on the physical i9-10900K (Wikidata x4,
838,800,784 B output, unpinned, 16 paired rounds, default codegen):

| threads | role | workers | wall base → head (median) | wall head/base [95% CI] | step | CPU head/base [95% CI] | peak RSS ratio |
|---:|---|---:|---:|---:|---|---:|---:|
| 6 | supporting | 6 | 0.244 → 0.244 s | 0.9969 [0.9923, 1.0015] | INCONCLUSIVE | 0.9907 [0.9835, 0.9980] | 0.9689 |
| 8 | supporting | 8 | 0.197 → 0.195 s | 1.0000 [0.9843, 1.0159] | INCONCLUSIVE | 1.0000 [0.9893, 1.0107] | 0.9885 |
| 9 | supporting | 9 | 0.196 → 0.190 s | 0.9663 [0.9472, 0.9859] | FASTER | 0.9815 [0.9606, 1.0029] | 0.9695 |
| **10** | **judged** | **8** | **0.197 → 0.195 s** | **0.9811 [0.9670, 0.9954]** | **FASTER** | **0.9891 [0.9798, 0.9986]** | **0.9846** |
| **12** | **judged** | **8** | **0.197 → 0.195 s** | **0.9898 [0.9726, 1.0073]** | **INCONCLUSIVE** | **0.9953 [0.9857, 1.0049]** | **0.9929** |

The frozen adoption rule required `FASTER` (95% CI upper bound `< 1.000`) at **both** judged counts (`10`
and `12` threads, both running 8 workers). `12` threads missed (`1.0073`), so the verdict is **`NOT
ADOPTED`** and PR `#1031` was closed without merge. Peak RSS was lower at all five counts (`0.969–0.993`).

The mechanism diagnostic (`in-maint.txt`, 3 timeline rounds at 8 workers / 32 slots) explains why the wall
effect was small: median scanner window maintenance dropped from **`10.689 ms` to `7.490 ms`** (**`-3.20
ms`, `-29.93%`**), far short of the `>= 80%` (`-11.2 ms`) reduction modeled from pre-`#1034` traces. Two
consequences change the system budget immediately:

- **Remove `#1031`'s unlanded `-13.35 ms` subtraction from `S'`.** Synthesis passes 5–12 subtracted
  `11.2 ms in_maint + ~2.15 ms coupling` from post-`#1034` `S'` assuming `#1031` would land. With `#1031`
  closed, post-`#1034` `S'` stays **`~10–13 ms` higher** **(derived)**: roughly `~141–148 ms` on default
  codegen and `~135–143 ms` aligned before `scanner-a2-fast-build` (instead of the ninth pass's
  `134–139 ms` *after* `A2`).
- **Scanner room at `k = 8` widens again.** That `~10–13 ms` restoration shifts `k = 8` back toward
  scanner-bound even on default-codegen Wikidata once worker improvements land, and deepens scanner
  binding on aligned Wikidata and off-Wikidata text. Both `scanner-a2-fast-build` (`~6.4–10.7 ms` derived)
  and slack-only `pooled-scan-hybrid` regain direct `k = 8` value rather than paying only at `k = 9–10`.
  Hardware `S'` must now be read directly from the upcoming `#1126`/`#1130` i9 bundle rather than summed
  from pre-`#1034` trace subtractions.

## 2. Shallow-runner i9 invariant (`#1122`–`#1130`) and dispatch hardening (`#1104`, `#1124`)

The first A6b i9 run (`#1122`, `7ad3e0fa`, `run-20261127T072936Z.tar.gz`) ended **`INCOMPLETE`** before
building because the workstation runner uses a `--depth 1` shallow checkout: fetching an unadvertised
commit into `FETCH_HEAD` and cloning the shallow runner via `git clone --local` dropped the fetched
commit.

Both active recipes are now repaired and merged:
- `#1126` (`c3db2e39`, live-verified on a real `--depth 1` checkout) fixed `bench/scanner_coalesced.py`,
  and `#1130` (`c152a4e8`) closed `scanner-coalesced-shallow-source` and resumed
  `scanner-coalesced-u8-i9-queue` on `#1126`.
- `#1127` (`9e5db348`) fixed `bench/a6b_endpoint.py` identically; `#1128` admitted
  `a6b-endpoint-successor-queue`, with PR `#1131` open to record the activated ready tag.
- Open PR `#1119` (`jcc-aligned-release-build`) already avoids history fetches entirely by verifying
  `HEAD:src`, `HEAD:Cargo.toml`, and `HEAD:Cargo.lock` tree hashes directly and compiling in place, backed
  by a real `git clone --depth 1` unit test.

**Standing operational invariant for all future i9 recipes** (`forward-emission-run`, `mtf-fold-run`,
`text-knee-i9`, `scanner-a2-fast-build`):
1. When comparing flags on identical source (`#1119`), verify `HEAD` tree hashes and build directly in
   place into separate `CARGO_TARGET_DIR`s.
2. When comparing two commits (`#1126`, `#1127`), initialise a private temporary git repo (`git init`),
   `git fetch --no-tags --depth <1|2> <origin_url> <full_sha>` directly into it, verify
   `FETCH_HEAD^{commit}` (and `FETCH_HEAD^` for parent comparisons), detach-checkout, and fetch
   `bench-data` directly from `origin`.
3. Include a unit test that reproduces a detached `--depth 1 --no-local` (`file://` URI) shallow runner.

On dispatch tooling, `#1104` (`1d1b23df`) merged ref-based run requests, but open PR `#1124`
(`fix/run-requests-security-gates`) identifies four concrete O2 authority/security defects in `#1104`:
non-blob `request.toml` DoS, fail-open non-passing check suites, fail-open skipped/neutral required
checks, and missing gates for parked leases, open flags, and draft PRs. `#1124` should merge before
relying on ref-based dispatch for any new queued run.

## 3. `#1034` 4T `pattern` CI flag: scanner refuted, mechanism isolated, policy disposition open

Two independent smooth-zone probes now explain the `+7.3% (+2.6%, +12.3%)` 4T `pattern` wall regression in
`#1034`'s exact-head CI run (`45947633032`) without editing or relaxing any policy rule:

- **Null-control bias (`#1108`, `d3bbe33f`):** across 18 non-scanner PR CI runs (180 paired rounds), 4T
  `pattern` is the only cell of 21 with a head-arm bias (`wall ×1.019`, run-level 95% CI `1.003–1.034`;
  `RSS ×1.017`), flagging *slower* in 3 of 18 runs (2 with identical decoder code). Net of that bias,
  `#1034` is `~+5.3%` (`1.6σ` above the null mean).
- **Scanner fixed-cost refuted + physical mechanism isolated (`#1111`, `30479058`,
  `scanner-per-block-fixed-cost` REFUTED):** on `pattern` (22/22 blocks pair-gated, `0.1%` scan share of
  1T decode), `#1034` cuts `A2` setup **−88.9%** (`3.93 → 0.44 µs/blk`, 9.0× faster, disjoint), total `S`
  **−77.1%** (`96.0 → 22.0 µs`, 4.36× faster, disjoint), and cold first-block latency `30.4 → 17.1 µs`.
  Cross-run CI telemetry (10 runs: `r(rss,wall) = +0.33..+0.84` in 10/10, `r(rss,cpu) = −0.19..−0.67` in
  9/10) and `occupancy` traces show the `+7.3%` wall / `+7.9%` RSS / `−2.2%` CPU signature is a
  pre-existing 6-on-4 thread oversubscription stalling strict-order `ready` draining and forcing fresh
  unrecycled `956 KiB` `Vec<u8>` allocations (`30/80 = 37.5%` high-RSS rate on 4 true null controls,
  including Python-only PR `#1033` run `45933661519` firing `+6.0% [+0.2%, +12.2%]` slower wall with zero
  Rust changes, vs `11/30 = 36.7%` across all 3 `#1034` CI runs).
- **Structural fix in flight (`#1129`, open):** `ready-cap-worker-gate` gates workers on
  `ready_bytes >= cap.bytes` and caps `free_out_bytes`, addressing both the `#1094` run-only RSS/fault
  blowup (`zeros_2g` RSS `0.488×`, `−572.5 MiB`; wall `−14.6%` at `t=4, slots=32`) and the `ready`
  backlog path.

Both `#1108` and `#1111` notes are appended to `frontier/flags/2026-11-27-scanner-coalesced-ci-acceptance.md`
(still `status: open`). Resolution stays with an independent agent different from `#1034`'s author,
combining exact-head CI run `45947633032`, `#1108`/`#1111`, and the resumed `#1126`/`#1130` i9 bundle
(`pattern` at `k = 2, 6`). `#1111` also confirms `scanner-a2-fast-build` will not move 4T `pattern`.

## 4. Pipeline state: worker pillars, 1T masking on `#1117`, and the single highest-leverage i9 run

- **`forward-emission` merged (`#1110`, `7a7a207a`):** paid `S = 16` bit-31 sampled-anchor 4-lane forward
  walk merged behind `--features forward-anchor-lanes`, leaving `forward-emission-run` open at P2. All
  three worker pillars are now on a concrete path to i9 validation: A6b (`#1053`, recipe `#1127` merged,
  successor queue `#1131` open), MTF-fold L1-batch redesign (`#1125`, admitted at P2 and held), and 4-lane
  forward emission (`#1110` merged, `forward-emission-run` open).
- **`scanner-a2-fast-build` (`#1117`, open; cold review + Miri clean):** paired 4-core CI `ab` reports
  `wikidata` 4T **−1.9% (−3.6%, −0.3%) faster** (`831 → 848 MB/s`), `text` 4T `−2.1% (−4.6%, +0.4%)`, and
  1T flat (`wikidata` `+0.5%`, `text` `−0.4%`) because `S` is only `~8–10%` of 1T `wikidata` wall (so a
  `−5..7%` `S'` cut is `~0.5%` of 1T wall, inside CI noise). Two 2T secondary cells flagged slower
  (`random` `+4.0% [+1.5%, +6.6%]`, `records` `+2.8% [+0.5%, +5.2%]`, both flat at 1T and 4T). Because
  `#1117`'s frozen stop test already requires a paired i9 `S'` (`A2 + A3`) check (`>= 3.0%` cut on
  `wikidata x4` plus `<= 2.0%` guard across `text/records/random/runs/onebyte`), it should be evaluated on
  that registered `S'` test (explicitly checking `random` and `records`) rather than judged on masked 1T
  CI wall. After `#1118`, `#1117` is the cleanest remaining `~6.4–10.7 ms` serial `S'` cut.
- **`jcc-aligned-release-build` (`#1119`, open, P1):** frozen C6 registration + shallow-safe recipe ready
  for independent C6 review and queueing. It remains the single highest-leverage i9 run because it settles
  both whether Comet Lake-S i9-10900K pays the JCC penalty and which station (`S` vs `W/8`) binds at
  `k = 8`.

## Release

Still none. Since `v0.5.14`, releasable work on `master` is `#1080`, `#1033`, `#1053` (feature-gated),
and `#1110` (feature-gated), plus `#1034`, whose open CI flag (`2026-11-27-scanner-coalesced-ci-acceptance.md`)
remains undispositioned. Release after an independent agent dispositions that flag (with `#1108`, `#1111`,
and/or the `#1126`/`#1130` i9 verdict in hand).

## Dead ends

All 35 rows in `frontier/dead-ends.tsv` stand (C2); nothing is revived. `#1118` records the real i9
`NOT ADOPTED` verdict for `scanner-seam-window` (`#1031`), and `#1111` refutes a `#1034` per-block scanner
fixed-cost regression.
