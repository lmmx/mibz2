mibz2 0.5.6 remains the released decoder. A proposed replacement for one of its block-decoding methods has passed correctness review, and one confirmatory run on Louis’s ten-core workstation has started to decide whether it enters 0.5.7. No result from that run is covered yet; if it does not meet its registered speed, memory and non-regression rules, the replacement will not ship.

## Why try this replacement?

The proposed [open decoder](https://github.com/lmmx/mibz2/pull/630) reconstructs selected blocks by a different route. Earlier [matched-kernel tests](https://github.com/lmmx/mibz2/pull/145#issuecomment-6994380056) found it 5–7% faster than the current method across two to ten workers, but those tests did not include the complete release path. The first explanation, that it reduced contention at high worker counts, failed that sweep; an independent [analysis](https://github.com/lmmx/mibz2/pull/145#issuecomment-6994466885) instead counted fewer instructions per output byte.

A [block-by-block check](https://github.com/lmmx/mibz2/pull/145#issuecomment-7005084600) found that open was faster only on blocks the current smaller-memory method already accepts. The proposed port therefore reuses that method’s existing record-count gates, rather than selecting blocks by their source corpus. It leaves other blocks on their current route. The [Guardian reviewed](https://github.com/lmmx/mibz2/pull/145#issuecomment-7007260113) the correctness evidence and allowed the adoption study to proceed. That review did not approve a merge or establish a speed gain. It also narrowed the claimed scratch-memory bound to logical elements, which is not a measured process-memory bound.

## What will the run decide?

The [frozen registration](https://github.com/lmmx/mibz2/pull/145#issuecomment-7009741436) compares the release build of the port with 0.5.6 at eight, ten and twenty threads. It includes held-out Wikidata samples, other data types and a sustained workload. It requires a gain on the main Wikidata sample at every count, no added peak memory there, a gain on held-out Wikidata, and explicit limits on other workloads and sustained performance. The Researcher [checked the corrected scorer](https://github.com/lmmx/mibz2/pull/145#issuecomment-7008650123), including complete input identities and the rule that a failed timed attempt does not open a retry. The [single attempt has started](https://github.com/lmmx/mibz2/pull/145#issuecomment-7009734722). A favourable result permits the port to merge and 0.5.7 to be released; a negative or invalid result leaves it out.

The [downstream Wikidata pipeline](https://github.com/lmmx/wikidata-pq/pull/227) already uses 0.5.6. Its measured quick sample stayed level, while its full-dump model estimated a small gain. This is separate from the open port, which is not yet in a release.

## What else has research found?

A separate experimental family improved in the isolated decoder test. Its best [scalar variant](https://github.com/lmmx/mibz2/pull/145#issuecomment-7004261019) ran 2–5% faster than ungated open at seven to ten workers with roughly equal peak memory. Its own earlier improvement over its predecessor was small, while an added counter [failed its incremental rule](https://github.com/lmmx/mibz2/pull/145#issuecomment-7001987495). None of these research-kernel results qualifies a production change; whether to scope the scalar variant for a port is Louis’s choice.

The Researcher [related the delimiter construction](https://github.com/lmmx/mibz2/pull/145#issuecomment-6997277733) to earlier parsing work without finding an identical statement of its finite run bounds. Independent mathematics verified the bounds. A separate [Neon investigation](https://github.com/lmmx/mibz2/pull/145#issuecomment-7008576600) found that a proposed fragment-count floor applies only to a specified in-place copying class. Its competitive high-run-density schedule still needs a way to label run transitions without walking every output row. That is the next research question, not an implementation gain.

## What happens next?

The team awaits the started workstation run and will apply its registered verdict once. No action is requested from Louis by this record. The independent research can continue without changing the port’s frozen rule.
