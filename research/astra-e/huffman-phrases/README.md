# Ten-bit Huffman phrases in the complete scanner

The historical two-symbol scanner made 13,707,325 table lookups on Wikidata. A four-symbol scanner with the **same 32-bit entry and 1,024 slots per group** made 12,034,635, a 12.20% reduction. With ordinary conditional output stores it consumed 1.5160 [1.4447, 1.5909] times the stock scanner’s thread CPU. An x86 BMI2 scatter store brought the complete scanner to **0.9976 [0.9815, 1.0139]** of stock in a fresh 96-round comparison. Wall time on that Xeon was 1.0018 [0.9833, 1.0207]. There is no verified target-machine gain from a phrase scanner. [Exact per-block counts](census.csv) · [96-round observations](scatter/confirm.csv) · [Ratios](scatter/confirm-contrasts.csv)

All paid scanner arms ran `Scanner::next` from header to final stream check, repeatedly reusing the original symbol Vec. They charged selector parsing, code lengths, canonical limits/perm, LUT construction, bit refills, body and tail decoding, symbol stores, EOB, item boundaries and concatenated streams. The scanner consumes each stored block CRC for its stream check. The subsequent worker computes the block CRC on expanded output. The corpus gate compared all 24,979,299 symbols, compressed bit positions, per-block metadata and CRC fields across all 431 blocks against the frozen 0.5.6 scanner. Each treatment passed the historical 4,000 generated item comparisons, including oversubscribed code lengths, long codes, corrupt data, a selector overrun, truncation and successful item results with the reader’s overrun flag set. [Native checking harness](native_driver.rs) · [Whole-corpus receipt](exact.txt) · [Malformed cases](synthetic.txt)

## Construction and lookup costs

| Lookup scheme | Total lookup invocations | Four-step body | Refilled tail | Single-code tail | Static triple / quadruple LUT entries |
|---|---:|---:|---:|---:|---:|
| Stock pair | 13,707,325 | 12,615,563 | 872,407 | 219,355 | 0 / 0 |
| Nine-bit first ID, up to three | 12,241,780 | 10,105,996 | 1,417,123 | 718,661 | 646,645 / 0 |
| Nine-bit first ID, up to four | 12,034,635 | 8,986,427 | 1,940,832 | 1,107,376 | 364,312 / 282,333 |
| Up to four with balanced small IDs | 12,033,378 | 8,985,579 | 1,940,484 | 1,107,315 | 362,841 / 283,804 |
| Up to four without ID payload limits | 12,033,378 | 8,985,579 | 1,940,484 | 1,107,315 | 362,833 / 283,812 |

The real stream uses 499,822 selector groups and constructs 1,796 tables. Every row still takes 170,331 long-code paths. Lookup frequencies are weighted by actual use; the static-entry column counts constructed slots, including slots that are never looked up. The unbounded row gives a ceiling for this fixed greedy body and tail policy. Retaining a nine-bit first ID costs only 1,257 extra complete lookups compared with the unbounded row. Excising the payload constraint alone has little headroom here. [Table census](totals.csv) · [Packed layout and exactness proof](PROOF.md)

In the first 24 rounds, the triple’s scanner CPU ratio was 1.3588 [1.2975, 1.4231], the quadruple’s 1.5160 [1.4447, 1.5909], and the same-source twin’s 1.0158 [0.9761, 1.0572]. Separately timed table construction took a median 6.82 ms per corpus on stock and 13.83 ms on the branch-based quadruple. Outside this region, the latter still took 1.4081 [1.3184, 1.5039] of stock time. Each attributed invocation includes two thread-clock reads per table. Absolute stage differences across binaries or run epochs do not decompose a particular uninstrumented wall observation. [Paid raw rows](times.csv) · [Original contrasts](contrasts.csv) · [Setup attribution](table-price/contrasts.csv)

The BMI2 version uses four arity-specific bit masks. One `pdep` deposits the payload into the low bits of up to four independent `u16` slots; one unaligned store writes the slots. The ordinary body runs only when four group positions are available. The scanner advances the already-reserved Vec by the actual arity. The historical selector-tail path still handles the final one to three positions. The directly matched branch-based quadruple cost 1.4641 [1.4405, 1.4882] times scatter’s CPU in the 96-round panel. In a separately metered run, scatter’s nonconstructor CPU was 0.9130 [0.8561, 0.9737] of stock. That subphase improvement did not establish a complete-scanner win. [Store and CPU instruction](scatter/) · [Separate attribution](interval/attribution/contrasts.csv)

## Directed follow-ups

* **Fill dyadic prefix intervals.** A construction with an explicit decision depth reproduced every one of the 1,839,104 packed input-table entries using 1,284,790 classification intervals. Its table time was 1.3554 [1.3143, 1.3977] times the original scatter constructor despite doing 30.1% fewer classifications. Its complete scanner-to-stock ratio was 1.0325 [1.0117, 1.0538]. [Invariant and source](interval/) · [Direct setup price](interval/attribution/contrasts.csv)
* **Consume an actually fitting phrase at a selector tail.** This saved 316,367 more lookups, bringing the total to 11,718,268, 14.51% below stock. It required additional classification and exact short stores. Its 48-round CPU ratio was 1.0561 [1.0136, 1.1004] against original scatter and 1.0340 [0.9872, 1.0829] against stock. The table lookup count alone ranked this treatment in the opposite direction to its directly measured cost. [Actual tail counts](tail/count.txt) · [Paired results](tail/contrasts.csv)
* **Choose table setup by the declared selector budget.** At most 64 declared selectors selected 198 blocks with 398 tables and only 8,295 symbols. For these blocks, skipping three/four-symbol promotion but still building full pairs produced 1.0132 [0.9842, 1.0431] of stock CPU. Retaining only the complete single-code LUT on those blocks took 0.9577 [0.9240, 0.9926] of original scatter, 0.9746 [0.9501, 0.9998] of the small-pair variant, and 0.9875 [0.9658, 1.0097] of stock. This is a measured incremental setup improvement on the original phrase scanner. [Raw selector use](selectors/usage.csv) · [40-round comparisons](selectors/contrasts.csv)

