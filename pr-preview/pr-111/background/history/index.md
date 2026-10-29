# History

- **In wikidata-pq.** mibz2 was written in [lmmx/wikidata-pq](<https://github.com/lmmx/wikidata-pq>), whose split decodes the Wikidata JSON dump. A profile there found the bz2 decode about half of the split's CPU, bound by the inverse BWT's chain of dependent loads, and a prototype walked it as interleaved chains (lmmx/wikidata-pq\#194). The production decoder followed behind a flag (`WIKIDATA_BZ2_DECODER=native`, lmmx/wikidata-pq\#197): on CI, the split's CPU fell 20.6% and its wall 16.6% against lbzip2.
- **The bake-off.** Before going further, it was timed against pbz2, fbz, the `lbzip2` crate, indexed\_bzip2, lbzip2 and libbzip2 on the dump's blocks. pbz2 0.2.0 tied it; the others were 1.5–1.9× slower at 4 threads (lmmx/wikidata-pq\#199).
- **Its own repository.** The decoder's history moved to [lmmx/mibz2](<https://github.com/lmmx/mibz2>) by `git filter-repo`, its paths renamed and each `#N` in its commit messages rewritten to `lmmx/wikidata-pq#N`, with its journals. It was published as 0.1.0 and 0.1.1 on crates.io and PyPI, the crate and the Python package at one version.

The [journals](<https://lmmx.github.io/mibz2/pr-preview/pr-111/background/journals/index.md>) record each step, and [Status](<https://lmmx.github.io/mibz2/pr-preview/pr-111/background/status/index.md>) says where the research stands now.
