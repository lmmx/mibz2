# Handing retained rank meetings to the token decoder

Every occurrence of a spelling shares the same forward node, crossing byte and backward node, because equal spellings take the same directional cohort steps and meet at the same depths. Conversely the triple reconstructs the exact forward path, crossing byte and reversed backward path. This gives a key for the spelling without comparing byte strings. The physical seam constructor from 81165dc yields these triples and affine source/destination bands in meeting order.

The writer interns each triple, spells it once, retains the minimum section source rank as its base, and processes the band labels in destination order. Either sort the small band array or scatter band indices at their destination starts in section-rank space and advance by band width. Consecutive equal IDs form maximal token-BWT runs. At each band of ID i it checks that the known true source equals base[i] plus the number of all prior occurrences of i in destination order. This certifies the full stable token LF permutation against actual rank partners. It charges both ID interning and ordering. For a section with disconnected buckets or nonstable owners, the certificate can decline. Disjoint marked cycles and declined objects use the original exact marked writer.

Native Compact uses a word ending with the next quote. If a section has a common first byte, moving that byte from the beginning to the end of every word gives that same convention. Locate the strictly preceding section rank under original LF and set skip to the number of steps minus one. For raw source-anchored words use the inclusively preceding section rank and set skip to the full number of steps. The second-level routine accepts either convention given its correct mark. A homogeneous token alphabet, a marked orbit avoiding the selected second marker, or an unrepresentable second packed link uses stable first-level links as an exact fallback. Word IDs are opaque. Measured source bases define the F-column order.

The FFI adapter supplies the committed Rust second induction and writer with the exact first-level root pieces, node bases, word slices and marked row/skip. It appends test interfaces inside candidate SHA-256 84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06 and gives the two-u32 Piece a C representation for the borrowed slices. The body of the second induction remains SHA-256 877dd1870572c4c5237cbdd1362b8d7124c4393abdcbcebb5194e1fadcffc992. The quote object matched Compact's independent mark/skip, every individual spelling, every source base and every maximal token-BWT root run, up to ID renaming, on all ten tapes. Both real section choices accepted, used second induction and matched all independent marked bytes. Native exhaustive binary columns through six rows, all nonempty sections and all marks checked 121,224 choices of section, orientation and ordering. This includes 23,350 token fallbacks and 61,080 object declines.

The earlier frozen F2 endpoint constructor inserted the primary into the section, ordered bands by entrance, and spelled 243,726 bytes over 5,009 entrance bands on its compound section. Here the mark/offset is recovered separately. Physical hits cover original section-to-section returns. Their destination ordering yields the token BWT after certification. Frozen quote has 6,133 hit bands and destination runs but only 2,716 unique words and 83,454 first-level spelling bytes. This experiment prepays all physical rank discovery and measures the remaining interchange and byte output. [Earlier endpoint study](../../../../../../f2-endpoint/README.md)

Twenty balanced five-arm repetitions per section on ten systematic blocks compared the same marked output and CPU. Paired observations sum all ten blocks in each repetition. Intervals are Student-t intervals on twenty log ratios. Old single-level writer receives already source-sorted bands. The new writers start with unsorted bands and retained spelling nodes. Both pay mark lookup, immutable dictionary assembly, token root publication, Rust first-level node adaptation, second induction, full n-byte output, and local scratch release. The second-only control receives the complete first-level object for free. The complete Compact control starts from its natural capped runs and frequencies. The 2,000 outputs all passed. These results do not price rank discovery, full worker decoding, or target-machine throughput.

| Stage / complete Compact | Quote [95% interval] | Frequent [95% interval] |
|---|---:|---:|
| Triple hash and destination sort + token decoder | 0.989 [0.939, 1.040] | 1.134 [0.919, 1.398] |
| Triple hash and destination scatter + token decoder | 0.921 [0.864, 0.982] | 0.959 [0.879, 1.046] |
| Already supplied token object, second decoder | 0.374 [0.336, 0.416] | 0.217 [0.201, 0.234] |
| Prior retained band writer | 1.457 [1.404, 1.512] | 0.906 [0.868, 0.946] |

The token interchange reduces the quote writer to 0.632 [0.596, 0.671] of the prior output method. The frequent comparison is unresolved at 1.059 [0.975, 1.150]. Prepaying discovery still consumes most of complete Compact time. The stronger single-interval source-bucket theorem suggests a more economical charged construction: rank the sparse source and destination band starts, group identical triples in source order and stream IDs in destination order, eliminating the hash and full destination-row scratch.

The CSV assembly memory covers actual standard-allocator C++ vector requests and growth while building the first token object; it excludes retained front geometry, the separate Rust second stage and its output. Frozen scatter peaks at 1,191,932 additional requested bytes on quote. Twenty samples per tape and family establish only the local stage envelope; there is no i9 or worker gain.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/seams/async/native/token-writer
bash "$d/build.sh" /tmp/astra-token-writer "$PWD/target/release/deps"
/tmp/astra-token-writer/writer
nice -n 10 taskset -c 4 python3 -B "$d/panel.py" /tmp/astra-token-writer/writer research/astra-e/dense/target/wikidata /tmp/token-writer.csv
python3 -B "$d/summary.py" /tmp/token-writer.csv
```

[Raw paired CPU and work](paired.csv) · [Paired ratios](summary.txt) · [Small adversarial checks](small.txt) · [Native writer](writer.cpp) · [Rust adapter](interface.rs)

The follow-up paid the complete original-rank producer and confirmed the conditional comparison in 60 additional repetitions. It verifies all 228 real admissions and the general single-rank-interval token theorem. [Paid result and reproduction](RANK.md) · [Single-interval proof](audit/README.md)
