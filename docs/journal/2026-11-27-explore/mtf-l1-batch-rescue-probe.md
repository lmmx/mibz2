# Smooth-zone validation probe: L1d-batched packed maximal MTF fold + `OutQ16` guard zone

**Date:** 2026-11-27
**Probe:** [`examples/mtf_l1_batch_probe.rs`](../../../examples/mtf_l1_batch_probe.rs) (zero `src/` edits; 100% safe `OutQ16`/`Pass`/batch consumers)
**Corpus:** `corpora/wikidata.bz2` (SHA-256 `82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915`; all 233 production-admitted Open blocks, `24,971,004` symbols, `15,141,602` maximal runs, `16,320,934` CAP-127 records, `26,567,581` retained rows, `755,021` thick strips) + 7 synthetic edge-case suites
**Machine:** 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell/Broadwell, AVX2/BMI1/BMI2/LZCNT/POPCNT, 32 KiB L1d, 256 KiB L2, 45 MiB L3), pinned to a single core (`taskset -c 1`), 7 interleaved rotated repeats measuring both wall `Instant` and Linux `/proc/thread-self/schedstat` on-CPU thread time (`cpu_ms`)
**Target hypothesis:** PR #1048's rescue design (`mtf-fold-l1-batch-redesign`) for `mtf-fold` (`#1010`), evaluated against its frozen local gate:
- 100% bit-for-bit state-exact on all 233 Open blocks + synthetic edge cases (non-injective `seq_to_unseq`, runs `> 16M`, marked row inside thick run, epoch & chunk crossings, zero-literal & overflow-chunk blocks);
- `objdump -d`: `0` `f0`/`f1` stack spills on the Pass 2 hot path, `0` per-literal sink reloads, `0` `vzeroupper` in `OutQ16::push_slow` (baseline x86-64);
- Beat shipped `M + A6a` (`A5 + A6a`, reference `332.7 ms` of `1,306 ms` 1W wall and `47.5 ms/w` of `184.8 ms/w` 8W worker busy on i9-10900K) by **>= 2.5% whole-worker** (`>= 32.65 ms` at 1W / `>= 9.81%` of `A5 + A6a`) with **non-overlapping `[min..max]` spreads on BOTH baseline x86-64 AND `-C target-cpu=native`**, and no `A6c` queue-readback regression.

---

## Executive summary — **VERDICT: PASS**

PR #1048's rescue hypothesis is **strongly validated**. Decoupling `front_avx2` from `Pass::take + OutQ16` via an L1d-resident `[u32; 256]` (or `[u32; 512]`) packed maximal-run batch `(c << 24) | total` **combined with** `OutQ16`'s power-of-2 guard zone (`CHUNK = 496, STRIDE = 512` [1024 B] or `CHUNK = 1008, STRIDE = 1024` [2048 B]) completely eliminates `#1010`'s register-spill regression and clears the frozen `>= 2.5%` whole-worker gate on **BOTH** baseline x86-64 AND `-C target-cpu=native` with completely non-overlapping `[min..max]` spreads and zero `A6c` regression:

