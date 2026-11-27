claim: jcc-aligned-release-build
by: antigravity
outcome: done
evidence: origin/evidence 77236472 (jcc-aligned-release/run-20261127T135104Z.tar.gz, sha256 608502c3520fa9e5f78ce4b45aa0f180c65638897b5c0fda575a8802973b6fb8); #1119 (dd935418); #1165

## Verdict: STOP (physical i9-10900K)
Valid physical i9-10900K run (Intel Core i9-10900K, family 6 model 165 **Comet Lake-S stepping 5**, 10C/20T, pinned `rustc 1.99.1`, `source 59cac2c5`, `recipe 256566b7`, 12 paired `ab run` rounds + 5×5 pinned `station_balance` rounds). Zero VOID conditions: bundle SHA-256 verified bit-for-bit, 28/28 `AB_SHA256=1` parity checks match `libbz2` with 0 mismatches, and `codegen_audit.json` confirms `ab` `.text` `1,654,406 -> 1,703,078 B` (`+2.94%`, `2,254 -> 0` unfused 32B crossings) and `station_balance` `.text` `411,869 -> 426,744 B` (`354 -> 0` crossings).

## Paired `ab run` results (`aligned / default`)
- **`wikidata/1`:** `1.0065 [0.9753, 1.0387]` (`1.3802 -> 1.4183 s`, RSS `0.9999x`) — **fails `<= 0.970`**.
- **`wikidata/4` (diagnostic):** `1.0045 [0.9967, 1.0125]` (`0.3592 -> 0.3602 s`, RSS `1.0022x`) — tight CI confirms null/slight loss.
- **`wikidata/6` (diagnostic):** `1.0064 [0.7778, 1.3022]` (`0.2998 -> 0.3017 s`, RSS `0.9956x`).
- **`wikidata/8`:** `0.9242 [0.7860, 1.0868]` (`0.3051 -> 0.2921 s`, RSS `1.0227x`) — **fails `<= 0.985` and RSS `<= 1.020`**.
- **`wikidata/10`:** `1.0226 [0.9861, 1.0605]` (`0.3027 -> 0.3082 s`, RSS `1.0010x`) — **fails `<= 1.010`**.
- **Variance note:** Multi-thread `ab` runs at `wikidata/6..10` (`wikidata/8` `0.195..0.332 s`) and guard corpora (`text`, `records`, `random`, where 6 guard cells also exceeded `1.010` and `random/8,10` RSS `1.023x`) showed elevated host variance; however, tight `wikidata/1`, `wikidata/4`, and pinned 1T `station_balance` settle the mechanism decisively without needing a re-run.

## Direct physical i9-10900K `station_balance` (`k* = W / S`, pinned CPU 0)
- **`wikidata`:** `S` `35.24 -> 35.32 ms` (`1.0017 [1.0001, 1.0033]`), `W` `333.48 -> 342.65 ms` (**`1.0305 [1.0149, 1.0464]`**, **+3.05% slower**), **`k* = 9.46 -> 9.70`** (`r_aligned = 1.0286 [1.0130, 1.0446]`).
- **`text`:** `S` `9.04 -> 9.09 ms` (`1.0057`), `W` `46.01 -> 46.15 ms` (`1.0082`), **`k* = 5.09`**.
- **`records`:** `S` `19.89 -> 19.94 ms` (`1.0024`), `W` `73.14 -> 72.73 ms` (`0.9933`), **`k* = 3.68`**.
- **`random`:** `S` `16.00 -> 16.01 ms` (`0.9999`), `W` `62.34 -> 63.58 ms` (`0.9988`), **`k* = 3.90`**.

## Architectural & Frontier Consequence
1. **Silicon fix confirmed:** Comet Lake-S stepping 5 carries the Intel JCC-erratum silicon fix; `-x86-branches-within-32B-boundaries` adds `+2.94%` `.text` NOP bloat and slows Wikidata worker execution `+3.05%` on i9. Do **not** ship as a release/wheel flag (`pyproject.toml` / `.github/workflows/CI.yml` untouched); retain strictly as a Skylake/Cascade Lake VM measurement control (`CG-JCC`).
2. **Station balance settled on physical i9:** Default `wikidata` (`k* = 9.46`) is **worker-bound at `k = 8`** (worker levers pay directly at the 8-worker knee), whereas `text` (`5.09`), `records` (`3.68`), and `random` (`3.90`) are **scanner-bound at `k = 8`**.
