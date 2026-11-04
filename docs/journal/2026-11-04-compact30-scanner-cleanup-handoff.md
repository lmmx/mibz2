# Compact30: the scanner and tile bounds cleanup, a design-only handoff

**What this is.** A design-only description of report 227's §D.2 candidate, removing the
redundant source bounds and control checks from constructor v2's scanner. It is written
as a self-contained case for the guardian, with:
- the frozen evidence;
- the safety argument;
- the frequencies;
- the budget and its risks;
- separate, falsifiable criteria for a possible future prototype.

**Its authority:** report 238, part 3, reinstated by report 240.

**Nothing was implemented or compiled, and nothing was timed.** No variant of the
constructor exists. Every instruction figure below is a static count from an archived
listing, multiplied by an untimed structural frequency, and no figure is converted to time.
This handoff makes no comparison with any other candidate (an SSE2 finaliser attempt among
them); that choice is the guardian's.

**The baseline, all frozen:**
- **The source:** `examples/c30walk/ctor2.rs` at `8beffe2`, constructor v2.
- **The machine code:** the audited listing archived at `cd14fa9`,
  `2026-11-03-compact30-constructor-v2-prototype/ctor2-build-false.asm`, which is
  `<c30walk::ctor2::Ctor2>::build::<false>`, 0x18e5 bytes, built with rustc 1.97.0
  (`2d8144b78`, LLVM 22.1.6). Addresses below are that listing's.

  The v2 construction harness's build (`ea2662dc…`) has the same function, identical after
  normalisation, at +0x16f0
  ([2026-11-03-compact30-construction-v2-harness/machine-code/](2026-11-03-compact30-construction-v2-harness/machine-code/)).
- **The frequencies:** the primary corpus's per-block counters, from v2's capture records in
  the audit's reproducibility package.
- **The independent audit:** report 227 §D.2, and the audit's report and scanner files in
  [2026-11-04-compact30-ultra-audit-evidence/](2026-11-04-compact30-ultra-audit-evidence/)
  (`report.txt` §E; `scanner/scanner-audit.txt` in the zip). I checked every address below
  in the listing myself, and recomputed every frequency.

**Kept separate:**
- **The finaliser:** the epoch-tagged prototype, ctor3 at `dbf5139`, is a formal STOP under
  its gate. This handoff neither uses nor combines with it.
- **The descriptor layout:** unchanged (the descriptor split is a separate candidate).

## 1. The scanner as frozen

**The source** (`ctor2.rs` at `8beffe2`):

```rust
// 98–100
fn rd(ll: &[u8], i: usize) -> u64 {
    u64::from_ne_bytes(ll[i..i + 8].try_into().unwrap())
}

// 557–576, W::tile
let u = self.y as u16;
let n1 = len.min(ROWS - self.p);
for (t, &c) in ll[i..i + n1].iter().enumerate() { … self.one(c, u + t) … }
for (t, &c) in ll[i + n1..i + len].iter().enumerate() { … self.one(c, u + n1 + t + 2) … }
if len == 16 { self.advance(16) } else { for _ in 0..len { self.advance(1) } }

// 823–863, scatter's epoch loop
while i < n {
    w.snapshot();
    let end = n.min(i + EPOCH);
    let mut held: Option<(u64, u64)> = None;
    while i + 16 <= end {
        let (a, b) = match held.take() { Some(v) => v, None => (rd(ll, i), rd(ll, i + 8)) };
        if a == b && a == a.rotate_left(8) {
            let mut e = i + 16;
            while e + 16 <= end {
                let (a2, b2) = (rd(ll, e), rd(ll, e + 8));
                if a2 == a && b2 == a { e += 16 } else { held = Some((a2, b2)); break }
            }
            … w.run(a as u8, e - i, forced_cfg) …; i = e;
        } else { … w.tile(ll, i, 16) …; i += 16; }
    }
    if i < end { … w.tile(ll, i, end - i) …; i = end; }
}
```

**Registers in the listing's scan:**

| register | holds |
|---|---|
| `%r14` | `ll.as_ptr()` (also spilled at `0x28(%rsp)`) |
| `%r13` | `ll.len()` (spilled at `0x8(%rsp)`) |
| `%r10` | `end` |
| `%r11` | i (fresh path) |
| `%r12` | e (lookahead) |

`p` lives at `0x28d8(%rsp)` and `y` at `0x28e0(%rsp)`.

## 2. The checks that would go, one by one

