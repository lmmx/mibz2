# Astra E research journal

Target decision requires a paired wall ratio at or below 0.80 versus 0.5.6 at the same 7–10-worker concurrency on Wikidata on the i9-10900K, with no increase in peak memory. The local reference is the scorer's explicit LazyCells arm. The shipping low-thread policy also uses eager Cells. Results below are experimental until independent target verification.

## Opening experiment

Master `9c31461df7bef24cecf79348545b27ee26846835` supplies the scorer and 0.5.6 decoder. Its `info` command prints the candidate identity and CPU count but has no version field. Each result includes the scorer path revision, toolchain, corpus digest, invocation, stdout, stderr, gate and individual runs. Corpus SHA-256 is `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`, 209,700,196 decoded bytes. Use the pinned compiler in rust-toolchain.toml. Build the corpus with `bash bench/moonshot/corpus.sh` (`lbzip2` required).

Start with ungated Open, density-gated Open and Lazy in the same round. This distinguishes a gate/policy effect from changes to the underlying inverse. Use explicit workers 1,2,3,4 for the four-CPU quota. Preserve raw per-arm samples and peak process RSS. New controlled arms investigate conjugating retained ranks, contracting literal paths, compact eager descriptors, predecoding trace pages and overlapping the two directions of the marked cycle. None has an established performance result at campaign opening.

The [continuation record](HANDOFF.md) gives the current independent target status, reproducible reference candidates and next experiments. The [performance register](REGISTER.md) links complete scorer receipts, individual timings, peak memory and independent evidence. The [formulation ledger](formulations/LEDGER.md) follows the separate search for objects that can eliminate reconstruction phases.

## Delimiter induction and independent measurements

The sustained mathematical programme follows one question. How cheaply can the original run column reveal the ordered return words and the marked position within one? The [ledger](formulations/LEDGER.md) follows the direct byte construction, contraction of unary paths, induction on tokens and the equivalence with direct induction on a compound byte marker. The raw observations distinguish output and construction costs.

The independently benchmarked one-edge lookahead beat opend at W1 and W2 on the AMD CI runner. The W4 interval included parity and eager Cells remained faster at W1, W2 and W4. The Coordinator has the frozen i9 recipe and successful cloud rehearsal. [Independent CI](https://github.com/lmmx/mibz2/actions/runs/44425372339) · [Workstation request and decision rule](workstation/lookahead-request.md)

Scorer v1 arrived at upstream path revision `604dac3f46569edd9ac9b6706b23a68d9d7d8f8d`. Local v1 reports also print the commit which imported that tree into the research branch, `6a8a366cd786d58a528000b58da12fd2cfb47be8`. The new series reruns the former local and independently measured leaders alongside the controls and the bounded admission candidate. Each v1 result includes its official JSONL register line. Older results keep their original corpus sample, proxy calibration and provenance.


The i9-10900K independently verified 1.115–1.153× throughput over 0.5.6 with lower peak RSS for the dense-block writer at 7–10 workers. The compact delimiter decoder independently improved on its predecessor by 7.8–8.7% at 1–4 workers. Current measurements, failed constructions and open hypotheses are linked from the continuation record. [i9 receipt](workstation/edge-i9.md) · [Compact delimiter CI](ci/evidence/compact-preload-44440381983.md) · [Continuation](HANDOFF.md)
