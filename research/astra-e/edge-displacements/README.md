# A smaller exact random-access residual successor at width 16

The band-to-queue construction stores the marked successor and its word label in **1.383 bytes per retained row** on the measured edge worker's 233 Wikidata admissions. It reduces the actual reused successor-plus-dictionary capacity from 1,585,440 to **614,472 bytes**. Median peak RSS across twelve fresh sequential single-worker processes fell from 19,816 to **18,570 KiB**. Its complete inverse took **1.8760× [1.8115, 1.9427]** the original thread CPU in a 24-round panel. This preserves a lower-memory implementation while leaving the measured i9 edge decoder as the speed incumbent. [Native measurements](summary.json) · [Scorer-facing entry at a098919](https://github.com/lmmx/mibz2/blob/a098919de8016b425fd7dd97186d1378c12eb41c/research/astra-e/edge-displacements/bucket/entry.rs)

The alternatives preserve every original marked hop and output width. Prior residual24 retained the complete successor and split high label bits across conditional page tables. Dictionary anchors changed the marked path and staged intervening literals. Reducing the original whole-run cutoff exchanged fewer edges for more endpoint and spelling work. This experiment keeps the i9 winner's cutoff and dictionary, and represents the actual function from physical row to next retained row by its local order. [Residual24 implementation and measured costs](../residual-24/README.md) · [Anchor study](../anchors/README.md) · [Width-cutoff experiment](../private-cache/README.md)

## Can each row encode a small displacement?

Let d(i) = next(i) − i. Over 26,567,581 marked edges, 99.20% of physical adjacent pairs have increasing next rank, while only 45.97% preserve the same displacement. An 8-bit signed displacement centered at zero covers 0.76% of rows. Even an optimally located 256-integer window for each terminal byte covers only 5.66%, and a separately chosen window for each full numeric word label covers 19.95%. At 12 bits the oracle per-label coverage is 29.64%. Locating these windows already consumes construction work. The measured zero-order entropies are 15.010 bits for d alone and 15.354 bits for the joint (label,d). They describe symbol frequencies without supplying row boundaries, a lookup dictionary or random access. [Full magnitude and entropy census](palette-summary.json)

A concrete two-byte representation uses some low bits for the residual within a fixed displacement bin and the other bits for an index of a frequent exact (label,bin) pair. Unrepresented pairs take the original four-byte edge, located by a 15-bit index in a 32,768-row page or by a bitmap and prefix rank. Every descriptor, escape and directory field is included. An oracle chooses the most frequent pairs after seeing the whole block. With eight-byte descriptors, even the best bit allocation per block averaged 4.581 bytes per row, with no block below four. Using the smaller packed descriptor width and choosing both the bit allocation and the directory type per block lowers this to 4.098. Some 116 blocks fall below four, but the best individual block still needs 3.660 bytes per row. These capacities already grant frequency discovery at zero cost. The bound is for these explicit formats. [Full original and compacted descriptor cases](palette_summary.py) · [Complete charged alternatives](palette-packed.csv)

## Two native representations

**Physical pages.** The first format fits successive next ranks in a 32-row physical page with a linear predictor and bounded exceptions. The independent numeric label uses a constant, palette or raw 16-row page. Both channels have addressable directories, and the block can use raw packed data as fallback. On these inputs 511,743 actual ranks are encoded as explicit exceptions, and 56,199 pages take a raw successor payload. The CPU must decode the next rank before it can locate the following page. The native constructor first creates the conventional packed table and then encodes it, retaining the capacity of that intermediate. [Exact page model, bounds and verified kernel](local/README.md)

**Band and monotone queue.** Let the original band start at physical d and consume the consecutive slice of the byte queue starting at k. Then next(i) = Q[k + i − d] throughout that band. Band boundaries in physical order, with a rank directory, identify the band of any i. Its four-byte descriptor recovers both its label and queue displacement. Groups of 32 queue entries use bounded Elias–Fano select with BMI2/POPCNT or a raw group. In all 233 blocks, 16,882 groups covering 539,733 rows took the raw fallback. The native constructor works directly from existing queues and band geometry. It uses no intermediate full successor table. The selected route pays a sequence of address-dependent reads for the band locator, descriptor, group header and low-rank payload. Other platforms and nonqualifying geometries take the original exact route. [Proof, eligibility, construction and native implementation](bucket/README.md)

| Native quantity | Measured edge | Physical P32 | Band/queue EF32 |
|---|---:|---:|---:|
| Initialized successor and numeric labels, all blocks | 106,270,324 B | 49,919,233 B | 36,745,030 B |
| Unchanged original spellings and offsets | 13,090,022 B | 13,090,022 B | 13,090,022 B |
| Complete initialized walk within 256 KiB | 0/233 | 145/233 | 203/233 |
| Complete initialized walk within 192 KiB | 0/233 | 6/233 | 80/233 |
| Maximum reused successor and dictionary capacity | 1,585,440 B | 2,240,936 B | 614,472 B |
| Single-worker peak RSS, median [min, max] KiB | 19,816 [19,792, 19,892] | 20,376 [20,308, 20,416] | 18,570 [18,512, 18,628] |
| Complete native inverse CPU / edge [95%] | 1 | 3.8901 [3.6938, 4.0968] | 1.8760 [1.8115, 1.9427] |
| Construction CPU / edge [95%] | 1 | 4.6542 [4.5652, 4.7451] | 1.1798 [1.1409, 1.2200] |
| Marked emission CPU / edge [95%] | 1 | 2.9463 [2.6580, 3.2659] | 2.7001 [2.5585, 2.8496] |

