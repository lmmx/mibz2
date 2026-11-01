# The resource attribution: amendment 1 (the maintainer's three refinements, and the implementation as built; before any i9 run)

**Status:**
- **Authorised:** the maintainer approved the protocol's design (draft `a64bdd1`), C1–C6,
  the correctness checks, the instrumentation's validation, the freeze and an untimed dry
  run.
- **Required before the freeze:** three refinements.
- **Committed before any i9 run.**
- **Unchanged from the draft:**
  - R1–R7's thresholds;
  - 1 against 4 workers;
  - the identical four-core allocation;
  - the walk trial's frozen corpora;
  - separate traced and uninstrumented runs.
- **No production change:** C6 shows the default-feature decoder is unchanged. Neither
  closed experiment is touched.

## 1. The three refinements

### 1.1 R2: the effective frequency of each workload, measured

**What G0 gives, for each (corpus, w):**
- cycles, ref-cycles, instructions and task-clock;
- **the effective clock** f = cycles / task-clock, the cycles per CPU-second while the
  decode runs, with cycles / ref-cycles beside it.

No earlier clock reading is assumed. The 4.9 GHz of `2026-10-28-workstation-scaling.md` is
not used.

**Since C = K / f** (CPU-s per byte = cycles per byte / clock):
- ln(C₄/C₁) = ln(K₄/K₁) − ln(f₄/f₁);
- the summary reports both shares of the CPU-s/GB change: from frequency, and from cycles.

**R2's rule is unchanged:** K₄/K₁ − 1 ≤ ⅓ (C₄/C₁ − 1).

**The cycles part is reported as additional execution cycles, not as memory stalls.**
- Only R3 attributes cycles to memory. It requires `stalls_l3_miss` evidence, and
  instructions per byte within 5%.
- Instructions per byte are reported beside it. That separates "more work" from "more
  cycles per instruction".

### 1.2 Hardware events: verified on the machine, never substituted

**The plan step (`attribution.py events`):** it runs on the i9 before any measurement, on
the primary at w = 1, and writes `events.json`.
- **Each group is tried whole first,** as a perf group (`{…}`: scheduled together or not
  at all). A group counts only if every event is counted for 100% of its enabled time.
- **A group that fails is tried event by event, each alone.**
- **An event that counts in neither way is recorded unavailable,** with perf's own
  message. No other event replaces it.

**The group sizes:**
- G1–G6 carry at most three programmable events each, so that a group fits beside the NMI
  watchdog's counter. The draft's G1–G4 are the same events, regrouped.
- G0 is the fixed counters and the task clock.
- IMC is machine-wide.

**The counters then follow `events.json` exactly.** Every pass's coverage is recorded, and
an incomplete pass is not interpreted.

**Sampled events are verified too:**
- `perf record` falls back to `cpu-clock` or `task-clock` when the PMU can't count the
  requested event, and says so only on stderr. The dry run VM showed this.
- So each perf.data file's event is read back (`perf evlist`). A file holding any other
  event is recorded as a mismatch and not used.

**What the samples are, and aren't:**
- They give the share of sampled cycles, and of sampled L3-miss stall cycles, by thread
  role and stage function.
- That is where those cycles were spent, with a few instructions of skid. It is not proof
  of what caused them.
- R3 uses them only beside the process-wide counts, and with its instruction condition.

### 1.3 Multiple bottlenecks, ranked by evidence

- **Each hypothesis H0–H5 gets its own status:** supported, not supported, falsified, or
  unresolved with the reason. The statuses are independent; more than one can be
  supported.
- **Each also carries the share of its question that its evidence accounts for:**
  - Q1 (CPU-s/GB): H0's frequency share, H1's build+walk share;
  - Q2 (idle capacity): H2's steady-state idle while the scanner reads, H3's head-of-line
    share, H4's startup+drain share, H5's run-queue share of the unused core-time.
- **The supported hypotheses are ranked within each question by that share.**
- The Q2 account names its largest owner (≥ 25%) and still lists all of them.
- No rule makes a classification exclusive. Head-of-line blocking can coexist with costly
  inversion, and a scanner ceiling with memory stalls.

## 2. The implementation as built, where it departs from the draft's §7 (disclosed)

| Draft | As built | Why |
|---|---|---|
| uninstrumented `bzcat` | `examples/attribution_decode.rs`: the occupancy workload exactly (`BzReader::parallel_with(File, w, 2w, …)`, its feeder `mibz2-read`, 4 MiB drains), default features | `bzcat` feeds stdin from an unnamed thread, so the traced and counted runs' threads wouldn't correspond |
| C3 in `occupancy`'s report | `occupancy` writes each traced run's raw marks (`OCCUPANCY_MARKS_DIR`); `bench/attribution.py` analyses them | smaller change to the 2,300-line instrument. The analysis is checked against the instrument's own idle shares on every timeline run (§3) |
| C4 in `bench/imc.py` | a new `bench/attribution.py` (the events, stat and record steps); `imc.py` untouched | no change to a tool other records depend on |
| R1: build + walk symbols | also a `memcpy` stage (the C library's copies, mostly the walk's segment assembly and RLE1's output, which samples can't split); R1 reports build + walk **without** (lower) and **with** (upper) them | **H1 is falsified only if the upper bound is below 0.5.** A lower bound under 0.5 with an upper bound over 0.5 is **unresolved** |
| R1's cross-check | the trace's walk phase (`WalkStart`→`InvertEnd`) includes the assembly's copies | the cross-check is compared with the upper bound's side of 0.5 |
| R4 "idle while the scanner reads" | the steady state only | idle while the scanner reads the first blocks is startup (H4), so it isn't counted twice |
| — | **the scanner after its last `Enqueued`** is "done": its final slot wait before it finds the stream's end is drain | no wider window could issue another block then |
| — | **the span:** from the first mark to the last `Taken` or `DecodeEnd` | the collector's own wait after the decode is excluded |
| — | **thread CPU:** the absolute thread CPU for the decoder's threads; the reader's from its first mark | the feeder's trace starts at its first hand-off, but its CPU before it still counts |
| — | **sampling periods:** cycles every 2,000,003; stall cycles every 200,003 | overhead checked (§3) |
| — | **perf:** as root, or `sudo -n perf` with sudo asked for once and kept alive; perf.data handed back to the user | the memory controller's counts are machine-wide |
| — | **corpora:** the walk trial's work directory, hash-checked; else rebuilt by that trial's own preparation at its frozen head (`f5c492d`), stopped before timing | byte-identical to the walk trial's |

## 3. Validity checks added (in `result.json`; any failure: the figure is not interpreted)

- **This analysis against the instrument:** on every timeline run, the idle shares this
  analysis computes (while the scanner reads; while it waits for input) must be within
  0.02 of worker time of the instrument's own.
- **The samples' overhead:** the `perf record` runs' wall time against the counting
  passes'. Above 3% is reported per w.
- **The idle siblings:** each is under 2% busy over M1–M3, from `/proc/stat` around them.
- **C6, sensitivity shown:** the same check against a trace-feature build of the same
  example reports 5 differing functions and 1 added (exit 1). So it detects the trace's
  code when it is present.
