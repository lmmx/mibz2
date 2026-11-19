# Paid native stage for the projected rank universe

Each timed call starts with the original capped runs, counts, immutable named section U and mark. It constructs the requested levels V1 through Vh, sorts their union F, performs early-root intersection, rebuilds signed/terminal LF descriptors and popcount index for the compressed universe, transfers the mark, discovers and spells all first words, merges the destination column, runs the existing second induction, and emits n verified bytes. The level builder uses a stable original-cap scan for each common prefix byte. The packed LF builder sweeps original caps and both source/arrival component boundaries. All these costs appear in setup/mark. The experiment gives every arm U without charging any data-dependent choice among candidate intervals.

The two native controls use the frozen Compact/preload implementation on quote, and the complete fast clipped Compact implementation on the same named frequent interval. The latter uses its one-bucket specialization with trailing-marker words. Projected h0 instead uses source-oriented raw words on U. For h>0 the common prefix moves from front to back while the compressed occurrence permutation and source bases are preserved. Small or periodic returns rotate modulo their exact length.

## Local paired measurement

Thirty-six balanced six-arm orderings on ten archived Wikidata blocks produced 2,160 complete outputs. Each ratio sums the ten paired thread-CPU samples per repetition before taking logs, with a Student-t interval on the 36 log ratios. Scratch remains with its arm during repetitions on a block. No sample was discarded. [Raw interleaved phase times](phase.csv) · [All ratios and phases](summary.txt)

| Native stage | Total / original Compact quote | Total / clipped Compact frequent | Discovery only / clipped Compact frequent |
|---|---:|---:|---:|
| Projected h0 | 1.088 [1.031, 1.148] | 1.010 [0.948, 1.075] | 0.947 [0.876, 1.024] |
| Projected h1 | 1.320 [1.258, 1.386] | 1.226 [1.140, 1.319] | 0.983 [0.901, 1.072] |
| Projected h2 | 1.283 [1.221, 1.348] | 1.191 [1.127, 1.259] | 0.950 [0.878, 1.029] |
| Projected h3 | 1.313 [1.249, 1.379] | 1.219 [1.141, 1.302] | 0.980 [0.904, 1.062] |

H3 reduces the frozen uncontracted live states from 100,644 to 91,846 and has 50,741 total compressed LF descriptors. It still spends 2.394 [2.246, 2.551] times the named control's setup and mark-transfer CPU. Discovery alone has no resolved advantage under this comparison. For the altered h3 and unshifted h0 both using the new native code, h3/h0 is 1.207 [1.170, 1.245].

An optimistic per-repetition substitution replaces the **entire** h3 setup/mark time with the control's measured setup/mark time, keeping actual discovery, merge and output. Its h3 / clipped Compact frequent ratio is 0.9735 [0.9172, 1.0332]. Giving h3 every setup and mark operation for free yields 0.7950 [0.7420, 0.8519]. These are accounting counterfactuals; they are not executable gains. The results do not support proposing a queue or LF-index timing change as the next milestone.

## All 228 admissions

Every admission supports every shift h=0,1,2,3. All 912 objects matched a separately stable-sorted original-rank walk: complete first word bytes, source bases, maximal incoming token runs, marked row and skipped bytes. Each object proceeded through the unchanged second induction and matched the entire independently reconstructed marked byte stream. A second build with debug assertions verifies every contiguous-child append and component exclusion. It also checks that each dictionary entry is exactly the corresponding raw h0 spelling rotated by h modulo its length, with the same source bucket and destination base/width sequence. The unmodified dense projector checks were repeated on 143,450 tiny marked cases, including genuinely empty fused roots and disjoint cycles. [All per-shift objects](admissions.csv) · [Debug conjugacy run](conjugacy.log) · [Exhaustive checks](small.txt)

| Total over 228 admissions | h0 | h1 | h2 | h3 |
|---|---:|---:|---:|---:|
| Live first-level spelling states | 28,053,831 | 27,247,807 | 26,441,791 | 25,651,055 |
| Compressed LF descriptors | 14,427,375 | 14,427,137 | 14,410,618 | 14,287,780 |
| Ranks deleted from the LF coordinate | 0 | 5,566,401 | 11,132,802 | 16,699,202 |
| Early-return destination rows | 0 | 0 | 1 | 2 |

Each column has the same 806,027 sum of per-block distinct word counts, 41,433,719 distinct first spelling bytes and 1,433,311 maximal incoming token runs. Stream 147 contributes an early return at h2 and h3. Stream 321 contributes the other at h3. The ten systematic inputs had none, so the larger corpus exercises the paid splice in real data. [Independent component, descriptor bounds and sharp families](audit/README.md)

Fresh processes decoded all 228 admissions sequentially with a reusable single worker scratch. Each marked output matched a per-stream FNV computed by a separate dense stable-sort process. Per-block exact byte comparisons precede this memory-only run. Maximum reserved bytes include marked output and live vector capacities, but exclude allocator metadata, growth transients and immutable caller caps. Process VmHWM supplies the separate process-level observation.

| Arm | Maximum retained bytes including output | Maximum process VmHWM, KiB |
|---|---:|---:|
| Original Compact quote | 8,473,542 | 8,212 |
| Clipped Compact frequent | 8,528,726 | 8,400 |
| Projected h0 | 8,561,670 | 8,440 |
| Projected h1 | 8,520,958 | 8,596 |
| Projected h2 | 8,520,054 | 8,536 |
| Projected h3 | 8,579,078 | 8,596 |

[Per-stream memory receipts](rss.csv) · [Fresh-process maxima](rss-summary.txt)

The rank projection gives a complete constructor whose lane cost does not grow with holes, including arbitrary early overlap. It reduces the encoded original LF descriptor count for h3, and it preserves the token object across every verified shift. Its paid native inverse loses against both production-relevant local controls. No CI or i9 run was requested. A different first-object producer would have to reduce the charged construction or the actual downstream discovery, merge or publication work.

## Reproduction

Use [the source check and build](README.md). The 10 systematic tapes, original 228-admission exporter and independent FNV generator are the same as in the clipped stage. The SHA-256 of the original admissions binary is da0529bf4e73b906fd997c4be87238a892f1feaaa3f549a729c6dd7584cecf26; the independent reference FNV CSV is 371c94a260c2f9a01cca5580cbee43ac687cc7b8d1a2b1a3d3e8b295c4131c10.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/shift-compact
out=/tmp/astra-shift-rust
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
python3 -B "$d/../../clipped/native/tapes.py" research/astra-e/dense/target/wikidata "$out/ten.bin"
"$out/check" check /tmp/astra-token-writer/admissions.bin > "$out/admissions.csv"
nice -n 10 taskset -c 3 "$out/price" price "$out/ten.bin" 36 > "$out/phase.csv"
python3 -B "$d/report.py" "$out/phase.csv"
bash "$d/memory.sh" "$out" /tmp/astra-token-writer/admissions.bin /tmp/astra-clipped/expected.csv 2
```
