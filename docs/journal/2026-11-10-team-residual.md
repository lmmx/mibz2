# 2026-11-10: The team path's last 4.4 instructions a byte: where they are

**Status: a reading of archived counts. Nothing was built, run or timed. Every figure is
callgrind's Ir from #330's and #334's archived profiles (counted), read from the source (read),
or arithmetic on those (model).**

- **The question** (the Coordinator): with the staged cooperative walk (#334) the team path's
  whole process is 30.21 Ir a decoded byte on Wikidata against one worker's 25.79. Where is
  the remaining 4.42, which of it is cooperation's own, and what is the cheapest next port?
- **Counted:** the build is +3.95 of it and MTF +0.60. The walk is 0.22 *below* one worker's.
  Everything else together is +0.09.
- **Cooperation's own share is about 0.3:** the checkpoints' validation (0.16), the pool and
  relay (0.07, a floor), and reserves. The other 4.1 is one-worker work not yet ported.
- **Inside the build, one loop is 2.3 to 2.5 of it:** a window of 16 equal bytes is written
  by 16 scalar stores, 77 instructions a window, on 82% of Wikidata's rows. One worker writes
  the same window with `run16`, about 33.
- **Cheapest next port:** `run16` in that window of `FusedBuild::share`. Predicted 30.21 →
  about 27.9 (model). It adds no invariant.

Labels: **counted**, **read**, **model**.

## 1. Sources

| arm | profile | binary at |
|---|---|---|
| one worker, `ab child mibz2 1` | `2026-11-10-team-kernel-gap-count/cg/one.*` (#330, master) | `ae44497e9` |
| staged team, `AB_STAGED_TRAVERSE=1 ab child mibz2-pipe3-both 9` | `2026-11-10-staged-traverse-count/cg/wikidata-staged.*` (#334, `dd5d8bf7`) | `c78c18d3d` |

Both decode the whole corpus (431 blocks, D = 209,700,196 bytes, N = 209,706,557 rows) to
the same hash. The parser here (`2026-11-10-team-residual/cgparse.py.txt`) reproduces both
totals, 25.792 and 30.207, and each function's figure in the two entries' tables.

**A limit that matters:** #334's count is FAIL-MEASUREMENT under its own filter (one helper
walked 69.5% of its team's helper bytes under valgrind's schedule). Its walk figure agrees
with its one-caller control (9.885). The build and MTF figures below are the same in its old
arm and in #330, so they do not rest on the staged arm's split.

## 2. The residual by stage (counted)

Ir a decoded byte.

| stage | one worker | staged team | team − one | missed one-worker work | cooperation's own |
|---|--:|--:|--:|--:|--:|
| scanner | 3.256 | 3.203 | −0.053 | | −0.05 (the channel's view in place of a memset) |
| MTF | 4.111 | 4.715 | **+0.604** | 0.60 | under 0.02 |
| build | 5.160 | 9.106 | **+3.946** | 3.78 | 0.16 |
| walk | 10.188 | 9.966 | −0.222 | | inside it: §5 |
| assembly, RLE1, CRC | 1.603 | 1.614 | +0.011 | | 0.01 (reserves) |
| coordination | 0.000 | 0.071 | +0.071 | | 0.07, a floor |
| other, harness | 1.473 | 1.533 | +0.060 | | 0.06 |
| **whole process** | **25.792** | **30.207** | **+4.415** | **4.38** | **about 0.3, less the walk's 0.22** |

## 3. The build: +3.95 (counted by line)

`FusedBuild::share` is 8.937 and `FusedBuild::new` 0.162; `bwt::build` is 5.160.

**`share` by source line** (`coop.rs` at `c78c18d3d`; Ir/D):

| part | lines | Ir/D | what |
|---|---|--:|---|
| the window loop and its test | 649, 651, `read_u64`, the compare | 1.10 | two 8-byte loads and a rotate a window |
| **16 equal bytes: 16 scalar stores** | 653–664 | **3.94** | 10,740,083 windows, 171.8 M rows (81.9%); 77 Ir a window, 3.1 of them a store |
| the one-row step | 634–644 | 2.76 | 37.9 M rows (18.1%), 15.3 Ir a row |
| the inner range and checked arithmetic | `range.rs`, `uint_macros.rs`, `index.rs` | 1.05 | the 16-row `for` of the one-row path |
| unattributed | | 0.07 | |

**`build` by source line** (`bwt.rs` at `ae44497e9`): the same 10,740,086 windows take
`ALL_JOINED`: 0.41 for its three lines and about 1.0 for `run16` (20 instructions a window:
model from the listing's shape). Its window loop is 6 instructions a window.

- **The gap is mostly one loop (model on counts):** the equal window costs 3.94 in the team
  and about 1.4 on one worker: **+2.3 to +2.5**.
- **The window loop and the row path's overhead:** about +1.2. `build`'s loop classifies a
  window by one SSE compare (`run_joins`) and has no inner range.
- **The rows outside equal windows cost the same in instructions on both paths,** about 15
  a row. #312's run windows are worth time there (they break a store-forward chain), not
  instructions on this corpus.
- **Cooperation's own:** `FusedBuild::new`, 0.162: `check_inputs` walks every checkpoint's 256
  counts three times. It is validation of the leader's checkpoints, which the helpers' windows
  rest on, so it stays; it could be one pass.

**Read: nothing in `share` blocks the port.** A share writes only inside its own window of
each bucket (`ftab[c]..lim[c]`), and windows of different shares are disjoint.
- **`run16` in the equal window:** the 16 entries are all final and already guarded by
  `s + 16 > lim[c]`. No new invariant.
- **`build_one_pass`'s whole loop** (`run_joins`, `few_runs`, `run16` for short runs): its
  16-wide write leaves provisional entries above the cursor. With `lim` the *window's* end
  they stay inside the share's own window and its own cursor rewrites them, as one worker's
  does inside the bucket. A run within 16 slots of its window's end takes the one-row steps:
  at most 15 rows a bucket a share, under 0.07 Ir/D.

## 4. MTF: +0.60, all of it missed work (counted)

`mtf_checkpoints_avx2` 3.983 against `mtf_avx2` 3.687 (+0.296: #172 and #207), its memset arc
0.709 against 0.424 (+0.285: #170), a memmove 0.022.
- **The checkpoints themselves cost nothing visible:** no line of the checkpoint code
  (`mtf.rs` 447–488) reaches 0.004 Ir/D.
- This port is C29, already in flight.

## 5. The walk: −0.22, with 0.3 still inside it

The staged team's 9.966 is below one worker's 10.188. Inside it, from #334's own counts:
- the unstaged tail: 3.64% of walked bytes at about 8 Ir a byte over the busy rate: +0.29;
- the flush: 0.139 against 0.123: +0.016;
- twice the segments: the sort +0.06.

The tail is the only one of size. One worker has a tail too; shortening the team's is a new
optimisation, not a port.

## 6. What each port would leave (model)

Each row brings a stage to the figure named and changes nothing else.

| after | team, whole process | saved |
|---|--:|--:|
| the staged walk (#334, counted) | 30.21 | |
| `run16` in `share`'s equal window (33 Ir a window for 77) | **about 27.9** | 2.3 |
| `build_one_pass`'s loop in `share` (5.16 + 0.07 + `new`'s 0.16) | about 26.5 | 3.7 in all |
| MTF to `mtf_avx2` (C29) | about 25.9 | 0.6 |
| one worker (counted) | 25.79 | |

- **Cheapest: the first row.** One loop body replaced by a call that exists, no new
  invariant, 60% of the build's gap.
- **Its falsifier:** the same count as #334's, old arm against the port, on I4: `share`
  below 6.9 Ir/D.
- **What remains is cooperation's:** about 0.3, of which 0.07 is a floor that real cores
  will raise (a waiting thread spins there).

**Limits.** Instructions are not time. On one worker #312 was worth more in time than its
instructions say, and at ten threads the build's writes contend for L3. The `run16` and
window-loop figures for one worker are split by reading the listing, not by line counts:
its SSE lines are pooled across the classifier and the writes. One corpus.

## Reproduce

```text
# unpack each arm's profiles (gzip, one file a thread), then:
python3 cgparse.py D PROFILES...                      # self Ir/D by function
python3 cglines.py DIR_OF_cgparse D 'FusedBuild>::share' SRC_DIR PROFILES...   # by source line
# D = 209700196; SRC_DIR is src/ at the arm's commit (c78c18d3d, ae44497e9)
```
