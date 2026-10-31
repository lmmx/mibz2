# Evidence pack: machine-code analysis of `bwt::room_segments::<32, PF_L1>`

Read-only. The question, the deliverable and its limits are in §6; everything before
that is evidence.

Labels: **measured** (from a named run), **deduced** (arithmetic on measured figures),
**model** (it depends on stated assumptions), **vendor** (published figures, not re-measured
here).

## 1. Provenance of the disassembly

| Item | Value |
|---|---|
| Repository, commit | lmmx/mibz2 `ae942bd` (master, 2026-10-31) |
| Toolchain | rustc 1.97.0 (stable), host x86_64-unknown-linux-gnu |
| Profile | `release`: `lto = "fat"`, `codegen-units = 1`, `debug = "line-tables-only"`, opt-level 3 |
| Flags | no `RUSTFLAGS`, no `.cargo/config*`: baseline x86-64 target, AVX2 only through `#[target_feature]` |
| Build | `CARGO_TARGET_DIR=<scratch> cargo build --release --example phases` |
| Binary | `target/release/examples/phases`, sha256 `730bee334613efe96acdaa84bfcafe0953055e82441028a918f94e3fad37cc81` |
| Symbol | `mibz2::bwt::room_segments::<32, 1>` at `0x29a50` (`PF_L1 = 1`) |
| Files | `room_segments_32_1.asm` (`objdump -d -l --no-show-raw-insn -M intel`, with source lines; the build paths are rewritten as `<repo>/` and `<target>/`); `room_segments_32_1.nolines.asm` (the same, without them); `room_segments.src.rs` (`src/bwt.rs:829–1012` at `ae942bd`) |

**Source identity (checked):**
- `src/bwt.rs:829–1012` is byte-identical at `c8f9740` (the commit CI run 40579732655
  measured) and `ae942bd`.
- An earlier build (`phases-sel.asm`, a different build of the same source) has the same
  16-instruction hot step at `0x2d240–0x2d273`.
- So the step's shape doesn't depend on that build.

**Caveat:** CI built with `dtolnay/rust-toolchain@stable` on the day of each run, not
necessarily 1.97.0. The step's shape was stable across the two builds checked, but
re-disassembling CI's exact binary is the strict reference.

## 2. The hot step (the full-lane pass, `'full` loop, all 32 lanes active)

Source (`room_segments.src.rs`, the `for off in 0..room { for k in 0..L { … } }` loop):

```rust
struct Lane { row: u32, seg: usize, p: *mut u8, end: *mut u8 }   // 32 B, row at +0
let mut base: [*mut u8; L] = from_fn(|k| lanes[k].p);           // on the stack
for off in 0..room {
    for k in 0..L {
        let l = &mut lanes[k];
        let q = unsafe { *t.add(l.row as usize) };               // T: u32, byte in bits 0..8
        if q & START != 0 { /* segment boundary: cold, ~0.057% of steps */ }
        l.row = q >> 8;
        prefetch::<PF_L1>(t, l.row);                              // prefetcht0
        unsafe { *base[k].wrapping_add(off) = q as u8 };
    }
}
```

Machine code (`0x2a530–0x2a563`, one lane-step; the outer `off` loop is at `0x2a4f0`):

```
2a530: lea    r10,[r8+0x1]               ; k+1
2a534: mov    ebx,DWORD PTR [r11]        ; LOAD  lanes[k].row     (stack, stride 0x20)
2a537: mov    eax,DWORD PTR [r15+rbx*4]  ; LOAD  q = T[row]       (the chain's load)
2a53b: test   eax,eax
2a53d: js     2a570                      ; START is bit 31 → cold path
2a53f: mov    edi,eax
2a541: shr    edi,0x8                    ; row' = q >> 8
2a544: mov    DWORD PTR [r11],edi        ; STORE lanes[k].row = row'
2a547: prefetcht0 BYTE PTR [r15+rdi*4]   ; PREFETCH T[row']
2a54c: mov    rdi,QWORD PTR [rsp+r8*8+0x70]  ; LOAD base[k]       (stack)
2a551: mov    BYTE PTR [rdi+rcx*1],al    ; STORE byte at base[k]+off
2a554: lea    rax,[r10-0x1]
2a558: add    r11,0x20                   ; &lanes[k+1]
2a55c: mov    r8,r10
2a55f: cmp    rax,0x1f
2a563: jb     2a530                      ; k loop, not unrolled
```

