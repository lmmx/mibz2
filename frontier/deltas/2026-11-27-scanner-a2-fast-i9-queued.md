claim: scanner-a2-fast-i9-run
by: antigravity
outcome: blocked
evidence: #1237 (independent C6 review PASS on d7544ed1, merged at 40a16165); tag ready/bench-scanner-a2-fast at 40a16165 (run-signal.yml run 46080356912); bench/workstation-queue.txt entry

## Independent C6 review and merge (#1237)
Reviewed #1237 (`d7544ed1`) independently as a non-producer participant before merging at `40a16165`:
- Confirmed `origin/build/scanner-a2-fast-head` (`689e55f7e4307167fa3fd0c744e16a3f85528490`) has second parent `2af67a675b39d9523e96686c98c72c984f799868`, zero diff against #1117 (`eb1ec40f`) on `src/block.rs` and `src/huff.rs`, and both arms match `PHASES_BLOB = 403cfbbdd160559826793d173b673dad5bcac682`.
- Confirmed shallow-clone safety (`--depth=2` fetch into a private temporary repo), isolated per-arm `CARGO_TARGET_DIR`, baseline x86-64 codegen checks (`shlx`/`shrx`, no `vzeroupper`), SHA-256 parity against `libbz2`, frozen gate-2 fidelity (`S' <= 0.970`, guard `<= 1.020`, RSS `<= 1.02`, wall `<= 1.020`), and `6/6` local test passes in `bench/test_scanner_a2_fast.py`.
- Answering the pointer in #1239 (`frontier/deltas/2026-11-27-scanner-a2-fast-i9-review-routing.md`): `examples/scanner_coalesced_run.rs` calls `BzReader::parallel_workers(data, workers, workers * SLOTS_PER_THREAD, Kernels::detect(), true)` directly, bypassing `trim_cell` in `decompress_parallel`, so the informative `real` Wikidata cells at `k = 1, 8, 9, 10` genuinely execute `1, 8, 9, 10` workers. The frozen C2 gate remains `S'` alone.

## State
`#1237` merged at `40a16165`, integration hold released, `ready/bench-scanner-a2-fast` confirmed at `40a1616520d7fddbc1fb3f156b9cb48fd9af5fee`, and `bench-scanner-a2-fast` appended to `bench/workstation-queue.txt` under standing workstation authorisation.

## Blocked on
The i9 bundle only (`just workstation-queue`). Nothing else to decide.

## Next (whoever reads the bundle)
Read the archived bundle from `origin/evidence` at the commit #145 names and verify its SHA-256 sidecars. Record the frozen gate-2 verdict in a delta:
- `PASS`: integrate #1117 (`scanner-a2-fast-build`), update surface row `S`, and mark this claim `done`.
- `STOP`: close #1117 unmerged, add a scoped row to `frontier/dead-ends.tsv` naming the bundle digest, and mark this claim `done`.
- `VOID` / `INCOMPLETE`: inspect the invalidation reason before deciding whether a corrected re-pin is justified.
