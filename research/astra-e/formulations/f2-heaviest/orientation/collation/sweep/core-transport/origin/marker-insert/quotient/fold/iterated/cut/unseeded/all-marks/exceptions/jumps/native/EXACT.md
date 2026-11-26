# Native physical LF jump inverse

The complete input consists of bzip2's capped last-column runs and the primary rank. The implementation constructs the physical context partition, per-component modal preceding bytes, sparse exceptional transitions, and the two retained bitmaps for one- and eight-byte jumps. It emits the marked pre-RLE1 bytes. [Index and emitter](jump.hpp) · [Run-only constructors](model.hpp) · [Mathematical proof](../PROOF.md)

For each physical component, the modal predecessor ranks land on a contiguous interval in a single poststate. The unit sweep discovers its base at the first matching run intersection. A stable per-letter cursor supplies LF destinations for all intersections. Each exceptional row writes its true target and packed poststate and byte into the exception stream in physical source order. Component-mode discovery guarantees at least one modal row in each component. At higher horizons a component can be empty. Its stored interval is a closed insertion boundary and the transition is read only at a set bit.

Doubling restricts each source word to the current component, counts its set bits, loads that many consecutive bits at the target and uses BMI2 PDEP to embed the result into the source word. Partial target reads cross at most one word boundary. Rank directories contain one 32-bit prefix count per 64 input rows at each layer. A global rank bias in an unsigned 32-bit word implements subtraction modulo 2³². All actual row positions are bounded by bzip2's 900,000 rows. Packed exception poststates have 24 bits. Prepacked eight-byte strings use x86 little-endian copies. An arbitrary marked column with a nondividing LF period uses the unit transitions to measure the period and repairs the output from its final lap.

The fixed partition uses the full capped run constructor at 3ddf456f. The variable partition uses the suffix-closed selection at 9e232479 with maximum depth 16 and minimum frequency 16. Modal bytes for each final physical component come from true run histograms. Both input paths coalesce capped runs, retain a full vector with capacity for the original caps, and build geometry and indices before emitting. The complete-arm measurement charges these allocations. The index has two 16-byte-per-component descriptor tables, two bitmaps with rank directories and eight bytes per exceptional row. Two intermediate horizon tables are built then released. The index-build diagnostic counts the active layer capacities plus exceptional rows. Empty placeholder layers can contribute another 16 bytes each. It excludes context acquisition, output, vector headers and allocator high water. Fresh-process memory and inclusive CPU are measured separately. [Preregistered benchmarks](bench/PLAN.md)

## Exact gates

The independent oracle expands each input into true rows, counts preceding letters to construct stable LF, inverts that permutation, and obtains the forward output using Ψ. Every row at horizons 1, 2, 4 and 8 must have the correct eligibility bit. Every set bit must produce the exact LF destination, target component, and complete spelling. The unit gate also checks each exceptional target, byte and poststate. On small columns it checks every primary mark. On larger random columns and actual blocks it checks the supplied primary and all physical rows' transitions.

The finite panel covers every ternary last column through length eight plus 380 seeded longer columns, in both partition modes. All 909,616 row/horizon checks and 148,384 complete marked outputs passed. Phase repair was required 75,202 times. For each of ten registered Wikidata blocks both variants passed every row/horizon check and matched all marked bytes. The true capped and independently derived raw SHA-256 values came from the [original extraction and libbzip2 receipt](../../../../../powers/xor/capped/receipt.json). [Finite receipt](small.txt) · [Real results](real.csv) · [Checks per file](gate.log)

| On 8,999,915 real rows | Fixed depth 8 | Variable depth ≤16 |
|---|---:|---:|
| Physical components | 815,844 | 184,004 |
| Exception rows | 409,432 | 818,537 |
| Eligible physical rows for an eight-byte jump | 6,528,221 | 6,607,841 |
| Eight-byte jumps actually taken | 915,165 | 904,708 |
| Share of emitted bytes using those jumps | 81.35% | 80.42% |
| Unit steps taken | 1,678,595 | 1,762,251 |
| Sum of per-block retained index capacities | 32,758,040 B | 15,812,000 B |
| Maximum retained index capacity on one block | 5,682,792 B | 2,343,944 B |
| Maximum active index capacity during build | 8,317,604 B | 2,914,148 B |

The physical eligibility fraction and the emitted-byte fraction weight rows differently. An eight-byte jump bypasses the next seven opportunities to start another jump. The ten real marks all completed at their start after n bytes, and each physical exception was visited once.

## Replay

Run from the repository root. The second gate uses the original donor's ten individually hashed capped and raw files.

```sh
d=$(dirname "$(rg --files | rg '/exceptions/jumps/native/jump.hpp$')")
python3 "$d/generate.py" /tmp/astra-physical-jumps
g++ -std=c++20 -Wall -Wextra -Werror -O3 -mbmi2 -mpopcnt -I/tmp/astra-physical-jumps "$d/gate.cpp" -o /tmp/astra-physical-jumps/gate
/tmp/astra-physical-jumps/gate 2>&1 | diff - "$d/small.txt"
python3 "$d/gate.py" /tmp/astra-physical-jumps/gate /tmp/astra-cut-caps | diff - "$d/real.csv"
```