**Census** (deduced from the listing):

| Count | Value |
|---|---|
| Instructions per lane-step | 16 |
| Macro-ops with test/js and cmp/jb fused | 14 |
| Per byte, with the outer `off` loop's 14 ops shared over 32 steps | ≈ 14.8 |
| Loads | 3: `row`, `T[row]`, `base[k]` |
| Prefetches | 1 |
| Stores | 2: `row`, the byte |
| Overhead, rather than chain work | `row` round-trip through the stack (1 load, 1 store); `base[k]` reload (1 load); the k loop's counter and pointer bookkeeping (`lea`, `lea`, `add`, `mov`, `cmp/jb`: 5 instructions, 4 macro-ops) |

**Dependencies:**
- The chain's own loop-carried path is `[r11]` → `T[row]` → `shr` → `[r11]`. It runs
  through memory, with store-to-load forwarding one round (32 steps) later.
- Successive lanes share only `r8`/`r10`/`r11` (the k counter and pointer) and `rcx`
  (`off`). These are cheap single-cycle chains.
- **No visible dependence makes lane k+1's `T` load wait on lane k's.** The listing
  supports this, but that is not proof about the scheduler; see question 2 in §6.

## 3. Measured inputs (each tagged by run and commit)

| Id | Figure | Source |
|---|---|---|
| M1 | Room walk **4.49 c/B**; flagged walk 6.10; bitmap (12 lanes) 13.25; **one plain chain 38.55 c/B**; assembly 0.16; stage total **10.78** (sum without assembly: 1.22 + 2.27 + 2.27 + 4.49 + 0.28 + 0.26) | CI run 40579732655, EPYC 7763 (Zen 3), 3.2407 GHz reported, at `c8f9740`, one copy |
| M2 | The same run with four concurrent copies: room 9.40, flagged 13.03, bitmap 25.15, chain 40.27 (room ×2.09) | same run |
| M3 | Lane sweep, one copy, `PF_L1`: L = 8: 8.01, 12: 6.36, 16: 5.35, 24: 4.87, 32: 5.00, 48: 5.15, 64: 4.96; L = 32 without prefetch 5.73 | CI run 40573190665, `docs/journal/2026-10-27-walk-sweep.md` |
| M4 | Lane sweep, four copies: 8: 11.97, 12: 10.60, 16: 10.23, 24: 10.28, 32: 10.37, 48: 10.46, 64: 10.83 | same |
| M5 | Per core, two copies on SMT siblings give 2 / 2.09 = **0.955×** one copy's room-walk throughput: the sibling adds nothing | deduced from M2; `refs/2026-10-30-n1-four-copy-sensitivity.md` |
| M6 | PR #114's bench, 7763: walk 6.64–6.69, total 12.93–12.98 c/B | lmmx/mibz2#114 bench comment (run 40970005807) |
| M7 | The 9V74's walk in c/B is the same in its 2.87 and 3.7 GHz groups (4.51–5.06 against 4.62–5.15) | bench comments, lmmx/mibz2 #69–#118 |

**Units:** "c/B" is cycles at the run's reported clock per output byte of the stage.

**Caveat (M7):** M7 says the walk's cost scales with the core clock, not with a fixed memory
time. That points away from a DRAM-latency floor on the 9V74.

## 4. Deduced and model figures

- **D1. The effective overlap.** ℓ/w = 38.55 / 4.49 = **8.6**: the walk produces bytes 8.6×
  as fast as one chain.
  - It does not say only 8.6 loads are in flight. Under prefetch, a lane's next demand
    load comes one round, 32 × 4.49 ≈ 144 cycles, after its prefetch. That is above ℓ ≈ 38.55
    on average, so most demand loads should hit.
  - **Caveats:** ℓ was measured on an unloaded single chain without prefetch. Prefetching
    and the 32-lane miss mix change both latency and the hit mix.
- **D2. Issue floors of today's step (vendor widths; model).**

  | Floor | Zen 3/4 | Comet Lake (i9) |
  |---|---|---|
  | Dispatch | 14.8 / 6 = 2.47 c/B | ≈ 14.8 / 4 = 3.7 c/B |
  | Memory ops (3 loads + 1 prefetch + 2 stores) | ≈ 2.0 c/B at 3 per cycle | ≈ 2.0–2.5 c/B (2 load ports, 1 store port) |

  Measured 4.49 on the 7763 is 1.8× its dispatch floor.
