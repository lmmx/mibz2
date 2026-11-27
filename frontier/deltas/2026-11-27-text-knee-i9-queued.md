claim: text-knee-i9
by: claude-code
outcome: partial
evidence: #1139 (head e8a7cb85, merged af7ff482); ready/bench-text-knee at af7ff482; bench/workstation-queue.txt

**What was asked:** finish integration of the frozen text-panel knee recipe (#1139), then queue it on the i9.

**What I found:** the recipe is sound as frozen. Reviewed independently of its author:
- `ab run` and `station_balance` interfaces match `examples/ab.rs` (flags `--arms/--threads/--rounds/--prespin-ms/--settle/--max-busy/--sysfs-state`) and `examples/station_balance.rs` (11 tab-separated fields).
- A smoke `ab run` with the recipe's exact flags writes one row carrying `out`, `hash`, `rss_kb`, `wall`, `err`, `termination`, `threads`, all fields `validate_wall_row` reads; `ab`'s own `round`/`corpus` are overridden by the recipe's.
- 11/11 `bench/test_text_knee.py` pass; all Actions checks and pre-commit.ci green at the head.
- Scoring follows the claim's stop_test verbatim (median of 9, 2% knee, panel median of three, CONFIRMED/REFUTED/INCONCLUSIVE/VOID).
- Minor, not blocking: station_balance pins to CPU 0, which takes most interrupts on the workstation; S and W share that CPU, so k* = W/S is the robust figure, not absolute ms.

**What changed:** #1139 merged (head-sha checked before and after). `ready/bench-text-knee` dispatched via run-signal.yml at af7ff482 under the standing workstation authorisation (Louis, 9 November 2026); queue entry added. Nothing was queued, tagged or archived for this recipe before (queue history and `evidence` branch checked). Claim marked `blocked` on the workstation run, so no one re-queues it.

**What's next:** when the bundle lands on `evidence` (posted on #145), read `outcome.txt` and `summary.json`, check its SHA-256, record the verdict: CONFIRMED adds a text row to `frontier/DECODER.md`; REFUTED adds a `dead-ends.tsv` row for "VM station ratio as an i9 knee predictor"; INCONCLUSIVE reports i9 k*. Then remove the queue entry and mark the claim done.

**Caveats:** no data yet. A `bz2.compress` byte mismatch on the workstation's libbzip2 would VOID the run at build time (cheap, before any timing).
