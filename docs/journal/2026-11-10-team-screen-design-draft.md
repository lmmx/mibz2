# 2026-11-10: The forced-team screen (D083): design (DRAFT, revision 2, not frozen)

**Status: DRAFT. Not frozen, not adopted, nothing run on a screen decode.**

**Revision 2 applies two reviews of revision 1 (`ffb56891`):**
- **The Researcher's REVISE:** #145 6759658271; report `state/reports/team-designs-6759589291`
  at `5326b6fd`.
- **The Optimiser's price:** #145 6759713909; report
  `state/records/reviews/inbox-6759587407` at `976e1478`.
- **The Coordinator's decision from the price:** two tiers. The PR tier is native coverage
  plus semantic probes; the full tier is the counted profiles.

**What it is not:**
- **Not an acceptance cell:** it adds none to any existing acceptance matrix, and removes
  none.
- **Not the earned registration:** this is not the forced-team registration that a pass of
  `2026-11-10-staged-traverse-count.md` would have earned. That entitlement is unearned
  (D084 G2.2), and D084's FAIL-MEASUREMENT stands.

**Making either tier merge-blocking is a prospective C6 admission change.** A separate
workflow file does not itself meet C6. Before any tier reduction, the Guardian resolves
whether D083 requires full matched counts on every affected PR (the Optimiser's §4).

**The artefacts, at this head:**

| file | what |
|---|---|
| `bench/team-screen/manifest.json` | the versioned manifest (`team-screen/0-draft`): roles, arms, fixtures, transitions, limits |
| `bench/team-screen/deltas/334.json` | #334's pinned delta |
| `bench/team_screen.py` | the checker stub: `outputs` for A1; `check` for the full tier's counted A2–A4 |
| `bench/test_team_screen.py` | its 24 negative controls: synthetic records and mocked profiles, no decoder |
| `examples/ab.rs` | the child's record now carries **structured `err_kind` and `err_block`**, from the reader's `mibz2::Error` |

**The structured error fields** are new at this head. The harness now emits the block
identity the contract claims; before this, it gave only error text. A local functional
check on a truncated generated corpus agreed across arms: `mibz2` and `mibz2-pipe3-both`
both reported `Truncated` at block 1, with the same prefix. That check is not a screen run.

## 1. The gap (read: #330 §7, D083)

**CI's A/B never runs a team:**
- its runners report 4 CPUs on 2 cores;
- `Plan::for_threads` gives `Workers` below 6 physical cores;
- the pipelined default needs exactly 10 physical cores, exactly 20 MiB L3 and threads of
  10 or more.

So these never execute on any PR: `mtf_checkpoints_avx2`, `FusedBuild::share`,
`coop::traverse` / `traverse_staged`, the relay, and the pipelined leader's solo path.
Generic teams engage on hardware from 6 cores.

## 2. Fixtures and arms (the manifest)

**The arms:** each a harness child, `ab child ARM 3 FILE`.

| arm | build | env | what |
|---|---|---|---|
| H-team | head | the delta's `head_team_env` (#334: `AB_STAGED_TRAVERSE=1`) | `mibz2-pipe3-both`: one pipelined team, a leader and 2 helpers |
| B-team | base (merge base) | none | the same |
| H-plain | head | none | `mibz2`: the ordinary pair, retained |
| B-plain | base | none | the same |

**The fixtures,** each frozen by digest:
- **W:** `wikidata.bz2` (`82f65ace…`). Its counted invariants:
  - 233 cooperative (T) blocks and 198 short (S) blocks;
  - M = 209,697,662 walked bytes;
  - 238,382 takes;
  - D = 209,700,196 decoded bytes.

  Its T/S block-class sequence is frozen from `scan`.
- **C-\*:** the six generated corpora.
- **Four native error fixtures,** each derived from W by one edit:
  - truncation inside a T block (not the first), and inside an S block lying between two
    T blocks;
  - one stored-CRC bit flipped in a T block, and in an S block.

**The error fixtures' frozen terms:**
- **Each fixture records:** its digest, its block index and the block's class, and the
  expected `err_kind` (`Truncated` or `BlockCrc`) and `err_block`.
- **Truncation:** at a byte offset, the block's midpoint from `scan`'s span.
- **A CRC flip:** a bit offset and a mask, at the stored CRC's last bit, `start_bit` + 48 +
  31. It is located structurally from the span's bit fields, which can be bit-aligned.
- **The promised prefix:** every block before the failed one, decoded, given as a length
  and the harness's hash.
- **lbzip2 must reject each one** (exit nonzero). That is an independent check only; its
  message and buffered output are not compared.

**The oracle** for each valid fixture: `ab child lbzip2 1 FILE`, the harness's hash over
lbzip2's output. That is 7 oracle decodes (W and the six C), whose results are frozen and
required again on every run.
- **The checksum's scope:** the hash is a 64-bit non-cryptographic regression checksum
  plus a length, not byte-for-byte proof. The screen states that scope; stronger identity
  would need a SHA-256 of the output stream (open question 1).

**Required transitions:** S→T, T→S, S→S and T→T; the stream ending after T and after S;
an error in a T block and in an S block; the leader joining a walk.
- **Coverage is shown, not inferred from block counts:**
  - in the PR tier, by the probes' ordered per-block records (§3, T2);
  - in the full tier, by W's frozen class sequence plus the profiles' role counts.
- **A missing transition gets a crafted fixture,** added for any required transition W's
  sequence lacks (S→S if W strictly alternates).

## 3. The PR tier (routine, on affected PRs)

### T1: uninstrumented exact results and errors (A1)

**The work:** the four arms on all 11 fixtures (44 children), the 7 oracle decodes and 4
lbzip2 rejections. All are the uninstrumented builds, so A1 is preserved as it would run
in production code.

**`team_screen.py outputs`** is an admission contract.

**What it rejects:**
- a missing, duplicate or unexpected (arm, fixture) pair, or an empty set;
- a wrapper or record without its full schema: arm, fixture, build, build SHA, executable
  SHA-256, env, exit status, signal, and the record with `out`, `hash`, `err`, `err_kind`
  and `err_block`;
- a malformed identity;
- a build or environment that differs from the pinned delta;
- a process killed by a signal.

**What each fixture must give:**
- **A valid fixture:** exit 0, an empty `err`, and the oracle's length and hash. Arms that
  agree on a failure do not pass.
- **An error fixture:** its frozen `err_kind` and `err_block`, and the promised prefix.
- **The matched arms:** they must agree on (exit, out, hash, kind, block).

The 24 negative controls include every counterexample both reviews executed: four empty
objects, one empty record, matching failures, an error without its block, and others.

**The prefix's meaning:** it is mibz2's streamed output before the error. Its
schedule-dependence under the parallel reader is an open item (open question 2). The
local check above found it equal across the plain and team arms.

### T2: per-thread semantic kernel and conservation probes

**The instrument is the `walk-stats` cargo feature,** shared with the native diagnostic
(`2026-11-10-staged-engagement-diagnostic-draft.md`, §2). It is not yet implemented.

**What it records:**
- **Per-thread entry counters** in each role's kernel: the leader's `mtf_checkpoints`, the
  helpers' `FusedBuild::share`, each walker variant (`traverse` and `traverse_staged`
  alike), the relay's `walk_and_leave`, and the solo path;
