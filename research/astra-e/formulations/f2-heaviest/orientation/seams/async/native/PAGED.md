# Paying for the bounded-rank predecessor

The two-sided ordered map can be replaced by a bounded-rank representation. A 64-bit mask and doubly linked ordered slot list represent the distinct start rows within each 64-row page. Higher layers mark nonempty pages or nonzero words in their child layer. Find the first or last eligible bit using these layers; find its packet by a popcount and traversal from the nearer page end. This takes at most 31 local links. An empty/nonempty transition changes at most one word at each layer. Every reused packet slot is removed from both local and cohort lists before publication. The underlying deque keeps addresses stable, so existing cohort links survive slot growth. A clipped packet's full interval is still stored once even if it covers many pages.

The maps now build both page tables, update masks, split opposite packets, rejoin eligible survivors and store two original source-run locator tables. Both experimental arms compile band spellings, scatter actual band IDs over section ranks and produce marked output through that direct index. This costs extra memory and replaces a binary search per emitted return. The matched controls reconstruct the same marked inverse from their natural run input. Backward Direct uses the actual registered section; Compact/preload uses its own native quote and second-induction policy for the complete inverse in both panels. Runs and frequencies are derived outside all timers. The 127-capped compact input and the two C++ run vectors are all resident in every process. Forced Compact invokes its exact candidate on every tape regardless of production routing. Every arm starts with fresh scratch and disposes of it inside the timing window while retaining only the issued byte buffer for comparison.

On frozen quote the mutable index replaces 2,424,040 comparison-tree key comparisons with 740,386 summary/page mask probes, 183,880 mask writes and 73,694 intra-page slot hops. Its peak full requested bytes grew from 5,799,157 to 6,573,509, including indexed marked publication. The diagnostic mask-probe counter covers searches in the page bitset and hierarchy. Slot popcounts also access the selected page mask. The memory instrumentation forwards to the normal standard allocator. It counts requested map, vector, and deque storage including growth transients; scoped verification and immutable caller input sit outside that count. Fresh-process high water provides a separate check that includes allocator overhead, executable and the input/output adapters.

Eight balanced four-arm paired repetitions on each of ten systematic tapes used a fixed local CPU, charged original translation construction, front updates, output indexing, marked output and scratch release. Each paired observation sums ten blocks. Intervals are geometric paired ratios with Student-t intervals on the eight log ratios. All 640 timed outputs plus the 80 isolated high-water outputs matched their independently expanded marked data. These are local inverse-stage CPU measurements. They do not establish worker throughput or i9 behavior.

| Complete marked stage | Quote ratio [95% interval] | Frequent ratio [95% interval] |
|---|---:|---:|
| Page index / comparison tree | 0.636 [0.574, 0.705] | 0.662 [0.601, 0.728] |
| Page index / native Direct | 6.626 [5.921, 7.415] | 7.528 [6.846, 8.278] |
| Page index / Compact/preload | 18.21 [14.13, 23.47] | 21.37 [18.96, 24.08] |
| Indexed publication and output alone / Direct | 0.630 [0.550, 0.721] | 0.466 [0.416, 0.523] |
| Indexed publication and output alone / Compact/preload | 1.730 [1.346, 2.224] | 1.324 [1.115, 1.572] |

Freeing the entire ownership discovery, priority queue and original edge construction still leaves this publication/output implementation slower than complete Compact. This bound is tied to the measured indexed writer's own work. Alternative publication or reconstruction could change it. Comparisons with Direct leave positive construction budgets after granting the existing writer for free at discovery. The complete paid paged constructor exceeds them substantially. Peak process VmHWM ranges across blocks extend to 17,420 KiB (quote) and 21,068 KiB (frequent); Direct extends to 9,868 and 9,728 KiB. The reduced tree work does not suffice for a whole-worker or workstation candidate.

To reproduce, start at the repository root with its Rust toolchain, release mibz2 library (with benchmark internals), a C++20 compiler, and Python with SciPy. The library exports a forced Compact/preload call by appending only the FFI adapter to candidate SHA-256 84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06. Its second stage is 877dd1870572c4c5237cbdd1362b8d7124c4393abdcbcebb5194e1fadcffc992 and native Direct is ca83b7e86111c055f7ed55b25e1e476cb2caa13425a2eb1cbec910877eb4e953.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/seams/async/native
bash "$d/build.sh" /tmp/astra-async-build "$PWD/target/release/deps"
/tmp/astra-async-build/check
python3 -B "$d/real.py" /tmp/astra-async-build/check research/astra-e/dense/target/wikidata /tmp/astra-async-check
nice -n 10 taskset -c 4 python3 -B "$d/match.py" /tmp/astra-async-build/match research/astra-e/dense/target/wikidata /tmp/astra-async-panel
python3 -B "$d/summary.py" /tmp/astra-async-panel
```

[Paired observations](panel/paired.csv) · [Fresh-process peaks](panel/fresh.csv) · [Paired summaries](panel/summary.txt) · [Small permutation and marked checks](paged-small.txt)
