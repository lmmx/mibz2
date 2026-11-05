# The compact T-vector pilot: a ψ table without the byte, on the VM (#79, research only)

- **Built:** 2026-10-28, on the VM.
- **Written up:** 2026-10-29. Until then the pilot was an unpublished branch. It is published now as evidence for #79 and for the link-only milestone recorded there.
- **Code:** `examples/compact.rs`.
- **Run:** `bench/compact.sh`, or `just bench-compact`.
- **Results:** `2026-10-28-compact-tvector/` (`compact-vm.md`, `.jsonl` and the corpus log).
- **Not run on the i9.** The VM's L3 holds all four workers' tables, so it cannot show a knee (below).

## 1. What it is

**The F formulation:**
- The walk emits F[row] and moves to ψ(row), starting at `orig_ptr`. This is `bwt::walk`'s output, since L[ψ(k)] = F[k].
- F[row] needs no storage. A table of each 2^5-row window's first byte (`FTable`, n/32 bytes, plus the 257 bucket starts) gives a candidate byte. A loop then steps past any bucket boundary inside the window.
- So the tables hold **the link only**, and the byte is recovered from F.
- The lookup's address is the current row, which is known before the row's ψ load issues. So the lookup is off the link chain.

**The arms.** All use the same build: production's counting sort with its 16-row run stores, writing ψ alone. That build is **store-only**: there is no init pass and no read-modify-write of the entry's high bits.

| arm | table | bytes a row | a step's random accesses |
|---|---|--:|--:|
| `p` | production's T-vector and room walk | 4 | 1 |
| `t` | **a 32-bit ψ, link only**: the control | 4 | 1 (plus the F lookup) |
| `c` | packed 20-bit ψ: one unaligned u32 at 5r/2, shifted 0 or 4 | 2.5 | 1 (plus F) |
| `b` | libbzip2's split ll16 + ll4 | 2.5 | 2 (plus F) |

- **The walk for `t`, `c` and `b`** is the cooperative pilot's lane walk on one core: 32 lanes and 512 segments. A segment start's entry is coded n + its segment number.
- **`p` keeps the room walk's passes.** So t against p conflates the loop form with the byte's recovery, and c against t isolates the representation alone.
- **Correctness:**
  - `selftest` checks every arm against the plain walk, byte for byte, at levels 1 and 9. Cases: one byte, two bytes, a 1 MB run, period 3, period 1000, 2 MB random and 2.4 MB text-like.
  - The grid's round −1 checks every arm's every block against production's output by hash.

## 2. The VM's result (431 blocks, 209.7 MB, 8 rounds, pinned and checked)

The T-vector phase only: the build and the walk, not the decoder. The table is `compact-vm.md`. The paired phase ratios below were computed from the `.jsonl`, as production's seconds over the arm's, mean (min–max):

| | build | walk | whole phase, MB/s |
|---|--:|--:|--:|
| t1 against p1 | **1.030** (1.007–1.049) | 0.826 (0.774–0.905) | 0.86 (medians 207 against 247) |
| t4 against p4 | **1.001** (0.857–1.091) | 0.794 (0.631–0.904) | 0.828 ±0.078 |
| c4 against t4 (the 20-bit packing alone) | 0.15 | 0.74 | 0.42 |
| b4 against t4 (the split layout alone) | 0.43 | 0.77 | 0.67 |

**What it shows:**

1. **The link-only, store-only build gains little on the VM.**
   - It is 3% at one worker, with a tight range.
   - At four workers it is within noise.
   - Production's init pass and read-modify-write cost about 3% of the build there. On the VM the build is about 24% of p1's phase (0.206 of 0.858 s).
2. **The walk's 0.79–0.83 is not the cost of recovering the byte.** It conflates three things:
   - the loop form, since `t` has no room passes; #77 measured the room loop at 1.03–1.06 in the cooperative harness, on the i9;
   - the start test, `v >= n` against the START bit;
   - the F lookup.
   The pilot has no arm that walks its lanes with the byte embedded, so the recovery cost is **not isolated**.
3. **Narrow stores are the packed build's cost.**
   - `c` writes each entry as byte stores, with the nibble it shares read-modify-written. Its build runs at 0.15× `t`'s (155 against 1,052 MB/s at one worker).
   - That is the single-thread form of the parallel-store problem a packed table has. A byte-aligned 24-bit entry avoids the nibble read-modify-write, but not the narrow stores, unless it uses wide stores where they're safe.
4. **The packing's read costs a quarter of the walk:** c is 0.74× t at both 1 and 4 workers. The causes are the shift, the unaligned load and line crossings (5r/2 crosses a line for some rows).
5. **No residency effect could appear on the VM.** Its L3 (32 MB) holds four 3.6 MB tables with room to spare. The hypothesis that a smaller table moves the knee is untested. It needs the i9 at 6–10 workers (#79's pass/fail).

## 3. What it changes

- **For #79:** its "0.7–0.8×" is the walk of `t`, `c` and `b` against production's. That conflates the loop form, the byte's recovery and, for c and b, the packing. Only c against t is the representation's own cost on the VM.
- **For the link-only milestone (#79's comment):**
  - Variant B, the 4-byte link-only table, **already exists** as `t`. Its VM build gain is about 3%.
  - What's missing:
    - an arm with the lane walk and the byte embedded, which isolates the recovery cost (the first-column packing, A′);
    - a 24-bit byte-aligned arm, with wide stores inside each (thread, bucket) sub-range and narrow ones at its last entry;
    - an F-window sweep (2^4–2^8);
    - the i9 at the knee and past it.
  - The pilot is the harness to extend, rather than a new one.
