claim: ci-pattern-4t-arm-bias
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/ci-pattern-4t-arm-bias.md

- **Verdict (frozen stop test): `NOT REPRODUCED`.** Both plain A/A (same binary: Arm B `0.9709 [0.9297–1.0140]`, Arm C `0.9784 [0.9115–1.0503]`) and two-build A/A at `d3bbe33f` (Arm A `0.9910 [0.9531–1.0303]`, Arm D `0.9678 [0.9007–1.0400]`) contain `1` on `pattern` 4T (`0/10` 10-round flags). Out-of-sample CI check on 6 subsequent strict-null PR runs (60 pairs, 4-vCPU runners) gives `0.9970 [0.9865–1.0075]` (`0/6` flagged); all 11 strict-null CI runs (110 pairs) give `1.0048 [0.9853–1.0261]`.
- **Static & ELF audit:** `bin/ab` vs `.base-src` `base/ab` have `1,606/1,606` identical `.text` symbol addresses and opcodes (`diff_addr = 0`); the 10 B path diff shifts only `.debug_str` and cold `.rodata` panic pointers. Both arms spawn child subprocesses with identical `name = "mibz2"`.
- **Mechanism from 24 non-scanner CI runs (240 pairs):**
  1. *Symmetric bimodal RSS/scheduling regime (`R² = 0.706` strict-null, slope `0.881`):* on 4 vCPUs / 2 cores, `pattern` 4T flips between `36–38 MB` and `>= 39 MB` (`+4–8 MB` queued output). `h_hi_b_lo` (`n = 27`) gives `wall 1.1075`, `cpu 0.9693`; `h_lo_b_hi` (`n = 22`) gives `0.9019`, `cpu 1.0280`; `both_low` (`n = 50`) gives `1.0001 [0.9869–1.0135]`. OLS RSS-adjusted `head/base` wall is `0.9964 [0.9872–1.0057]` (strict-null) and `1.0001 [0.9920–1.0082]` (all 24 runs).
  2. *Round-0 cold-start asymmetry:* `ab run` has no warm-up and always runs head first in `r0`. Across 24 CI runs, `r0` gives `wall 1.0605 [1.0027–1.1216]`, `rss 1.0542`, high-RSS `12/24` vs `3/24` (`p = 0.0114`; strict-null `5/11` vs `0/11`, `p = 0.0351`), contributing `~+0.6%` to a 10-round mean. In `r1..r9`, all 11 strict-null runs give `wall 1.0012 [0.9831–1.0196]` and `33/99` vs `33/99` high-RSS (`p = 1.0000`).
- **Evidence for `#1034` flag (not resolved here):** In run `45947633032`, `782f4feb` hit `>= 39 MB` on `5/10` head vs `0/10` base rounds (`wall 1.0734`, `rss 1.0794`, `cpu −2.2%`); RSS-adjusted wall (strict-null slope `0.881`) is `1.0035 [0.9725–1.0355]`.
- **Policy boundary:** No bench, corpus, guard, or statistics change follows from this claim alone (`PERFORMANCE-POLICY` §2).
