# Draft workstation request for edge-budget admission

The combined writer reached 1.097–1.127× LazyCells throughput at 7–10 workers on the i9-10900K. Edge-budget uses the same implementation on the original admissions. Its additional width guard admits five dense Wikidata blocks whose certified tables range from 796,756 to 926,400 bytes. It scans capped records when testing a potential admission. The question is whether those five decisions conserve process memory at target concurrency, and how their construction costs affect complete wall time. [Combined i9 result](combined-i9.md) · [Structural census](../edge-budget/README.md)

Independent Xeon CI measured edge/combined median peak RSS of 0.9383, 0.8904 and 0.8334 at W1, W2 and W4. Paired wall ratios were 1.0071 [0.9986, 1.0156], 1.0096 [1.0031, 1.0161] and 1.0027 [0.9995, 1.0058]. W2 resolved a slowdown. This follow-up tests the tradeoff at each target worker count. [Five-arm CI receipt and raw runs](../ci/evidence/edge-budget-44431030145.md)

## Frozen comparison

The local registration is `bench/astra-e-edge-budget` at `e00b278656838aad9487ffa8fb7333eaa0c4705e`. The protocol implementation is `9b0d9637f1d2fcdae2be5d13ac97943c301f4533`. A checkout of that registration exposes these two commands.

```sh
just bench-astra-e-edge-budget-dry
just submit bench-astra-e-edge-budget
```

The edge decoder is built from `badb2aec3624bd65606c429b2e6f79e9d9d0050f`, entry `research/astra-e/edge-budget/candidate.rs`. A second detached checkout builds the combined reference from the exact measured commit `8c9640d8ab186f81a0500f48e606e0c21c2a6669`. Its entry `research/astra-e/narrow-short/ahead.rs` is identical to the first combined source `80d8409359214b4e5abfd4430213921e82e9cd30`. The bundle records the intervening library diff, both binary hashes, source trees and arm mappings. LazyCells and density-gated opend execute in the measured combined checkout's binary. The second candidate arm in that binary invokes the measured combined writer. The edge arm invokes the separate edge executable.

The runner fetches the live scorer and applies exactly the same projection to both worktrees, with Rust 1.99.1 and equivalent release options. Before decoding, it builds and rehearses both interfaces, including a synthetic 256-row two-binary rotor. It tries the pinned last-working scorer only for a demonstrated build or interface incompatibility and records the fresh diagnostic. Both candidates then pass the full byte/CRC gate. LazyCells and opend pass the quick controls. In an i9 dry run, each of the four arms also passes the quick Wikidata gate separately under affinity sized to W7, W8, W9 and W10. Verify the four arm labels and all sixteen worker-specific gate transcripts before measuring. [Scorer selection](refresh.py) · [Matched runner](edge.py)

## Preregistered decision

Use four concatenated copies of Wikidata, SHA-256 `0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e`, 838,800,784 output bytes per sample. Run unpinned on the ten-core i9-10900K with all 20 logical CPUs available. The four arms use equal W workers and 4W slots at W7, W8, W9 and W10. Sixteen complete forward/reverse rotated rounds, two seconds between processes, produce 256 rows. Pair observations within each worker count and round. A wrong decoded length, wrong executable mapping, failed correctness case, missing or reordered run, or nonpositive or nonfinite wall/RSS invalidates the attempt.

The campaign target passes only if every W has edge/Lazy upper 95% paired wall ratio ≤0.800, upper paired RSS ratio ≤1.000, median peak ratio ≤1.000, and maximum observed edge peak ≤ maximum Lazy peak. Report the per-worker contrasts against opend and the frozen combined writer as well. Each paired interval is a two-sided 95% Student interval on the natural logs of the round ratios.

Report a memory advantage over the measured combined writer at a given W if the upper paired RSS limit and median ratio are both below one and the maximum observed edge peak is at most the combined maximum. A claim over the complete W7–10 range requires all four cells. For wall time, an upper limit below one resolves an improvement, a lower limit above one resolves a regression, and an interval containing one remains unresolved. Report every W even when timing and memory move in opposite directions. Raw per-round observations and both process peak maxima remain in the upload. [Assessment](assess.py) · [Binary identities and tradeoff rule](edge.py)

A Git bundle preserves the registered commit and its parent requirement. From a checkout containing the protocol commit, restore a missing local branch with the following command. [Registration object](rehearsals/edge-registration-e00b278.bundle)

```sh
git fetch research/astra-e/workstation/rehearsals/edge-registration-e00b278.bundle refs/heads/bench/astra-e-edge-budget:refs/heads/bench/astra-e-edge-budget
```


## Untimed cloud receipt

The frozen registration passed its cloud dry run on four logical CPUs. Both independently built candidates passed their 338-case differential gates, including Wikidata and the mutation suite. Each control passed 98 cases. The replay verified both corpus hashes, both candidate source hashes, two distinct executable hashes, the projected Git blobs and the exact 256-row scheduled order. The cloud affinity mask did not cover W7–10. Those sixteen arm/worker gates remain for the target dry run. The archive contains no timed decoder samples. SHA-256 `d023bb4f60734ddb67e7cd52387d7c841384acfd9451fb59417041c5fb69d836`. [Full rehearsal archive](rehearsals/edge-matched-e00b278.tar.gz)

The fresh projection and scorer path revision were both `75297a1d57b2bdf262c3a19f3064848dedf2c493`. The executable interface rehearsals printed v1 at `9c31461df7bef24cecf79348545b27ee26846835+dirty` from the edge checkout and v1 at `9a9c169d70639c1c3e56cb85b3679fa7529e516c+dirty` from the combined checkout. The scorer obtains that string from each checkout's HEAD and working status. All 31 tracked blobs under `bench/moonshot`, `.just` and `Justfile` matched each other and the fresh upstream tree. The distinct release binary SHA-256 values were `8e2de672bca926d043288b39ff53496e06f9844fa3028341f697d60aca252536` for edge and `037f804a5076f967306a55dd150ba3cd47a040d0c609b3f799b0bf77773afb86` for combined.
