# Astra E research journal

Target decision requires a paired wall ratio at or below 0.80 versus 0.5.6 at the same 7–10-worker concurrency on Wikidata on the i9-10900K, with no increase in peak memory. The local reference is the scorer's explicit LazyCells arm. The shipping low-thread policy also uses eager Cells. Results below are experimental until independent target verification.

## Opening experiment

Master `9c31461df7bef24cecf79348545b27ee26846835` supplies the scorer and 0.5.6 decoder. Its `info` command prints the candidate identity and CPU count but has no version field. Each result includes the scorer path revision, toolchain, corpus digest, invocation, stdout, stderr, gate and individual runs. Corpus SHA-256 is `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 209,700,196 decoded bytes. Use the pinned compiler in rust-toolchain.toml. Build the corpus with `bash bench/moonshot/corpus.sh` (`lbzip2` required).

Start with ungated Open, density-gated Open and Lazy in the same round. This distinguishes a gate/policy effect from changes to the underlying inverse. Use explicit workers 1,2,3,4 for the four-CPU quota. Preserve raw per-arm samples and peak process RSS. New controlled arms investigate conjugating retained ranks, contracting literal paths, compact eager descriptors, predecoding trace pages and overlapping the two directions of the marked cycle. None has an established performance result at campaign opening.

The [performance register](REGISTER.md) links complete scorer receipts, individual timings, peak memory and independent evidence. The [formulation ledger](formulations/LEDGER.md) follows the separate search for objects that can eliminate reconstruction phases.
