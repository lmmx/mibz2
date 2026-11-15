# The 16-byte slot (buffer diet step 2): rejected

PR #398 (closed unmerged; branch `perf/cells-slot16`, head f3ff1c01). The design moved each
word's length out of the slot and into the word: a 4-byte header on small arena words, a rule's
length as the sum of its two factor lengths, and the walk's row's letter on its own marked
entry. Output was byte-identical, and the safety material is complete on the branch (a written
argument, Miri, fuzz-smoke at seed 42 with 28,104 cases, and ten mutants in `mutants.md`).

- **Net bytes:** on the bench's 196 Wikidata blocks, r₀ = 11,090,867, U = 9,365,628 and
  14,117,160 small-word headers. The slot saving is 4(2r₀+U) = 126,189,448 B, the headers add
  56,468,640 B, so the net saving is 69,720,808 B: 0.33 B per output byte, against step 1's
  0.60 B. The headers give back 45% of the slot saving.
- **The CI paired bench** (run 43645031562, against b180d61): no primary gain at any thread count.
  `wikidata` CPU at 4 threads was +4.4% (+1.6, +7.3): one more dependent load per letter read.
- **Decision** (#145 LOG 6884914379): no merge under PERFORMANCE-POLICY.md §2. The i9 Q₂ run was
  not queued, since it would only have decided step 3, which is withdrawn because the arena is
  zero-filled only on growth (#145 LOG 6884895257). The diet ends at step 1, the 20-byte slot
  (#391).
