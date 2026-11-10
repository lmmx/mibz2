# The run-record front's harm: touched-slot geometry and code placement, a static check

10 November 2026. This is an untimed check, with no CI bench and no registration, made
under Guardian D082 (#145 6758456856) and with the Optimiser's ranked falsifiers folded in
(#145 6758640506, `inbox-6758452562/report.md` §A2).

**The question:** whether boxing `runs::Runs` (this branch's only source change) restores
master's geometry for the slots the unchanged hot loops actually touch.

**D082's rejection test, fixed before this data:** if boxing doesn't restore master's
geometry for the touched slots, the layout hypothesis is unsupported, and no successor is
registered on it.

**D083 (G3), noted for any later registration:** MTF has twins.
- **The plain-worker path:** `decode_block`, which #324 changed.
- **The team and pipelined paths:** `mtf_checkpoints` and `coop`, which #324 deliberately
  left unchanged. CI's A/B doesn't execute them.

A registration must state each path's change or non-change, the reason, and the evidence
scope, and claim only plain-worker benefit unless separately evidenced.

**The premise, corrected.** This entry's first version read `counts` against `mtf_avx2`'s
frame modulo 4 KiB, after #277 §2. #277's own §6 withdrew that discriminator: `counts` is
1 KiB, so modulo 1 KiB it meets every residue, and what matters is the slots actually accessed.
That version's conclusion (exposure identical in all builds) was a property of the window
arithmetic, not of the accessed slots. It is superseded by §2 here. Its raw gdb probes stay in
`probes.txt`.

## Builds

`examples/ab` built as CI builds it (`cargo build --release --example ab`, rustc 1.99.1, fat
LTO, `codegen-units = 1`):
- **master:** **5e7b9954**, CI's base for #324;
- **head:** **e88c0631**, #324;
- **boxed:** e88c0631 with `Scratch::runs: Box<runs::Runs>`.

Every check uses the bench's own 1-thread arm, `ab child mibz2 1 FILE`. That is `ab::drain` →
`BzReader::read` → `block::read_item` (the Huffman decode) and `decode_block::<Segmented>` →
`mtf::mtf` → `mtf_avx2`, `bwt::build` and `room_segments`, on fallback-only inputs. The inputs
are one block of CI's `random` corpus and one of `records` (their first 890,000 bytes,
recompressed with `bzip2 -9`). Both take the column's path in every build.

## 1. Code: bodies, placement and shared callers (the Optimiser's rank 2)

**The hot kernels' code is identical.** Normalised disassembly (addresses and rip-relative
operands masked) hashes the same in all three builds for:
- `mtf_avx2` (246 instructions);
- `block::read_item` (1,908);
- `bwt::build` (888);
- `room_segments::<32, 1>` (head's and boxed's `<32, 1, false>`);
- `unrle_avx2`;
- `crc::clmul_update`.

`mtf_avx2`'s prologue and frame (0x1c0, realigned to 64) are the same.

**`finish` is not outlined in the production binary.** `decode_block::<Segmented>` calls
`unrle_avx2` and `crc32` directly on the column's path, as master does. `finish` is inlined
twice, once on each path. The Optimiser's +48 `finish` instructions belong to the diagnostic
driver's binary (`drv`, base 3e1a7ec3), not to `ab`.

**The shared callers changed:**

| caller | master | head | boxed |
|---|---|---|---|
| `decode_block::<Segmented>`: size, frame | 0x282, `sub $0x68` | 0x431, `sub $0x48` | 0x431, `sub $0x48` |
| `ab::drain::<BzReader<File>>`: size, frame, normalised hash | 0x2b5c, `sub $0x5b8`, 41850027 | 0x2c61, `sub $0x5c8`, 8cc1fe4f | 0x2c0e, `sub $0x5a8`, b50ddd33 |
| stack from `drain`'s entry to `mtf_avx2`'s entry | 1,688 B | 1,672 B | 1,640 B |

`drain` inlines `BzReader::read`, so its body and frame differ in each build.

**Placement of the hot code** (address mod 64, mod 4 KiB):

| function | master | head | boxed |
|---|---|---|---|
| `mtf_avx2` | 32, 0xca0 | 32, 0x520 | **0**, 0x000 |
| `block::read_item` | 0, 0xcc0 | 0, 0x680 | **48**, 0x630 |
| `bwt::build` | 0, 0x9c0 | 0, 0x340 | **16**, 0x290 |
| `room_segments` | 32, 0xee0 | 32, 0xce0 | **0**, 0xdc0 |
| `unrle_avx2` | 0, 0x880 | 0, 0x200 | **16**, 0x150 |
| `crc::clmul_update` | 16, 0x1d0 | 16, 0xb50 | **32**, 0xaa0 |
| `decode_block` | 16, 0xa10 | 32, 0x420 | 0, 0xf00 |

- **Head** keeps every hot kernel's alignment modulo 64 and moves them all within the page.
- **Boxed** changes the alignment modulo 64 of all six hot kernels, so it is further from
  master's code placement than #324's head is.
- **Every hot branch PC differs in both** (the Optimiser's rank 4). The predictors' indexing
  is undocumented, so no static aliasing conclusion is drawn.

## 2. Touched slots: candidate false store-to-load dependences (D082 point 1)

> **SUPERSEDED (D086, #145 6760052047).** `agg.c`, which produced this section's class
> counts, recorded only the class of each load's **first** matching store in ring order. A
> stack store could take a load that also matched a heap store, so the "fixed" hh and ss
> classes weren't independent of stack placement (D086's witness). The counts below stay as
> they were recorded. Corrected, independent per-class counts for master, #324's head, this
> branch's boxed build and the later candidate are in `2026-11-10-run-front-fallback.md`,
> §"Corrected counts", on `research/run-front-fallback`.
>
> **What the correction changes for this entry:** boxing's `mtf_avx2` D←H count (the
> `counts`→`STEPS` pair) equals #324's head's (4.06× master on random), so the verdict that
> boxing doesn't restore master's geometry stands under the corrected counter too.

**The method** (`agg.c`, `run.sh`, `all.sh`; raw output in `lackey/`):
- `valgrind --tool=lackey --trace-mem=yes` traces every data access of `ab child mibz2 1`.
  Lackey replaces no allocator, so glibc's heap layout within pages is the process's own.
- For each load issued by a hot function, `agg` checks the last 48 stores, from any function,
  for one whose bytes match the load's **modulo 4 KiB** (and separately **modulo 1 KiB**) at a
  different real address. That is a candidate false dependence on the actually touched slots.
- Each candidate is classed by where the two accesses sit:
  - **hh,** heap with heap: fixed for a build (page offsets of its allocations);
  - **ss,** stack with stack: fixed for a build (frame-relative offsets);
  - **sh,** stack with heap: set per process by stack randomisation, so it is sampled by
    padding the environment by 0, 336, 1,360 and 2,400 bytes.
- These are candidate counts, not penalties. No timing was made.

**Per hot function** (`random`: four paddings; `records`: padding 0). The 4 KiB ss count
was 0 for `mtf_avx2` and `bwt::build`, and 13–19 for `read_item`, in every build.

| function | corpus | build | 4K hh (fixed) | 4K ss (fixed) | 4K sh by stack padding 0/336/1360/2400 | 1K hh | 1K ss (fixed) | 1K sh |
|---|---|---|--:|--:|---|--:|--:|---|
| block.read_item | rand | master | 13,878–13,897 | 13–19 | 24,817 / 24,269 / 23,832 / 24,012 | 55,658–55,721 | 17,796–17,800 | 96,746 / 96,200 / 96,205 / 96,405 |
| block.read_item | rand | head | 14,053–14,066 | 13–19 | 23,862 / 24,036 / 24,548 / 24,112 | 55,737–55,788 | 70–81 | 96,598 / 96,506 / 96,515 / 96,343 |
| block.read_item | rand | boxed | 14,024–14,034 | 13–19 | 23,784 / 23,891 / 24,810 / 24,344 | 55,752–55,796 | 40,751–40,802 | 96,675 / 96,457 / 96,462 / 96,309 |
| block.read_item | rec | master | 14,260 | 19 | 23,494 | 56,534 | 17,804 | 92,880 |
| block.read_item | rec | head | 14,185 | 13 | 23,286 | 56,196 | 60 | 92,813 |
| block.read_item | rec | boxed | 14,227 | 13 | 23,352 | 56,351 | 62,585 | 92,692 |
| bwt.build | rand | master | 43,999–44,077 | 0 | 86,907 / 86,681 / 88,705 / 86,826 | 160,337–160,744 | 163,819–163,908 | 327,930 / 327,198 / 327,198 / 327,514 |
| bwt.build | rand | head | 43,950–44,136 | 0 | 86,676 / 86,913 / 88,081 / 86,738 | 160,508–161,006 | 163,758–163,927 | 325,300 / 327,129 / 327,129 / 325,917 |
| bwt.build | rand | boxed | 43,961–44,051 | 0 | 86,850 / 86,957 / 87,884 / 87,367 | 160,283–160,683 | 163,560–163,915 | 326,779 / 326,937 / 326,937 / 327,396 |
| bwt.build | rec | master | 43,193 | 0 | 88,572 | 160,211 | 79,896 | 338,634 |
| bwt.build | rec | head | 43,277 | 0 | 88,912 | 160,333 | 80,103 | 338,718 |
| bwt.build | rec | boxed | 43,201 | 0 | 89,187 | 160,457 | 79,875 | 337,919 |
| bwt.room_segments | rand | master | 7,795–7,807 | 16,705–16,708 | 41,998 / 41,714 / 41,792 / 52,675 | 31,023–31,089 | 19,385–19,392 | 123,670 / 131,620 / 131,620 / 131,748 |
| bwt.room_segments | rand | head | 7,656–7,669 | 16,700–16,708 | 44,969 / 56,977 / 41,979 / 41,830 | 30,832–31,060 | 19,374–19,404 | 125,457 / 135,819 / 135,819 / 136,733 |
| bwt.room_segments | rand | boxed | 7,678–7,695 | 16,703–16,709 | 44,016 / 56,569 / 41,655 / 41,861 | 30,711–30,889 | 19,362–19,421 | 125,625 / 136,274 / 136,274 / 135,639 |
| bwt.room_segments | rec | master | 11,535 | 16,198 | 41,918 | 41,244 | 18,854 | 126,239 |
| bwt.room_segments | rec | head | 11,637 | 16,199 | 41,442 | 41,149 | 18,868 | 128,855 |
| bwt.room_segments | rec | boxed | 11,699 | 16,198 | 41,589 | 41,198 | 18,866 | 129,567 |
| mtf.mtf_avx2 | rand | master | 25,410–27,046 | 0 | 170,046 / 169,692 / 150,247 / 1,217,044 | 283,586–359,699 | 0–1 | 1,616,935 / 2,014,532 / 2,014,532 / 2,014,736 |
| mtf.mtf_avx2 | rand | head | 58,830–69,195 | 0 | 1,055,773 / 1,291,237 / 95,163 / 95,481 | 275,828–279,113 | 0–1 | 2,006,369 / 2,039,628 / 2,039,628 / 2,040,349 |
| mtf.mtf_avx2 | rand | boxed | 58,987–69,228 | 0 | 856,015 / 1,292,764 / 95,163 / 95,483 | 276,277–365,167 | 0–1 | 1,598,272 / 2,040,951 / 2,040,951 / 2,040,953 |
| mtf.mtf_avx2 | rec | master | 24,105 | 0 | 133,219 | 355,648 | 0 | 1,686,549 |
| mtf.mtf_avx2 | rec | head | 34,017 | 0 | 1,091,191 | 247,000 | 0 | 2,304,982 |
| mtf.mtf_avx2 | rec | boxed | 34,751 | 0 | 851,398 | 361,193 | 0 | 1,676,841 |

**Reading it:**
- **`mtf_avx2`, heap with heap modulo 4 KiB, fixed for a build:**
  - **On `random`:** master 25–27 k, head 59–69 k, boxed 59–69 k.
  - **On `records`:** master 24.1 k, head 34.0 k, boxed 34.8 k.
  - **That is the largest fixed difference between the builds, and boxing keeps #324's
    value, not master's.**
- **`read_item` (the Huffman decode), stack with stack modulo 1 KiB, fixed for a build:**
  - **On `random`:** master 17.8 k, head 0.07 k, boxed 40.8 k.
  - **On `records`:** master 17.8 k, head 0.06 k, boxed 62.6 k.
  - **Boxing moves it to a third value,** further from master than #324's.
- **`mtf_avx2`, stack with heap modulo 4 KiB, set per process:**
  - **Bimodal in every build:** about 0.1–0.17 M candidates, or about 0.86–1.29 M.
  - **The "high" mode turned up:** at 1 of 4 paddings for master, and at 2 of 4 for head and
    for boxed.
  - **Four samples can't estimate each build's share of high-mode processes.** That share is
    what a per-process random effect averaged over the bench's 10 rounds would depend on.
- **`bwt::build`, `room_segments`:** every fixed class within about 2% across the builds.

## 3. The other ranks, as far as static evidence reaches

| rank | finding |
|---|---|
| 1, geometry | Fixed classes differ (§2). Boxing doesn't restore master's in `mtf_avx2` (hh 4 KiB) or `read_item` (ss 1 KiB) |
| 2, inlining and placement | Hot bodies are identical. `finish` is inlined in production. `decode_block` and `drain` changed. Head keeps hot alignment mod 64; boxed changes it in all six kernels (§1) |
| 3, stack, page and translation footprint | The stack depth to `mtf_avx2` is −16 B (head) and −48 B (boxed) against master. Head's `Scratch` carries `Runs` inline (+2,120 B, zeroed once at construction); boxed adds 8 B plus a separate 2,120-B allocation. Neither touches `Runs` per block on a fallback block, so there is no added per-block page touch. Translations were not traced |
| 4, branch history and PCs | All hot PCs moved (§1). The selector adds one branch a block. Not resolvable statically |
| 5, paired samples and runners | The harms were flagged on EPYC 7763 and 9V45 (runs 2 and 3), not on run 1's Intel 8370C. Run and host are confounded; nothing was reanalysed here |
| 6, the selector | Once a block, in `decode_block`: a call to `reads_runs` (inlined to `true`), two compares and a branch |
| 7, `Runs` allocations | Head: none on a fallback-only process (its Vecs stay empty; the arrays are inline). Boxed: one 2,120-B allocation, zeroed, at `Scratch` construction, freed at drop, never dereferenced on a fallback block |

## 4. What boxing costs on the fallback path (D082 point 2)

- **Allocation:** one `malloc` of 2,120 bytes a `Scratch`, so once a worker a process. None per
  block.
- **Zeroing:** 2 KiB once, at construction. Head zeroes the same bytes inline.
- **Indirection:** none on a fallback block, since `Runs` isn't read there. One pointer load a
  block on the records' path.
- **Lifetime and memory:** as long as the `Scratch`; +2,120 B of heap per worker, as head's
  inline bytes. Its `drop` is one `free`.
- **Code:** it changes the hot kernels' alignment modulo 64 (§1), which head did not.

## Verdict under D082

**Boxing does not restore master's geometry for the touched slots.**
- **`mtf_avx2`'s fixed heap-with-heap 4 KiB candidates** stay at #324's level: 59–69 k against
  master's 25–27 k on `random`.
- **`read_item`'s fixed stack-with-stack 1 KiB candidates** move to a third value.

**By the rule fixed in advance,** the layout hypothesis is unsupported as a basis for a
boxed successor, and no successor is registered on it. **This doesn't show layout is not the
cause.** The fixed and per-process classes do differ between master and #324, so layout stays
a live explanation; boxing just isn't a test of it.

## Not done (would need its own authorisation or more machine time)

- **The per-process stack-with-heap distribution:** `run.sh` at 64 paddings per build, about
  1.7 minutes a run, about 5.5 hours for three builds.
- **A decomposition of `mtf_avx2`'s heap-with-heap candidates by buffer:** which stores and
  which loads (`counts`, `ll` or the symbols).
- **Any timing.**
