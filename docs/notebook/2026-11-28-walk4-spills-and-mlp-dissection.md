# Dissection: `Walk::run<4>` Stack Spills, Inline `LONG`, 1-Step Pipelining & `edges[next]` Prefetch (`src/open/lanes.rs`)

**Scope & honesty notice:** All timings, RSS readings, and disassembly metrics in this note were measured on a shared Linux KVM VM (`Intel(R) Xeon(R) CPU @ 2.30GHz`, 2 vCPUs, 45 MiB L3, `rustc 1.99.1`, `[profile.release]` `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`) on top of `#1312`'s bounded-slab four-lane forward walk (`18d46bf8`, `--features forward-anchor-lanes`, `<= 80 KiB/worker` `Arena`, `#![forbid(unsafe_code)]` in `src/open/lanes.rs`). **These are VM whole-decoder (`examples/ab.rs`) measurements, NOT physical i9-10900K adoption evidence.**

## Verdict: `STOP` — `#1312` `Walk::run<4>` Sits on a Tight x86-64 16-GPR Pareto Optimum

Across **34 distinct `Walk::run<4>` variants** spanning 1-step `edges[next]` software prefetch (`T0`/`T1`/`T2`), cold-outlined `LONG` register-diet kernels, packed `(off << 32) | j` state, zero-`memcpy` overlapping-16B `17..=32 B` inline `LONG` (both rolled and explicitly unrolled), batched 4-lane all-short fast-paths, and 100% safe 1-step software-pipelined `edges` lookahead — **no variant cleared the JCC-32B-verified whole-decoder win bar (`>= 1.5%` Wikidata CPU / `>= 4%` `A7` at `<= 1.005x` RSS)**:

- In the **definitive standalone `n = 11` alternating paired showdown** (`ab_w0` vs standalone `ab_w4a_t2`, our strongest screening finalist, `63/63` exact `AB_SHA256=1` parity vs C `libbzip2` across all 7 corpora, `0 vzeroupper / 0 ymm`, `<= 80 KiB/w` `Arena`):
  - **`JCC-32B` (`n = 11`):** **0/4 cells disjoint faster** (`1W` warm `-0.88% [0.9747, 1.0080]`, `1W` cold `+0.69% [0.9802, 1.0344]`, `8W` warm `-2.58% [0.9476, 1.0017]`, `8W` cold `+1.07% [0.9831, 1.0391]`; RSS `1.0024x..1.0277x`).
  - **`base` (`n = 11`):** **2/4 cells disjoint slower** (`1W` warm `+0.57% [0.9951, 1.0164]`, `1W` cold `+0.99% [0.9881, 1.0322]`, `8W` warm **`+2.58% [1.0074, 1.0445]` DISJOINT SLOWER**, `8W` cold **`+4.63% [1.0127, 1.0810]` DISJOINT SLOWER**; Wall `+5.61%`).
- **Root cause (§4):** `W0`'s `81` stack reloads (`28` loads + `9` stores executed per normal 4-lane step) all hit L1d (`4-cycle` latency, 2 loads/cycle) and retire completely in the shadow of the 4 random `edges[j[0..3]]` L2/L3 loads (`14–45` cycles over `445 KiB` avg `edges`). Every rewrite that eliminates `W0`'s `4 × call *memcpy` or cuts stack spills trades hidden L1d stack ops for a strictly larger penalty on the critical path (`1.71M` cold `LONG` calls, rolled inner-loop branch/index ops, `23.3%` batch-guard branch mispredicts, or `139–180` stack spills when keeping extra live registers).

---

## 1. Disassembly & Spill Census of Shipped `#1312` `Walk::run<4>` (`W0`)

`objdump -d` on standalone `ab_w0_base` (`0x12f810`, `2,111 B`, `490 insns`) and `ab_w0_jcc` (`0x1364c0`, `2,209 B`, `505 insns`):

