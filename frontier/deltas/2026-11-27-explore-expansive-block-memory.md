claim: explore
by: claude-code
outcome: done
evidence: docs/journal/2026-11-27-explore/expansive-block-memory.md; master 1a61f5b, examples/bzcat; src/parallel.rs:402 (Shared::full), :961/:972 (free_out pool), :1292 (worker buffer); local bzip2 -9 corpora text/zeros/mixed (not checked in)

**What was asked:** the recurring `explore` claim: a phenomenon no claim covers, cold, from primary evidence.
**What I found:** every memory and admission figure in the frontier is priced on wikidata (~1 MB a block). I timed the shipped `bzcat` on run-only blocks (~46 MB decoded each) and measured peak RSS (`wait4`), 4-vCPU VM, best of 3.
- *The 12 MiB ready cap does not bound memory; the window does.* zeros.bz2 (2 GiB of zeros in 1,522 bytes) at t = 4: 457 / 591 / 591 / 943 MiB at 4 / 8 / 16 / 32 slots. The library default window (4 x threads) takes 1.27 GB at t = 8 and 1.82 GB at t = 16 (capped by the stream's 46 blocks): ~840,000x amplification. Text at the same settings: 99-148 MiB.
- *Mechanism:* `Shared::full` applies `READY_BYTES` only at the scanner's issue. Run-only blocks scan in microseconds, so the scanner fills the window before any block is ready; workers never check the cap, each decodes into its own buffer (grown to ~46 MB by RLE1), and the `free_out` pool keeps up to `slots` of them until drop. The module doc's bound (slots x ~46 MB) holds; the cap's does not.
- *Time, too:* at t = 4 the 16- and 32-slot windows are 24-28% slower than 8 slots on zeros (513 / 497 vs 402 ms), matching +90k first-touch page faults (~370 MB zeroed).
- *Nothing changes on text:* RSS 99-110 MiB and wall 518-561 ms at t = 4, 8 vs 16 slots.
**What changed:** the note; claim `ready-cap-worker-gate` at `idea` (P3, striated `src/parallel.rs`, stop test frozen: worker-side cap gate + byte-capped pool). No surface row changed: the P row prices wikidata, where the cap never binds. `explore` stays open.
**What's next:** `ready-cap-worker-gate` is disjoint from `scanner-seam-window` and `runt-exempt-window` in intent but shares `src/parallel.rs`; take it when those holds clear. A Python-level RSS check would confirm the same exposure through `mibz2.open`.
**Caveats:** VM only, no i9; t = 8/16 oversubscribed (RSS is the point there). Wall figures are best-of-3, not interleaved CIs. Zeros is the extreme; real run-heavy data (sparse images, disk dumps) sits between it and text.
