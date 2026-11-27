# Adaptive permutation compression — independent software incubation

26 November 2026 · working title; not a published crate or decoder feature.

**Owner's revised investment criterion (26 November 2026):** no standalone implementation programme solely for general-purpose compression or memory savings. First identify a concrete computational advantage for mibz2, with a credible construction-plus-consumer budget and a measured relevant improvement over matched controls. The separate decoder investigations establish that case; the standalone package then generalises the useful mathematics while preserving its independence. Until that application case exists, keep this design as an unadmitted idea, not an implementation task. The scoped #998 STOP stands; do not reinterpret a storage-only result as a decoder win.

**Purpose:** reusable mathematical software for discovering, constructing, selecting and evaluating exact compressed representations of finite permutations. The library learns which representation fits the input and the operations requested, including the cost of finding and building it. The object is the permutation, not a compressed text or a particular inverse-BWT algorithm.

- [Full research and application specification](DESIGN.md)
- [First swarm work item](../../frontier/claims/permutation-compression-incubation.toml)
- [Permutation-patterns discussion, preserved verbatim](../../docs/journal/2026-11-24-algorithm-level/permutation-patterns/chatgpt-response.md)
- [Ideal permutation structures](../../docs/journal/2026-11-24-algorithm-level/ideal-permutation-structures/README.md)
- [Earlier Nanda-inspired research-software survey](../../docs/journal/2026-11-24-algorithm-level/leads/research-software.md)
- [Standalone-crate precedent in reshape](../../docs/journal/2026-10-27-bz2-decoder.md)

This directory is reserved for the independent package, its documentation, tests and generic benchmarks. It must not become part of the root decoder's Cargo graph, API, releases or performance gates. The initial work item is a bounded feasibility slice, not permission to publish, create another repository, change root dependencies or commit to a permanent API.
