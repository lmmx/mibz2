# Slack-only hybrid scanning: design and frozen pre-data kill/adoption registration

*Registered 27 November 2026 under claim [`pooled-scan-hybrid`](../../../../frontier/claims/pooled-scan-hybrid.toml). Design and pre-data evaluation protocol only — zero `src/` edits, zero i9 hardware data. Frozen under C1a, C2, and C6 before any workstation execution.*

## 1. Context, zero-data freeze declaration, and distinction from killed `#942`

**Zero-data freeze check (verified 27 November 2026 at `origin/master` `f0fdd8b9`):**
- `origin/evidence` contains zero `slack-hybrid-scan` or `hybrid-scan` bundles (latest bundles: `c0b2341c` `scanner-seam-window/run-20261127T070349Z`, prior `c71c4a56` `pooled-scan/run-20261126T133034Z`).
- `bench/workstation-queue.txt` and `origin` `refs/tags/{ready,hold}/*hybrid*` contain zero entries.
- The killed all-pooled prototype's frozen verdict (`#932`, `c71c4a56`, `#942` **KILL**) and its row in `frontier/dead-ends.tsv` remain completely untouched (C2).

**Why `#942` was killed and why slack-only hybrid is a distinct mechanism:**
The all-pooled prototype (`pooled-scan`) eliminated the dedicated serial scanner thread and loaded all `~175 ms` of scanning onto the `k` workers (`P(k) = S(k) + scan/k` to within `1.5%`). Below the knee where workers bind, adding `scan/k` slowed wall by `+10.5%` at `k = 6`, `+10.0%` at `k = 7`, and `+5.2%` at `k = 8` (killing K2), while serial `pool_find` (`6.11 ms`) and `pool_copy` (`5.27 ms`) taxed the chain thread.