- **claims and walked bytes,** counted per segment and per tile chunk, not per byte;
- **records** keyed by (build, team, thread, block), written into per-thread storage and
  published once, at the run's end.

**What the probes prove:**
- **Execution:** a counter compiled into a kernel counts its entries. That proves the
  kernel executed.
- **Absence:** the forbidden walker's counter must be zero, which proves absence for that
  build. Symbol presence (`nm`) and sampled traces can prove neither.

**The assertions:**
- **the roster:** exactly one leader and two helpers, identified by role;
- **the kernels on the same helpers:** each helper shows share = walker = relay = 233, and
  the forbidden walker's counter is 0;
- **liveness:** each helper made at least one successful claim over the run;
- **the leader:** 198 solo blocks, and at least one join;
- **conservation, per block:** walked bytes = segment lengths, and successful claims = the
  deduplicated starts;
- **the transitions** of §2, seen in the per-block records ordered by place.

**The cost and the limits:**
- **One more fat-LTO build:** the head, with the feature. It is charged in §5.
- **A changed binary:** the probes run on the instrumented build only, never in place of
  T1's uninstrumented A1. A probe result claims semantic execution and conservation in
  the instrumented execution: **neither native balance nor performance.**
- **The base gets no probes** until the instrument is on master. Until then the base arm
  has T1 only, and the probes run on the head alone.
- **No balance bar:** that belongs to the native diagnostic.

**Until T2 exists,** the PR tier is T1 alone, and the kernel and engagement evidence is the
full tier's.

## 4. The full tier, or a separate diagnostic: matched counted profiles

**Callgrind,** one execution each of H-team and B-team on W, uninstrumented builds:
`--dump-instr=yes --separate-threads=yes`. It runs on master pushes, from the Actions tab,
or as a separately registered diagnostic, never as a routine PR gate.

**`team_screen.py check`** gives A2 and A3 from the counts, as a cross-check of the
probes, and A4's table.

**Rules for the counts:**
- **Each profile matched to its own binary and source:** `--binary` and `--coop-src` are
  that arm's own and required. Thread IDs are never compared across builds.
