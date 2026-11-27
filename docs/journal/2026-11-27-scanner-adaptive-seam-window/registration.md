# Registration: adaptive per-block scanner lookahead and one-shot seam window (`scanner-adaptive-seam-window`)

*Frozen before any workstation timing run (C2, C6). Claim [`scanner-adaptive-seam-window`](../../../frontier/claims/scanner-adaptive-seam-window.toml).*

---

## 1. Change under test

Three files in the striated zone, **zero new `unsafe`** (`src/stream.rs` retains `#![forbid(unsafe_code)]`; no `unsafe` added to `src/parallel.rs` or `src/trace.rs`):

1. **`src/stream.rs` (`Scanner::next`):** replaces the sticky `LOOKAHEAD = 2 MiB` window request with a non-sticky 16-block sliding-window peak lookahead:
   - First block requests `INITIAL_LOOKAHEAD = 128 KiB`.
   - Each subsequent block computes `want = (max16 + (max16 >> 2) + 4 KiB).clamp(64 KiB, 1 MiB)` from the compressed byte lengths of the last 16 blocks.
   - Retains the exact `br.overran()` retry loop, doubling `want` (and `retry_step` starting at `128 KiB`) up to `MAX_ITEM = 64 MiB` on any block whose compressed length exceeds the initial lookahead, and emitting `BlockTooLarge` / `Truncated` identically to `master`.
2. **`src/parallel.rs` (`ChannelSource`):** replaces unconditional chunk-append (`buf.extend_from_slice(&chunk)` + `buf.drain(..drop_n)`) with a one-shot seam window:
   - Incoming `4 MiB` chunks are scanned in place without copying.
   - When `from + want` crosses the end of `buf` while unreleased bytes remain, the next chunk is parked in `next` (`next_base`).
   - Only when `released < next_base` does `ChannelSource` copy the unreleased tail plus a one-shot slice `next[at..min(need + 128 KiB - next_base, next.len())]` into a reused `seam` buffer, avoiding incremental per-block `WindowResume` copies.
   - Once `released >= next_base`, `next` is swapped directly into `buf` with zero copy and `seam` is retained for reuse.
   - Checks cancellation lock-free via `self.shared.stopped.load(Ordering::Acquire)` (`Shared::cancel_locked` publishes with `Ordering::Release`).
   - Recycles up to `INPUT_QUEUE + 2` (`4`) spent input chunks into `free_in` (matching the `chunks` footprint ledger count), dropping any surplus outside the `Shared` lock.
   - Preserves `ready-cap-worker-gate` (`#1129`) untouched.
3. **`src/trace.rs`:** adds `Kind::WindowResume` and `Kind::WindowWorkEnd` so scanner window-maintenance spans (`InputGot` / `WindowResume` -> `WindowWorkEnd`) are instrumented directly on `head` without patching `src/parallel.rs`, updates `LOOKAHEAD` to `1 MiB` (`MAX_LOOKAHEAD`), and updates the `chunks` and `window` ledger notes.

---

## 2. Mechanism and relationship to prior `#1031` verdict (C2)

This is a **distinct mechanism** from `scanner-seam-window` (`#1031`), whose frozen **NOT ADOPTED** verdict in `#1118` (`origin/evidence` `c0b2341c`, `scanner-seam-window/run-20261127T070349Z.tar.gz`) stands untouched:

- In `#1118` on the physical i9-10900K (`wikidata ×4`, unpinned, 16 paired rounds), `#1031` reduced `in_maint` at 8 workers by only **`-29.93%`** (`10.689 -> 7.490 ms`, `-3.20 ms`) and missed adoption at `12` threads (`0.9898 [0.9726, 1.0073]`, `INCONCLUSIVE`; `10` threads was `0.9811 [0.9670, 0.9954]`, `FASTER`).
- Smooth-zone dissection (`docs/journal/2026-11-27-explore/in-maint-residual-dissection.md`, `examples/in_maint_dissection.rs`) reproduced the exact i9 span counts (`base = 12`, `#1031 = 879` on `wikidata ×4`) and showed that `src/stream.rs`'s sticky `LOOKAHEAD = 2 MiB` (`50%` of `CHUNK = 4 MiB`) forced `#1031` into the seam halfway through every chunk (`51.0%` of Wikidata blocks), copying `21.70 MiB` (`seam_init`) plus `21.04 MiB` (`extend` across `867` `WindowResume` spans) — **`97.2%` of `#1031`'s `7.490 ms` residual `in_maint`**.
- Across all three compiler builds in the dissection (`baseline`, `base_jcc32b`, `nat_jcc32b`), the adaptive lookahead + one-shot seam redesign (`seam_adaptive`) cut `wikidata ×4` bytes moved by **`-95.7%`** (`53.02 -> 2.29 MiB`), spans `879 -> 34`, overrun retries `0`, peak window capacity `16.00 -> 4.38 MiB` (`-72.6%`), and `in_maint` by **`-96.1%..-97.7%`** (derived i9 projection `10.689 -> ~0.42 ms`, `~10.3 ms` serial scanner wall recovered).