- **D3. B2's interval fit (model, `refs/2026-10-30-b2-gathered-walk.md`).**
  - w = U/D + (miss term), with U ≈ 14.8 ops/B and D = 6.
  - Fitted miss term ≈ 2.02 c/B on the 7763.
  - Implied average misses in flight ≈ 7.8, against 24 L1 MABs on Zen 3/4 (vendor).
  - It reproduces 4.49 only with an effective 8–9 outstanding misses: m = 8 gives 4.70.
- **D4. The lane curve (M3).**
  - From 16 to 32 lanes, cost falls only 5.35 → 5.00 (−7%). A pure latency bound would
    halve it.
  - From 24 to 64 lanes it is flat (4.87–5.15). Prefetch is worth 5.73 → 5.00.
  - Under four copies (M4) the curve is flat from 16 lanes. That fits a shared or
    issue-bound limit more than a per-lane latency limit.
- **D4′. The SMT tension (deduced from M5 and D2).**
  - One thread runs at 1.8× its own dispatch floor, so dispatch width is not saturated.
  - Yet a second thread on the same core adds no throughput.
  - So some per-core resource is saturated at one thread. Candidates: miss-handling
    (MABs, L2 miss queue), the load queue, or a statically partitioned structure (Zen 4
    halves its retire, store and op queues under SMT; vendor).
  - That fits Path A, or a mixed reading, better than pure Path B. B2's interval model
    reconciles this through its miss term. Question 6 asks whether the machine code
    supports that reconciliation.
- **D5. The batching trap.**
  - Four groups of eight run group by group expose 8 chains.
  - Latency floor ≥ 38.55/8 = **4.82 c/B**, above today's 4.49.
  - Batching helps only if each group's loads stay in the window together with the
    others'.

## 5. Targets (by revision; conditional)

| Basis | Total | −20% decoder time: walk ≤ | +20% throughput: walk ≤ | +20% throughput with P74's 0.65 c/B saved elsewhere: walk saving needed |
|---|--:|--:|--:|--:|
| Run 40579732655 (`c8f9740`), 7763 | 10.78 | **2.33** (0.52×) | **2.69** (0.60×) | **1.147** (walk ≤ 3.34) |
| PR #114 (run 40970005807), 7763 | 12.93 | — | — | **1.505** |

- **Condition:** P74's 0.65 c/B is a stage-level saving.
  - At decoder level it is measured only exploratorily: CI run 41196164480 on an EPYC 7763
    gave +5.3% at w = 1 and +3.6% at w = 2 on different-dump Wikidata.
  - The registered four-core trial hasn't run yet.
  - If P74 is negative, the needed walk saving rises to 1.80 c/B (10.78 basis).
- **Mixing caution:** don't combine figures across runs. An earlier draft used 4.10 c/B,
  a mixed-run figure. The correct figures are 2.33 or 2.69 on one consistent run.
- **Comparing a 1.15 c/B saving with D2:** removing the stack round-trip, the `base[k]`
  reload and the k-loop bookkeeping takes ~6–7 of 14.8 ops/B. At the 7763's dispatch rate
  that is ~1.0–1.2 c/B of issue. Path B can reach the 1.147 target only if the step is
  issue-bound and the removed work isn't replaced by other overhead.

## 6. The question for read-only analysis

**Main question.** Why does the 32-lane walk get 8.6× one chain and not more?
- **Path A, insufficient overlap:** the generated code or the core's resources serialise
  or throttle independent `T` loads. Candidates:
  - ROB or scheduler occupancy: 16 instructions per step × 32 lanes = 512 instructions per
    round, against a 256-entry ROB on Zen 3 and 224 on Comet Lake;
  - load-queue entries;
  - MAB or fill-buffer limits: 24 on Zen 3/4, 12 on Comet Lake;
  - prefetches that occupy miss buffers.
- **Path B, too much work per byte:** overlap is sufficient and the instruction stream
  (14.8 ops/B, 6 memory ops) sets the cost.

Focused questions:

1. **Dispatch accounting.** From the listing, what is the step's fused-domain op count,
   port pressure and best-case cycles per step on Zen 3 (7763), Zen 4 (9V74) and Comet Lake
   (10900K)? Does 4.49 fit "dispatch floor plus a miss term", and how large is each part?
