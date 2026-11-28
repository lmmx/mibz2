claim: forward-emission-bounded-slab-rescue
by: antigravity
outcome: dropped
evidence: origin/evidence 22f2d7d6 (forward-adopt-rescue/run-20261128T164925Z.tar.gz, SHA-256 01c248a06a85250bf44cce4d71ee368624d2d691d7b142cc8c71968e1556b6dd, commit 575fa98a, rustc 1.99.1); bench/forward_adopt_rescue.py score; docs/notebook/2026-11-28-forward-adopt-rescue-i9-verdict.md; frontier/dead-ends.tsv; #1382 (f47a2fda) Clause 1 feature test at e125ef95

**What was asked:** read back, SHA-256 verify, and offline-score the physical i9-10900K C6 adoption bundle `origin/evidence 22f2d7d6` for `#1312` bounded-slab `forward-anchor-lanes` (`<= 80 KiB/worker`, `#![forbid(unsafe_code)]`), record the unrelaxed C2 verdict, and dequeue `bench-forward-adopt-rescue`.

**What I found (unrelaxed C2 verdict: `DO NOT ADOPT` — 10/11 clauses PASS, 1 sub-clause FAIL):**
- **Exactness & Clause 1:** `exactness_ok: PASS` (`34/34` untimed `AB_SHA256=1` checks across 7 corpora + `544/544` timed repeats vs C `libbzip2`), `distinct_binaries: PASS`, CI Miri `0` UB + `#1382` `cargo test --release --locked --features forward-anchor-lanes` (`218/218` lib + all suites PASS at `e125ef95`).
- **Wikidata wall & CPU (all 6 speed clauses PASS by wide margins):** `wikidata/1` `0.8773 [0.8751, 0.8795]` (`-12.27%`, insns `18.34B -> 15.36B`), `wikidata/6` `0.8732 [0.8703, 0.8762]` (`-12.68%`), `wikidata/8` `0.9187 [0.9039, 0.9337]` (`-8.13%` wall, paired med `0.9153`, CPU `0.8999`, `-10.01%`), `wikidata/10` `0.9175 [0.8977, 0.9378]` (`-8.25%`), `wikidata/12` `0.9132 [0.8999, 0.9268]` (`-8.68%`).
- **Guards (12/12 wall & RSS PASS):** `text/1 0.9839x`, `text/8 1.0007x`, `records/1 0.9939x`, `records/8 0.9934x`, `random/1 0.9815x`, `random/8 0.9845x (-1506 KiB)`, `runs/1 1.0000x`, `runs/8 0.9858x`, `pattern/1 0.9989x`, `pattern/8 0.9857x`, `onebyte/1 1.0009x`, `onebyte/8 1.0081x`; all 1T guard `delta_rss` in `-80..+80 KiB`.
- **Sole failing sub-clause (`wikidata_rss_ok`):** every RSS GM bound PASSED (`wikidata/1 1.0143x <= 1.020x`; `wikidata/6, 8, 10, 12` all **lower** than base at `0.9852-0.9940x`, `-972 / -722 / -132 / -462 KiB`), but `wikidata/1` `delta_median_kib = +868.0 KiB > +512.0 KiB` (`33,116 -> 33,984 KiB`, `t_ci95 [0.9705, 1.0601]`). Per-round `wikidata_1t.jsonl` shows identical 3-mode pipeline-buffer quantization (`32.9 / 34.0 / 37.2 MiB`, `~900 KiB` ready-block step): `base` lands `11/2/3` and faster `cand` lands `6/6/4`, stepping the 16-sample median across one `900 KiB` pipeline quantum (`base` min `32,860` vs `cand` min `32,880`, `+20 KiB`).

**What changed:**
- Recorded complete 17-cell verdict & root cause in `docs/notebook/2026-11-28-forward-adopt-rescue-i9-verdict.md`.
- Removed `bench-forward-adopt-rescue` from `bench/workstation-queue.txt` (`pytest bench/test_workstation_queue.py` PASS).
- Closed `frontier/claims/forward-emission-bounded-slab-rescue.toml` (`status = "dropped"`, frozen `stop_test` untouched) and appended a scoped `DO NOT ADOPT` row to `frontier/dead-ends.tsv`.

**What's next:** do not promote `#1312` under this closed C6 claim; decide cleanly between composing `#1312` with `u8 SymBuf` / `#1328` (`-1.1 MiB` 1W RSS) or a fresh follow-on C6 registration with a quantum-aware 1W RSS check.
