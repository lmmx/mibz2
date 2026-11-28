claim: explore
by: claude-code
outcome: done
evidence: origin/evidence 870e61d (character, sha256 bb72107b…), e98948b6 (forward-lanes, db53fe2c…), 5d259c8 (kernel-choice, dbb88594…), d6bc8ca2 (adaptive seam, c14e53b4…), each checked; docs/notebook/2026-11-28-explore-scanner-inflation-is-clock.md

**What was asked:** `take one`, as the explore share. The target was the nineteenth pass's untested premise that "faster workers inflate the scanner" (+10% base, +15% lanes, 8W over 1W, i9). The pass named L3 or bandwidth as the cause.
**What I found:**
- **The i9's clock falls with the number of busy cores.** By cycles over ref-cycles it is 5.28 GHz at k = 1, 5.01 at 2, 4.96 at 3, and 4.91 from 4 to 20 (character report). The `cores.json` pacing loop gives 5.29 GHz with one core busy; I checked by disassembly that LLVM folds 8 LCG steps into one 4-cycle `imul+add`. Kernel-choice and the seam bundle read 4.87-4.90 GHz at 3-12 threads.
- **The scanner follows that clock.** In the character k = 1..20 grid its busy time is 172.8 / 179.9 / 183.7 / 183.4-183.7 / 185.0 ms at k = 1 / 2 / 3 / 4-7 / 8. The clock alone predicts 182.1 / 183.9 / 185.8 / 185.8. The shape is a step and then flat, where contention would grow with k. A3 rises +6.4% against the clock's +7.5%. Only window maintenance grows with k (+3.6 ms by 8). Past 10 threads, SMT sharing begins.
- **Across the whole process,** k = 1 against k = 8 runs the same instructions, with +5.7% cycles (84% of them memory stalls) and +13.7% time. So 7.5 points of the time rise is clock.
- **In forward-lanes (post-#1034),** the clock explains 11.7 of base's 16.2 ms and 11.8 of the lanes' 22.9 ms. The clock-adjusted residual is +4.5 ms (+2.7%) on base and +11.1 ms (+6.6%) with lanes. Only that residual is the coupling to the workers, and the lanes add +6.6 ms of it.
**What changed:**
- New notebook entry.
- DECODER.md: a correction appended to the nineteenth-pass bullet.
- surface.tsv `S` row: one parenthesis added.
- New idea claim `scanner-contention-residual`, an i9 per-thread PMU probe of the residual.
**What's next:**
- A 1T-derived serial S' (the fourteenth pass's 115-119 ms) compares with an 8W W/8 only after ×~1.075, about 124-128 ms before contention.
- A scanner lever's 1T saving in ms is worth ~1.075× at 8W, not less.
- The clock part can't be removed, because desktop Comet Lake cores share one clock.
**Caveats:**
- Forward-lanes has no PMU. Its 1W clock is taken from the character run on the same machine.
- Its 1W trace is n = 1.
- Character predates #1034.
- The residual's mechanism is untested.
