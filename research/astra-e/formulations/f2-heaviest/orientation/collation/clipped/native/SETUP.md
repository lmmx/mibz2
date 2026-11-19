# Two endpoint cuts with Compact's named-section cost

For U equal to a whole F byte bucket, every L run is wholly terminal or wholly live. The second setup variant takes this branch once per block, tags each newly encountered maximal run by its original byte and leaves artificial cap extension untouched. It never searches for a split in this case. For a partial F bucket or a section spanning multiple buckets, it groups capped copies of an original L run first. It emits its at most three uniformly tagged translation pieces, using the two endpoint offsets in the complete run image. These setups yield identical flagged LF tables. They use the same one-comparison unary loop and exact terminal publication as the original clipped implementation.

Both constructors now have exact paid stage and reserved-buffer receipts on ten systematic inputs, covering setup/mark, discovery, stable merge and total inverse. Reusable scratch stays with its arm and block through forty eight-arm balanced repetitions. Each paired observation sums the ten blocks; intervals use their forty log ratios. The registered free section choice excludes any data-dependent selector search. The generic CLI accepts any additional frozen interval per block as CSV with columns stream,lo,hi. The original and clipped arms start from the same counted capped input and marked output. All 3,200 timed outputs matched their independent stable-sort reference; each block/arm also passed a complete first-object and mark comparison before timing. [Raw interleaved samples](setup-phase.csv)

| Complete local inverse comparison | Ratio [95% interval] |
|---|---:|
| Tagged whole-byte quote / first clipped quote | 0.9424 [0.9116, 0.9743] |
| Tagged whole-byte quote / original Compact quote | 0.9851 [0.9548, 1.0163] |
| Grouped-run frequent / first clipped frequent | 0.9723 [0.9145, 1.0337] |
| Grouped-run frequent / original Compact quote | 1.0481 [1.0087, 1.0891] |
| Grouped-run fifth / first clipped fifth | 0.9925 [0.9512, 1.0356] |

The whole-byte specialization cuts quote setup and mark transfer to 0.7296 [0.6962, 0.7647] of the first clipped implementation. On the original Compact comparison, no complete speed difference is resolved. Their retained capacities including the output match on every systematic quote tape. Grouping original caps before splitting reduces setup and mark time on frequent and fifth to 0.9226 and 0.8662 of the first implementation. The resulting full-stage comparisons remain unresolved on a matched section. [All ratios and phase intervals](setup-summary.txt)

The new and previous constructors agree with every dense first object, mark and emitted byte on all five policies of all 228 admissions. Both pass the same 19,493 arbitrary tiny intervals and 132,098 marked outputs. The small comparison also checks equality of the two exact mixed-branch and physical-split counts for each outcome. All 40 systematic geometries match the independent Python formulation. No worker default, allocator policy or shipping decoder changed. [Full block verification and capacities](fast-admissions.csv) · [Ten registered tape checks](fast-ten.csv) · [Small exact receipts](fast-small.txt)

The named branch has no extra first/second vector capacity. For a general multi-byte section the native adapter additionally retains one base per discovered first-level word. Other memory differences chiefly reflect the sizes of the selected first/second dictionaries and root streams. Per-block reserved bytes include the state vectors and marked output; they exclude allocator metadata, stack, caller input and transient reallocation overlap. The fresh-process companion checks resident high water separately with one scratch switched between monomorphic terminal-byte specializations.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/clipped/native
out=/tmp/astra-clipped
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
"$out/check"
python3 -B "$d/tapes.py" research/astra-e/dense/target/wikidata "$out/ten.bin"
"$out/check" fast_verify "$out/ten.bin"
nice -n 10 taskset -c 4 "$out/profile" price "$out/ten.bin" 40 > "$out/setup.csv"
python3 -B "$d/setup-report.py" "$out/setup.csv"
```

To substitute a preregistered interval, pass its CSV path as the final argument to `profile price`. The fifth section is replaced by that interval in both implementations and checked, leaving the named controls in the same run. This is an inverse-stage experiment and has no CI or independent i9 result.

A fresh-process repeat processed all 228 admissions consecutively per arm, preserving one reusable scratch across changes of starting F symbol. The shared adapter specializes methods at compile time on one common state. The original control and tagged whole-byte quote arm reached **8,036 KiB process VmHWM** and **8,473,542 retained bytes** apiece. Their buffer capacities agree after **all 228 blocks**. Frequent reached 8,440 KiB and 8,528,726 bytes; fifth reached 10,540 KiB and 11,650,874 bytes; half reached 9,416 KiB and 9,512,582 bytes. These are isolated one-process inverse measurements in this environment. They do not establish i9 peak memory at 7–10 workers.

The memory process reads original capped records and checks the emitted marked bytes against a per-stream checksum generated by an independent stable-sort walker in a separate process. The reference-checksum CSV SHA-256 is 371c94a260c2f9a01cca5580cbee43ac687cc7b8d1a2b1a3d3e8b295c4131c10. The separate full-object receipts check the exact bytes directly. [Per-stream high-water records](rss.csv) · [Capacity and process maxima](rss-summary.txt)

```sh
bash "$d/memory.sh" "$out" /tmp/astra-token-writer/admissions.bin 3
python3 -B "$d/rss-report.py" "$out"
```