Both native layouts contain read guards in these numbers. Complete footprint counts include the unchanged spelling and offset arrays. They leave room usage by the output stream and other state unresolved, and cannot establish actual cache residency on the i9. The 203 complete EF32 fits cover 182,691,444 final output bytes and 80.0% of historical complete-block CPU. All alternatives preserve the same 4,894,668 nonliteral word visits and their 188,024,749 emitted coded bytes. [Per-block native sizes](checked.csv) · [Native code and output oracle](checked.txt) · [Exact word visits](../edge-spellings/README.md)

## Scope of verification and timing

The frozen control is `badb2aec3624bd65606c429b2e6f79e9d9d0050f`. All native successor and numeric-label pairs were checked against its actual packed data on every admitted row, including rows other than maximal-run starts. All 233 resulting coded byte streams matched the independent Segmented inverse, and their RLE1 expansions matched the corresponding libbzip2 oracle bytes and block CRCs. Together the 431 corpus blocks produced 209,700,196 oracle bytes and passed the twelve combined stream CRCs. Synthetic stable-LF checks covered 17,064 marked cases per kernel. Additional per-format controls cover partial pages, nonmonotone queue boundaries, multiple marked and unmarked cycles, an active compressed representation and wide-node fallback. [Native corpus gate](checked.txt) · [Marked adversarial tests](adversarial.txt) · [Page audit](local/audit.txt) · [Queue audit](bucket/marked.txt)

The direct band/queue worker at `a098919de8016b425fd7dd97186d1378c12eb41c` also passed the full refreshed Moonshot v1 correctness gate, **338 cases, zero failures**. The master path revision was `75297a1d57b2bdf262c3a19f3064848dedf2c493` and the scorer printed `v1 at 81585172f5fb9e7a59f330124779ba084fd6bf04`. The dedicated checkout generated its research kernel from the frozen i9 constructor. This run produced correctness evidence on one available CPU. [Authoritative receipt](bucket/official/score/summary.txt) · [Frozen entry and dependencies](bucket/official/manifest.txt)

The 17-block CPU panel includes the first large admission from each real stream and the five dense additional admissions. Across 24 paired rounds each arm occupied every position six times for each block. Times enclose paid admission, all construction and packing, and complete marked output. The identical-code twin's ratio was 0.9601 [0.9281, 0.9932]. The size of the two observed slowdowns exceeds this control variation. The local processor reports 2 MiB private L2 and a four-CPU-equivalent quota. Twelve fresh processes per arm independently ran a complete sequential single-worker decode under CRC checks to measure RSS. These results leave target W7–10 performance and memory unverified. [Preregistered panel](price-plan.md) · [All paired observations](paid.csv) · [Fresh-process RSS](memory.csv) · [Build manifest](manifest.txt)

## Reproduction

The included lossless tape reconstructs all 119,523,508 bytes of original packed tables and band records. It stores the original retained-queue byte indexed by successor rank, then restores each band from stable byte order. The SHA-256 of the exact restored tape is `670b97cbe1f6ba513df848ba68e428e9004a80bf74807254e507fb263ebf0697`. The same data can be rebuilt directly from the original corpus and pinned constructor. The ordinary scorer gate for the EF32 research entry can be run with its generated constructor in a clean checkout. [Tape receipt](restore.txt) · [Native input capture](dump.inc.rs)

```bash
d=research/astra-e/edge-displacements
out=/tmp/astra-edge-displacements
python3 "$d/build.py" "$out"
python3 "$d/tape.py" restore "$d/edges.select.xz" "$out/edges.bin"
"$out/dump" corpora/wikidata.bz2 "$out/regenerated.bin"
cmp "$out/edges.bin" "$out/regenerated.bin"
OPENBLAS_NUM_THREADS=1 python3 "$d/palettes.py" "$out/edges.bin" "$out/models"
python3 "$d/price.py" "$out"
bzip2 -dc corpora/wikidata.bz2 > "$out/wikidata.raw"
"$out/price" adversarial
"$out/price" check corpora/wikidata.bz2 "$out/wikidata.raw"
nice -n 16 taskset -c 3 "$out/price" price corpora/wikidata.bz2 "$out/wikidata.raw" 24 > "$out/paid.csv"
nice -n 16 taskset -c 1 python3 "$d/memory.py" "$out/price" corpora/wikidata.bz2 > "$out/memory.csv"
python3 "$d/results.py"
```
