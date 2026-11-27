# Input-chunk seam window (`scanner-seam-window`): adoption registration

*Frozen 27 November 2026 before any workstation data, under claim [`scanner-seam-window`](../../../frontier/claims/scanner-seam-window.toml) (RULES C6 adoption registration: requires one independent review by a different agent before queuing on the workstation; C2 is never relaxed after data is seen).*

---

## 1. Mechanism and scope

In `src/parallel.rs`, `ChannelSource::view` supplies compressed bytes to the serial scanner thread from 4 MiB chunks arriving on a channel. On `master`, whenever a block's 2 MiB look-ahead reaches past the current chunk while earlier bytes are still held, `ChannelSource::view` drains up to `COMPACT` (4 MiB) bytes and calls `self.buf.extend_from_slice(&chunk)`, re-copying the entire incoming 4 MiB chunk on the serial scanner thread.

In the pooled-scan and stage-1 characterisations (`c71c4a56`; [`fifth-pass.md`](../2026-11-26-synthesis/fifth-pass.md) table 1), this window maintenance (`in_maint`) costs **11.2 ms** at 8 workers and **12.8 ms** at 10 workers on `wikidata ×4` (out of ~185 ms total scanner demand).

**The input-chunk seam window** (ported from PR #857 `perf/scanner-window`, `946ba572`, touching `src/parallel.rs`, `src/trace.rs`, and `docs/ledger.md` only; zero new `unsafe`):
- Keeps the incoming chunk (`buf`) in place while views stay within it.
- When a view reaches past `buf`'s end while bytes remain held, holds the incoming chunk beside it (`next`, at offset `next_base`) and moves only the held tail plus the straddling prefix of `next` that views actually reach into a single reused `seam` buffer (allocated once at the first boundary at `max(held + add, 2 * want)`, reused at every subsequent boundary).
- As soon as `released >= next_base`, swaps `next` directly into `buf` in O(1) without copying the rest of `next`, and retires the `seam` buffer for reuse at the next boundary.
- Single-chunk inputs never allocate a seam buffer. Inputs with chunks shorter than the 2 MiB look-ahead absorb whole chunks into the seam and compact as `master` did.

**Scope isolation:** PR #846's symbol-buffer high-water change (`src/block.rs`) is intentionally excluded from this branch and registration; it corresponds to lever 3 of `scanner-coalesced-u8-decode` (`owned_paths = ["src/huff.rs", "src/block.rs", "src/bits.rs"]`), keeping the two holds strictly disjoint.

---

## 2. Prior evidence and why this re-registration is valid

Combined with #846 as #861, this mechanism ran twice on the i9-10900K (journal [`2026-11-24-algorithm-level.md`](../2026-11-24-algorithm-level.md) §§77, 79; [`scanner-plumbing-combined/summary.txt`](../2026-11-24-algorithm-level/math-targets/data/scanner-plumbing-combined/summary.txt)):
- At 10 and 12 threads (then 7 workers): **FASTER** (`0.982 [0.966, 0.998]` and `0.977 [0.961, 0.994]`), with peak RSS `0.967–0.974`.
- At explicit 9 and 10 lazy workers: **`0.921 [0.884, 0.958]`** and **`0.936 [0.889, 0.984]`**.
- It was **NOT ADOPTED** solely because the prior registration judged 9 threads (the noisiest count on the i9, ±4–5%: `0.973 [0.936, 1.011]`).

Journal §79 explicitly closed the 9-thread registration (no repeat of the same test) and called for a new registration once production moved to 8 workers from 10 threads (§76, `UNPINNED_FROM_TEN`), judged at production's default counts (10 and 12 threads). At the current 8-worker tie (`fifth-pass.md` §1: scan-bound wall `184.4 ms`, worker-bound wall `185.0 ms`, shipped `193.0 ms`), realised 8-worker wall gain from a scanner-only lever is capped near ~4% (~8 ms coupling recovery), and transfers ~1:1 wherever the scanner binds (9–10 workers, or 8 workers once worker-side wins land).

---

## 3. Exactness and safety obligations (VOID if any fail)

Frozen before data, verbatim from `frontier/claims/scanner-seam-window.toml`:

> #857 rebased onto master is 100% byte- and error-identical on `cargo test`, fuzz smoke and wikidata x4 (including inputs that end mid-seam and single-chunk inputs), with no new `unsafe`.

Specifically:
- Zero new `unsafe` in `src/parallel.rs` or `src/trace.rs`.
- `cargo test` (including `channel_source_tests`, `phase`/`phase_off`, `service_default`, and all integration suites) and CI `fuzz-smoke` pass cleanly.
- Every timed and diagnostic decode on `wikidata ×4` must match the independent output reference as specified below.

**Pre-data exactness clarification (27 November, review of aa737185).** The original wording required SHA-256 on every timed and diagnostic decode. The timing harness computes SHA-256 inside the decode clock and reserves that mode for untimed checks. Before timing, stream the registered input through an independent libbz2 decoder and record the SHA-256 and byte count. For each binary, arm and registered thread count, run an untimed `ab child` with SHA-256 enabled. Require the independent SHA-256, exactly 838,800,784 bytes and the same 64-bit streaming hash on all sixteen checks. Require that byte count and streaming hash on every main, lazy and occupancy record across both builds. A missing check, an output mismatch or a decode error voids the experiment. The timed binaries retain their usual sink. The estimators, counts and thresholds in §5 are unchanged.

---

## 4. Workstation experiment design

- **Recipe:** `just submit bench-scanner-seam-window <HEAD_REF> <HEAD_SHA> 100` (defined in `.just/bench.just`, verdict by `bench/scanner_seam_window.py`, unit-tested by `bench/test_scanner_seam_window.py`).
- **Machine (C5):** Maintainer's i9-10900K (10 physical cores, 20 logical CPUs, no CPU quota). Any other host voids the run.
- **Toolchain:** Both `base` (exact `git merge-base <HEAD_SHA> origin/master`) and `head` (`<HEAD_SHA>`) built cleanly from detached worktrees with `rustc 1.99.1`. Build inputs (`Cargo.toml`, `Cargo.lock`, `build.rs`, `rust-toolchain.toml`, `examples/ab.rs`) must be identical between `base` and `head`.
- **Corpus:** `wikidata ×4` (`4` copies of `corpora/wikidata.bz2`, SHA-256 `0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e`, `838,800,784` decoded bytes), verified by SHA-256 before timing.
- **Judged A/B run:**
  - Arms: `mibz2@build-base` (reference) vs `mibz2` (head), production default decoder.
  - Placement: **unpinned** (`cpus = "all"`, no `--spread`, no spinner).
  - Thread counts: **`6, 8, 9, 10, 12`** (judged: **`10, 12`**; supporting no-regression counts: **`6, 8, 9`**).
  - Repeats: **16 paired rounds**, arms interleaved within each round with rotated order, `2 s` settle gate, `100 ms` pre-spin (`--prespin-ms 100 --machine-state`) recording `cycles_per_ref` for the §81 clock split.
- **Estimator:** Paired per-round log wall ratios `log(wall_head / wall_base)`, geometric mean and two-sided 95% Student's *t* confidence interval (`bench/two_arm.py`: `step`, `label`):
  - **FASTER:** 95% CI upper bound `< 1.000`.
  - **SLOWER:** 95% CI lower bound `> 1.000`.
  - **INCONCLUSIVE:** 95% CI spans `1.000`.

---

## 5. Frozen adoption decision rule (C2 applies; never relaxed after data)

Exact frozen `stop_test` from `frontier/claims/scanner-seam-window.toml`:

> On the i9, wikidata x4, unpinned, 16 paired rounds against the merge base, it ADOPTS iff 10 and 12 threads (production's default counts, 8 workers) are both FASTER (95% CI upper bound < 1.000), none of 6, 8 and 9 threads is SLOWER (lower bound > 1.000), and median peak RSS head/base <= 1.10 at every count. The traced diagnostic, which decides nothing, should show `in_maint` at 8 workers falling by >= 80%. A NOT ADOPTED here is final for this mechanism under this production (no repeat of the same registration). Record it in `dead-ends.tsv`.

In checklist form (`bench/scanner_seam_window.py`):
- **ADOPTS** iff ALL of:
  1. `10` threads (8 workers) is **FASTER** (95% CI upper bound `< 1.000`);
  2. `12` threads (8 workers) is **FASTER** (95% CI upper bound `< 1.000`);
  3. None of `6`, `8`, and `9` threads is **SLOWER** (none has 95% CI lower bound `> 1.000`);
  4. Median peak RSS ratio `median(rss_kb_head) / median(rss_kb_base) <= 1.10` at every count (`6, 8, 9, 10, 12`).
- **NOT ADOPTED** otherwise. A **NOT ADOPTED** verdict is **final** for this mechanism under this production (no repeat of the same registration); record a scoped row in `frontier/dead-ends.tsv`.
- **VOID** (exit 1; no verdict; may be repeated once with the cause recorded) if ANY of:
  - Machine is not i9-10900K (10 cores, 20 CPUs, no CPU quota);
  - Any record is missing, duplicated, unexpected, non-`success`, non-finite/non-positive, pinned (`cpus != "all"`), or run beside a spinner;
  - Corpus name, decoded byte count (`838,800,784`), or output hash differs across any run;
  - Median observed worker count (`mibz2-N` in `observed_threads`) differs between `base` and `head` at any thread count;
  - Any judged or supporting 95% CI is undefined.

---

## 6. Diagnostics (reported only; decide nothing)

1. **Traced diagnostic (`in_maint` at 8 workers, decides nothing):**
   - Built with `--features trace` (`examples/occupancy`) on both `base` and `head` at 10 threads (production's 8 workers) on `wikidata ×4`.
   - Reports `window_live_max_mb`, `window_cap_max_mb`, scanner timeline shares, and `InputGot -> Window` / `ChannelSource::view` maintenance time (`in_maint`), checking the expected **`>= 80%` drop in `in_maint`** at 8 workers.
2. **Explicit-worker lazy diagnostic (decides nothing):**
   - `mibz2-workers-lazy@build-base` vs `mibz2-workers-lazy` at explicit `8, 9, 10` workers, 16 paired rounds, unpinned, on `wikidata ×4` — directly exposing the scanner-bound regime at 9 and 10 workers.
3. **Clock split (§81, decides nothing):**
   - `cycles_per_ref` ratio and common-clock wall ratio at each count from `--machine-state`.

---

## 7. Queuing gate (C6)

This is an adoption registration deciding a production default. Under `CLAUDE.md` (RULES C6), **do not edit `bench/workstation-queue.txt` or set `ready/bench-scanner-seam-window`** until:
1. One independent agent (different from the author) has reviewed both the code diff and this frozen registration;
2. All CI checks on the PR head are green.

Once reviewed and green, queue `bench-scanner-seam-window` under the standing workstation authorisation (`CLAUDE.md`, 9 November 2026) pinning the exact reviewed head SHA.