Slack-only hybrid scanning keeps the dedicated serial scanner (`k + 1` active threads: `1` scanner + `k` workers) and allows a worker to scan ahead **only when `job_queue` is empty** (idle slack). When workers bind (`k <= 7` or worker-bound `k = 8`), `job_queue` stays non-empty and helpers almost never trigger (`0.997×–0.999×` in `#1056`'s discrete-event replay of primary i9 traces). When the serial scanner binds (`S' - W'/k > ~5.5 ms`), idle workers absorb excess scan demand up to the `(k + 1)`-thread work bound:

$$\text{wall}_k \approx \frac{k \cdot (W'/k) + S'}{k + 1} + \delta_{\text{coupling}}$$

with `#1056` deriving $\delta_{\text{coupling}} \approx 4.85\text{ ms}$ (`+3.50 ms` with small chunk 0) at `k = 8` on `wikidata x4`, clearing `140.0 ms` (`6 GB/s`) once `W'/8 <= 133.9 ms` at `S' = 145 ms`.

**Current system budget (incorporating synthesis passes 9, 10, 12, and 13):**
1. **`#1118` (`scanner-seam-window` `NOT ADOPTED`, `c0b2341c`):** `#1031` was closed without merge (`in_maint` fell only `10.689 → 7.490 ms`), restoring `~10–13 ms` to post-`#1034` serial `S'` (`~141–148 ms` default, `~135–143 ms` aligned before `A2`; [`thirteenth-pass.md`](../../2026-11-27-synthesis/thirteenth-pass.md)).
2. **Corpus knee (`#1075`):** Typical text compresses less tightly than `wikidata` (`17.7×`), giving derived i9 `k* ≈ 4.6–6.8` — scanner-bound at `k = 8` by `~0.15–0.43 S'` on default codegen.
3. **Codegen knee (`#1107` CONFIRMED, [`twelfth-pass.md`](../../2026-11-27-synthesis/twelfth-pass.md)):** JCC 32B branch alignment cuts `wikidata` `k*` by `r_aligned = 0.871` (base) / `0.911` (native), derived i9 `k* ≈ 7.0–7.3`, making aligned `wikidata` scanner-bound at `k = 8` if the i9-10900K pays the JCC penalty (settled by `#1119`).
4. **High worker counts (ninth pass):** Even on unaligned `wikidata` with today's workers, `k = 9–10` is scanner-bound (`#1056` derives `0.952×` at `k = 9` today, `0.855×` post-worker-stack).
5. **Memory safety baseline (`#1129`):** `ready-cap-worker-gate` (`1429f54f`, merged on `master`) gates workers on `ready_bytes >= cap.bytes` and bounds `free_out_bytes`; slack-only hybrid MUST preserve this invariant untouched.

---

## 2. Architecture and mandatory invariants

The implementation lives behind an opt-in research feature `--features slack-hybrid-scan` (zero behavioural or codegen change with the feature off; zero new `unsafe` outside any already-justified SIMD `Finder` path, and preferring 100% safe Rust where `Finder` meets K4):

1. **Feeder-offloaded 48-bit magic `Finder` on `mibz2-read` (zero startup tax):**
   - Primary `character-stage1` traces show `mibz2-read` is `91.6%` idle at `k = 8` (`12.34 ms` active / `134.10 ms` `feed_wait`).
   - Wrap each compressed input chunk in immutable `Arc<Chunk { base: u64, bytes: Vec<u8>, cands: OnceLock<Vec<Cand>> }>`.
   - **Chunk 0:** push `Arc::clone(&chunk0)` to the scanner channel **before** running `Finder` (`~0.41 ms` for 4 MiB at `9.84 GB/s`), then populate `chunk0.cands` via `OnceLock::set`. Serial scanning of block 0 pays `0.00 ms` startup delay.
   - **Chunks `1..N`:** carry the trailing 7 bytes of chunk `j − 1` as overlap so no 48-bit block or stream-end magic straddling a chunk boundary is missed; populate `cands` while the buffer is L2/L3-hot, then send `Arc<Chunk>`.
   - **Buffer recycling:** recover `chunk.bytes` via `Arc::try_unwrap(chunk)` on retire into `st.free_in`, avoiding steady-state reallocations.
2. **Zero-copy `Arc<Chunk>` slicing skipping seam blocks (`pool_copy = 0.00 ms`):**
   - Helpers slice candidate bytes directly from `&chunk.bytes[start..end]` with zero copies and zero allocations whenever the candidate and its successor magic (+ 16 B margin) lie within the same chunk.
   - The `~0.64%` (`~11 / 1,724`) of blocks straddling chunk boundaries (`last > chunk.bytes.len()`) are skipped by helpers and decoded normally by the serial scanner's existing chunk window.
3. **Candidate gating (`full_only`, `stream_level == 9`, and block-0 TTFB gate):**
   - **`full_only` filter:** helpers claim only candidates whose byte gap to the next magic is `>= 512 B`, eliminating `35%–41%` wasted helper handoffs on lbzip2/pbzip2 tail runts (`~2–3 µs` each, left to the serial scanner).
   - **Level-9 gate:** helpers stay dormant until the serial scanner parses a stream header with `stream_level == 9` (resetting on stream-end magic). Level 1–8 streams incur zero speculative scans.
   - **Block-0 TTFB & slack gate:** workers ALWAYS pop `job_queue` first; helper scanning activates **only when `pushed_blocks > 0 && jobs.is_empty()`**, `k >= 2`, and admission (`in_flight < slots`, `ready_bytes < READY_BYTES`) has room.
4. **Policy A jump-ahead scanner with asynchronous chain verification & full rollback (anti-`#770`):**
   - Maintain a lock-synchronized `serial_cursor` tracking the serial scanner's exact bit position and claimed/completed helper intervals.
   - When the serial scanner reaches a candidate currently in `HelperScanning` state, it **MUST NOT block** (Policy B caused `#770`'s `+55%` chain-head serialization); it jumps ahead to the next unclaimed candidate within the admission window (Policy A) and verifies `(bit, end_bit, crc)` asynchronously when the helper completes.
   - Any false-positive magic, missed magic, non-matching bit boundary, or candidate decode error triggers a clean discard/rollback: the serial scanner decodes sequentially from the exact bit cursor, guaranteeing byte-identical output and identical `Error { kind, block, bit }` on every corrupt, truncated, or adversarial input.
5. **Bounded speculation and cache locality:**
   - **`MAX_LOOKAHEAD = 8`** candidates ahead of the serial chain cursor, bounding worst-case speculative CPU and memory on dense false-magic streams.
   - **Worker-local scan scratch:** workers that enter helper mode lazily allocate a worker-local `ScanScratch` + `Vec<u16>` (`~200 KiB/worker`, `<= 1.6 MiB` at `k = 8`; `0 B` when helpers never trigger), executing `block::read_item_with_scratch` + `decode_block` end-to-end on the same core without cross-core `free_syms` coherence ping-pong, while respecting `#1129`'s `wait_worker_cap` and `push_free_out` byte bounds.

---

## 3. Experimental grid, codegen, and shallow-workstation recipe contract

### 3.1 Machine, timing protocol, and codegen
- **Target machine:** Louis's i9-10900K (Comet Lake-S, 10C/20T), unpinned, `100 ms` pre-spin, `120 s` per-decode timeout.
- **Pairing & rounds:** `>= 12` paired interleaved randomised rounds (fixed seed recorded in the recipe) plus `3` traced rounds per cell for activity ledgers (`job_wait`, `slot_wait`, `feed_wait`, helper steals/rollbacks, scan CPU). Every timed repeat verifies decoded output SHA-256 against `libbzip2`.
- **Codegen (explicitly specified per synthesis passes 10 and 12):**
  - **Primary build:** shipped default baseline x86-64 release flags (`--release`, no `-C target-cpu=native`, since `#1107`/`#1109` §3 showed `target-cpu=native` slows `S` by 5–8% on Haswell-CPUID Xeon VMs).
  - **Alignment contingency:** if `jcc-aligned-release-build` (`#1119`) adopts x86-64 32B branch alignment into shipped release flags before the executable recipe is frozen, the recipe tests `base` vs `hybrid` under the shipped flags (recording the exact flags in the bundle); otherwise an optional secondary `base_jcc32b` vs `hybrid_jcc32b` leg at `k = 8` on `wikidata x4` may be included as a reported diagnostic.

### 3.2 Corpora
1. **`wikidata x4`** (`47,419,732` B compressed, `838,800,784` B decoded, `1,724` blocks, level 9) — primary high-compression corpus.
2. **Pinned `text` panel** (deterministic multi-block level-9 text corpus `~200–400 MB` decoded, matching `text-knee-i9` / `ab gen` text) — primary scanner-bound corpus at `k = 8` (`#1075`).
3. **Guard & exactness corpora** (preflight + `k = 1, 6` wall/RSS guard): `runs`, `random`, `records`, `onebyte`, `pattern`, a level-1 multi-stream corpus, and a synthetic run-only high-expansion `zeros` stream (verifying `#1129` RSS preservation), plus adversarial false-magic and corrupt/truncated error-parity tests.

### 3.3 Worker counts and arms
- **Worker counts:** `k ∈ {1, 6, 7, 8, 9, 10}`:
  - `k = 1`: shipped-path identity check (helpers disabled; `0` helper scans).
  - `k = 6, 7`: non-regression guard (the exact regime that killed `#942`).
  - `k = 8`: primary judged count on `text` and on `C`-paired / post-worker-stack `wikidata x4` (also reported and non-regression-checked on standalone `wikidata x4`).
  - `k = 9, 10`: primary judged counts on standalone unaligned `wikidata x4` (where `S'` binds even on today's workers).
- **Arms:**
  - `base`: shipped default (`slack-hybrid-scan` off).
  - `hybrid`: `--features slack-hybrid-scan` on.
  - **Optional `C`-paired arm at `k = 8, 9` on `wikidata x4` (`base_C` vs `hybrid_C`):** paired with zero-copy `ideal-construction` oracle (or the composed merged worker stack `a6b-endpoint-redesign + forward-anchor-lanes + mtf-fold`) to test capture of the 9-core work bound `(8·W'/8 + S')/9 + 4.85 ms` when workers sit at `W'/8 <= 135 ms`.

### 3.4 Standing shallow-workstation recipe contract (`#1133` §2)
Any future executable recipe implementing this registration MUST:
1. Build `base` and `hybrid` from the **same** reviewed `HEAD` tree (verifying `HEAD:src`, `HEAD:Cargo.toml`, `HEAD:Cargo.lock` hashes) into separate `CARGO_TARGET_DIR`s — avoiding `git clone --local` history fetches that fail on `--depth 1` workstation runners (`#1122`). If comparing two commits, initialise a private `git init` directory and `git fetch --no-tags --depth 2 <origin_url> <full_sha>` directly (`#1126`/`#1127`).
2. Include an automated Python unit test reproducing a detached `git clone --depth 1 --single-branch --no-local` checkout (`file://` URI).
3. Run `cargo test --features slack-hybrid-scan` (including false-magic rollback, level 1–8 dormancy, error-order parity, and run-only RSS cap tests) in preflight before timing.
4. Verify `libbzip2` SHA-256 on every timed repeat and follow `.just/` single-tarball bundle hygiene.

---

## 4. Frozen pre-data decision rules (C1a / C2)

For each `(corpus, arm_pair, k)`, let `R = median(wall_hybrid / wall_base)` across the `>= 12` paired rounds, with exact 95% confidence interval `[lo, hi]` (order-statistic / paired-ratio CI). Every threshold below is frozen before any hardware data exists and is never relaxed after data.

### 4.1 VOID (no verdict; fix harness/preflight only)
The run is **VOID** if any of the following holds:
- Target machine is not Louis's i9-10900K, or preflight tests fail, or any repeat fails `libbzip2` SHA-256 byte-identity or errors out.
- Any 95% CI bound (`lo`/`hi`) is undefined (`< 2` valid paired rounds).
- **Activation check:** `base` (or `base_C`) dispatches `> 0` helper scans; OR `hybrid` at `k = 1` dispatches `> 0` helper scans; OR `hybrid` dispatches `0` accepted helper blocks on `wikidata x4` at `k = 9, 10` AND `0` on `text` at `k = 8` (indicating the feature was inactive).
- Any false-positive magic or rollback alters decoded bytes or error coordinates.

### 4.2 KILL (stop the slack-only hybrid line; record scoped dead-end row)
**KILL** if ANY of **K1–K4** holds:
- **K1 (non-regression guard at `k = 1, 6, 7` and standalone `k = 8`):**
  - `lo > 1.010` (`> +1.0%` wall slowdown, 95% CI excluding `+1.0%`) at any `k ∈ {1, 6, 7}` on `wikidata x4` or `text`; OR
  - `lo > 1.005` (`> +0.5%` wall slowdown) on standalone `wikidata x4` at `k = 8`; OR
  - `lo > 1.020` (`> +2.0%` slowdown) on any guard corpus (`runs`, `random`, `records`, `onebyte`, `pattern`, level-1) at `k ∈ {1, 6}`.
- **K2 (no real scanner-bound speedup):** `hi >= 1.000` on ALL primary scanner-bound cells (`wikidata x4` at `k = 9`, `wikidata x4` at `k = 10`, `text` at `k = 8`, and `C`-paired `wikidata x4` at `k = 8` if included) — i.e., no statistically significant speedup anywhere the scanner binds.
- **K3 (memory & run-only cap):** median peak RSS ratio `hybrid / base > 1.050` at any `k ∈ {1, 6, 7, 8, 9, 10}` on `wikidata x4`, `text`, or the run-only `zeros` stream (verifying `#1129` is preserved).
- **K4 (finder cost):** standalone single-thread `Finder` median over `wikidata x4` (`47.42 MB`, 11 timed passes, pinned) exceeds `10.0 ms`.

### 4.3 PASS (continue to production adoption / default-enable evaluation)
**PASS** iff:
1. None of VOID, K1, K3, or K4 triggers; **and**
2. At least ONE primary scanner-bound criterion clears **`>= 3.0%` speedup (`hi <= 0.970`)**:
   - **(a) `wikidata x4` at `k = 9, 10`:** both `k = 9` and `k = 10` satisfy `hi < 1.000`, and at least one satisfies `hi <= 0.970`; **OR**
   - **(b) `text` at `k = 8`:** satisfies `hi <= 0.970`; **OR**
   - **(c) `C`-paired / post-worker-stack `wikidata x4` at `k = 8`:** satisfies `hi <= 0.970` (and captures `>= 50%` of the remaining 9-core work-bound room `(8·W'/8 + S')/9` below `base_C`).

### 4.4 RESCUE (bounded diagnosis; no silent gate relaxation)
**RESCUE** iff none of VOID, K1, K3, K4 triggers, at least one primary scanner-bound cell achieves `hi < 1.000` (statistically real `0.1%–3.0%` speedup) without reaching `hi <= 0.970`, AND the trace ledger isolates a concrete removable overhead (e.g., `serial_cursor` lock contention, chunk-0 `OnceLock` delay, or excessive rollback work) with modeled headroom `>= 3.0%`. Any rescue requires a distinct follow-on claim and fresh pre-data registration; this verdict itself stands verbatim (C2).

---

## 5. Reported diagnostics (non-gating)

Every run records these diagnostics beside the verdict to catch `#770` serialization or `#942` overhead immediately:
- Helper blocks claimed, accepted, skipped (seam / `< 512 B` runt), and rolled back / wasted per decode at every `k`.
- Total scan CPU (`serial S + helper_scan`) vs shipped `S` CPU (expected `<= 1.05×`; catches `#770` scan duplication).
- Policy A jump-ahead skip count and scanner wait time (`slot_wait`, `in_wait`, cursor lock wait).
- Per-worker `job_wait` idle share and `mibz2-read` feeder utilization (`find` vs `feed_wait`).
- Realized wall vs `(k·W'/k + S')/(k + 1)` work bound and margin to the `140.0 ms` (`6 GB/s`) contour at `k = 8` and `k = 9`.

## 6. Out of scope here

- Production default-enablement (requires a separate adoption review once PASS is measured on i9-10900K, including `text` and `k = 1..20` scaling).
- Re-opening `#942`'s killed all-pooled prototype (`c71c4a56`).
- Modifying worker kernels (`a6b-endpoint-redesign`, `forward-anchor-lanes`, `mtf-fold`) or `#1129`'s `ready-cap-worker-gate`.
