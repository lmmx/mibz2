claim: explore
by: claude-code
outcome: done
evidence: docs/notebook/2026-11-27-explore-two-lane-scan.md; docs/journal/2026-11-27-explore/two-lane-scan/ilp_probe.rs.txt (probe on master 3d8fc666); corpora/wikidata.bz2 SHA-256 82f65ace…9915

**What was asked:** explore off the map for a direction that could move the coupled scanner/worker bottleneck, and probe it cheaply in the VM.

**What I found:** the scanner's A3 decode is one serial lookup chain per block, and nothing on the surface or in dead-ends uses the core's spare issue slots during it. The existing parallel-scan work all puts the scan on more threads: pooled scan, slack-hybrid, and within-block self-sync.
- **The direction.** A two-lane scanner decodes blocks i and i+1 in the same thread with their lookups interleaved. It adds no thread and no CPU-seconds.
- **VM probe** (Emerald Rapids-class, 48 KiB L1d, unpinned, VM only): a rolling two-lane schedule ran A3 at a paired median of **0.844x** of production `decode_runs` on all 431 wikidata blocks (33/33 repeats faster, min-of-repeats 0.842). On a 17-block text corpus it ran at **0.888x** (30/33).
- **Exactness.** Symbols matched production on every block.
- **Codegen.** It decides the result. With the lane `Reg` spilled to the stack (store-forwarding on the chain), the gain was only 0.923x. Static pairing, without rolling, gave only 0.966x.

**What changed:**
- Notebook entry `docs/notebook/2026-11-27-explore-two-lane-scan.md`.
- The probe source and the `pub(crate)` visibility diff under `docs/journal/2026-11-27-explore/two-lane-scan/`.
- New claim `scanner-two-lane-interleave` at status `idea`, P2, with a frozen stop test: a VM paid-scan gate first, then an i9 registration.
- Surface row S: the state, evidence and claims columns.

**What's next:** the smallest discriminating step is the claim's step 1. Port the lockstep loop into a `Scanner`-shaped two-lane path that takes lane B's start from a magic finder, and price the finder inline and on the feeder. Measure paid A2+A3 against the gates, default and JCC-32B. Order it behind the holders of `src/block.rs` and `src/huff.rs` (scanner-a2-fast-build #1117, scanner-a3-k3-pipe-u8).

**Caveats:**
- These are VM ratios for A3 alone, with headers and tables prepared outside the timing. The finder, seams, ordering, admission and error parity are unpaid.
- The i9 has a 32 KiB L1d and the JCC erratum; the VM has neither.
- The S and wall projections in the claim are derived, not measured. On i9 Wikidata at the shipped 8T/10T plans the cut is masked (worker-bound); it pays on text and off-Wikidata corpora.
- Three lanes were not tried.
- Brief showed "release due" at the start. `release-0-5-18` was opened at P1 by another session meanwhile (#1247), so this session did not duplicate it.