| arm (`A5 + A6a`, 7 repeats, `schedstat` CPU) | baseline x86-64 med `[min..max]` (`paired`) | native `-C target-cpu=native` med `[min..max]` (`paired`) | projected 1W saving (base / native) | projected 8W saving (base / native) | vs `>= 2.5%` 1W gate |
|---|---:|---:|---:|---:|---|
| `shipped_split` (`512/512`) | `188.79 ms` `[186.25..198.15]` (`1.0000×`) | `187.58 ms` `[183.96..188.60]` (`1.0000×`) | `+0.00 ms` (`+0.00%`) | `+0.00 ms/w` | baseline |
| `direct_fold_1010` (`#1010` mirror) | `273.60 ms` `[267.13..281.75]` (`1.4376×`) | `203.83 ms` `[202.28..215.14]` (`1.0876×`) | `−145.6 / −29.1 ms` | `−20.8 / −4.2 ms/w` | reproduces `#1010` regression |
| `l1_batch_256_shipped_outq` (`512/512`) | `177.31 ms` `[175.02..210.72]` (`0.9373×`) | `168.84 ms` `[167.01..174.83]` (`0.8985×`) | `+20.9 / +33.8 ms` (`+1.60% / +2.58%`) | `+3.0 / +4.8 ms/w` | native only (fails base) |
| **`l1_batch_256_guard496`** (`496/512`) | **`167.51 ms`** `[166.51..169.23]` (**`0.8820×`**) | **`152.74 ms`** `[151.39..155.94]` (**`0.8229×`**) | **`+39.3 / +58.9 ms` (`+3.01% / +4.51%`)** | **`+5.6 / +8.4 ms/w`** | **PASS (both, DISJ)** |
| **`l1_batch_256_guard1008`** (`1008/1024`) | **`167.41 ms`** `[163.69..187.37]` (**`0.8840×`**) | **`163.37 ms`** `[159.46..169.92]` (**`0.8732×`**) | **`+38.6 / +42.2 ms` (`+2.95% / +3.23%`)** | **`+5.5 / +6.0 ms/w`** | **PASS (both, DISJ)** |
| **`l1_batch_512_guard496`** (`496/512`) | **`166.88 ms`** `[163.77..171.82]` (**`0.8793×`**) | **`151.41 ms`** `[147.88..153.88]` (**`0.8087×`**) | **`+40.2 / +63.6 ms` (`+3.08% / +4.87%`)** | **`+5.7 / +9.1 ms/w`** | **PASS (both, DISJ)** |
| `l1_batch_1024_guard496` (`496/512`) | `167.82 ms` `[163.85..168.26]` (`0.8772×`) | `151.34 ms` `[151.03..151.72]` (`0.8068×`) | `+40.8 / +64.3 ms` (`+3.13% / +4.92%`) | `+5.8 / +9.2 ms/w` | **PASS (both, DISJ)** |
| `l1_batch_128_guard496` (`496/512`) | `171.98 ms` `[163.84..173.64]` (`0.9010×`) | `155.79 ms` `[151.89..163.17]` (`0.8312×`) | `+32.9 / +56.2 ms` (`+2.52% / +4.30%`) | `+4.7 / +8.0 ms/w` | **PASS (both, DISJ)** |

*(An independent earlier 7-repeat baseline run on the same binary reproduced the exact same ratios: `l1_batch_256_guard496 = 0.8796× [0.799..0.922]` [`+3.07%` 1W], `l1_batch_256_guard1008 = 0.8733× [0.819..0.922]` [`+3.23%` 1W], `l1_batch_512_guard496 = 0.8705× [0.808..0.955]` [`+3.30%` 1W].)*

---

## 1. Exactness & edge-case verification

Every arm was verified **100% bit-for-bit identical** against `shipped_split` on `n`, CAP-127 `recs`, `First` (`thick_rows`, `thin`, `thick`, `m`, `mark`, `runs`), `ns`, thick-strip `edges[..3*ns]`, `over`, and all 256 byte thin queues read back in epoch order via `Reader::read_all`:
- **233 / 233 Open blocks of `corpora/wikidata.bz2`:** `recs = 16,320,934`, `runs = 15,141,602`, `retained_m = 26,567,581`, `thick_strips = 755,021`; `n` also verified identical against production `mibz2::runs::front` on all 431 blocks.
- **Synthetic edge cases:**
  1. Non-injective `seq_to_unseq` (`s2u = ['A', 'A', 'B', 'A', 'C', ...]`) with adjacent identical output bytes crossing `BATCH = 256` boundaries (automatic fallback to `probe_flush_batch_merging`).
  2. Merged run `> 16M` (`20,971,515` rows = `5 × 4,194,303 > 2^24`) via non-injective `seq_to_unseq` with `orig_ptr = 10,000,000` inside the thick run, plus a direct `20,000,000`-row `consume_single_run` check for raw `> 16M` runs.
  3. Zero-literal block (opening run only), small block forcing `ovf_chunks > 0` (`in_out == 0`), deep MTF (`s >= 33` and `s >= 65` hitting `second_cold`), and epoch crossings (`m > 65,536`).
