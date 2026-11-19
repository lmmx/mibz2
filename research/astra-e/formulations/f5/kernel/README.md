# Native phrase-action decoder

`candidate.rs` uses the research-only final-output hook. Its admission and endpoint constructor match opend. Each dictionary entry gets a 20-byte action. Words without a count token borrow their encoded bytes. Other entries may keep a decoded spelling while the total number of live decoded dictionary bytes is at most one eighth of the block extent. A word that would exceed this budget uses the streaming scalar transition instead. The worker's previous vector capacity can persist across blocks. A clipped word and an unfinished count preserve the RLE1 contract. Marked reconstruction keeps the live parser state between cycles. The normal block CRC and rollback occur in the enclosing hook, followed by the normal combined stream CRC.

A separate Rust parser groups equal bytes into runs and checked the production action on 479,553 cases including every binary spelling of up to nine symbols, a foreign pending-count state, a zero decoded dictionary allowance, all prefixes and long periodic inputs. [Native checks](actions-check.txt)

All 228 admitted large Wikidata blocks confirmed exact agreement with opend's coded inverse followed by the stock RLE1 stage and the stored CRC, covering 205,197,794 coded and 205,191,352 decoded bytes. Frozen required 1,326 actions, 25 live decoded dictionary bytes, and 40,960 bytes of action vector capacity. Across blocks these respective maxima were 3,635 actions, 6,567 bytes and 81,920 bytes. [Raw census](geometry.csv)

Twenty paired, rotated rounds on the nine admitted systematic blocks measured from the capped input records through output CRC. In each complete repetition the sum of the input observations is paired. Compared with opend, F5 wall time was 1.210 [0.963,1.522] and thread CPU time 1.073 [1.015,1.134]. Shared-host scheduling widened wall uncertainty. This fixed-input stage comparison prices constructor, dictionary setup, traversal, output, RLE1 and CRC. It omits the entropy-decoding front and parallel workers. [Observations](phase-raw.csv) · [Phase summary](phase-summary.txt)

```sh
d=research/astra-e/formulations/f5/kernel
python3 "$d/prepare.py"
rustc --edition=2021 -O "$d/check_actions.rs" -o /tmp/check-f5-actions
/tmp/check-f5-actions
scratch=$(mktemp -d)
cargo build --manifest-path bench/moonshot/Cargo.toml --release --locked --target-dir "$scratch/build"
deps="$scratch/build/release/deps"
python3 "$d/phase.py" "$scratch/src"
rustc --edition=2021 -O -C codegen-units=1 "$scratch/src/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$scratch/probe"
"$scratch/probe" corpora/wikidata.bz2 "$scratch/raw.csv" > "$scratch/geometry.csv"
bash research/astra-e/score.sh phrase-action-repeat "$d/candidate.rs" --workers 1,4 --with-opend
```

A second 20-round run clocked construction and marked traversal separately on both arms. The nine-input sums of per-block medians were 14.55 ms for opend construction and 14.75 ms for F5. F5 booked another 0.560 ms for its dictionary. F5's 11.05 ms marked walk compares with opend's 9.62 ms walk and 0.985 ms RLE1 scan. The full thread CPU ratio was 1.053 [0.994,1.115]. These stage medians identify the cost of handling dictionary and literal boundaries on this implementation. [Second run](phase2-raw.csv) · [Complete accounting](phase2-summary.txt)

The candidate passed the official 338-case scorer gate. Eight standalone crafted and real blocks also forced the action route on scalar and detected kernels, checking append, libbzip2 bytes, CRC rollback and retry, including an unfinished periodic run. [Official gate](../../../results/v1-f5-action-gate/score/summary.txt) · [Forced-route checks](check_block.rs) · [Receipt](block-check.txt)