* **Omit never-selected tables.** Exactly 190 of 1,796 declared tables were absent even from the full encoded selector list. All code lengths still undergo their original validation. The scanner can omit each unreachable group’s complete canonical/LUT object and map active selectors compactly before symbol decoding. The 80-round stock-skip/stock CPU ratio was 0.9880 [0.9661, 1.0104]. Adding the same elision to the small-singleton phrase worker gave 1.0090 [0.9895, 1.0288] against that worker. This rules out a clear additional gain from elision in the measured setup. The peak requested group capacity within these particular small blocks can decrease. The six-group maximum over the full corpus remains. No process RSS measurement enters this stage. [Reachability and registration](unused/PLAN.md) · [80-round raw rows](unused/times.csv) · [Full contrasts](unused/contrasts.csv)

The strongest measured stage candidate was the small-singleton phrase scanner. In the final panel it consumed 0.9652 [0.9458, 0.9850] of the original stock scanner’s CPU seconds and 0.9774 [0.9533, 1.0021] of its same-source twin. Its preceding 40-round stock interval crossed 1. The final panel ran under contention, with median whole-corpus stock thread CPU and elapsed wall of 78.9 and 237.8 ms. Code layout, concurrent work and thermal conditions can change close comparisons. The best-preserved constructor is at `78c2fa6`, generated by `selectors/build.py` as `singletiny.rs`. The unused-group treatment is separately recoverable and has a smaller per-tiny-header group allocation. In the subsequent parallel-decoder experiment, the unchanged scalar-cap worker and a single scanner-source difference gave complete-wall ratios of 0.9441 [0.8023, 1.1108] at W1 and 1.0122 [0.9513, 1.0770] at W4 across 32 paired rounds. Each complete executable passed all 338 cases in the scorer gate. The independent i9 effect is unresolved. [Matched whole-worker measurements and cross-built release references](integrated/README.md)

## Scope and reproduction

The i9’s measured Lazy scanner CPU/wall was .773 at W9 and .778 at W10 in the specific older Wikidata×4 run. Scanning overlaps workers, input and output. These ratios describe role occupancy. They cannot convert a Xeon thread-CPU stage ratio to a target-wall gain. The earlier independent i9 parallel-scan study reported a 0.981 find/scan rate ratio on the level-9 Wikidata corpus at W10. The register-resident study had already moved the hot bit cursor out of spills. All scanner copies here retain that normal producer and use its baseline source commit `530ed404a014a414ef0b62adfd947fa14d4533e6`. The treatments are research-only generated modules. None has independent whole-decoder CI or i9 measurement in this study. [Role measurement](../budget/target.md) · [Historical scanner experiment](../../../docs/journal/2026-11-05-huffman-register.md) · [i9 parallel-scan result](../../../docs/journal/2026-11-10-par-scan-result.md)

Use Rust 1.99.1 on x86-64. BMI2 is required for the scatter descendants. The canonical corpus SHA-256, toolchain, initial executable hashes and link addresses are recorded. The native measurements used CPU1 at nice +12 on a shared four-equivalent Xeon Platinum 8573C. The first structural census ran at nice +16. The scanner-stage panels used complete forward/reverse position-balanced rotations and two-sided paired t intervals on log ratios within each run epoch. Each row records both thread CPU and elapsed wall time, including header and symbol work. Complete-worker output and process peak RSS are reported in the separate integrated experiment. [Environment](environment.txt)

```bash
d=research/astra-e/huffman-phrases
b=/tmp/astra-huffman-replay
input=corpora/wikidata.bz2
python3 -B "$d/build.py" census "$b/census"
"$b/census/census" "$input" > "$b/census.csv"
python3 -B "$d/summarize.py" "$b"
python3 -B "$d/build.py" native "$b/native"
python3 -B "$d/scatter/build.py" "$b/native" "$b/scatter"
python3 -B "$d/table-price/build.py" "$b/native" "$b/table-price"
python3 -B "$d/interval/build.py" "$b/scatter" "$b/interval"
python3 -B "$d/table-price/build.py" "$b/interval" "$b/interval-price"
python3 -B "$d/tail/build.py" "$b/scatter" "$b/tail"
python3 -B "$d/selectors/build.py" "$b/scatter" "$b/selectors"
python3 -B "$d/unused/build.py" "$b/selectors" "$b/unused"
"$b/interval/audit/native" tables "$input"
"$b/tail/audit/native" count "$input"
"$b/selectors/audit/native" usage "$input" > "$b/usage.csv"
for a in native scatter interval tail selectors unused; do
    "$b/$a/native" exact "$input"
    "$b/$a/native" synthetic 4000
done
taskset -c 1 nice -n 12 "$b/scatter/native" time "$input" 96 > "$b/confirm.csv"
python3 -B "$d/report.py" "$b/confirm.csv"
```

The `time` subcommand takes the corpus and a round count. The ratio reporter uses the identical `paired_ratio` source blob `237a4fbedddda74c9a1ad7e0fb053f63390d6f8a` found at scorer path `75297a1d`. These are standalone scanner measurements. The official inverse-kernel scoring interface cannot substitute a scanner. The branch-based, scatter, constructor-attribution and interval screening panels used 24 rounds. The tail panel used 48, selector-budget panel 40, and unused-group panel 80. The native binary rebuilds scanners and their tables at every sample and includes no preconstructed table cache.
