# 2026-11-10: The forced-team screen (D083): design (DRAFT, not frozen)

**Status: DRAFT for pricing (Optimiser) and PROCEED (Researcher). Not frozen. Nothing in it
has been run.** It changes no workflow, and it adds no cell to any existing acceptance
matrix (bench.yml's A/B, its report or its assessment). The stub `bench/team_screen.py` has
only been syntax-checked.

Authority: the Guardian's D083 (#145 6758740649) and D084 (#145 6759487306, §3). This is
**not** the forced-team registration that a pass of `2026-11-10-staged-traverse-count.md`
would have earned. That entitlement is unearned (D084 G2.2). This is the routine coverage
design D083 asked for.

## 1. The gap it closes (read: #330 §7, D083)

**CI's A/B never runs a team:**
- the runners have 4 vCPUs on 2 cores;
- `Plan::for_threads` gives `Workers` below 6 physical cores;
- the pipelined default needs exactly 10 physical cores, exactly 20 MiB L3 and threads of 10
  or more.

So these never execute on any PR: `mtf_checkpoints_avx2`, `FusedBuild::share`,
`coop::traverse` / `traverse_staged`, and the relay. A change to them reads as "no change".
Generic teams engage on hardware from 6 cores, so a shared-kernel port reaches more than
the i9.

## 2. The screen

**The arms:** each a harness child (`ab child ARM THREADS FILE`) of a build made in the
screen's own job.

| arm | build | command | role |
|---|---|---|---|
| **H-team** | head | `ab child mibz2-pipe3-both 3 FILE`, with the head's research selector set where the PR adds one (for #334: `AB_STAGED_TRAVERSE=1`) | forced head: one pipelined team of three (leader + 2 helpers), park and small |
| **B-team** | base (merge base) | `ab child mibz2-pipe3-both 3 FILE` | forced base |
| **H-plain** | head | `ab child mibz2 3 FILE` | the ordinary pair, retained (`Workers` on the runner) |
| **B-plain** | base | `ab child mibz2 3 FILE` | the ordinary pair, retained |

**Why `pipe3-both@3`:**
- it is one pipelined team: its relay, the leader's join, its solo path below `MIN_COOP`,
  and the levers the default runs with;
- a team of two alone does not cover the relay (D083);
- `@9` (three teams) is in the full tier (§5).

**The inputs** (frozen by digest at registration):
- **W:** `corpora/wikidata.bz2` (SHA-256 `82f65ace…c2c29915`). It has 431 blocks: 233 of at
  least `MIN_COOP` rows, alternating with 198 short ones, so **every short/large transition
  of the pipeline** occurs (the leader's solo path, lever B's run of short blocks, the team
  resuming).
- **C:** the bench's generated corpora at the bench's scale (`ab gen corpora 4`): text,
  random, records, runs, pattern, onebyte.
- **E1:** W truncated mid-block, at a fixed byte offset frozen at registration.
- **E2:** W with one byte of a large block's stored CRC flipped, at a fixed offset frozen
  at registration.

### The assertions (each a FAIL of the screen)

**A1. Exact outputs and errors.** For each input, the four arms agree:
- the same hash and length (`out`, `hash`), or the same error kind at the same block, with
  the same bytes before it;
- the outputs also equal `lbzip2 -dc`'s for W and C.

**A2. The executed kernels.** One callgrind execution each of H-team and B-team on W:
`valgrind --tool=callgrind --dump-instr=yes --separate-threads=yes`, one file a thread.

| arm | must execute | must not execute |
|---|---|---|
| H-team, for #334 | `coop::traverse_staged`, on **both** helper threads | `coop::traverse` |
| B-team | `coop::traverse`, on both helper threads | `coop::traverse_staged` |
| both | `mtf::mtf_checkpoints_avx2` (the leader), `FusedBuild::share` (both helpers), `Relay::walk_and_leave` | |

- **Other ports:** the must and must-not lists are the PR's own, given in its registration.
  The "both" row stays for every port.
- **Shared code generation can change** (D084 §3): the plain and solo paths keep their own
  checks in A1 even where their source is unchanged.

**A3. Cooperative engagement** (H-team and B-team, from the same profiles):
- each helper's walker calls = its `FusedBuild::share` calls = **233**, the cooperative
  blocks of W;
- each helper made at least one successful claim, so walked at least one segment;
- the walked bytes, summed over threads (take + full phase + tail), equal W's cooperative
  walked bytes (209,697,662);
- the claims equal the takes plus one exhausting claim per walker call.

**No balance bar here.** Callgrind runs one thread at a time, so the helpers' byte split
is valgrind's schedule (D084 §2). The split is archived (A4), not judged. A balance
predicate belongs to a native diagnostic (`2026-11-10-staged-engagement-diagnostic-draft.md`),
not to this screen.

**A4. Matched per-thread instruction counts, archived separately.** The harness has no
counter of its own. The profiles of H-team and B-team are gzipped into the job's artifact
`team-screen-<head>`, beside a per-thread table from `bench/team_screen.py` (§3), as
`team_gap_count.py staged` prints it:
- each thread's total Ir and its build, walk and coordination Ir;
- its walker calls, claims, walked bytes by kind, and fragments;
- the walk and inversion stages in Ir/D, for both arms.

**Never in the bench comment's acceptance table.** It is a screen artifact: no
performance inference, no merge criterion.

### The dispositions

- **Any of A1–A3 failing** fails the screen job; the PR's merge is blocked. The usual CI
  incident classes apply (`docs/team/CI-INCIDENTS.md`).
- **A4** carries no disposition.
- **The screen qualifies nothing.** No performance claim, no target or native
  qualification: a forced, oversubscribed arm cannot qualify target performance (D083).

## 3. The harness stub (`bench/team_screen.py`, in this PR, not run)

- **`check`** reads one arm's per-thread profiles with `team_gap_count.py`'s parser.
  - It applies A2 (`--require SYMBOL=MIN_THREADS`, `--forbid SYMBOL`) and A3
    (`--coop-blocks`, `--walked`), and with `--table` writes A4's per-thread table.
  - It exits nonzero on any failed assertion, naming it.
- **`outputs`** compares the harness's child JSON records (`out`, `hash`, `err`) across
  arms for A1.
- **Still to write before freezing:**
  - the workflow, below;
  - the E1/E2 offsets;
  - the per-PR must/must-not lists' home (proposed: a block in the PR's registration,
    copied into the workflow's dispatch input).

**The workflow, proposed:** `.github/workflows/team-screen.yml`, a job of its own, never a
step of `bench.yml`, so that no acceptance cell is added and none removed.
- It is triggered on pull requests touching `src/coop.rs`, `src/mtf.rs`, `src/parallel.rs`,
  `src/bwt.rs`, `examples/ab.rs` or `bench/team_screen.py`.
- It builds head and base as bench.yml does (`cargo build --release --example ab`, the base
  in a worktree), installs valgrind and lbzip2, and makes C with `ab gen`.
- It runs §2 in a fixed order: B-team, H-team, B-plain, H-plain natively on W, C, E1 and
  E2; then callgrind B-team, then H-team, on W.

## 4. The price (model, from counted and logged figures; not measured on CI)

**The sources:**
- bench run 42758830562 (#334's, EPYC 7763 runner, warm cache): checkout 8 s; lbzip2 25 s;
  head build 27 s, base build 25 s; corpora 7 s; Wikidata windows 10 s;
- this machine: callgrind of `pipe3-both@9` on W took about 30 s an arm (the archived run's
  file times), at about 8.0 G Ir;
- `@3` executes about the same Ir (the same kernels on the same blocks; fewer threads).

| part | seconds, warm cache | cold cache |
|---|--:|--:|
| setup: checkout, apt (valgrind, lbzip2), toolchain | about 45 | about 45 |
| builds: head and base `ab` (fat LTO) | about 55 | 300–600 |
| inputs: corpora, Wikidata windows, E1/E2 | about 20 | about 20 |
| native arms: 4 arms × (W, C's 6, E1, E2) = 36 decodes, about 0.1–0.6 s each | about 15 | about 15 |
| callgrind: B-team and H-team on W, about 30–60 s each | 60–120 | 60–120 |
| checker: parsing about 6 profiles with `--dump-instr`, 2 arms | about 20–40 | about 20–40 |
| artifact upload (about 1–2 MB gzipped) | about 5 | about 5 |
| **job total** | **about 3.5–5 min** | **about 8–14 min** |

**Seconds a round:** the screen has no rounds. It is one execution per arm; Ir counts are
deterministic but for the schedule. The measurement part (native, callgrind, checker) is
about **95–175 s per screened PR**.

**Where it costs:**
- in runner minutes: a second job of about 4 min, in parallel with bench.yml's 8 min;
- not in the PR's wall: it adds none to the bench job and nothing to its 10-round A/B.

**The tier (proposed):**
- **PR tier, on the affected paths above:** §2 as written, one team (`@3`). The cost fits a
  parallel job of about 4 min, and it is the bounded one-team screen D083 prefers.
- **Full tier** (push to master, the Actions tab, or a `bench:full`-style label if one is
  adopted):
  - add `pipe3-both@9` (three teams: the relay's interaction across teams), counted in
    callgrind the same way;
  - archive the counts, about +60–120 s.

  The three-team counts stay out of the PR tier (D083: "broader counts and three-team
  interaction in full tier").

**What the price leaves out:** queueing for a runner; a cold-cache base build on its first
run; a re-run after a red setup (class B).

## 5. Open questions for the Researcher

1. **A3's engagement:** is "every helper walked at least one segment in total" enough under
   serialised execution, or should A3 require at least one claim per block per helper?
   Under callgrind a helper can legitimately find a block's queue exhausted.
2. **The must and must-not lists:** per PR (as proposed), or a standing list per kernel?
3. **The E1/E2 offsets:** should they be chosen inside a block the team walks, a block the
   leader solos, or both (two more inputs)?
4. **Should A4's table also record the old/new walker's Ir/D on W at `@3`?** As a figure
   it is diagnostic only. Recording it risks it being read as a gate.