- **Static totals (identical across `base` and `jcc`):** `0 vzeroupper / 0 ymm`, **`81` `(%rsp)` loads + `33` `(%rsp)` stores (`114` total)**, `17` calls (`4` inline `call *memcpy@GLIBC_2.14` + `6` cold `spill_word` + `7` panic stubs), `55` `cmp`/`test`, `76` jumps.
- **Why LLVM spills:** The `else if len == LONG` fast-path emits **4 non-cold `call *memcpy@GLIBC_2.14`** right inside the hot loop (`0x12fa16`, `0x12fae4`, `0x12fbb0`, `0x12fc7d`). Because each `memcpy` clobbers all 9 caller-saved GPRs (`rax, rcx, rdx, rsi, rdi, r8..r11`), LLVM has only 6 callee-saved GPRs (`rbx, rbp, r12..r15`) for ~25 live values (`lanes`, `inline.ptr`, `edges.{ptr,len}`, `lens.ptr`, `nw`, `spelling.{ptr,len}`, `bits`, `mask`, `done`, `iters`, `j[0..3]`, `off[0..3]`, `e[0..3]`).
- **Dynamic hot-path stack ops per normal (non-`LONG`, non-spill) 4-lane iteration:** **`28` stack loads + `9` stack stores = `37` stack ops/iter**, including reloading `bits` (`0x78(%rsp)`), `nw` (`0x60(%rsp)`), `mask` (`0x3c(%rsp)`), and `lens.ptr` (`0x70(%rsp)`) **4 times each per iteration**, plus 4 reloads of `lanes[i].slab` from `(%r15)` because `&mut lanes[i]` passed to `spill_word` prevents LLVM from proving `Box<[u8; SLAB]>` pointers loop-invariant.

---

## 2. Candidate Design Space, Disassembly Audit & Stage-1 Screening

All variants preserve `#![forbid(unsafe_code)]` in `src/open/lanes.rs` (software prefetch variants isolate a non-faulting `PREFETCHT0`/`T1`/`T2` helper in `src/rle.rs`), `<= 80 KiB/worker` `Arena`, `0 vzeroupper / 0 ymm`, and 100% SHA-256 exactness (`3f075e30f4f8c734...` on `wikidata.bz2`).

### 2.1 Disassembly Census (`JCC-32B`, `Walk::run<4>` Variants)