---

## 3. Exactness and safety obligations (VOID if any fail)

Frozen before data, matching `frontier/claims/scanner-adaptive-seam-window.toml`:

1. **Safety:** zero new `unsafe` in `src/stream.rs`, `src/parallel.rs`, or `src/trace.rs`; `src/stream.rs` keeps `#![forbid(unsafe_code)]`.
2. **Test parity:** `cargo test` (including `stream::tests`, `parallel::channel_source_tests`, `parallel::plateau_tests`, `trace::tests`, `decode`, `service_default`, `pipelined`), `cargo test --features trace`, Miri (`miri_adaptive_lookahead_clamps_and_decays_after_16_blocks` and existing `miri_` suite), and CI `fuzz-smoke` pass cleanly.
3. **CI guard corpora:** paired CI `bench` satisfies `PERFORMANCE-POLICY.md` §2 across all 7 CI corpora (`wikidata`, `text`, `random`, `records`, `runs`, `pattern`, `onebyte`), and traced peak `window` capacity is `<= 1.00x` shipped `base`.
4. **Workstation untimed SHA-256 verification:** before timing, stream the registered `wikidata ×4` input through an independent `libbz2` decoder and record its SHA-256 and exact decoded byte count (`838,800,784` bytes). For each built binary (`base` and `head`), arm, and registered thread count (`6, 8, 9, 10, 12`, plus any diagnostic counts), run an untimed `ab child` with SHA-256 enabled and require the independent SHA-256, exact `838,800,784` decoded bytes, and identical 64-bit streaming hash. Require that exact byte count and streaming hash on every timed and traced record across both builds. Any missing check, output mismatch, or decode error **VOIDS** the experiment.

---

## 4. Workstation experiment design & shallow-runner invariant

- **Machine (C5):** Maintainer's i9-10900K (10 physical cores, 20 logical CPUs, no CPU quota). Any other host **VOIDS** the run.
- **Shallow-checkout hardening (standing thirteenth-pass invariant):** the workstation runner executes in a `--depth 1` shallow checkout. The comparison harness MUST NOT clone from the shallow local repository or rely on local history:
  - Initialise two private temporary repositories (`git init`), fetch `<BASE_SHA>` (exact merge-base of `<HEAD_SHA>` and `origin/master`, resolved against the remote or unshallowed history) and `<HEAD_SHA>` directly from `<origin_url>` with `git fetch --no-tags --depth 1 <origin_url> <full_sha>`, verify `FETCH_HEAD^{commit} == <expected_sha>`, verify `base_sha != head_sha`, detach-checkout each commit, and fetch `bench-data` directly from `origin`.
  - Verify identical build inputs (`Cargo.toml`, `Cargo.lock`, `build.rs`, `rust-toolchain.toml`, `examples/ab.rs`) between `base` and `head` and identical `rustc` version.
  - Back the harness with a unit test reproducing a detached `--depth 1 --no-local` (`file://`) shallow runner before queueing.
- **Corpus:** `wikidata ×4` (`4` copies of `corpora/wikidata.bz2`, SHA-256 `0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e`, `838,800,784` decoded bytes), verified by SHA-256 before timing.
- **Judged wall/RSS A/B run:**
  - Arms: `mibz2@build-base` (reference) vs `mibz2` (`head`), production default decoder.
  - Placement: **unpinned** (`cpus = "all"`, no `--spread`, no spinner).
  - Thread counts: **`6, 8, 9, 10, 12`** (judged: **`10, 12`**; supporting no-regression counts: **`6, 8, 9`**).
  - Repeats: **16 paired rounds**, arms interleaved within each round with rotated order, `2 s` settle gate, `100 ms` pre-spin (`--prespin-ms 100 --machine-state`) recording `cycles_per_ref`.