- **Attribution is not assumed:**
  - Under LTO, the unmodified merge base may have no standalone `traverse` symbol. The
    base is **never patched** to obtain attribution.
  - An unattributable kernel is reported as UNATTRIBUTED, never as zero calls; the checker
    fails A2 on it.
  - Only an attributable symbol's call arcs show execution.
  - **The base arm's A2 for the walker is therefore unresolved** until its frozen binary
    is inspected. The probes (T2) are the means of resolving it, once the instrument is on
    master.
- **A4, diagnostic only:**
  - raw Ir per thread and per stage;
  - the whole process, and the inversion (build, walk and assembly);
  - **Ir/D** with D = 209,700,196 decoded bytes, and the walk's **Ir/M** with M =
    209,697,662 walked bytes;
  - unattributed Ir, and the tool and build identities.

  It carries no threshold, no badge and no acceptance row.
- **`@9` is deferred** until an explicit team identity exists. Each team owns a subset of
  W, so "233 per helper" does not hold, and A3 needs per-team assigned-block counts and a
  stable team identity, not equal totals. The probes' team key would supply them, on an
  instrumented build.

## 5. Limits, triggers, contract, price

**Hard limits** (the manifest's `limits_s`, in seconds):

| stage | limit |
|---|--:|
| the whole job | 1,200 |
| each build | 720 |
| each native child | 60 |
| each callgrind child | 300 |
| the checker | 120 |
| the artifact | 20 MB |

A timeout is missing coverage: never a pass, never retried silently.

**The trigger:** pull requests touching any of:
- `src/**`, `Cargo.toml`, `Cargo.lock`;
- `examples/ab.rs`, `tests/common/**` (the corpora's recipe);
- `bench/team_screen.py`, `bench/team_gap_count.py`, `bench/team-screen/**`;
- the screen's workflow file.

So a change to the covered binary, its inputs, or the screen's own machinery re-runs it.

**Running it:**
- one run per head/base pair;
- an obsolete PR attempt is cancelled, but its status and artifacts are kept.

**The contract** is versioned by the manifest's `version`. Any change to a fixture, an
oracle value, a role, a transition or a limit is a new version, reviewed as the first.

### The price (models; the Optimiser's equations)

**Revision 1's rows,** as the Optimiser corrected them: **220–300 s warm, 465–845 s cold.**

**Before omissions, revision 1 left out:**
- the 7 oracle decodes;
- Valgrind's apt delta (unmeasured on CI: run 42758830562 installed lbzip2 only);
- the cache save, compression and tails.

**Its "about 4 runner-minutes"** was one optimistic cache-hit case, not an expected cost.

**The two-tier price, rebuilt the same way** (in seconds; models, not CI observations):

| part | PR tier, warm | PR tier, cold | full tier adds |
|---|--:|--:|--:|
| checkout, toolchain, apt (lbzip2) | about 45 | about 45 | the Valgrind apt delta (unmeasured) |
| builds: head and base (27.08 and 25.14 s logged, cache hit), plus head with `walk-stats` | about 80 | 450–900 | none |
| fixtures: corpora, windows, 4 E edits | about 25 | about 25 | none |
| T1: 44 children, 7 oracle decodes, 4 lbzip2 rejections (0.1–0.6 s each) | 6–33 | 6–33 | none |
| T2: the instrumented H-team on 11 fixtures | 2–7 | 2–7 | none |
| checker, artifact, cache save | about 15 | about 15 | none |
| callgrind, H-team and B-team on W | none | none | 60–120 |
| parsing and disassembly | none | none | 20–40 |
| **total** | **about 175–205** | **about 545–1,025** | **80–160, plus apt and tails** |

**The cold cache:**
- The cold range adds a third fat-LTO build: T2 costs a build.
- With cold probability p, the expected PR-tier time is about 190 + 600p seconds before
  omissions. No value of p has been established.

**The merge delay is not zero.**
- **The equation:** with the existing required checks finishing at B, and the screen's
  queue Qs and duration Ts, the added merge-ready latency is **max(0, Qs + Ts − B)**.
- **The bench's B:** about 473 s (job log 129639441404).
- **What that gives:** a warm PR tier with no queue adds about 0; a cold one adds up to
  about 550 s; a queue or a rerun can make even a warm screen critical.
- **Runner-minutes always rise,** and a screen sharing a physical runner could disturb the
  bench's isolation. A separate job definition does not prove separate resources.

## 6. Open, for the Researcher and the Guardian

1. **Valid outputs:** add a SHA-256 of each output stream to the record (stronger than
   length plus a 64-bit checksum), or freeze the checksum's scope as stated?
2. **The error prefix:** compare it exactly (length and hash) across the matched arms, as
   proposed, or define its schedule semantics first?
3. **S→S:** if W strictly alternates, should the crafted fixture be a low-level C-style
   corpus, or a W-derived stream with one T block dropped?
4. **The Guardian:** does D083 require counted profiles on every affected PR, which would
   make the full tier a PR tier, or does the two-tier split stand?
