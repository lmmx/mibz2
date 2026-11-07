# 2026-11-07: Two "left-field" directions checked against the archive

An outside review (a ChatGPT chat) proposed two directions:
1. libsais's two-byte (super-alphabet, ψ²) inverse BWT;
2. a run-aware compact table.

This entry checks each claim against what the project has already measured. The aim is to
avoid both repeating a failed attempt and dismissing a live one. **No timing was run for
it.** Its sources are the journal on master, the research branches and the PRs, each named
where it is used. Each claim is marked **confirmed**, **corrected** or **unsupported**.

Branch names are `origin/<branch>` at the commits given. "Inversion" means the research
harness's T-vector phase (build + walk, each block's last column made outside the timing),
not the decoder.

## Direction 1: libsais's two-byte inverse BWT

### The claims, figure by figure

| claim | verdict | what the record says | source |
|---|---|---|---|
| mibz2 3×3 at 1,942 MB/s, inversion only, on the i9 | **confirmed, with a caveat** | 3x3's median in the two-arm confirmation run `coop-20261029T081605Z` (p4 against 3x3, 20 rounds; p4 1,463). It was **the research kernel**: two-pass build with its recount, plain cooperative walk, pilot commit `c2620b8` | `research/coop-fused` (PR #95; the pilot is PR #73), `docs/journal/2026-10-28-cooperative-inversion.md` §9.1, results `coop-20261029T081605Z.md` |
| libsais 3×3 at 2,355 MB/s | **confirmed, from another run** | s3x3's median in `coop-20261029T082204Z` (the E2 "kernels at a fixed footprint" run, 20 rounds), where our 3x3 ran at **1,955**, not 1,942. The discovery grid `coop-20261028T233945Z` (38 rounds) had s3x3 at 2,356 against 3x3 at 1,912 | same journal §7.1, §9.5; results `.md` files |
| 1.20× | **confirmed as a rounded figure; the pair quoted mixes two runs** | The paired in-run figure is s3x3/3x3 = **1.195 (1.169–1.222)** (run 3, §9.5); the discovery decomposition gave 1.20 (§7.5). 2,355 / 1,942 = 1.213 compares two runs. The Status page puts both figures in one table (`docs/site/background/status.md`, PR #86, last touched `5a1ddeea`) | §7.5, §9.5 |
| "inversion only" | **confirmed** | Both arms were build + walk at the T-vector phase. libsais's build is inside its walk call, so it has no separate attribution. It ran on OpenMP with k threads, `libsais_unbwt_cyclic_ctx`, 2.12.1 at `c284ba8` | §7.5; `docs/journal/2026-10-29-libsais-rematch.md` (on `research/libsais-rematch`, PR #107), "Where 2,356 MB/s comes from" |
| the two-byte port is "PRs #6 and #34" | **corrected** | #6 is the issue (LF², KKP 2012; closed not planned). #29 was the first LF² experiment (`perf/lf2`), and #34 the faithful ψ² port of libsais 2.12.1 (`perf/pair-walk`, head `a3687d8f`). Neither merged | issue #6, PRs #29 and #34 |
| "the ψ² kernel with run lanes" | **confirmed** | #34's build is a bigram counting sort with `RUN_LANES` (16 equal bytes whose LF rows' bytes are also equal go in with one store) | PR #34 body; `docs/journal/2026-10-27-pair-walk.md` (`f398966e`, PR #33) |
| it lost on CI because construction and per-step costs outweighed the saved lookups | **confirmed** | CI A/B against 0.1.4: Wikidata **+40.6%** wall at 1 thread, **+24.3%** at 4; every corpus +21% to +52%. Locally on Wikidata, the build was 7.1–7.6 c/B against ours at 2.5. The walk took half the steps, but each cost about twice as much (≈10 c/B against ≈10.4–10.8) | `2026-10-27-pair-walk.md` |
| later i9 tests showed a benefit only under heavy contention | **confirmed** | E1, the whole decoder, pairwalk / v0.1.4: **0.691, 0.816, 0.991, 1.107, 1.117** at 1, 4, 6, 8 and 10 threads. Its best was **0.856 (0.833–0.881)** of v0.1.4's best (759 MB/s at 6 threads against 881 at 4) | `docs/journal/2026-10-29-expert-review.md` §7 E1 (`155bb99b`, PR #83) |

One more caveat on the premise. #34 was closed on "the walk is instruction-bound, not
latency-bound". The walk round later reversed that premise: on CI the walk is bound by misses
in flight (`2026-11-05-walk-round.md`). The binding census says the rejection still stands.
Our port is still at least 10% slower at 4 threads on CI, and its build adds about 5 c/B
(`2026-11-05-binding-census/nogo.md`, `38af9b6f`).

### What the archive says explains libsais's lead

**The 1.20× was mostly our research kernel's own overhead, not libsais's design.**
- At 4 independent workers (§7.5, reproduced in §9.5):
  - libsais was 1.065–1.083 × production;
  - our cooperative research kernel, one core a block (4x1), was 0.865–0.868 × production.
- Their ratio, s4x1 / 4x1 = 1.227, accounts for the whole 3×3 gap.
- Cooperation lifted both kernels by the same factor: 3x3 / 4x1 = 1.531 and s3x3 / s4x1 = 1.491.
- The research 3x3 ran **35.3** instructions a byte, against libsais's **23.2** at 3×3 and
  production's 23.8 (Status page, counters of PR #84).

**Against the kernel that ships, libsais trails at the same inversion boundary.**
- The rematch (v0.2.0 at `1140793` plus harness, run at `192fca6`, result `536fe0f6`) put the
  library's `bwt::Cooperative` (the fused one-pass build and the room walk) in the very
  harness that measured 2,356:
  - **l3x3 / s3x3 = 1.114 (1.027–1.208)**, median 1.081, 19/20 rounds. l3x3 ran at 2,430 MB/s
    against s3x3's 2,260;
  - **l4x2 / s4x2 = 1.343 (1.303–1.385)**;
  - the follow-up run (`cc9febae`) reproduced l3x3 / s3x3 at **1.096 (1.080–1.112)**, with l3x3
    at 2,584 MB/s.
- **At the decoder's boundary, the MTF stage included:** a tie at 9 cores, l3x3 / s3x3 =
  **1.022 (0.986–1.059)**. At 8 cores l4x2 / s4x2 = 1.218.
- **Teams of 2, at every block count from 1 to 5:** the library leads libsais by
  **1.23–1.41 ×** (`2026-10-29-libsais-knee.md` on the same branch, result at `d5460e9`).

**libsais's surviving edge is one core a block, under contention.**
- In the matched knee sweep (`libsais-knee.md`), sNx1 / pN was:
  - **0.989** at 1 block: equal alone;
  - 1.062 at 4;
  - 1.251 at 8;
  - 1.320 at 10.
- Both curves peak at 4 blocks, so libsais's curve is higher and falls less steeply, but its
  knee is in the same place.
- The rematch's own one-core figure was p4 / s4x1 = 0.930, so libsais was 1.075 × faster.

**By mechanism:**

| candidate | what the archive says |
|---|---|
| **two bytes a step** | **The one supported candidate, and only under pressure.** On the shipped kernel's counters, libsais makes 0.897 reads past L2 a byte against our 1.366 (l3x3). It meets full fill buffers 0.080 times a cycle against 0.194. Its lead grows with live blocks, both for libsais (E2: 1.065 → 1.343 from 4 to 10 workers) and in our own port (E1). It is called "a candidate, not a finding" (`libsais-knee.md`). |
| construction | libsais's setup is ≈4.2 c/B against our port's 7.1–7.6 (`pair-walk.md`). Its ψ² build is cheaper than ours, but no faster than our own ψ build (2.5). Its parallel build recounts L and ignores the counts it is given (`expert-review.md` §2). It isn't a source of its lead over the shipped kernel. |
| walk loop | **Already transferred.** libsais's shared-offset chain decode, "room" steps with no per-chain end check, is mibz2's room walk since 0.1.5 (PR #37; `docs/site/concepts/prior-work.md`). Its other mechanics: 24 lanes (CI found 24–64 flat and 16 slower), one round of prefetch lead, chains from hashed marks, and two copies after the walk. |
| memory layout | Not a difference. Both use 4 bytes a row, 3.6 MB a full block. libsais's entry is `(ψ²(row) << 8) \| symbol`, with a 28 KB first-symbol table for the second byte. |
| OpenMP scheduling | Not an advantage. It has six parallel regions a block and four barriers in the build. One s3x3 round had a 13 ms stall in one team; the fallback path is a hypothesis for it (§7.5; `expert-review.md` §2). |
| the machine | On CI's EPYC, libsais's kernel took **1.44 ×** our time (bench 3.2.0, `2026-10-29-bz2ngrs.md` §7). The later CI kernels table gives libsais / mibz2 at 1.05 × (1.04–1.06) in time (`binding-census/accounting.md`, run 41980640481). |

### Is it today's decoder?

**The 1.20× is not.** It was measured at `c2620b8`, the research pilot: two-pass build with
its recount, cooperative walk without room passes, 4x1 at 0.865 × production.

**The rematch is current for the inversion.**
- It measured the shipped team kernel, `bwt::Cooperative`.
- `src/coop.rs` has changed by 4 lines from v0.2.0 to v0.2.5 (`git diff --stat v0.2.0 v0.2.5`).
- 0.2.5's three-team policy (#234) runs l3x3, the arm the rematch measured.
- Since 0.2.0, the changes that touch the comparison are in the MTF stage (#172, #207) and in
  `Segmented`'s walk (#157, #164, #171, #181), not in the team kernel.
- So the rematch's pass-2 figures (MTF included) predate two MTF speed-ups.
- No libsais run on the i9 postdates 0.2.0.

### What is still untried

- **ψ² inside the cooperative teams.** It was never built. The teams keep their tables in the
  L3 (l3x3 has a hit share of 0.966, against 0.951 for p4), so the pressure where ψ² pays is
  absent at 3×3. At one block alone, libsais's two bytes a step buy nothing (0.989).
- **A ψ² build at libsais's cost** (≈4.2 c/B, against our port's 7.1–7.6), and **E6**,
  libsais's walk parameters in our ψ² port. E6 was conditional on E1 passing, and E1 did not
  pass.
- **An attribution of libsais's low-pressure lead** by `perf record` of the timed rounds only.
  The rematch's symbol profiles were withdrawn as confounded by setup (`88f387f7`).
- Not libsais's, but pointed at by the same counters: **L2 prefetch in the fill-buffer-bound
  teams on the i9.** CI's sweeps found L1 prefetch fastest for one-core walks
  (`2026-11-05-walk-round.md`; `binding-census/nogo.md`). No i9 team run with L2 prefetch was
  found in the archive.

### Verdict

**Closed, because the shipped decoder already does at least as well as libsais at its operating
points.**
- **On the i9 at 3×3:** inversion 1.096–1.114 × libsais; a tie with the MTF stage included.
- **On CI's 4 one-core workers:** libsais is the slower kernel (1.05–1.44 × our time), and our
  ψ² port lost by 24% at 4 threads.
- **Where two bytes a step pays:** only one core a block, past the knee (6–10 live tables).
  The team policy exists to avoid that regime. At 4 live tables it is worth 1.06 ×.
- **It reopens only on a condition named here:**
  - a target machine and plan that keep 6 or more independent tables live; or
  - a ψ² build shown untimed (callgrind) at no more than today's one-pass build plus 1 c/B.

  Either then goes to a registered CI A/B.

## Direction 2: a run-aware compact table

### The claims

| claim | verdict | what the record says | source |
|---|---|---|---|
| "about 77.6% of rows in 32-row translation groups" | **confirmed (v1), refined (v2)** | 77.6% is census v1's share of rows in aligned 32-row groups whose ψ span is exactly 31: ψ(j) = base + (j & 31), a group inside one run of L. Each block's partial last group counts as its own. Census v2 gives **77.39%** at G = 32 (81.95% at 16, 72.18% at 64). Per full block it ranges 63.7 / 72.2 / 78.0 / 80.9 / 88.0% (min / p10 / median / p90 / max). On the 2026-10-26 corpus it is **76.80%** (FOR-14 census). | `docs/journal/2026-10-29-succinct-review.md` §9 (v1, `9b6408ba`), §13.1 (v2, `ca0ae6bb`), §18.2 (plateau-1 = RUN, `c32042b6`), on `research/psi-census` (PR #94); `2026-11-01-for14-stage1.md` §1 on `research/for14-records` (PR #130) |
| what the figure counts | **clarified** | It is row-weighted, and equal to group-weighted and step-weighted: every row is visited once, and only a block's last group is short. It counts the full blocks of the bench corpus, which is Wikidata. **It is workload-specific:** RUN is 0% of random or compressed data, 8.9% of prose, 22% of Rust source and 35% of an ELF binary (§13.2) | §13.2, §14.2, §18.1 |
| 8 B a group plus 4 B entries for the rest takes 3.6 MB to about 1 MB | **confirmed by arithmetic, for Wikidata's median block only** | A full block has 28,125 groups. 28,125 × 8 B = 225 KB of descriptors. 22.61% of rows × 900 k × 4 B = 814 KB of entries. That is **≈1.04 MB, 9.2 bits a row**. By block (non-RUN 12.0–36.3%) it is 0.66–1.53 MB. On random or compressed input it is 3.8 MB, **larger than today's 3.6 MB T-vector**. (Computed here from the census shares; not counted by `psi_census`.) | §14.2's per-block quantiles |

### Each related attempt: what it measured, and why it lost

| attempt | layout | measured | outcome | source |
|---|---|---|---|---|
| **Two-tier design** (census §9's "two-level variant", sized in §14.2) | **8-byte descriptor a group** (base 20, byte 8, format 2, start mask) + 64-byte FOR-14/EF payloads for non-RUN groups + 128 B exception entries | Untimed, exact sizes: **5.70 bits a row, 0.641 MB a full block** (descriptors 225 KB, payloads 398 KB, exceptions 18 KB). Second dependent load on 12.0–36.3% of steps (median 22.0%). Analysis (latencies assumed, §14.4): about 43 cycles a RUN step, about 90 a FOR step, about 53 on average, against today's 43. Its format branch is unpredictable in an interleaved walk. | **Never built or timed.** It was predicted to lose at or below the knee and possibly win past it. It was ranked second behind the fixed line, and the fixed line's stop pre-empted it. | `succinct-review.md` §9, §14.2–14.5 (`4ca8b398`) |
| **e arm** | the pilot's lane walk with the byte embedded (A′) | Not run. It was the missing control that would isolate the cost of recovering the byte. That cost was later isolated by compact-Ψ's `v` arm. | superseded | `succinct-review.md` §2; `2026-10-28-compact-tvector.md` (PR #91) |
| **Compact T-vector pilot** (#91, VM only) | `t` 32-bit link-only, `c` packed 20-bit, `b` split ll16 + ll4 | VM, no knee possible (32 MB L3). c's walk is 0.74 × t's, its build 0.15 × (narrow stores). b's walk is 0.77 ×, its build 0.43 ×. | negative | `research/compact-tvector` `7bbd7e5e` |
| **Compact-Ψ stage A** (o / v / u) | `o` production's entry (control); `v` a 32-bit link, byte from F; `u` a 24-bit link, 2.7 MB | i9, 20 rounds: the smaller table **moves the knee 4 → 6** (u6 / p4 1.100), and the hit share follows its bytes (v6 holds p's bytes and gets p's share). But u1 / o1 = **0.681**, and u6 / l4x2 = 0.687. Recovering the byte from F costs 16–28% of the walk. | fails stage B's gate (u1 / o1 ≥ 0.85) | `2026-10-29-compact-psi.md` §5 on `research/compact-psi` (`42794eba`, PR #112) |
| **FOR-14 / raw, stage 1** (the `h` arm; RUN folded into FOR-14, EF as RAW) | 64-byte record a group: 32 × 14-bit offsets + header; RAW groups 128 B more | 1.868 MB a block (0.519 ×). **38 instructions an ordinary step** against production's 16 (37 with BMI2). i9: **W1 = h1 / o1 = 0.475 (0.473–0.477)**, W4 = 0.688. The four-block penalty fell by 69% (0.716 → 0.223 ns a row), but the one-block cost rose by 1.401 ns a row, three times the saving. | **STOP by rule 1**. A general budget is set: an ordinary step must keep W1 ≥ 0.774 (conditional on the phase-cost model) | `2026-11-01-for14-stage1.md` §5–§9 (i9 result `204896c2`), PR #130 |
| **compact30** (30 `u16` fields + header in 64 B, RAW area) | 2.16 bytes a row, 1.95 MB a block | #155 (default): Callgrind walk **+31%** instructions, whole block +14.7%; closed. #195 (under the staged walk): **12 instructions and 3 loads a step against 8 and 2**. CI Wikidata 4-thread wall **+4.8% / +7.7%** slower (EPYC 9V74 / 7763), 1-thread +11–14%. The core term is 3–5 × the footprint term. Walk-only: −9.4% where it fits a 2 MB L2 alone (Xeon 8573C), +21% on Zen 5's 1 MB L2, and the advantage gone at four copies. | rule failed, #195 closed. Residency at four copies would need about 1 byte a row (8573C) or 0.5 (Zen 4/5) | `2026-11-05-compact30-staged.md` (`3d2ee2ed`, PR #196); PR #155; `research/compact30-census` (PR #132) |
| **Move-structure walk** (#42) | length-capped run intervals, 8-byte records; 0.64–0.82 bytes a row, 574–742 kB a mean block | VM: one chain is 59–63% faster than the plain one-chain walk. The 32-lane room walk over it is **+46% to +85%**, and build + walk **+90% to +108%** against today's. 0.34–0.57 fast-forwards a step. | negative on #42's criterion. A cheaper dependent step, but a dearer interleaved one | `2026-10-27-move-structure-walk.md` (`50d404c8`, PR #46) |
| **Few-plateau descriptor** (#99) | two bases and a split, for two-plateau groups | Two-plateau groups in one bucket are 4.18% of rows (18.5% of non-RUN rows); 42.8% of non-RUN rows are in groups of 9 or more plateaus | marginal; non-RUN groups default to payloads | `succinct-review.md` §18 |

### Is the proposed layout new?

**It is the same idea as the census's two-tier design.**
- It has the same 8-byte group descriptor (base, byte, format, start mask) and the same RUN
  test.
- The one change is the payload for non-RUN groups: whole 32-bit entries instead of FOR-14/EF
  payloads. That gives:
  - a simpler decode on the minority path (no 14-bit extraction, the cost that sank `h`);
  - a table 1.6 × as large (≈1.04 MB against 0.641 MB);
  - a worse worst case (4.25 bytes a row on high-entropy input).
- "One cache-line access in the common case" holds for RUN steps, as it does for today's step.
  The minority, 12–36% of steps by block, takes two dependent loads, behind a format branch
  that §14.4 predicts mispredicts on about a fifth of steps. That branch cost is analysis with
  assumed latencies, never measured.

**Never measured:** this layout and the two-tier design were never built, and no RUN fast path
(a format branch on the common step) was ever timed. FOR-14 folded RUN into its one format
precisely to avoid the branch.

**The indirect evidence against it is strong.**
- **The step budget:** compact30 lost on CI at 12 instructions a step, against `Segmented`'s 8
  (#195). FOR-14's budget asks W1 ≥ 0.774 on the i9.
- **The regime:** on the i9 the 3×3 policy keeps three 3.6 MB tables in the L3 (hit share
  0.95–0.97), so residency has little left to buy at the operating point.
  - Compact-Ψ found five compact teams no better than four full ones (≈1.01 × l4x2, §5.2).
  - The knee journal found 4 teams at about 320 MB/s a table at k = 2, 3 and 4 alike
    (`2026-11-07-knee-mechanism.md` E1).
- **CI:** at 1.16 bytes a row, the median block misses the four-copy residency threshold even
  on the 8573C (about 1 byte a row), and is far above Zen's (0.5).

### Verdict

**Not a new direction. It survives only as one cheap, untimed falsifier.**
- **The figures:** the 77.6% is right as v1 counted it, 77.39% in v2 and 76.80% on the current
  corpus. The ≈1 MB is right for Wikidata's median block only.
- **The design:** it is the two-tier design with raw payloads. Its family has been sized and
  analysed, and its siblings with one load a step (u, h, compact30) have each lost on step
  cost.
- **What is untested:** whether the RUN-path step can stay near `Segmented`'s 8 instructions,
  and whether the format branch really mispredicts as predicted.

**The next step, with its kill condition stated beforehand:**
1. Build the layout's RUN and non-RUN steps as a walk-only arm, untimed.
2. Count its ordinary-step instructions from the listing and its mispredicts per step
   (callgrind `--branch-sim=yes` on the bench corpus, then CI's PMU if the gate passes).
3. **Close without timing** if:
   - the RUN step exceeds 10 instructions a lane in the staged walk; or
   - the weighted step exceeds compact30's 12; or
   - mispredicts exceed 5% of steps.

   These thresholds are bounded by #195's loss at 12 instructions.
4. **Only if it passes,** run a `sweep.yml` walk-only arm (outside the PR tier), alone and at
   four copies. It must beat the room walk at four copies beyond noise on at least one runner
   model.

If the coordinator prefers not to spend even that, the alternative verdict is: closed, because
its family has lost on step cost three times, and the operating point leaves residency little
to buy.

## What changes

- **The Status page's 1,942 / 2,355 pairing** mixes two runs, and both are research-kernel or
  libsais figures from 29 October. They predate the rematch, which shows the shipped kernel
  ahead of libsais at the same inversion boundary. A correction of that page is a separate
  docs change. It isn't made here.
- **No experiment is queued by this entry.** Direction 2's falsifier waits for the
  coordinator's decision.
