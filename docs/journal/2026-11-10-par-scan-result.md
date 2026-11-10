# 2026-11-10: The scan without the scanner, on the i9: FASTER at level 1 and on records, not at level 9

**Status: the registered diagnostic's result (`2026-11-10-parallel-scan.md` §5). One attempt,
VALID. A diagnostic: it qualifies no production change.**

- **The readings, as registered:**
  - `wikidata`: **NOT FASTER**, at 0.981 of scan's rate;
  - `wikidata-l1`: **FASTER**, at 1.615;
  - `records`: **FASTER**, at 1.780.

  Each is find's rate over scan's at 10 workers, as the median of five paired rounds.
- **Against the predictions written before the data:**
  - `wikidata`: NOT FASTER or MIXED was predicted (r 0.95–1.12). Inside it.
  - `wikidata-l1`: FASTER was predicted (r 1.3–2.2), with the ratio rising from 4 workers.
    Inside it: the ratio is 0.81 at 4 workers, 1.13 at 6, 1.53 at 8 and 1.62 at 10.
  - `records`: FASTER was predicted (r 1.4–2.5). Inside it.
  - At 1 worker, `find` slower on every input (r 0.7–0.95) was predicted. Inside it: 0.892,
    0.833, 0.764.
- **What the registration directs** (§5): FASTER on `wikidata-l1` or `records` sends the design
  to the Coordinator as a build candidate for the parallel decoder, with §6's list as its
  obligations. FASTER is a descriptive threshold for selecting a construction to build next. It
  is not a test of a population gain, and it qualifies nothing for production.

Labels: **registered** (the readings, by the frozen rule); **measured** (the bundle's figures,
on the i9, Louis's run under RULES C5); **descriptive** (patterns in the same figures that no
rule was registered for).

## 1. The run

- **The bundle:** `par-scan/20261110T030608Z.tar.gz` on the `evidence` branch at
  `727674e185744b91077dd4a862f2cd9e1485f0d0`, SHA-256
  `140a0e5b0f7e8b741e3f81d5bc533df0b916fa8d57c3a439dde8abecbab4d152` (checked here on the
  fetched file).
