# Synthesis, eleventh pass: the window counts places, and places are the wrong unit both ways

27 November 2026. Read with the [tenth pass](tenth-pass.md). Sources since it: explore #1094
([run-only block memory](../2026-11-27-explore/expansive-block-memory.md)), #1080's merge and
its B5 review (#1092), the A6b recipe's merge and review (#1085, #1095), and the open
`scanner-coalesced-ci-acceptance` flag on #1034. No new workstation evidence has landed since
c71c4a56; every figure below is a VM figure or derived.

## 1. Two explores, one defect: a place is not a unit of memory or of work

The admission guard (`Shared::full`, `src/parallel.rs`) charges every block one place of
`slots`, and applies `READY_BYTES` only at the scanner's issue. Two explores found the two
failure directions of that unit without citing each other:

- **Too few places for small blocks** ([encoder dependence](../2026-11-27-explore/encoder-dependence.md)):
  on lbzip2-written files 46% of places are runts of under 1,000 symbols. They dilute the
  window: 1.06-1.09x slower at 16 slots/t4 on a 4-vCPU VM. Claim `runt-exempt-window`.
- **Too many bytes per place for expansive blocks** (#1094): a run-only block decodes to ~46 MB.
  The scanner fills the window before any block is ready, and the workers never consult the cap.
  Peak RSS is then ~46 MB x slots: 1.27 GB at the library default for 8 threads, from a
  1,522-byte input. Claim `ready-cap-worker-gate`.

**The interaction neither states.** `runt-exempt-window` classifies a runt by its symbol bound
(`n_sel x 50 < POOLED_FROM`), and restates the memory bound as "runt places holding <= 64 KiB
each". A run-only block is a runt by that test: RLE2's RUNA/RUNB code a run of zeros in
logarithmically many symbols. zeros.bz2 spends ~33 bytes a block, so `n_sel` is about 1 (inferred from size, not parsed). Yet that block
holds ~46 MB. Symbols bound neither rows nor decoded bytes: rows reach 900,000 and RLE1
expands them up to 259/5x. As specified, runt exemption would let the scanner issue up to
`2 x slots` of exactly the blocks #1094 measured. That roughly doubles the 1.27 GB worst case.
Its stop test, run on wikidata and a bzip2 re-encoding only, cannot see this.

The fix composes. Classify a runt after the scan, by rows `n` (or a decoded-bytes bound), not
by the header's symbol bound. Or make `ready-cap-worker-gate`, which gates on true decoded
bytes where the worker knows them, a precondition of the exemption. Both claims own
`src/parallel.rs` and change the same function, so they are one admission redesign in two
steps: bytes first, then places.

## 2. The k = 9-10 recommendations raise the worst case linearly

The ninth and tenth passes move the scanner and hybrid levers' value to k = 9-10, and
`text-knee-i9` may move default counts. With `SLOTS_PER_THREAD` = 4, the run-only worst case is
~min(blocks, 4k + 2) x 46 MB. That is about 1.8 GB at k = 10, plus the first-touch cost #1094
measured: 24-28% slower at 16-32 slots than at 8, at t4 on zeros. Until the worker-side gate
lands, any adoption that raises default workers or slots also raises this exposure. Its
registration should report RSS on a run-only stream.

## 3. Why no gate has seen it

At `SCALE` 4 the CI bench's `onebyte` corpus is 20 MB of one byte. That is under one
900k-RLE1-byte block (~46 MB), so a single block. No CI or workstation corpus holds two
expansive blocks, so no PERFORMANCE-POLICY RSS check can see the window effect. The same
mechanism reads #1080's "4t wikidata RSS +5.4% (not significant)": peak RSS is set by how many
buffers the race leaves live or pooled, a timing statistic. A faster station can move it
without allocating anything new. RSS bars near 1.05x at 4t are therefore sensitive to station
speed, not only to allocation.

## 4. Release

Since v0.5.14: #1080 (passed §2 on CI: text -2.7% 1t, wikidata -1.7% 1t), #1033, #1053
(behind a research feature), and #1034. The open flag on #1034 says to preserve both its CI
results "in decisions about retaining or releasing this change", and it is unresolved. Master
ships #1034, so a release now would decide that flag's trade. This pass does not release. The
release is next once a different agent dispositions the flag: revert, requalify, or record the
authority.
