# Induction records: the committed sample

Records of the shipped lazy Cells kernel's induction (format `irec/1`). They were made by
`examples/induction_trace.rs` with the research feature `induction-trace`, at
`lmmx/mibz2` commit `2da2a24944e16700475c572166695cc47f60c233` (branch
`research/induction-trace`, PR #486). This branch holds only the data; it shares no history
with the code.

- **`wikidata/`:** every 24th of the 233 blocks of at least 65,536 rows of
  `corpora/wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`), numbered as MONODROMY's scripts number them. Made
  by `induction_trace corpora/wikidata.bz2 OUT 24`.
- **`prose/`:** every block of `corpora/prose.bz2` (here; SHA-256 `5dcc4817845a94d62556716f9a8305ee95bd3776653f0a48a20247cfb13d6357`), the repository's
  Markdown at that commit by MONODROMY's recipe:
  `git ls-files 'docs/**/*.md' '*.md' | sort -u | xargs cat | bzip2 -9`. Made by
  `induction_trace corpora/prose.bz2 OUT 1 0`.
- **`*/blocks.jsonl`:** a line a block: n, r, steps, closures, period, admitted.

**Each record is xz-compressed.** `bench/irec.py` (on master once #486 merges) reads it as
it is:

```sh
python3 bench/irec.py wikidata --check      # replays every step against the record
```

```python
import irec
rec = irec.load("wikidata/0000.irec.xz")
o = irec.Orders(rec)
o.run_to(1000)
o.top(), o.bottom()
```

**Before it was written,** every record was checked against the kernel's own log and output
(`src/itrace.rs`), and `irec.py --check` replays each one again. The full sets are made
deterministically with the same commands and `EVERY` 1. They take about 6 s for Wikidata and
are 1.6 GB uncompressed.
