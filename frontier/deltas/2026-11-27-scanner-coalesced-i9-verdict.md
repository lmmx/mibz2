claim: scanner-coalesced-u8-i9-queue
by: antigravity
outcome: done
evidence: origin/evidence 9de7b6e0e3be71679ef6894680ccfa293ea0b03e (scanner-coalesced/run-20261127T082548Z.tar.gz, sha256 0a5e398f7bd4465b0f634c5736badcdeb04a48ae420aaf1a91c9294ab4422550); recipe 96e574a7b3b1; head 5ed20c1d51af vs base 5c7fe5d24432

Read and verified the completed physical i9-10900K (Comet Lake-S, stepping 5, 10C/20T, rustc 1.99.1) evidence bundle for `#1034` (`scanner-coalesced-u8-decode`). Archive SHA-256 matches `0a5e398f7bd4465b...` bit-for-bit; all parity checks passed (`0` failures), `bmi2.asm` verified `22 shlx`, `8 shrx`, `0 vzeroupper`, and all 38 cells executed cleanly with zero VOID conditions.

**Frozen Verdict (registration §3.2 verbatim, C2 preserved):** **STOP** (`10-worker Wikidata misses the 0.970 upper bound or non-overlapping ranges`).

Key measured ratios (`head / base`, 10 paired rounds, 95% CI, `[min, max]`):
- **Serial scan `phases/wikidata/1`:** `0.8950` (`-10.50%`, CI `[0.8923, 0.8978]`, disjoint `[159.7, 160.9]` -> `[143.1, 144.1]` ms) — **PASS** (`<= 0.920`).
- **Construction-free oracle:** `C/wikidata/8` `0.9220 [0.9132, 0.9308]` (`-7.80%`, disjoint, RSS `1.0010`); `C/wikidata/10` `0.9424 [0.9336, 0.9512]` (`-5.76%`, disjoint, RSS `1.0037`).
- **Real `wikidata x4`:** `1T` `0.9880 [0.9703, 1.0061]`, `2T` `0.9990`, `6T` `0.9996`, **`8T` `0.9807 [0.9626, 0.9991]` (`-1.93%` FASTER)**, **`10T` `0.9598 [0.9457, 0.9742]` (`-4.02%` FASTER)** — misses the frozen `<= 0.970` upper CI bound by `0.0042` and ranges overlap (`[0.1889, 0.2011]` vs `[0.1967, 0.2059]`). Peak RSS `<= 1.0122x` on Wikidata (`<= 1.0416x` across all 35 real cells).
- **Guard corpora (30/30 cells, 0 regressions):** `random` 6T `0.9677` / 8T `0.9754` FASTER, `pattern` 6T `0.9724` FASTER (`1/2/8/10T` neutral `0.972–1.016`, clearing the `#1084` CI `pattern` regression concern on real hardware), `onebyte` 8T `0.9714` FASTER; `text`, `runs`, `records` neutral.

Scoped dead-end recorded in `frontier/dead-ends.tsv` and `frontier/surface.tsv` row `S` updated. Keep-vs-revert of already-merged `#1034` stays with Louis (per `release-0-5-15`); open CI flag `2026-11-27-scanner-coalesced-ci-acceptance.md` remains untouched for independent resolution.