| Variant | Mechanism | `jcc` size / insns | `rld / rst` (tot) | `pf` | `memcpy` | `cmps / jmps` | Loop shape |
|---|---|---:|---:|---:|---:|---:|---|
| **`W0`** (`#1312`) | Shipped `Walk::run<4>` (`lanes` in `%r15`, inline `LONG` `memcpy`) | `2,209 B` / `505` | `81 / 33` (`114`) | `0` | `4` | `55 / 76` | Unrolled 4× |
| **`W1_t0/t1/t2`** | `W0` + 1-step `edges[next]` prefetch (`T0`/`T1`/`T2`) | `2,393 B` / `550` | `103 / 36` (`139`) | `4` | `4` | `57 / 80` | Unrolled 4× |
| **`W2a`** | Cold `spill_or_long`, hoisted `&mut [u8; SLAB]`, countdown `rem`, `PF=0` | `1,749 B` / `396` | `85 / 36` (`121`) | `0` | `0` | `30 / 43` | Unrolled 4× |
| **`W4a_t0/t1/t2`** | `W2a` + 1-step `edges[next]` prefetch (`T0`/`T1`/`T2`) | `1,765 B` / `395` | `87 / 37` (`124`) | `4` | `0` | `32 / 45` | Unrolled 4× |
| **`W2b` / `W4b_t*`** | `W2a` + zero-`memcpy` overlapping-16B `17..=32 B` `LONG` (`PF=0` / `T0,T1`) | `1,233–1,251 B` / `278–287` | `29–32 / 30–35` (`62–64`) | `0` / `4` | `0` | `25 / 33–34` | **Rolled** by LLVM |
| **`W2c` / `W4c_t*`** | `W2a` + packed `u64` `(off << 32) | j` (`PF=0` / `T0,T1`) | `1,754–1,916 B` / `399–434` | `72–85 / 38–39` (`110–124`) | `0` / `4` | `0` | `29–32 / 43–46` | Unrolled 4× |
| **`W3` / `W4p_t0`** | `W2a` + 1-step safe pipelined `ne = edges[nj]` (`PF=0` / +2-step `T0`) | `1,749–1,820 B` / `380–409` | `85–88 / 50–51` (`136–138`) | `0` / `4` | `0` | `27–28 / 36–37` | Unrolled 4× |
| **`W5_0` / `W5_t*`** | 4-lane all-short batch guard `(len0\|..\|len3) < 32` + cold `emit4_slow` | `1,529–1,577 B` / `345–360` | `42–50 / 45–46` (`88–95`) | `0` / `4` | `0` | `27–30 / 30–31` | Unrolled 4× |
| **`W6_0` / `W6_t*`** | Explicitly unrolled macro + `first_chunk/last_chunk::<16>` `<=32B` `LONG` | `2,667–2,676 B` / `577–587` | `105–117 / 49–51` (`156–166`) | `0` / `4` | `0` | `58–60 / 77–81` | Unrolled 4× |
| **`W6p`** | `W6_0` + 1-step safe pipelined `ne = edges[nj]` (`#![forbid(unsafe_code)]`) | `2,619 B` / `563` | `111 / 69` (`180`) | `0` | `0` | `57 / 73` | Unrolled 4× |
| **`W7_0` / `W7_t*`** | `W0` (`lanes` in `%r15`) + zero-`memcpy` `<=32B` `LONG` | `1,181–1,221 B` / `268–280` | `35–39 / 24–32` (`63–67`) | `0` / `4` | `0` | `25 / 34` | **Rolled** by LLVM |
| **`W8`** | `W0` (`lanes` in `%r15`) + 1-step safe pipelined `ne = edges[nj]` | `2,444 B` / `544` | `95 / 52` (`147`) | `0` | `4` | `52 / 74` | Unrolled 4× |
| **`W9_0` / `W9_t*`** | Cold `spill_or_long_lane` keeping `lanes` in `%r15` (`PF=0` / `T0,T1,T2`) | `1,579–1,686 B` / `365–388` | `69–87 / 28–35` (`97–122`) | `0` / `4` | `0` | `30–32 / 42–47` | Unrolled 4× |

### 2.2 Stage-1 Whole-Decoder Screening (`wikidata.bz2`, `1W lazy`, `JCC-32B` & `base`)

Paired geometric mean CPU vs `W0` (`n = 5..7` alternating interleaved rounds, pinned CPU 0):