2. **Overlap.** Does any instruction make lane k+1's `T[row]` load wait on lane k's
   (false dependence, store-forwarding stall, memory-disambiguation replay between the byte
   store `[rdi+rcx]` and the next `[r11]` load)? With 16 instructions per step, how many
   lane-steps fit in each core's ROB and scheduler? Is the prefetch the miss that
   occupies the MAB/fill buffer, or the demand load?
3. **Prefetch distance.** At ~144 cycles per round, is one round enough lead time for
   L3/DRAM misses on these cores? Would a two-round lead, prefetching `T[T[row]]`
   (impossible without the value) or by another scheme, matter, or does D4 rule it out?
4. **The 32-lane state.** LLVM keeps `row` in a 32-byte `Lane` on the stack and reloads
   `base[k]`.
   - Which of these could leave the hot step without reducing exposed chains? For
     example: full unrolling of k with constant offsets; a dense `[u32; 32]` row array; an
     interleaved output tile `out[off*32 + k]` that removes the `base[k]` load (at the
     price of a transpose pass); or two groups of 16 interleaved per off.
   - Per D5, any grouping must keep all 32 chains within the window.
5. **Clock scaling (M7).** Does M7 point to issue-bound (Path B) on Zen 4? Is there a
   case where it doesn't?

6. **The SMT tension (D4′).** Which per-core resource can be saturated by one thread
   running this step at 4.49 c/B, while dispatch is not? Does the answer change the
   choice between Path A and Path B?
7. **The incumbent candidate.** B2's kernel "B" (`refs/2026-10-30-b2-gathered-walk.md` §5–6)
   is a scalar tiling:
   - lanes unrolled ×4;
   - rows kept as one load and one store;
   - the byte stored into an 8-byte tile;
   - a deferred commit, with replay of failed lanes using the `START` sign bit;
   - 8.0–9.0 macro-ops/B and 3.25 loads + 2.125 stores per byte, against 14.8 and 4 + 2.03.

   Its interval model (iv) predicts **3.30–3.42 c/B** on the 7763: −1.07 to −1.19 c/B,
   −10% to −11% decoder time. That is close to the 1.147 bar of §5 and below the 2.33 bar.
   Does the machine-code reading support the model's premises (issue 2.47 → 1.80,
   miss term 2.02 → 1.46)? Or does it point to a cheaper transformation with the same
   effect? For example, full k-unrolling alone, which needs no tile, commit or replay.

**Deliverable:** at most **one** transformation, or an explicit "no". If one, give:
- the source-level change and the **expected disassembly signature**: instructions,
  macro-ops, loads and stores per lane-step, and whether k is unrolled;
- a **conservative** c/B estimate on the 7763's basis (run 40579732655), with the
  assumptions;
- a **falsification criterion** stated before it is measured. For example: "if the walk
  at L = 32 doesn't fall below X c/B on CI in the paired stage bench, or the lane sweep
  doesn't shift as predicted, the Path B reading is wrong";
- what it costs elsewhere: the transpose pass, code size, the cold `START` path, `unsafe`
  invariants, Miri coverage.

**Out of scope:**
- gathers (`vpgatherdd`: 39/42 µops on Zen 3/4, throughput 8 cycles; B2 rejected it),
  unless evidence is compelling;
- allocator changes;
- changes to the T-vector format;
- any change to production dispatch;
- any edit or run. This is analysis only.

## 7. Files

- `room_segments_32_1.asm`: the whole symbol, with source lines.
- `room_segments_32_1.nolines.asm`: the same, without source lines (1,496 lines). The hot
  step is at lines 450–465 and the outer pass at 425–449.
- `room_segments.src.rs`: the source, `src/bwt.rs:829–1012` at `ae942bd`.
- `refs/2026-10-30-b2-gathered-walk.md` (the interval model, kernel B, the gather
  rejection) and `refs/2026-10-30-n1-four-copy-sensitivity.md`. These are copied
  unchanged from branch `docs/research-portfolio` at `9723f06`, where they live; they are
  not on master.
- On master, in `docs/journal/`: `2026-10-27-walk-sweep.md`, and
  `2026-10-27-mlp-scheduling-and-run-structures.md` (vendor MAB and latency figures).
- `implementation-prompt.md`: the conditional implementation prompt. It is used only if
  the analysis returns one transformation.
- The blog post `docs/site/blog/2026-10-31-room-walk-overlap.md` (the public framing).
