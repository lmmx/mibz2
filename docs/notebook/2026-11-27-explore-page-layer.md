# Explore: the page layer (THP, faults, allocator churn) under the current worker stack

27 Nov 2026, `explore`, source `66511da9c3b8` (master), VM (Xeon @ 2.30 GHz, 4 vCPU, no HW PMU,
THP `madvise`/defrag `madvise`, glibc 2.39), `corpora/wikidata.bz2` (sha256 `82f65ace0731`)
and its x4 concatenation (47.4 MB in, 838.8 MB out). Scripts and raw A/B:
`docs/journal/2026-11-27-explore/page-layer/`.

**Why off the map.** The only prior look at huge pages was the 5 Nov binding census
(`docs/journal/2026-11-05-binding-census/nogo.md`: "NOGO Huge pages … authority-gated,
0 to -3% at 4t, sign unknowable without counters"), on the pre-Open T-vector walk, and the
Oct compute-frontier kernel probe ("no change"). Design is now delegated (C6), the production
worker is `OpenCells`, and the cheap zero-code A/B (`GLIBC_TUNABLES=glibc.malloc.hugetlb=1`,
which `madvise(MADV_HUGEPAGE)`s malloc's heaps and mmapped chunks) was never run. Nobody had
counted faults, `mprotect`s or kernel share for the shipped decoder either.

No decision rule was frozen before the data: this is a characterisation, and its outcome is a
scoped STOP plus supporting evidence for an existing claim, not an adoption.

## 1. Where kernel time goes (perf `cpu-clock`, `examples/bzcat`, x4)

| t | kernel share | `memmove` (libc) | page-fault entry | sched/wakeup (irqrestore + task switch) |
|--:|--:|--:|--:|--:|
| 1 | 5.4% | 9.3% | 0.34% | 2.7% |
| 4 | 6.3% | 9.7% | 0.81% | 1.5% |

- `memmove` is ~9.5% of all CPU and ~8.1/9.4 points of it are under `CellsKernel::invert_runs`
  (fp call graph): the segment stitch. This is the known stitch cost (`#1058` vectored delivery,
  `#1045`), not new; it is visible here because t <= 6 on a 4-vCPU box plans the `Cells` kernel.
- The fault path itself is <= 0.8% of samples. Wakeups/futex (~6,000 futex calls per x4 decode,
  ~3.5 per block) are ~1.5-2.7%: small, and partly a VM artefact.

## 2. Faults and `mprotect` by thread count (strace -c, perf stat page-faults, x4)

| t | kernel used | mprotect | page faults (x 4 KiB) |
|--:|---|--:|--:|
| 1 | Cells | 339 | 19,458 (76 MB) |
| 2 | Cells | 1,321 | 27,247 |
| 4 | Cells | 2,347 | 41,725 (163 MB) |
| 6 | Cells | 3,640 | 50,478 |
| 8 | OpenCells | 103 | 18,162 (71 MB) |
| 10 | OpenCells | 129 | 20,051 |

- The `mprotect`s are one-page `grow_heap` calls in glibc's per-thread arenas; gdb (frame-pointer
  build) puts ~97% of them under `room_segments<32,1,true>` in `walk_back_room_with_kernels`:
  the `Cells` route's per-segment `Vec<u8>` buffers (`s.bufs`, `reserve(2n/ns+64)`, doubling in
  `grow`). `glibc.malloc.trim_threshold=4G` does not remove them (3,193), so they are growth,
  not shrink-regrow churn.
- On the production `OpenCells` route (t >= 8) both vanish: 103 `mprotect`s and ~18k faults for
  839 MB of output. The page layer is not a cost there.
- The 4-6T `Cells` route faults 2.2-2.8x what 8T does. That is more evidence for
  `plan-schedule-and-lazy-threshold` (move t = 4..7 to `OpenCells`; stale `LAZY_FROM_THREADS`), not
  a separate lever.

## 3. THP A/B (`GLIBC_TUNABLES=glibc.malloc.hugetlb=1` vs default)

- Takes effect: 10 MiB of 17.7 MiB RSS on `AnonHugePages` in `station_balance`; faults fall
  43-57% at 2-4T (27.6k -> 11.9k; 40.8k -> 23.5k).
- **Single thread, pinned, warm, `OpenCells` (`station_balance`, 10 interleaved pairs x 5 passes):**
  W `342.8 -> 343.9 ms`, paired ratio median `1.0044` [min 0.981, max 1.027]; S `47.55 -> 47.59 ms`,
  `1.0035` [0.982, 1.029]. Null.
- **Whole decoder (bzcat, 8 interleaved pairs):** 2T CPU `2077.7 -> 2013.0 ms` (-3.1%, sys
  `125 -> 90 ms`), wall `-1.9%`; 4T CPU `1867.4 -> 1872.9 ms` (+0.3%), wall `+0.2%`. Within VM noise
  at 4T; the 2T gain is in sys time on the `Cells` route only.
- **RSS (3 runs each, `ru_maxrss` median):** 1T `58.8 -> 60.2 MiB` (1.024x), 2T `76.7 -> 82.3`
  (1.073x), 4T `104.4 -> 109.6` (1.050x), 8T `69.8 -> 71.1` (1.019x). At 8T this breaches the
  `<= 1.01x` 8W RSS invariant (`#1053`'s L3 law) for no measured CPU gain.

## Verdict

STOP, scoped: process-wide THP (`madvise` via glibc's `hugetlb=1`, or a hand-placed
`MADV_HUGEPAGE` on worker scratch) is not a lever on the shipped `OpenCells` worker: W and S are
null single-threaded, the production route has no fault or arena churn to remove, and RSS rises
1.9% at 8T. The page layer's only measurable cost lives on the `Cells` route at t <= 6, which
`plan-schedule-and-lazy-threshold` is already set to retire.

**Not shown:** TLB counters (no PMU on this VM); L3-cold or 8-worker i9 behaviour (the i9's page
walks were 1-2% of cycles at 2-10 workers in the Nov census, which bounds any THP gain there
near that size); THP `always` systems (`/sys` is read-only here; on `always` the default arm
already gets huge pages, so the comparison would be inverted, not opened up).