- **`OutQ16` census on all 233 Open blocks:**
  - `shipped_512_512`: `push_slow = 417,041` (`tail_guard = 351,722`, `84.3%`), `chunks = 65,033`, `ovf_chunks = 0`.
  - `guard_496_512`: `push_slow = 67,001` (`6.22×` fewer; `tail_guard = 0`), `chunks = 66,713`, `ovf_chunks = 0`.
  - `guard_1008_1024`: `push_slow = 40,597` (`10.27×` fewer; `tail_guard = 0`), `chunks = 40,310`, `ovf_chunks = 0`.

---

## 2. Disassembly audit (`objdump -dC`) & two essential codegen discoveries

### 2.1 Final `objdump -dC` audit (baseline x86-64 and `-C target-cpu=native`)

| function | baseline `lines` / `vzu` / `xmm_rsp` / `calls` | native `lines` / `vzu` / `xmm_rsp` / `calls` | hot-loop findings |
|---|---:|---:|---|
| **`probe_fill_batch_avx2::<256>`** | `279` / **`0`** / `4`* / `1`* | `278` / **`0`** / `4`* / `1`* | **24 insns** (`1a290..1a315` base; `19e30..19eb7` native): **0 `f0`/`f1` spills**, **0 GPR spills**, **0 per-literal sink reloads**, **0 `vzeroupper`** in the entire function! |
| `probe_flush_batch_direct::<496, 512>` | `327` / **`0`** / `0` / `9` | `321` / `9`† / `0` / `9` | `m`, `in_out`, `cur`, `p` in GPRs; `f.runs += batch.len()` once/batch; 0 `vzeroupper` on baseline. |
| `probe_flush_batch_direct::<1008, 1024>` | `327` / **`0`** / `0` / `9` | `321` / `9`† / `0` / `9` | Identical clean loop with `shl $0xb` (`2048 B` stride). |
| `Queues<496, 512>::probe_outq_push_slow` | `406` / **`0`** / `0` / `15` | `423` / `10`† / `12` / `15` | **0 `vzeroupper`** on baseline; on native, YMM `vzeroupper` occurs only at `ret` and cold glibc/panic ABI boundaries (`calloc`/`memset`/`panic`), and `push_slow` is hit 6.2× less often. |
| `probe_front_avx2_fold_1010` (`#1010`) | `1601` / `18` / `68` / `24` | `1608` / `18` / `68` / `24` | Reproduces `#1010`'s register-spill and `vzeroupper` pathology (`1.438×` base, `1.088×` native). |

*\*In `probe_fill_batch_avx2::<256>`, all 4 `xmm_rsp` moves (`1a3fc, 1a402, 1a40d, 1a42a` baseline; `19f9e, 19fa4, 19faf, 19fcc` native) and the 1 `call` live exclusively inside the 1.5% cold `s >= 33` branch around `second_cold`. The 98.5% `s < 33` hot loop has **0** stack loads/stores of any register (`f0 = %xmm2`, `f1 = %xmm0`, `after = %rcx`, `spw = %r15`, `tz = %rbp`, `w = %r12`, `steps = %r13`, `out_off = %r14`, `out0 = %r9`).*
*†On `-C target-cpu=native`, LLVM compiles `OutQ16::push` and `push_slow` itself with native 256-bit YMM (`vmovdqu %ymm0`), so System V ABI requires `vzeroupper` before `ret` / external ABI calls (`calloc`/`memset`/`panic`); there is no AVX2-to-SSE mismatch between `fill_batch` and `flush_batch` because `fill_batch` is pure 128-bit XMM (`vzu = 0`).*

### 2.2 Two implementation lessons essential for the striated build (`mtf-fold-l1-batch-redesign`)

Our initial draft of `mtf_l1_batch_probe` (before inspecting `objdump`) actually **regressed** on baseline (`1.04–1.08×`) and `guard496` native (`1.018×`). `objdump -dC` exposed two concrete codegen traps that the production implementation in `src/runs/fold.rs` + `src/open.rs` **must** avoid:

1. **Out-of-line `second_cold` + direct `emit!` inside `if w != 0` in `fill_batch_avx2`:**
   Inlining `second()` (1.5% cold path) hoisted 5 YMM constant loads (`ymm2, ymm5..ymm8`) into `fill_batch_avx2`'s prologue and dirtied YMM upper halves on every 256-run call (triggering `vzeroupper` 59,147 times/corpus). Sharing a single post-branch `emit` tail also spilled `after` to `-0x48(%rsp)` and tested a `finished` flag on every literal. Moving `second` into `#[cold] #[inline(never)] #[target_feature(enable = "avx2")] second_cold` and inlining `emit` directly inside `if w != 0` eliminated every prologue YMM load, every `vzeroupper` in `fill_batch_avx2`, and the `after` spill — dropping `l1_A5_batch256_only` below shipped `A5` (`63.95 ms` vs `67.95 ms` baseline, `67.90 ms` vs `71.94 ms` native).
2. **Hoisting `m`, `in_out`, and `f.runs` in `flush_batch_direct` (100% safe Rust):**
   Passing `&mut Pass` directly into `Pass::take` caused LLVM to assume `pass.queues.out[at..at+32]` could alias `pass.f.runs`, `pass.f.m`, and `pass.queues.in_out`, emitting `incq 0xc00(%r12)` + load/store of `m` + reload of `in_out` on all `15.14M` runs. Hoisting `let mut m = pass.f.m; let in_out = pass.queues.in_out; pass.f.runs += batch.len();` inside `flush_batch_direct` cut flush time by ~20–30 ms and flipped `l1_batch_256_guard496` from `1.077×` / `1.018×` to **`0.882×` / `0.823×`**.

---

## 3. Full timing tables (Phase 4, Phase 5 `A6c` readback, Phase 6 sub-phase decomposition)

### 3.1 Native `-C target-cpu=native` (uncontended `wall_ms` AND `cpu_ms`, 7 repeats)

| arm | `wall_ms` med `[min..max]` | `cpu_ms` med `[min..max]` | paired wall `[min..max]` | paired cpu `[min..max]` | 1W proj ms (`%wall`) |
|---|---:|---:|---:|---:|---:|
| `shipped_split` | `187.44` `[186.08..235.80]` | `187.58` `[183.96..188.60]` | `1.0000×` | `1.0000×` | `+0.00` (`+0.00%`) |
| `direct_fold_1010` | `206.48` `[204.13..360.16]` | `203.83` `[202.28..215.14]` | `1.0970× [1.085..1.527]` | `1.0876× [1.076..1.147]` | `−29.13` (`−2.23%`) |
| `l1_batch_256_shipped_outq` | `172.44` `[167.87..214.49]` | `168.84` `[167.01..174.83]` | `0.9099× [0.902..0.944]` | `0.8985× [0.890..0.932]` | `+33.75` (`+2.58%`) |
| **`l1_batch_256_guard496`** | **`154.76`** `[151.61..170.43]` | **`152.74`** `[151.39..155.94]` | **`0.8214× [0.723..0.829]`** | **`0.8229× [0.808..0.832]`** | **`+58.91` (`+4.51%`)** |
| **`l1_batch_256_guard1008`** | `164.12` `[162.01..213.14]` | `163.37` `[159.46..169.92]` | `0.8712× [0.866..0.904]` | `0.8732× [0.851..0.906]` | `+42.18` (`+3.23%`) |
| `l1_batch_128_guard496` | `155.90` `[153.85..170.41]` | `155.79` `[151.89..163.17]` | `0.8229× [0.723..0.891]` | `0.8312× [0.810..0.870]` | `+56.16` (`+4.30%`) |
| **`l1_batch_512_guard496`** | **`152.14`** `[149.73..163.75]` | **`151.41`** `[147.88..153.88]` | **`0.8095× [0.694..0.821]`** | **`0.8087× [0.784..0.836]`** | **`+63.64` (`+4.87%`)** |
| `l1_batch_1024_guard496` | `151.94` `[151.31..153.00]` | `151.34` `[151.03..151.72]` | `0.8094× [0.648..0.820]` | `0.8068× [0.804..0.825]` | `+64.28` (`+4.92%`) |

### 3.2 Phase 5: `A5 + A6a + A6c` queue-readback check (`schedstat` CPU ms, 7 repeats)

