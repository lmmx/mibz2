# Removing leaf streams and redundant coalescing from bottom-up merge

A contracted node with one selector contains only its own terminal. Every selection of that child therefore emits the child's id for the full selector width. No caller needs a stored leaf run, a cursor advance or a width comparison for that child. The pure root still writes its one output piece. Inner nodes with a terminal and child choices retain their full local streams.

The [earlier splice proof](../../f2d/splice/) at `a93eee798ad94ee7c6216ac97066e9518b094942` established that distinct selector choices contribute disjoint labels when relinking mutable run cells. The same property also simplifies the original contiguous-vector merge. There is a second consequence of stable partitioning. Inside one child selector, adjacent partial emissions originate from consecutive runs of that child's stream, whose labels differ by induction. At the next selector, the choice changes because selectors are coalesced. Distinct choices have disjoint terminal supports. The self-terminal is also distinct from every descendant label. Therefore each emission is exactly one maximal local run, including partial consumption followed by later re-entry into a child. An unconditional append can replace the old last-id test and width update. In test builds this induction is checked at every append.

The [full kernel](kernel.rs) has two independent compile-time switches. [Leaf absorption](leaf.rs), [unconditional append](maximal.rs) and [their combination](candidate.rs) compile separately; the fourth factorial arm uses the original dynamic refill source. The section builder, selector vector and second induction are identical. Absorption saves exactly one stored pair per nonroot terminal leaf. On 228 admissions it cuts the materialized arena from 5,390,514 to 4,686,557 cumulative pairs, removing **703,957 (13.06%)**. The actual per-block geometry agrees with the independently discovered leaf counts on all nine common inputs. On frozen it removes 2,631 pairs (21,048 bytes of logical payload) from 18,994. Vec capacity thresholds and process peak RSS require separate measurements.

The combination passed 11,264 small marked cases, the pure-root and cyclic reopening regressions, and all 205,197,794 inverse bytes on the 228 real admissions. [Property output](property.log) · [Corpus check](corpus.log) · [Block census](geometry.csv)

The paired panel rotates the four factorial arms and Lazy in each pass, with one warm-up and 40 measured passes over nine systematic inputs. Every output is checked after every inversion. In units of the original refill merge's thread CPU, leaf-only measured 1.029 [.935,1.133], unconditional append .909 [.815,1.013], and their combination 1.020 [.914,1.138]. The corresponding complete inverse ratios were 1.015 [.983,1.048], 1.007 [.976,1.039] and 1.020 [.986,1.056]. The combined merge took 1.122 [1.020,1.234] of the unconditional-append merge. The uninterrupted mixed-stage measurements establish a reduction in stored pieces; they give no whole-inverse lead. [Raw panel](phase.csv) · [Paired summary](summary.txt)

The full v1 scorer gate passed 338 cases against libbzip2, including CRCs. The refresh identified scorer source `604dac3`; the scorer printed `6a8a366`. The receipt is preserved at `f84b3ee`. [Scorer report](../../../results/v1-delimiter-bottom-leaf-gate/score/summary.txt)

Reproduce from the worktree root with a release scorer build and frozen corpus present.

```sh
d=research/astra-e/formulations/f2-rootstream/bottom
export MOONSHOT_CANDIDATE="$PWD/$d/candidate.rs"
cargo test --release --locked --manifest-path bench/moonshot/Cargo.toml candidate:: -- --nocapture
F2_CORPUS="$PWD/corpora/wikidata.bz2" F2_CSV=/tmp/bottom-geometry.csv cargo test --release --locked --manifest-path bench/moonshot/Cargo.toml candidate::checks::corpus -- --ignored --nocapture
taskset -c 4 bash "$d/price.sh" /tmp/f2-bottom-price "$PWD/bench/moonshot/target/release/deps"
PYTHONDONTWRITEBYTECODE=1 python3 "$d/summarize.py"
```