| Group | Variant | `jcc` `1W` warm vs `W0` | `jcc` `1W` cold vs `W0` | `base` `1W` warm vs `W0` | `base` `1W` cold vs `W0` | Outcome |
|---|---|---:|---:|---:|---:|---|
| **W1 (`W0+PF`)** | `w1_t0` / `t1` / `t2` | `+1.18%` / `+3.93%` / `+4.03%` | `-0.67%` (`t0`) | `+2.06%` (`t0`) | **`-3.32%`** (`t0`) | Spills `114->139`; loses `jcc` warm |
| **W2 (diet)** | `w2a` / `w2b` / `w2c` | `+3.60%` / **`+5.01%`** / `+3.38%` | `+1.89%` (`w2a`) | `+1.24%` (`w2a`) | `-4.83%` (`w2a`) | `1.71M` `LONG` calls (`w2a/c`) or rolled (`w2b`) |
| **W3 (pipe)** | `w3` / `w4p_t0` | `+2.09%` / `+3.85%` | `+0.71%` (`w3`) | `+3.10%` (`w3`) | `-0.50%` (`w3`) | `136–138` spills + `1.71M` calls |
| **W4a (finalist)** | `w4a_t0` / `t1` / `t2` | `-1.79%` / `-2.48%` / `-2.58%` *(run 2: `+0.45%` / `-1.02%` / `-1.23%`)* | `-0.11%` / `-1.98%` / `-2.48%` *(run 2: `-1.05%` / `+1.10%` / `+0.84%`)* | `+1.65%` / `-0.50%` / `+3.26%` *(run 2: `-0.09%` / `+3.36%` / `-0.67%`)* | `-4.75%` / `-2.27%` / `-1.33%` | **Promoted to `n=11` standalone test** |
| **W4b/c** | `w4b_t0` / `w4c_t0` | `-1.00%` / `-1.24%` | `-1.13%` / `+1.76%` | `+2.88%` / `+3.63%` | `-3.91%` / `-1.60%` | Dominated by `W4a` |
| **W5 (batch)** | `w5_0` / `t0` / `t1` / `t2` | `+6.15%` / `+1.73%` / `-0.63%` / `+0.09%` | **`+8.45%` / `+3.76%` / `+5.71%` / `+6.62%`** | **`+4.59%` / `+3.68%` / `+4.44%` / `+3.49%`** | — | **DISJ SLOWER** (`23.3%` batch mispredicts) |
| **W6 (unroll)** | `w6_0` / `t0` / `t1` / `t2` / `w6p` | `+0.48%` / `+0.68%` / **`-1.43%`** / `-0.66%` / `-1.98%` | **`+2.39%`** / `+1.98%` / `+2.35%` / `+1.86%` / `+4.15%` | — | — | `156–180` spills; regresses L3-cold |
| **W7/W8/W9** | `w7_*` / `w8` / `w9_*` | **`+6.81%..+9.27%`** / `+3.57%` / `+1.13%..+4.49%` | — | — | — | Rolled (`W7`), `147` spills (`W8`), `lanes` reload (`W9`) |

---

## 3. Definitive Standalone `n = 11` Paired Evaluation (`W4a_t2` vs `W0`)

To eliminate multi-variant binary-layout coupling, `W4a_t2` (`run4_diet::<3>`, `1,729 B` `base` / `1,765 B` `jcc`, `0` `memcpy`, `4` `prefetcht2`, `0 vzeroupper / 0 ymm`, zero `OnceLock` dispatch) was compiled as a clean standalone binary and benchmarked directly against untouched standalone `W0` (`18d46bf8`, `--features forward-anchor-lanes`) across **`n = 11` alternating interleaved rounds** (`df = 10`, `t = 2.2281`), `64 MiB` LLC flush between cold runs, and **`63/63` exact `AB_SHA256=1` parity vs C `libbzip2`** across all 7 corpora (`wikidata`, `text`, `runs`, `onebyte`, `records`, `pattern`, `random`):

