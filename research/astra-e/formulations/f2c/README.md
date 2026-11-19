# F2c. Reparse the phrase BWT before constructing its row links

**Replaced computation.** F2b reconstructs a quote-factor BWT as a run stream and then builds/dereferences one Ψ entry per token occurrence. The second level takes the *run stream*, the original token bucket starts and the dictionary as its input. It chooses the most frequent token as delimiter, breaking ties by its first-column bucket. One scan of the residual runs computes token frequencies and LF run starts. A thread-local array indexed by token ID replaces the 256-entry byte alphabet at this level. The first dense token link table is never built on a two-level admission.

**Exactness.** The F2 partition and stable-merge proof applies over any totally ordered alphabet as long as its first-column bucket starts respect that order. F2b already supplies those starts for its arbitrarily named dictionary words. A second-level phrase is a sequence of first-level tokens ending at a chosen token delimiter. Competing second-level children emit disjoint labels because their left extension symbols differ at the same depth. A terminal emits the current word, distinct from all its proper extensions. Within a coalesced selector span the consumed child runs already have distinct neighboring labels. Consequently each consumed child fragment is itself an output run; the second inverse merge needs no extra adjacent-label test.

The mark has two coordinates after the first contraction, a token row and an offset *in bytes* in its outgoing phrase. While rewinding to a second-level boundary, add the full **byte length** of each traversed first-level token. At that boundary the sum is the byte offset into its outgoing second-level phrase. The dictionary spells second-level words by copying the original bytes of their first-level constituents. Walking the second-level Ψ and emitting slices with that initial offset gives the original `n` bytes even on a short covered cycle. The one-level walk handles a cycle without the second delimiter, an unprofitable all-delimiter second section, or a link-field overflow. The first-level gate still applies. [Independent weighted small cases](small.txt)

**Structural evidence.** Ten systematic Wikidata blocks, including frozen and one block outside the production pre-gate, were tested at three frequency targets (most frequent, nearest to 1/8, nearest to 1/32). All 30 reconstructed the complete raw oracle output. On the most-frequent policy the 1,357,067 first-level link writes became 245,445 second-level writes. These results charge 70,030 second-stage lookups (including mark searches), 115,252 selectors, 230,346 merged fragments and 1,926,457 additional dictionary bytes assembled from 212,691 first-level spans. On frozen there were 24,165 second-level rows and 24,166 final phrase visits against the original 135,716. The 1/32 target reduced the ten-block final table to 41,846 rows at a price of 5,711,135 dictionary bytes and 862,941 copy spans. A smaller final table can increase the cost of spelling it. [Exact structural receipts](geometry.csv)

**Native check.** Both levels actually engaged on 228/233 large Wikidata blocks in the native scorer candidate. They matched all 205,197,794 raw inverse bytes from Segmented. Across them the second level built 5,566,401 links, assembled 41,433,719 dictionary bytes from 4,418,276 spans, and issued 5,566,621 final dependent phrase visits. For the nine doubly admitted systematic blocks, the independent Python construction agrees with the native table size, number of nodes, positions merged, copied dictionary spans and bytes, and final phrase visits in every row. [All native block counts](native.csv) · [Native run log](native.log)

**CPU pricing.** A separate Linux thread-CPU diagnostic compares the common stages, one-/two-level finishing, and shipped LazyCells' inversion. It preloads the same capped records, verifies each arm's raw output outside its timed region, warms up once and rotates the three arms for 12 measured rounds per block. It measures only the kernel on this host. Its Rust executable is optimized with one codegen unit and links the research-enabled release crate. The CSV gives each observation. Summaries take the median of each phase per block and sum across the nine equally weighted admitted systematic blocks. The median of totals and sum of phase medians can differ. [Raw thread CPU observations](phase-cpu.csv) · [Phase summary](phase-summary.csv)

| Nine blocks | Run/page construction and mark | Trie discovery and spelling | Stable merge | Residual and output | Whole inverse | Ratio to Lazy |
|---|---:|---:|---:|---:|---:|---:|
| One level | 4.219 ms | 15.783 ms | 1.420 ms | 9.902 ms | 31.799 ms | 1.432 |
| Two levels | 4.348 ms | 15.862 ms | 1.426 ms | 5.931 ms | 27.852 ms | 1.254 |
| LazyCells | | | | | 22.211 ms | 1.000 |

The one-level common work alone consumes 96.5% of the comparator's entire inverse CPU; the two-level arm reads 97.4%. On frozen the whole inverse ratios were 1.451 and 1.164 respectively. This local observation makes further residual-only recursion a poor path to 0.80. The first-level discovery cost would have to fall materially even if residual construction and output became free. The independent executable scorer decides wall time and memory.

## Reproduction

```sh
study=research/astra-e/formulations/f2c
scratch=$(mktemp -d)
CARGO_TARGET_DIR="$scratch/build" bash "$study/export.sh" "$scratch"
python3 "$study/second.py" --small
python3 "$study/second.py" "$scratch/export"
cargo build --release --manifest-path bench/moonshot/Cargo.toml --target-dir "$scratch/build"
bash "$study/price.sh" "$scratch/price" "$scratch/build/release/deps" > "$scratch/phases.csv"
python3 "$study/summarize.py" "$scratch/phases.csv"
export MOONSHOT_CANDIDATE="$study/candidate.rs"
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV="$scratch/native.csv" \
  cargo test --manifest-path bench/moonshot/Cargo.toml candidate::checks::native_second_level -- --ignored --nocapture
bash research/astra-e/score.sh recursive-f2c-round1 "$MOONSHOT_CANDIDATE" --with-opend
```
