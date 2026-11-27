claim: ready-cap-worker-gate
by: antigravity
outcome: done
evidence: src/parallel.rs; src/parallel/plateau_tests.rs; cargo test --release --lib -- parallel:: (52/52) + decode/pipelined/service_default (30/30); paired bzcat zeros_2g/text/wikidata; #1121

Bounded high-expansion run-only block memory in `src/parallel.rs` by the ready cap (`READY_BYTES`) instead of the full slot window, with **zero new `unsafe`**.

**Mechanism (`src/parallel.rs`):**
- Added worker-side ready-cap gate (`worker_cv` + lock-free `ready_bytes: AtomicUsize` fast-path): workers wait before decoding `place >= taken + cap.floor` while `ready_bytes >= cap.bytes`, preserving unconditional forward progress for `taken..taken + cap.floor` (strict FIFO `Jobs` guarantees deadlock freedom) and waking waiting workers on `advance()` and `cancel_locked()`. When `ready_bytes < cap.bytes` (normal `wikidata`/`text` blocks), workers take the single atomic load fast-path with zero extra locking.
- Tracked `free_out_bytes` and bounded `push_free_out()` by `cap.bytes + cap.floor * 2 MiB` (while retaining up to `threads` active buffers so large run-only buffers recycle without `munmap`/`mmap` page-fault thrashing while never scaling with `slots`).
- Added deterministic high-expansion memory-cap and mid-stream cancel wakeup regression tests in `src/parallel/plateau_tests.rs`.

**Stop-test verification (all 3 criteria PASS):**
1. **Exactness & safety:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (0 warnings), `parallel::` **52/52** (incl. 180 plateau cells, unwind/panic, Miri, cancel wakeup, high-expansion cap), `decode` **14/14**, `pipelined` **12/12**, `service_default` **4/4**, and byte-exact `sha256` parity on `zeros_2g` (`1,522 B` -> `2 GiB`), `text` (`35.6 MB` bz2), and `wikidata` (`11.9 MB` bz2).
2. **`zeros_2g` (`bzip2 -9`, 3 interleaved repeats vs `origin/master` `c5eb2bc2`):**
   - `t=4, slots=8`:  RSS `588.9 -> 545.2 MiB` (`0.926×`), minflt `150.7k -> 139.4k`, wall `1268.7 -> 1268.4 ms` (`1.000×`)
   - `t=4, slots=16`: RSS `810.6 -> 545.2 MiB` (`0.673×`; **`1.000×`** of `t=4/8 <= 1.15×`), minflt `207.4k -> 139.5k`, wall `1443.1 -> 1350.7 ms` (`0.936×`)
   - `t=4, slots=32`: RSS `1117.9 -> 545.4 MiB` (`0.488×`, **−572.5 MiB**; **`1.0004×`** of `t=4/8 <= 1.15×`), minflt `286.1k -> 139.5k`, wall `1383.4 -> 1181.8 ms` (`0.854×`), CPU `2079 -> 1603 ms` (`0.771×`)
   - `t=8, slots=32`: RSS `1528.0 -> 955.1 MiB` (`0.625×`, **−572.9 MiB**), minflt `391.0k -> 244.4k`, wall `2028.0 -> 1600.9 ms` (`0.789×`), CPU `2463 -> 1911 ms` (`0.776×`)
3. **`text` & `wikidata` (`1` and `4` workers, 7 interleaved repeats):** all 95% CIs span `1.000×` (`text t=4` `0.966×` median / `1.001×` mean; `wikidata t=4` `0.996×` mean; `wikidata t=1` `1.011×` mean); dedicated pinned AMD EPYC CI paired `bench` verifies zero regression under `PERFORMANCE-POLICY.md` §2.
