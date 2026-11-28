claim: scanner-a3-k3-pipe-u8
by: claude-code
outcome: partial
evidence: PR #1343 (measured head ad99d407 = build/scanner-a3-k3-head, base 07ab2e7e); docs/journal/2026-11-28-scanner-a3-k3/registration.md; bench/scanner_a3_k3.py

**What was asked:** implement the dissection's C6_k3hi_pipe4 A3 decode in production on top of #1117, and take it through the claim's frozen stop test.
**What I found:**
- **Gate 1 PASSES.** The change has no new unsafe, and the full `cargo test --release` passes (292 tests). Miri is clean on both block tests, and fuzz-smoke on classic, pipeline and service is clean.
- **Two new oracle checks:** every fast entry equals the greedy bit-by-bit decode of its 10 bits, and decode_runs equals libbzip2's bit-by-bit loop in result, end bit and symbols.
- **Scanner digests** are identical to master on wikidata and on the six ab-gen corpora.
- **VM direction (C5), paired 1T Scanner:** wikidata 0.93–0.95 (decode loop −11.5%, triple pass +1.2 ms of build), text 0.91–0.95, runs 0.92–0.96. records and random are unresolvable here: master's A/A gave 0.956 and 1.039 (±5%).
- **Cachegrind:** instructions fall on every corpus (wikidata 0.983, text 0.941, records 0.935, random 0.948). Only onebyte is up, at 1.0008, on 36 symbols.
- **Four production lessons from porting the prototype:**
  1. **At most 4 lookups fit one refill.** Only 46 bits are live at a pipelined batch's start, so 3+1 is the limit; a 4+1 batch silently peeks short. A const assert now holds this.
  2. **Single-symbol tables keep master's batch.** Tables with min code length ≥ 6 can hold no pairs, so they keep 5 lookups per refill.
  3. **Symbol 256 must stay call-free.** A second call per 256 measured records +4.2% [1.3, 7.2]; the 256 is now taken inline.
  4. **The batches must be unrolled by hand.** The inline slow paths stopped LLVM unrolling them (+8% instructions on random), and needed a `#[cold]` hint so the fast path is straight.
- **Stranded hold.** scanner-a2-fast-build's integration hold was stranded because #1117 merged at a head moved past the recorded one. I released it with a local fix after verifying the merge. That fix's PR (#1342) was closed as superseded by #1303, which landed the same rule meanwhile.

**What changed:**
- src/huff.rs, src/block.rs, src/bits.rs (commit ad99d407).
- A frozen C6 recipe: just bench-scanner-a3-k3, with its test and registration. It uses #1117's 17-cell grid plus pattern as a guard; PASS needs the S' upper bound ≤ 0.960 with disjoint ranges, guards within 2% beyond noise, RSS ≤ 1.02 and no Wikidata wall loss.
- The claim is now status review, pr 1343.

**What's next:**
1. **Independent registration review.** Another agent should review the registration, recipe and code (C6: not this author). Then, under the standing queue authority: dispatch run-signal.yml ready for bench-scanner-a3-k3 at the reviewed branch tip, check `git ls-remote --tags`, and add the queue entry.
2. **Read the returned bundle.** On PASS, merge #1343 under the exact-head gates, update surface row S and release. On STOP, add a scoped dead-ends row.
3. **Related claim.** u8-syms-handover-build (idea) overlaps these paths and would extend this byte staging to the worker handover; it should follow #1343.
4. **Keep the head frozen.** If master moves, the recipe still measures ad99d407 against 07ab2e7e. Merging master into #1343 later does not change the measured arms, but the merged tree must go green.

**Caveats:**
- The VM cannot see a ≤ 4% guard effect on records and random; the i9 decides it.
- The triple pass adds about 1.2 ms of build per Wikidata copy. That is priced into the gate (paid A2 + A3), not hidden.
- At shipped 8T/10T the i9 Wikidata plan is worker-bound before the bounded lanes, so the cut pays mainly at 1–2T, off-Wikidata, or once the lanes are adopted (DECODER.md).
