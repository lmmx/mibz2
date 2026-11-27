claim: explore
by: claude-code
outcome: done
evidence: docs/journal/2026-11-27-explore/encoder-dependence.md; corpora/wikidata.bz2 82f65ace; re-encodings bzip2 817e6d89, lbzip2 3241376c, pbzip2 60f48c09 (not checked in)

**What was asked:** an `explore` contribution covering a phenomenon no claim covers, or a falsification attempt on a baseline premise.
**What I found:** I tested whether the decoder baseline is an artefact of the one compressor that wrote the bench corpus. I re-encoded its exact 209.7 MB with bzip2 1.0.8, lbzip2 2.5 and pbzip2 1.1.13 on a 4-vCPU VM, with interleaved rounds.
- *The dump is lbzip2-shaped (inferred):* lbzip2 reproduces its runt structure (188 against 198), its tables-a-block histogram and its code lengths to 0.06 points. Reference bzip2 writes no runts (234 blocks). pbzip2 writes 234 streams and 201 runts.
- *The scanner is encoder-invariant, so the falsification failed:* lookups a symbol are 0.5556-0.5558 on all four files, and the scan's best-of-8 is 44.5-45.8 ms. The S row carries over to other encoders.
- *The whole decode is not encoder-invariant:* at `examples/bzcat`'s default 2 x threads window, the dump is 1.17-1.23x slower than its bzip2 re-encoding at t1, t2 and t4. User CPU is equal (509 against 491 ms at t1), and the scanner/worker overlap is lost. Sweeping slots at t4, the ratio falls from 1.35 (4 slots) through 1.06-1.09 (16, production's default) to 1.01 (32) and 0.99 (64). This is slot starvation: `Shared::full` charges a runt a whole place, and 46% of the dump's places are runts. #1035 priced the same dilution at -0.76% at k = 8 on the i9. On a small, oversubscribed machine it is several times larger.
**What changed:** new note `docs/journal/2026-11-27-explore/encoder-dependence.md`; new claim `runt-exempt-window` (idea, P3, striated `src/parallel.rs`) with a frozen stop test; the surface P row now records the runt dilution and its encoder dependence.
**What's next:**
- Implement `runt-exempt-window`: a runt doesn't charge a full place, exempt places are capped at `slots` more, and a bzip2-reencoded corpus serves as the control.
- Two small harness drifts are left for whoever next touches those files (both striated, so not fixed here):
  - `examples/bzcat.rs` still defaults to 2 x threads, while production uses `SLOTS_PER_THREAD` = 4 (#171). Timing bzcat on lbzip2 input therefore measures a starved pipeline.
  - `bench/occupancy.sh`'s usage line says "default 2 x threads".
**Caveats:**
- These are VM figures. The VM is bimodal (~45/~55 ms scans), and the residual at 16 slots was about 1% in one of three sweeps.
- No i9 run, no bit-identity check against lbzip2, no slot-wait trace at t4.
- The window sweep used `StreamDecoder::new` through bzcat, not `BzReader::parallel`, although both use the same admission code.