### C1. The fresh pair's bounds checks (`rd(ll, i)`, `rd(ll, i + 8)`)

**The compiled sequence:** eight instructions on the success path, between the retained held
test (`33ae3 test $0x1,%dl; 33ae6 jne`) and the retained loads.

| address | instruction | role |
|---|---|---|
| `33ae8` | `lea 0x8(%r11),%rsi` | i + 8: the first slice's end, **and the second load's address** |
| `33aec` | `cmp $0xfffffffffffffff7,%r11` | i + 8 overflow (`i ≤ usize::MAX − 8`) |
| `33af0` | `ja 34898` | → `slice_index_fail` |
| `33af6` | `cmp %r13,%rsi` | i + 8 ≤ `ll.len()` |
| `33af9` | `ja 34898` | → `slice_index_fail` |
| `33aff` | `lea 0x10(%r11),%rax` | i + 16, only for the next compare |
| `33b03` | `cmp %r13,%rax` | i + 16 ≤ `ll.len()` (the second `rd`) |
| `33b06` | `ja 348bf` | → `slice_index_fail` |

The loads retained are `33b0c mov (%r14,%r11,1),%r9` and `33b10 mov (%r14,%rsi,1),%rcx`.

**The dominating invariant:** the loop guard i + 16 ≤ end (`33acc lea 0x10(%r11),%rsi; 33ad5
cmp %r10,%rsi; 33add ja`), and end = min(n, i₀ + 61,440) ≤ n = `ll.len()`.

**The release-mode argument:**
- **Overflow.** `build` rejects n > MAX_N = 983,010 before `scatter`, the ordinary runtime
  test at line 701. i only increases from 0, by 16 per tile, by e − i ≥ 16 per run, or to
  `end` at a tail; every assignment keeps i ≤ end ≤ n. So i + 16 ≤ n + 16 < 2²⁰ ≪ usize::MAX
  on 64-bit targets, and none of i + 8, i + 16, e + 8 or e + 16 can wrap.
- **Bounds.** When the guard holds, 0 ≤ i < i + 8 < i + 16 ≤ end ≤ n. Both [i, i + 8) and
  [i + 8, i + 16) lie inside the slice.
- **Initialised memory.** `ll` is a `&[u8]`: every byte of it is initialised and immutable
  for the call, and nothing in `build` writes to it. The output arenas are distinct
  `&mut Vec`s, so they can't alias it.
- **Alignment.** None is promised: `ll` may be any subslice. A replacement must use unaligned
  reads.

**Its frequency** is A = S + Q fresh pairs (§3).

**The replacement:** `p.add(i).cast::<u64>().read_unaligned()` and `p.add(i + 8)…`, with
p = `ll.as_ptr()`, under a `// SAFETY:` comment citing the guard. The intended shape is two
loads, `mov (%r14,%r11,1),%r9` and `mov 0x8(%r14,%r11,1),%rcx`, with the +8 folded into the
second load's displacement. The 8 instructions go.
- **It needs `unsafe`:** a raw-pointer read. The proof is the guard above.
- **A safe alternative is compiler-contingent.**
  - Take `let src = &ll[..end]` once per epoch: one check per epoch.
  - Write the guard so that LLVM can see it can't wrap, for example `while end - i >= 16`
    with i ≤ end. Rust's `+` doesn't tell LLVM "no unsigned wrap", and that is why
    `33aec`'s overflow test exists at all.
  - Then index `src[i..i + 16]`.

  Whether LLVM drops every check is unknown until compiled.

### C2. The lookahead pair's bounds checks (`rd(ll, e)`, `rd(ll, e + 8)`)

**The compiled sequence:** five instructions per lookahead pair, after the retained guard
(`33b30 mov %rax,%r12; 33b33 add $0x10,%rax; 33b37 cmp %r10,%rax; 33b3a setbe %r9b; 33b3e
ja`).

| address | instruction | role |
|---|---|---|
| `33b44` | `lea 0x8(%r12),%rsi` | e + 8, **used only by the compare at `33b53`** |
| `33b49` | `cmp $0xfffffffffffffff7,%r12` | e + 8 overflow |
| `33b4d` | `ja 347ba` | → `slice_index_fail` |
| `33b53` | `cmp %r13,%rsi` | e + 8 ≤ `ll.len()` |
| `33b56` | `ja 347ba` | → `slice_index_fail` |

