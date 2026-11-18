# Reproducing the live-reuse study

The [research record](../2026-11-18-cells-live-reuse.md) contains the identities, counterexamples and outcome. The [cost ledger](cost.md) states the reference operations and prices. Run the Python scripts with assertions enabled. The local run used Python 3.12.14, rustc 1.90.0 for the existing reference tools, and Cachegrind 3.24. Production pins a different compiler for its workstation measurements.

## Data and the frozen screen

The frozen block has stream index 1, large-block ordinal 0, 899,931 rows and primary 264,045. The ten published Wikidata traces are every 24th large block. The [data branch](https://github.com/lmmx/mibz2/blob/2e4b34e086ac95e8218f1ee822a4652c8db5bb19/README.md) also supplies all six prose traces. From the root of this checkout:

```sh
study="$PWD/docs/journal/2026-11-18-cells-live-reuse"
tmp=$(mktemp -d)
git fetch origin refs/heads/data/induction-records
git archive 2e4b34e086ac95e8218f1ee822a4652c8db5bb19 wikidata prose | tar -x -C "$tmp"
python3 "$study/census.py" --examples "$tmp/wikidata/0000.irec.xz"
python3 "$study/positions.py" --small
python3 "$study/positions.py" --lf --overwrite "$tmp/wikidata" "$tmp/prose" > "$tmp/sample.csv"
python3 "$study/boundaries.py" "$tmp/wikidata" > "$tmp/boundaries.csv"
python3 "$study/prices.py"
```

SHA-256 of `sample.csv` is `dd34d8a5555f7f5ae8abd0a01bf346151ce3635201f063d4479da15a9f59ae74`. SHA-256 of `boundaries.csv` is `ff77917efdfcf0337538d71adb0a21266180e9bb46962c405fee979a1169a8b3`. The first mismatch witness on a direction switch is a triple of cut number, previous carried link and actual link on the other side. Slot numbers are those of the executable reference model.

## Whole-corpus evidence

The pinned compressed Wikidata stream has SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`. The repository's `bench/corpus.sh` reproduces it. We reused the 233 records generated directly from this stream in #575. Their producer was the unmodified recorder at `642be1bc3efc7c702773e85e8a3a696b97a1821d`. The relevant induction, recorder and writer are identical at the present study's base `82f2767d317aa9b359925866c257d8fa05714641`. The six prose records are from the data branch, with corpus SHA-256 `5dcc4817845a94d62556716f9a8305ee95bd3776653f0a48a20247cfb13d6357`. [Compressed-input recorder](https://github.com/lmmx/mibz2/blob/642be1bc3efc7c702773e85e8a3a696b97a1821d/examples/induction_trace.rs) · [Trace and output checks](https://github.com/lmmx/mibz2/blob/82f2767d317aa9b359925866c257d8fa05714641/src/itrace.rs)

```sh
wiki="$PWD/corpora/wikidata.bz2"
code=82f2767d317aa9b359925866c257d8fa05714641
git worktree add --detach "$tmp/kernel" "$code"
cargo build --manifest-path "$tmp/kernel/Cargo.toml" --target-dir "$tmp/target" \
  --release --locked --features induction-trace --example induction_trace
"$tmp/target/release/examples/induction_trace" "$wiki" "$tmp/all" > "$tmp/recorder.jsonl"
python3 "$study/census.py" "$tmp/all" "$tmp/prose" > "$tmp/blocks.csv"
python3 "$study/positions.py" "$tmp/all" "$tmp/prose" > "$tmp/schedules.csv"
cmp "$tmp/blocks.csv" "$study/blocks.csv"
cmp "$tmp/schedules.csv" "$study/schedules.csv"
python3 "$study/prices.py" --per-block > "$tmp/priced.csv"
```

[blocks.csv](blocks.csv) contains one row per input and includes the frozen block. The eager flag reconstructs the two density gates on the count of 127-capped run records and the $\lfloor4n/25\rfloor$ cell budget. All 233 reconstructed record counts, letter counts, row counts and completed step counts match the three archived i9 censuses used by `prices.py`. The lazy flag is the current recorded admission outcome. The older separately published trace of ordinal 144 has a different lazy admission bit. Its cut arrays agree. A forced trace for a rejected block prices the mathematical operation only. The 196 eager and 228 lazy populations overlap.

[schedules.csv](schedules.csv) contains the count and byte volume of each kind of output action and the number of sources at a later output address. An example is the first such command in execution order, recorded as (destination, source, length). The two evidence file digests are `aaff10011c57df6ed92a7b2139ee270acfe1d648f5b181bf1d65f93621b927e5` and `7d55e2dfc3af8fbc8d1669d05a0b8f4185a262fad6bd51d2f3506361d0eb9d77`.

## Independent checks inside the executables

- [census.py](census.py) uses exact short byte strings, unshared production descriptor identities and recursively interned tree identities to classify prior occurrences. The sets and tree dictionary exist only in this offline classification.
- [positions.py](positions.py) transcribes the ordinary short arena and stored rule pairs from each recorded grammar. The proposed schedule receives those ordinary descriptors, root, root rotation and period. Its optional comparison to the live cut types is an assertion on a decision already made by comparing neighboring descriptors. The baseline comparator uses least positions. The general overwrite comparator selects a reachable parent's offer. A single cyclic coordinate convention lets the comparator include marked rules explicitly.
- The oracle supplies already-known marked bytes. A coverage bitmap checks that every destination is within the period and filled exactly once, that every copied source is disjoint and complete at the instant of the copy, that the byte spans agree, and that the marked singleton lands one byte before the cut. Each whole-corpus row agrees exactly with the least-position schedule on literal and copy counts and byte volumes. The optional dense LF check reconstructs the source column from the initial sites, inverts its stable sort, and checks each expected byte and the return to the primary. It passed on all 16 published traces. The schedule has no entropy reader, production buffer path or timing harness.
- Every binary column through length 9 and every ternary column through length 6 is tested at every primary, including columns with multiple cycles. The test runs 14,209 cases with the shipped threshold of 16 and again with every nonliteral stored as a rule. Each suite has 2,017 full-period cases. The latter cutover exposes copy dependencies in small examples. The two explicit 35- and 36-row witnesses use the shipped threshold. All compare against dense LF. A missing owner in an unreachable component is expected only in the deliberate unguarded test.
- [boundaries.py](boundaries.py) executes predecessor forwarding in #575's lazy slot transducer on the ten representative Wikidata records, with an assertion on every bypassed field. Its frontier names, widths, read modes, closures, final word and mark agree with the recorder. In eager Cells the strict-cut writes and same-side link identity follow directly from the shipped source. Only the lazy slot version of this secondary candidate was executed.

## Baseline profile

Two instruction and data-reference profiles price the existing frozen-block writer in its eager and lazy callers. The measured program is the repository's unmodified `lever_counts` on an already-available production kernel. There is no candidate performance run.

```sh
python3 "$study/../2026-11-18-open-return/frozen.py" "$wiki" "$tmp/frozen.bz2"
cargo build --manifest-path "$tmp/kernel/Cargo.toml" --target-dir "$tmp/target" \
  --release --locked --features memory-lever --example lever_counts
for arm in cells lazy
do
  valgrind --tool=cachegrind --cache-sim=yes --I1=32768,8,64 --D1=32768,8,64 \
    --LL=20971520,20,64 --cachegrind-out-file="$tmp/$arm.cg" \
    "$tmp/target/release/examples/lever_counts" "$tmp/frozen.bz2" "$arm"
done
python3 "$study/prices.py" --profiles "$tmp/cells.cg" "$tmp/lazy.cg"
```

[reference-profile.csv](reference-profile.csv) gives the function-attributed counts. External memcpy/memmove and other callees are outside the function line totals. Cache simulation and target-machine DRAM traffic are different measurements. The i9 writer time shares use the ratio of summed row-weighted `c_write` to summed row-weighted `c_whole` on the eager-admitted population. [Historical measurement definitions](../monodromy/2026-11-10-rauzy-veech.md) · [Existing reference program](https://github.com/lmmx/mibz2/blob/82f2767d317aa9b359925866c257d8fa05714641/examples/lever_counts.rs)