| Cell (`wikidata.bz2`, `n = 11`) | `W0` med CPU (ms) / RSS (KiB) | `W4a_t2` med CPU (ms) / RSS (KiB) | `W4a_t2 / W0` CPU [95% CI] | `W4a_t2 / W0` Wall [95% CI] | `W4a_t2 / W0` RSS [95% CI] |
|---|---:|---:|---|---|---|
| **`jcc` `1W` lazy warm** | `874.00` / `25,008` | `856.20` / `25,124` | `-0.88%` `[0.9747, 1.0080]` (ovlp) | **`-6.14%`** `[0.8834, 0.9972]` | `1.0059x` `[0.9960, 1.0158]` |
| **`jcc` `1W` lazy cold** | `850.10` / `24,980` | `850.50` / `25,000` | `+0.69%` `[0.9802, 1.0344]` (ovlp) | `+3.42%` `[0.9122, 1.1724]` | `1.0024x` `[0.9913, 1.0136]` |
| **`jcc` `8W` mibz2 warm** | `886.20` / `52,056` | `874.00` / `53,384` | `-2.58%` `[0.9476, 1.0017]` (ovlp) | `+0.29%` `[0.9307, 1.0807]` | `1.0106x` `[0.9857, 1.0361]` |
| **`jcc` `8W` mibz2 cold** | `912.40` / `52,624` | `949.50` / `53,720` | `+1.07%` `[0.9831, 1.0391]` (ovlp) | `+2.09%` `[0.9727, 1.0714]` | `1.0277x` `[1.0036, 1.0525]` |
| **`base` `1W` lazy warm** | `932.80` / `24,968` | `942.70` / `24,876` | `+0.57%` `[0.9951, 1.0164]` (ovlp) | `-4.40%` `[0.8747, 1.0447]` | `0.9963x` `[0.9848, 1.0079]` |
| **`base` `1W` lazy cold** | `922.30` / `24,948` | `931.20` / `24,876` | `+0.99%` `[0.9881, 1.0322]` (ovlp) | `+2.18%` `[0.9827, 1.0626]` | `1.0011x` `[0.9855, 1.0170]` |
| **`base` `8W` mibz2 warm** | `942.60` / `52,764` | `971.30` / `52,508` | **`+2.58%`** `[1.0074, 1.0445]` (**DISJ SLOWER**) | `+3.76%` `[0.9980, 1.0787]` | `0.9971x` `[0.9561, 1.0398]` |
| **`base` `8W` mibz2 cold** | `937.60` / `53,576` | `958.70` / `52,920` | **`+4.63%`** `[1.0127, 1.0810]` (**DISJ SLOWER**) | **`+5.61%`** `[1.0135, 1.1004]` | `0.9904x` `[0.9451, 1.0379]` |

---

## 4. Mechanistic Takeaway

1. **Why `#1312`'s `Walk::run<4>` (`W0`) wins despite `81 rld / 33 rst` and `4 × call *memcpy`:**
   - Every stack slot reloaded in `W0` (`bits`, `mask`, `nw`, `lens.ptr`, `edges.ptr`, `done`, `iters`, `j[0..2]`, `off[1..3]`) lives in the top `136 B` of the thread stack — 100% L1d-resident (`4-cycle` hit, 2 loads/cycle) — and retires out-of-order while the 4 random `edges[j[0..3]]` loads wait `14–45` cycles on L2/L3.
   - `LONG` words (`> 16 B`) occur on `6.43%` of Wikidata steps (`1.71M` steps / `23.3%` of 4-lane rounds), and `99.7%` fit in the current `4 KiB` slab. Outlining in-slab `LONG` to a function call (`W2a`/`W4a`/`W9`/`W5`) pays `1.71M` calls + branch overhead (`+1.9%..+8.5%`), which exceeds any L1d spill savings; keeping `LONG` inline without `memcpy` either forces LLVM to roll the `0..4` loop (`W2b`/`W7`: `+5.0%..+9.3%` slower) or inflates code and spills when explicitly unrolled (`W6`: `2,676 B`, `166` spills, `+2.39%` cold).
   - Adding 1-step `edges[next]` prefetch (`T0`/`T1`/`T2`) recovers `~3–5%` when paired with cold `LONG` (`W4a`), but only brings `W4a` back to parity with `W0` on `JCC-32B` (`-2.58%..+1.07%`, 0/4 disjoint) while regressing `base` `8W` by `+2.58%..+4.63%` (disjoint slower), and adding prefetch directly on `W0` (`W1`) pushes spills from `114 -> 139` (`+1.18%..+4.03%`).
2. **Disposition:** Closed as a scoped `STOP` row in `frontier/dead-ends.tsv`; scratch worktree removed cleanly. Keep `#1312`'s `Walk::run<4>` intact as the `forward-anchor-lanes` walk kernel. Any future `A7` win beyond `#1312` must change the memory/algorithmic structure (not micro-schedule spills, `LONG` outlining, or 1-step `edges` prefetch inside `Walk::run<4>`).