The loads retained are `33b5c mov (%r14,%r12,1),%rdx` and `33b60 mov 0x8(%r14,%r12,1),%rsi`.
The +8 is already folded here, and LLVM has already dropped the second slice's check.

**The dominating invariant** is the lookahead guard e + 16 ≤ end, under the same argument as
C1.
- e starts at i + 16 ≤ end, and advances by 16 only under the guard.
- A held pair (`33b6f`/`33b74`, a mismatching tile kept for the next outer iteration) was
  read under that guard in the same epoch, at exactly the next outer tile's index i = e.
- `held.take()` prevents its reuse, and each epoch starts with `held = None`.

**Its frequency** is J = U − Q lookahead pairs (§3).

**The replacement** is the same unaligned read at e. The intended shape keeps the two
existing loads, and the five instructions go, the LEA among them, since it feeds only the
check. It needs `unsafe` as C1 does; the safe alternative is also as in C1.

### C3. A nonuniform full tile's slice checks (`ll[i..i + n1]`, `ll[i + n1..i + 16]`)

**The compiled sequence:** five instructions per nonuniform full tile.

| address | instruction | role |
|---|---|---|
| `33ba9` | `jb 348aa` | after `33ba6 add %r11,%r15` (i + n1, retained: the second loop's start), overflow of i + n1 |
| `33baf` | `cmp %r13,%r15` | i + n1 ≤ `ll.len()` |
| `33bb2` | `ja 348aa` | → `slice_index_fail` |
| `343b0` | `cmp %r15,0x30(%rsp)` | i + n1 ≤ i + 16 (the second slice's start ≤ its end) |
| `343ba` | `jb 348d4` | → `slice_index_fail` |

**The dominating invariants:** i + 16 ≤ end ≤ n (as C1), and the phase invariant 0 ≤ p < 30.

**The release-mode argument for the phase invariant,** by induction over every write to p:
- `build` starts p = 0.
- `advance(k)` (lines 297–306) is called with:
  - k = 16, a full tile;
  - k = 1, a tail row;
  - k = m, from `fill`, with m ≤ the open portion's room ≤ 30. A portion never passes its
    group's end.
- For 0 ≤ p < 30 and 0 ≤ k ≤ 30, p + k ≤ 59. `advance` subtracts 30 exactly when p + k ≥ 30,
  so p stays in [0, 30).
- `complete` adds 32·whole to y and doesn't touch p: whole groups preserve the phase.
- Malformed counts end the build early, but never enlarge k.

**Hence, for a full tile:**
- n1 = min(16, 30 − p) satisfies 1 ≤ n1 ≤ 16.
- Both slices are well-ordered: i ≤ i + n1 ≤ i + 16 ≤ end ≤ n.
- i + n1 can't wrap.

**Its frequency** is S nonuniform full tiles.

**The replacement:** iterate over the 16 bytes at i without slice construction.
- **Either** a raw pointer and counts, with `unsafe` and the proof above;
- **or** one safe 16-byte window from the per-epoch reslice of C1, split with
  `split_at(n1)`. LLVM can see n1 ≤ 16 from the `min`, so the split needs no check; the window
  itself is compiler-contingent, as in C1.
- The intended shape keeps `33ba6 add %r11,%r15` (the second segment's start) and the
  retained second-segment presence test (`343c0 cmp $0xf,%r12; 343c4 ja`), and drops the five
  instructions.

### C4. The "impossible phase" test (an empty first segment)

**The compiled sequence:** `33bc0 cmp $0x1e,%rax` (p == 30?) and `33bc4 jne 34340` (p ≠ 30 →
the first segment's loop).
- On every valid tile the branch is taken.
- The fall-through detour, `33bca`–`33bd5`, runs only if p == 30, which the invariant
  excludes. It is dynamically dead, and no credit is taken for its bytes.

**The invariant** is that of C3: p < 30, so n1 ≥ 1 and the first segment is never empty.

**Its frequency** is S.

**The replacement:** give the first segment a do-while shape. Its body runs once
unconditionally, then loops while t < n1.
- The valid path then falls through into the loop, and both instructions go.
- **Done safely,** with checked indexing or the window, this relies on the invariant only for
  correctness, not for memory safety. If p were 30 (impossible), the body would append one
  wrong row, but would read in bounds.
- **Alternatively,** `std::hint::assert_unchecked(self.p < 30)` (`unsafe`; undefined
  behaviour if false, which the induction excludes) lets LLVM drop the test.
- The credit holds only if the emitted layout really falls through. Deleting the test while
  leaving an unconditional jump in its place earns nothing.

### Not removed, and earning no credit

These stay as they are:
- **The scan's own control:**
  - the outer guard (`33acc`–`33add`) and the lookahead guard (`33b30`–`33b3e`);
  - the held test (`33ae3`–`33ae6`);
  - the 16-byte uniformity classifier (`33b14`–`33b27`, seven instructions);
  - the lookahead comparison and the held store (`33b65`–`33b7e`);
  - the second-segment presence test (`343c0`–`343c4`);
  - every row loop's exit test;
  - the tile advance (`33a90`–`33ac0`).
- **The tail tile,** 0 < end − i < 16, the path at `339b7` onward: at most one per block's
  last epoch (61,440 is divisible by 16), 1,875 rows in all. It must keep its exact-length
  reads, and must never take a 16-byte read past `end`.
- **The destination safety checks:**
  - `one`/`refill`'s bucket exhaustion (`34361`/`34366` and `refill`);
  - `run`'s whole-run d.rank() + m ≤ lim;
  - `scatter`'s final rank check.

  None is a source check, and none may weaken. On malformed counts the first rejected
  append, and the empty arrays, must stay exactly as v2's.
- **Not credited, though it might also go:** `343b5 mov 0x8(%rsp),%r13` reloads `ll.len()`
  after the first segment. With the checks gone the length may no longer be needed in the
  loop. Any such change, up or down, is to be measured from the compiled listing, not
  assumed.

## 3. The frequencies (primary corpus, one construction)

**From the per-block counters** of v2's capture (434 blocks, 207,006,515 rows, the
`counters` and `meta` records), recomputed here:

| symbol | meaning | how | value |
|---|---|---|--:|
| T | full 16-row tiles | Σ over epochs ⌊epoch rows / 16⌋ = Σ over blocks ⌊n_b / 16⌋, since 61,440 ≡ 0 (mod 16) | **12,937,790** |
| tail | rows after the last full tile | Σ n_b mod 16 | 1,875 |
| U | uniform tiles (in runs) | run_rows / 16 = (fragment + bulk) / 16 = 168,711,632 / 16, exact | **10,544,477** |
| S | nonuniform full tiles | T − U | **2,393,313** |
| R | coalesced runs | Σ runs | **610,521** |
| H | runs ending on a held, mismatching full tile | not recorded | 0 ≤ H ≤ R |
| Q | R − H: runs ending because no full tile remains in the epoch | at most one per epoch per block, and at most the block's runs | 0 ≤ Q ≤ Σ min(epochs_b, runs_b) = **3,450** |
| A | fresh pairs | S + Q | 2,393,313 … 2,396,763 |
| J | lookahead pairs | U − Q | 10,541,027 … 10,544,477 |

**Two checks of the derivation:**
- 16·S + tail = 38,293,008 + 1,875 = **38,294,883** = the recorded scalar rows, exactly.
  Every nonuniform full tile appends 16 one-row labels, and every tail row one.
- T computed epoch by epoch equals T computed block by block.

**Why A and J are as stated:**
- Every full tile is loaded exactly once as a qword pair: fresh in an outer iteration, or in
  a lookahead.
- The outer iterations are S + R: one per nonuniform tile, one per run.
- Of those, H start from a held pair and load nothing. So A = S + R − H = S + Q, and
  J = T − A = U − Q.

**The removed instructions:**

```text
8A + 5J + 7S = 8(S + Q) + 5(U − Q) + 7S = 5T + 10S + 3Q
             = 64,688,950 + 23,933,130 + 3Q = 88,622,080 + 3Q
gross        ∈ [88,622,080, 88,632,430]
```

This matches report 227's lower end and the audit's tightened upper end (`report.txt` §E).
The audit also reconstructs this scan's whole baseline as 19S + 16U + 12R − 2Q + 2W ∈
[221,503,931, 222,731,873], where W ≤ H counts held mismatches whose first qword matched.
The candidate removes about 40% of it.

## 4. The 87.62 M budget, reconciled

| part | instructions | its status |
|---|--:|---|
| C1 checks without the address LEA (7 × A) | 16,753,191 + 7Q | **mathematically removable**: dominated by the outer guard and n ≤ MAX_N |
| C1's LEA `33ae8` (1 × A) | 2,393,313 + Q | **compiler-contingent**: its value is also the second load's address, so it goes only if the +8 folds into that load's displacement |
| C2 (5 × J), its LEA included | 52,722,385 − 5Q | **mathematically removable**: the LEA feeds only the removed compare, and the load already folds +8 |
| C3 (5 × S) | 11,966,565 | **mathematically removable**: the guard and the phase invariant |
| C4 (2 × S) | 4,786,626 | **removable if the emitted layout falls through**; otherwise compiler-contingent |
| **gross** | **88,622,080 + 3Q** | |
| allowance for work moved elsewhere | ≤ 1,000,000 | an **acceptance cap**, not an estimate |
| **conditional net** | **87,622,080 … 87,632,430** | report 227's 87.622 M |

**One correction to the audit's fallback.** The audit (`report.txt` §E) gives 74,684,290 as
the target "if only the +8 address folding fails", which charges all T LEAs to folding. In
the frozen listing, only the fresh pair's LEA (`33ae8`) carries a load address. The
lookahead's LEA (`33b44`) feeds only its bounds check, and its load (`33b60`) already uses
`0x8(%r14,%r12,1)`.
- So if fresh-pair folding alone fails, the loss is A ≈ 2.39 M, not T.
- If both lookahead and fresh addressing regressed to separate LEAs, the audit's figure
  applies.

Either way, this is an observation about the existing listing, not a credit: only the
compiled replacement decides it.

**Work that would move elsewhere,** to be charged against the cap if emitted:
- a per-epoch reslice or pointer setup: a few instructions × 3,654 epochs, about 20 k at most;
- a guard rewritten to make no-wrap visible (`end − i ≥ 16`), which may cost one instruction
  of setup per epoch or none;
- a do-while restructure that, laid out badly, adds an unconditional jump per nonuniform
  tile, up to S ≈ 2.39 M. That alone would exceed the 1 M cap, so the layout must be
  inspected;
- new base registers, spills or reloads in the scan loop, from changed register pressure;
- any change in the tail path's code.

**Unchanged by intent:**
- the source reads (T qword pairs plus the tail bytes) and every output write;
- the row classification, run coalescing and epoch splitting.

## 5. Criteria for a possible future bounded prototype (not authorised)

### 5.1 Correctness: falsifiable, against frozen v2 and the oracle

**What it would be.** A separate variant: v2 at `8beffe2`, copied, with C1–C4 only. No
finaliser or descriptor change, and `ctor2.rs` untouched.

**Each case would be checked** as v2's `check_block` does:
1. the oracle `C30::build`;
2. the candidate's counted and uncounted builds, in poisoned reused arenas, with byte-exact
   `rec` and `raw`;
3. v2's counted build on the same input, with **every counter equal field by field**:
   - scalar, fragment and bulk rows, runs, run rows and runs split at an epoch;
   - complete groups, every finish class and the normalisations;
   - epochs, epoch searches and RAW work;
   - every portion-shape cell.

   Any difference in run, fragment or scalar classification rejects it.
4. the walker's counted and audited output against production's and the oracle's;
5. the arrays again after each walk.

**The suites:** v2's, all of them:
- the 11 hash-checked corpora × 4 configurations;
- the adversarial set;
- the full synthetic suite.

**New classes, each byte-exact and counter-equal:**
1. **Short tiles and the 1–15-row tail:**
   - n = 1..=47 at every n;
   - blocks with n mod 16 = 1..=15 at the end of a full epoch and of a partial one;
   - a tail beginning at every phase p = 0..=29.
2. **Epoch boundaries:**
   - n = 61,440·k + d for d ∈ {−17, −16, −15, −1, 0, 1, 15, 16, 17};
   - a uniform run ending exactly at, one tile before, or one tile after an epoch's end;
   - a run straddling the boundary (split, `runs_split_at_epoch` equal to v2's).
3. **Held tiles and lookahead:**
   - a run followed by a different byte's uniform tile;
   - a run followed by a nonuniform tile;
   - a run followed by a tile matching in its first qword only (W), and in its second only;
   - a run whose lookahead finds no full tile (Q > 0);
   - consecutive held tiles across alternating bytes.
4. **Phase and the tile split:**
   - nonuniform full tiles starting at every p = 0..=29, so n1 = 16 at p ≤ 14 and n1 = 1..15
     at p = 15..29;
   - tiles whose label hole falls at every position;
   - sequences driving p through 29 → 30 → 0 in `advance(16)`, `advance(1)` and a run's
     `fill`;
   - runs of every length 16..=64 at every entry phase.
5. **Odd alignment** (for unaligned reads): `ll` passed as a subslice at byte offsets 1..=7 of
   its allocation, for tiny, epoch-boundary and full blocks.
6. **Malformed counts:**
   - the full v2 class (4,000 perturbations; n = 983,011 and 2²⁴);
   - rejection exactly when the counts aren't L's own, both builds agreeing, empty arrays;
   - and the same first-failing append as v2 (a counted-build counter of the rows accepted
     before rejection, equal to v2's).
7. **Allocation reuse:** v2's repeated-size class, growing and shrinking, unpoisoned and
   poisoned, with n = 0 through both builds.
8. **Empty and tiny inputs:** n = 0, 1, 2, 15, 16, 17, 31, 32, 33, with σ = 1 and 2.
9. **Miri** on a targeted selection: n ≤ 48 at offsets 0..=7, every tail length, the
   phase sweep, a held-tile run, an epoch-boundary block, and a malformed case.
   - It must report no undefined behaviour.
   - Whatever it doesn't reach is listed, not counted.

**Rejected if:**
- any byte differs;
- any counter differs;
- a read lands outside `ll`, under Miri or by inspection of the tail;
- a malformed input publishes a nonempty array or fails at a different append;
- the safety argument of §2 doesn't cover every `unsafe` block introduced.

### 5.2 Compilation: falsifiable, from the listing

**Toolchain:** rustc 1.97.0 (`2d8144b78`, LLVM 22.1.6), `--locked --release`,
`CARGO_ENCODED_RUSTFLAGS=` empty.

**Targets per success path,** counted from the guard that remains:

| path | frozen v2 | target |
|---|--:|--:|
| fresh pair, from the held test to the second load (`33ae3`–`33b10`) | 12 | **4**: test, branch, two loads, the second with folded +8 |
| lookahead per matched tile (`33b30`–`33b6d`) | 16 | **11**: the guard (5), two loads, the two compares and branches |
| nonuniform full tile, overhead outside its 16 row appends | 41 + 7X | **34 + 7X**: the five slice checks and the phase test gone; X counts tiles taking the second segment |
| the row loops (the four 12-instruction scalar copies, both fragment fills, the emitter and its odd tail) | as listed | **identical instruction for instruction** |
| the tail tile, the guards, the uniformity classifier, and the destination checks | as listed | present and unweakened |

**Also required:**
- no `slice_index_fail` call site reachable from the full-tile and lookahead paths. Those at
  `34898`, `348aa`, `348bf`, `348d4` and `347ba` go; any the tail keeps are listed;
- no new spill, reload or base register inside the scan loop;
- no unconditional jump replacing the phase test;
- the whole function's instruction delta, and the explicit loads and stores on each path,
  reported;
- every added instruction outside these paths, times its frequency, ≤ 1,000,000 in all.

**Rejection conditions:**
1. Any credited check survives in an equivalent form: an overflow compare, a length compare,
   a reversed-range compare, or a p == 30 test.
2. The fresh pair's second load is not folded, and the shortfall isn't reported as such (A
   fewer instructions).
3. An instruction is added per tile or per row on any path.
4. A spill or reload appears in the scan loop.
5. Added work elsewhere exceeds 1,000,000 instructions on the primary corpus.
6. Any row loop's instructions change.
7. The tail reads past `end`.
8. Any correctness criterion in §5.1 fails.

**The ledger** would be written from the compiled paths times the frequencies in §3, with
the interval in Q kept. The 87.62 M would not be credited automatically.

## 6. Risks and open questions

- **The safe formulations are compiler-contingent.** Whether rustc removes every check is
  known only from a listing. The unsafe formulations are deterministic in what they remove,
  but each adds an `unsafe` obligation, discharged by §2's arguments.
- **Layout.** The C4 credit, and the absence of new jumps, depend on block placement.
- **Register pressure.** Freeing `%r13` (the length) may help, and a new base pointer may
  hurt. Neither is known.
- **The frequencies are exact except Q**, which isn't recorded: 3Q ≤ 10,350 instructions.
- **Unmeasured:** whether the removed checks cost anything at run time. They are predicted
  branches and simple compares, and their removal may or may not shorten any critical path.
  The audit's solved requirement, about 1.10–1.65 ns per full tile for a 10.28% completion
  reduction (`report.txt` §H, as a standalone exposure model), is a requirement, not a
  prediction.
- **Platform:** x86-64 and the pinned toolchain only, as for v2.
