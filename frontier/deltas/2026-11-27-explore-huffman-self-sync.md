claim: explore
by: claude-code
outcome: done
evidence: docs/journal/2026-11-27-explore/huffman-self-sync.md; scripts huffman_self_sync.py, huffman_index_hypotheses.py beside it; corpora wikidata.bz2 82f65ace, ab gen text.bz2 7149b995

**What was asked:** an exploration off the map that is not a dead end or a rediscovery.
**What I found:** the 24 November FSM-composition lead named one cheap measurement to take before any code: the self-synchronisation length of bzip2's Huffman tables, and the selector-count obstruction. Nobody had taken it. I measured it offline and exactly on all 431 wikidata blocks (24.98M symbols), with `text` as a control. Every block decodes to its next magic.
- *The code contracts fast:* from a random bit with oracle tables, the first true codeword boundary comes after 3 codewords at the median, 43 at p99 and about 25 bits on average (text: 3 and 34).
- *The symbol index does not, and it picks the table:* started on a true boundary with the index off by d, the decoder re-locks within 5,000 symbols for 82% of d = -1 but 0.29% of +25 and 0.05% of +50. The error walk drifts one way.
- *The selectors don't locate a chunk:* expected code lengths place a bit to +-8 groups at the median (text: +-62). 801 index hypotheses leave about 131 distinct states after 16 kbit, and none of 699 collapses to one. That is about 30x a block's serial scan in redundant decoding.
- *Dominated anyway:* the magic-candidate scan (2026-11-10) already parallelises across blocks with no redundant decoding, since each block's start state is known exactly. Both approaches move scan CPU onto workers, so the pooled-scan argument applies to both.
**What changed:** new note and two scripts in `docs/journal/2026-11-27-explore/`; one STOP row in `frontier/dead-ends.tsv`. No claim opened: nothing measured here earns one. No surface row changed: S's cost and room are unchanged.
**What's next:**
- Don't propose within-block speculative Huffman scanning without an exact source for the symbol index; bzip2 stores none.
- A corpus property: `wikidata.bz2` is 12 byte-aligned streams and x4 is 48, while the lbzip2-written dump is (inferred) one stream. A lever that uses stream heads would be a bench artefact and must be judged on a single-stream input.
**Caveats:** this is offline counting, with no timing and no Rust. E1's estimator is one simple choice; a better one shrinks the window, but the slow collapse (E2) is the cost. The drift's mechanism is not tested. D and E are capped at 5,000 symbols and 16 kbit.