- **What it ran:**
  - commit `efc1bd88c73b07d917617eddfdf008dffc691e3d` (#317's ready head), whose measured
    inputs are those frozen at `243051248de80ebc88066c8f2db3b3852c47d1f8`;
  - the outcome: `VALID: wikidata: NOT FASTER; wikidata-l1: FASTER; records: FASTER;`.
- **The Coordinator's reading:** #145 comment 6757860786.
- **The machine:**
  - an Intel Core i9-10900K, 20 logical CPUs (2 threads a core);
  - the process's mask 0–19, with no cgroup CPU quota at the mounted path;
  - nothing pinned;
  - every thread's mask was 0–19 in each 10-worker check;
  - load average 0.49 before, 2.90 after.
- **Before the cells:**
  - **faults:** all 16 fault cases ended;
  - **checks:** `mibz2::decode` equal to libbzip2 on all three inputs, and every arm equal to
    it at 1–4 and 10 workers;
  - **inputs:** each at its expected size, anchored apart from the run (§2).
- **Peak resident memory**, an observation and not a cap: 611, 474 and 182 MB for the three
  inputs' cells.

## 2. The inputs (recorded)

| input | file bytes | file SHA-256 | decoded bytes (expected) | decoded SHA-256 |
|---|--:|---|--:|---|
| `wikidata` | 11,854,933 | `82f65ace0731962d…` | 209,700,196 | `3f075e30f4f8c734…` |
| `wikidata-l1` | 19,018,084 | `2ae39adffbfbaab6…` | 209,700,196 | `3f075e30f4f8c734…` |
| `records` | 19,175,457 | `78bc91db7d198cb8…` | 19,200,000 | `93880bf79d4cb0c1…` |

The level-1 file was made by Python 3.12.2's `bz2` (libbzip2 1.0.8). It is byte-identical to
the registration's local one (`2ae39adf…`), and it decodes to the corpus's own bytes.

## 3. The cells (measured)

Each figure is the median over 5 rounds. MB/s is decoded output a second. CPU s is the process's
CPU seconds for the decode. The ratio is find's MB/s over scan's, paired round by round, as the
median, with its min–max over the rounds.

**`wikidata` (level 9): NOT FASTER**

| workers | scan MB/s | find MB/s | scan CPU s | find CPU s | find / scan | min–max |
|--:|--:|--:|--:|--:|--:|--:|
| 1 | 455 | 409 | 0.575 | 0.568 | 0.892 | 0.822–0.981 |
| 2 | 812 | 765 | 0.636 | 0.605 | 0.927 | 0.892–1.088 |
| 4 | 1,145 | 1,127 | 0.853 | 0.803 | 0.978 | 0.923–1.018 |
| 6 | 956 | 968 | 1.468 | 1.380 | 1.006 | 1.001–1.023 |
| 8 | 741 | 744 | 2.457 | 2.358 | 1.006 | 0.993–1.037 |
| 10 | 631 | 619 | 3.693 | 3.627 | **0.981** | 0.880–1.028 |

**`wikidata-l1` (the same text at level 1): FASTER**

| workers | scan MB/s | find MB/s | scan CPU s | find CPU s | find / scan | min–max |
|--:|--:|--:|--:|--:|--:|--:|
| 1 | 413 | 348 | 0.705 | 0.662 | 0.833 | 0.817–0.890 |
| 2 | 792 | 659 | 0.720 | 0.698 | 0.839 | 0.792–0.845 |
| 4 | 1,524 | 1,255 | 0.737 | 0.728 | 0.813 | 0.804–0.887 |
| 6 | 1,628 | 1,842 | 0.747 | 0.740 | 1.132 | 1.127–1.143 |
| 8 | 1,600 | 2,429 | 0.787 | 0.745 | 1.525 | 1.489–1.678 |
| 10 | 1,598 | 2,563 | 0.818 | 0.840 | **1.615** | 1.597–1.958 |

**`records` (binary records, level 9): FASTER**

| workers | scan MB/s | find MB/s | scan CPU s | find CPU s | find / scan | min–max |
|--:|--:|--:|--:|--:|--:|--:|
| 1 | 116 | 89 | 0.221 | 0.221 | 0.764 | 0.746–0.833 |
| 2 | 205 | 170 | 0.237 | 0.228 | 0.830 | 0.788–0.843 |
| 4 | 283 | 285 | 0.257 | 0.252 | 0.992 | 0.957–1.034 |
| 6 | 294 | 375 | 0.255 | 0.272 | 1.303 | 1.234–1.338 |
| 8 | 288 | 423 | 0.255 | 0.312 | 1.460 | 1.392–1.557 |
| 10 | 254 | 426 | 0.293 | 0.344 | **1.780** | 1.454–1.987 |

## 4. What the figures show beyond the readings (descriptive, not registered verdicts)

No rule was registered for these. They are patterns in the same figures, offered as context and
as candidates for a registered question. They are not conclusions.

- **Level 9 has a knee in both arms.**
  - Both arms peak at 4 workers (scan 1,145 MB/s, find 1,127), then fall to 631 and 619 by
    10.
  - CPU seconds per decode rise from 0.58 at 1 worker to 3.69 at 10 (scan; find 0.57 → 3.63).
  - Taking the scanner away changes neither the peak nor the fall: here something else bounds
    ten workers, in both arms. This fits §4's prediction that the memory knee binds first at
    level 9. It does not establish that memory is the cause. The bandwidth titration left the
    cause unresolved, and this run has no counters.
- **At level 1 the find arm scales, and scan stops.**
  - `find` rises to 2,563 MB/s (2.56 GB/s) at 10 workers, its CPU seconds nearly flat
    (0.66 → 0.84).
  - `scan` stops at about 1.6 GB/s from 6 workers on (1,628, 1,600, 1,598).
  - That plateau is what one scanner thread feeding the workers would give. It is close to the
    record's earlier level-1 plateau near 1,460 MB/s, here and there measured on different
    harnesses.
  - The flat CPU says the extra rate comes from more cores working, not from extra work
    beside them.
- **Records** show the same shape on a smaller scale:
  - `scan` holds at about 283–294 MB/s from 4 workers on, and falls to 254 at 10;
  - `find` rises to 426, with its CPU up by about half (0.22 → 0.34).

## 5. Caveats

- **One attempt, five rounds.** The ratios' min–max over five rounds are not confidence
  intervals. NOT FASTER at level 9 covers small gains and small losses alike.
- **The i9's two modes** (`2026-11-10-two-modes.md` on #313, at
  [`ce6954e6`](https://github.com/lmmx/mibz2/blob/ce6954e65f285953b0cb8dd18f2390b4dc2cc91b/docs/journal/2026-11-10-two-modes.md))
  are unresolved. This run has no instrument that says which mode each round ran in, and no
  round was excluded. Alternating the arms' order does not remove that state dependence.
- **Placement is unpinned**, on 20 logical CPUs (10 cores with 2 threads each). At 10 workers
  each arm runs 12 threads: the workers, the finder or scanner, and the joiner. Where they ran
  is not recorded.
- **The arms are complete constructions.** They differ in who reads the symbols, and also in
  buffer ownership, locking and speculative work (§5). `find` keeps every candidate it finds,
  a list that grows with the input.
- **Scope:** in-memory inputs, the harness's own worker loop (not the library's teams), three
  fixed inputs and one machine.
- **FASTER selects a construction to build next.** It is not production qualification. A
  production version owes §6's obligations: streaming input, no level guess, a bound on wasted
  work, and the one-thread path unchanged. It would then need CI's paired bench for adoption
  (RULES C3, C5).

## 6. What it bears on

- **The share-nothing direction** (the Pathfinder's
  [`2026-11-10-share-nothing.md`](https://github.com/lmmx/mibz2/blob/97e0067dc6defab2c77aeb1c2b5a6b1105f9ec69/docs/journal/2026-11-10-share-nothing.md)
  on #313).
  - It proposes that a table small enough (under about 1.2 MB a block) lets ten workers each
    run the one-thread decoder and share nothing, given the parallel scan.
  - This run is the parallel scan's first workstation measurement. With a 0.4 MB level-1
    table and nothing shared between workers but the input, the finder's design ran 10
    workers at 2.56 GB/s with nearly flat CPU, where one scanner capped at about 1.6 GB/s.
  - At level 9's 3.6 MB tables it gained nothing, and both arms showed the knee.
  - This is consistent with that entry's reading. It is not a test of it: no table size was
    varied here.
- **#10 (the scanner bound):** reopened by the registration, and now measured. One scanner
  thread binds ten workers at level 1 and on records on the i9. At level 9 it does not bind
  before something else does.