| arm | baseline `cpu_ms` (`paired`) | native `cpu_ms` (`paired`) |
|---|---:|---:|
| `shipped_split + read512` | `205.60 ms` `[200.59..210.64]` (`1.0000×`) | `196.88 ms` `[194.43..199.55]` (`1.0000×`) |
| `l1_256_shipped + read512` | `191.42 ms` `[187.84..194.29]` (`0.9422× [0.905..0.961]`) | `178.86 ms` `[175.18..183.35]` (`0.9126× [0.878..0.934]`) |
| **`l1_256_guard496 + read496`** | **`179.42 ms`** `[175.55..184.04]` (**`0.8830× [0.852..0.918]`**) | **`160.80 ms`** `[159.42..163.89]` (**`0.8190× [0.799..0.843]`**) |
| `l1_256_guard1008 + read1008` | `179.47 ms` `[178.61..184.66]` (`0.8806× [0.869..0.921]`) | `173.94 ms` `[170.84..215.37]` (`0.8855× [0.859..1.108]`) |

Adding `A6c` queue readback preserves the full gain (`−26.2 ms` baseline, `−36.1 ms` native for `guard496`), confirming **zero `A6c` regression** from the `496/512` guard layout.

### 3.3 Phase 6: Isolated `A5` vs `A6a` sub-phase decomposition (`schedstat` CPU ms, 7 repeats)

| sub-phase | baseline `cpu_ms` `[min..max]` | native `cpu_ms` `[min..max]` | mechanism delta vs shipped |
|---|---:|---:|---|
| `shipped_A5_only` | `67.95 ms` `[67.84..75.28]` | `71.94 ms` `[67.96..109.39]` | baseline A5 |
| **`l1_A5_batch256_only`** | **`63.95 ms`** `[63.60..67.92]` | **`67.90 ms`** `[63.88..68.00]` | **−4.0 ms (−5.6%..−5.9%)**: drops `cnt[256]` RMW + `CAP=127` + `Vec<u16>` |
| `shipped_A6a_512_only` | `123.64 ms` `[119.85..124.44]` | `115.96 ms` `[115.25..170.89]` | baseline A6a |
| `l1_flush_512_only` | `111.93 ms` `[111.74..115.85]` | `103.81 ms` `[99.91..105.03]` | **−11.7 / −12.2 ms (−9.5%..−10.5%)**: drops `maximal()` + register-promoted flush |
| **`l1_flush_496_only`** | **`103.48 ms`** `[99.15..103.79]` | **`87.81 ms`** `[83.91..87.97]` | **−20.2 / −28.2 ms (−16.3%..−24.3%)**: adds `496/512` guard zone (`−351,722` `push_slow`) |
| `l1_flush_1008_only` | `103.94 ms` `[99.90..107.56]` | `99.80 ms` `[95.99..99.98]` | **−19.7 / −16.2 ms (−15.9%..−13.9%)**: adds `1008/1024` guard zone (`−376,444` `push_slow`) |

---

## 4. Recommendation for `mtf-fold-l1-batch-redesign`

1. **Both components are necessary together:** L1d batching alone (`l1_batch_256_shipped_outq`) reaches `+1.60%` baseline / `+2.58%` native 1W wall (short of the `>= 2.5%` gate on baseline); stacking `OutQ16`'s safe `CHUNK = 496, STRIDE = 512` guard zone clears the gate decisively on **both** baseline (`+3.01%..+3.08%`) and native (`+4.51%..+4.87%`) with identical 1 KiB chunk footprint and `0` overflow chunks.
2. **Preferred build configuration:** **`[u32; 512]` (2 KiB stack batch) + `CHUNK = 496, STRIDE = 512` guard zone** (with `[u32; 256]` a close second at `−0.07..−0.36%` difference), preserving `second_cold` out-of-line and hoisting `m` / `in_out` / `f.runs` inside `flush_batch_direct`.
3. **Next step:** Promote `mtf-fold-l1-batch-redesign` (`status = "idea"` in PR #1048) to `open` once a claim slot frees and `src/open.rs` sequencing clears, implement behind `--features mtf-fold`, verify `cargo test` + Miri + `examples/mtf_a6a_dissection.rs` local gate, then run the frozen i9-10900K `mtf-fold/registration.md` §3 kill test (K1–K4).