- **Judged traced `in_maint` check (8 workers):**
  - Built with `--features trace` (`examples/occupancy`) on both `base` and `head` at `10` threads (production's `8` workers / `32` slots) on `wikidata ×4`, `3` paired timeline rounds.
  - Both arms confirm `8` constructed workers and `32` slots and sum exact scanner maintenance spans (`InputGot` / `WindowResume` -> `WindowWorkEnd`, excluding channel wait and intervening block decoding; `head` already emits these marks natively, and `base` uses a minimal trace-only `WindowWorkEnd` mark at the end of `ChannelSource::view`'s chunk iteration).
  - Report median `in_maint_ms`, `in_maint_spans`, `window_live_max_mb`, `window_cap_max_mb`, and percentage drop `(base - head) / base`.
- **Estimator:** paired per-round log wall ratios `log(wall_head / wall_base)`, geometric mean and two-sided 95% Student's *t* confidence interval (`bench/two_arm.py`):
  - **FASTER:** 95% CI upper bound `< 1.000`.
  - **SLOWER:** 95% CI lower bound `> 1.000`.
  - **INCONCLUSIVE:** 95% CI spans `1.000`.

---

## 5. Frozen adoption decision rule (C2 applies; never relaxed after data)

Verbatim from `frontier/claims/scanner-adaptive-seam-window.toml`:

> Workstation i9-10900K registration (frozen before any hardware run, shallow-safe per thirteenth-pass invariant, paired A/B against `master` on `wikidata x4`, unpinned, 16 paired rounds): must cut traced `in_maint` at 8 workers by `>= 85%` (`10.689 -> <= 1.6 ms`) AND pass wall adoption at both judged 8-worker default counts (`10` and `12` threads both `FASTER`, 95% CI upper bound `< 1.000`, no `SLOWER` at `6`/`8`/`9` threads, median peak RSS `<= 1.05x` at every count). Otherwise STOP and record a scoped dead-end in `frontier/dead-ends.tsv`.

In checklist form:

- **ADOPTS** iff ALL of:
  1. Traced `in_maint` at 8 workers drops by **`>= 85%`** vs paired `base` (`head / base <= 0.15`, matching `10.689 -> <= 1.6 ms`) with `window_cap_max_mb` `<= 1.00x` `base`;
  2. `10` threads (8 workers) is **FASTER** (95% CI upper bound `< 1.000`);
  3. `12` threads (8 workers) is **FASTER** (95% CI upper bound `< 1.000`);
  4. None of `6`, `8`, and `9` threads is **SLOWER** (none has 95% CI lower bound `> 1.000`);
  5. Median peak RSS ratio `median(rss_kb_head) / median(rss_kb_base) <= 1.05` at every count (`6, 8, 9, 10, 12`).
- **NOT ADOPTED** otherwise. A **NOT ADOPTED** verdict is **final** for this mechanism under this production; record a scoped row in `frontier/dead-ends.tsv`.
- **VOID** (exit 1; no verdict; may be repeated once after fixing the recorded cause) if ANY of:
  - Host is not i9-10900K (10 cores, 20 CPUs, no CPU quota);
  - `base_sha == head_sha`, build inputs differ, or shallow checkout fails exact SHA verification;
  - Any untimed SHA-256 check fails or any record is missing, duplicated, non-`success`, non-finite/non-positive, pinned (`cpus != "all"`), or run beside a spinner;
  - Corpus SHA-256, decoded byte count (`838,800,784`), or output streaming hash differs across any run;
  - Median observed worker count differs between `base` and `head` at any thread count, or traced diagnostic does not confirm 8 workers / 32 slots on both arms;
  - Any judged or supporting 95% CI is undefined.

---

## 6. Diagnostics (reported only; decide nothing)

1. **Explicit-worker lazy diagnostic:** `mibz2-workers-lazy@build-base` vs `mibz2-workers-lazy` at explicit `8, 9, 10` workers, 16 paired rounds, unpinned, on `wikidata ×4` (exposes the scanner-bound regime at 9–10 workers directly).
2. **Clock split (§81):** `cycles_per_ref` ratio and common-clock wall ratio at each count from `--machine-state`.

---

## 7. Queuing gate (C6)

This is an adoption registration deciding a production default. Under `CLAUDE.md` (RULES C6), **do not merge into `master`, edit `bench/workstation-queue.txt`, or set a `ready/` signal** until:

1. One independent agent (different from the author) has reviewed both the code diff (`src/stream.rs`, `src/parallel.rs`, `src/trace.rs`) and this frozen registration;
2. All CI checks on the PR head (including `rust`, `miri`, `fuzz-smoke`, `occupancy`, and paired `bench`) are green.
