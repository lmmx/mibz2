# 2026-11-10: The forced-team screen (D083): design (DRAFT, revision 3, not frozen)

**Status: DRAFT. Not frozen, not adopted; no mibz2 arm has run on any screen fixture.**

**The history of this draft:**

| revision | at | what it answered |
|---|---|---|
| 1 | `ffb56891` | — |
| 2 | `25b778cd` | the Researcher's REVISE (#145 6759658271) and the Optimiser's price (#145 6759713909) |
| 3 | this head | the Researcher's recheck (#145 6760210997; report and counterexamples `state/reports/team-designs-recheck-6760095177` at `ddd3479c`) and the Guardian's D087 (#145 6760280679; `state/records/D087/decision.md` at `5059ce63`) |

**The standing terms (D087 and earlier):**
- **No per-PR callgrind mandate** (D087 (2)). T1 plus an implemented, validated and
  scope-adequate T2 can discharge executed-path correctness and engagement.
  - **What T2 must validate:** that the forbidden counters are complete; exact build,
    input and arm bindings; an independent reconciliation of calls against records,
    zero-work walkers included; and the actual transitions and roles.
- **Until T2 exists** (D087 (3)):
  - an affected candidate needs full-tier or equivalent evidence **before admission**, or
    it is held;
  - UNATTRIBUTED is not zero;
  - the base is never modified to obtain coverage.
- **S2 stays** (D087 (4)). A change to a default kernel needs matched whole-decoder Ir
  accounting (the full or diagnostic tier), with no "Ir must decrease" veto.
- **This is not the earned registration.** It is not the forced-team registration a pass
  of `2026-11-10-staged-traverse-count.md` would have earned: that entitlement is
  unearned, and FAIL-MEASUREMENT stands (D084).
- **Merge-blocking is a C6 admission change.** Making either tier merge-blocking is a
  prospective C6 admission change; a separate workflow file does not meet C6 by itself.
- **No acceptance cells:** none added to, and none removed from, any existing acceptance
  matrix.

**The recommendation** (the Researcher's answer 4): for affected PRs, **T1 plus an
implemented T2**; full profiles for the broader and diagnostic cases (§4). T1 alone is not
executed-kernel evidence.

**The artefacts, at this head:**

| file | what |
|---|---|
| `bench/team-screen/manifest.json` | `team-screen/0-draft.3`: roles, arms, read buffers, fixtures with their made and checked digests and oracle values, transitions, limits, record schema |
| `bench/team-screen/deltas/334.json` | #334's pinned delta |
| `bench/team-screen/make_fixtures.py` | the fixtures, Python only and deterministic, from W and its block map |
| `bench/team-screen/w-spans.jsonl`, `w-classes.txt` | W's block map: each block's span, CRC, rows and class |
| `bench/team_screen.py` | `outputs` (A1, the admission contract); `check` (the full tier's counted A2–A4) |
| `bench/test_team_screen.py` | 36 negative controls and positive cases, synthetic (§3) |
| `examples/ab.rs` | the child record, schema `ab-child/2` (§2): from #336 (bench 3.6.0), merged into this branch |
| `examples/walk_count.rs` | `spans`: the block map |

## 1. The gap (read: #330 §7, D083)

CI's runners report 4 CPUs on 2 cores, and `Plan::for_threads` gives `Workers` below 6
physical cores. The pipelined default needs exactly 10 physical cores and a 20 MiB L3.

So these never execute on any PR: `mtf_checkpoints_avx2`, `FusedBuild::share`, the
walkers, the relay, and the pipelined leader's solo path.

## 2. The harness's record, `ab-child/2` (#336, bench 3.6.0)

**`examples/ab.rs`'s child now writes:**

| field | what |
|---|---|
| `schema` | `"ab-child/2"` |
| `err_kind`, `err_block` | the kind's name and block, from the reader's `mibz2::Error`. **Both null on success** |
| `sha256` | with `AB_SHA256=1`, a streaming SHA-256 of the output beside the 64-bit hash. Null without it |

**`AB_READ_BUF=BYTES`** sets the read buffer; unset, it is the 4 MiB every run has read
with.

**The timed bench sets neither variable**, so its arms read and hash exactly as before.

**Checked here:**
- The SHA-256 equals `sha256sum` of `lbzip2 -dc` on all 12 oracle inputs.
- A truncated corpus gave `Truncated` at block 1 in the `mibz2` and `mibz2-pipe3-both`
  arms, with the same prefix (revision 2).

**An ab child returns normally on a decoder error.** It reports the error in its record
and exits 0, so an expected decoder error is never required to exit nonzero. A nonzero
exit, a timeout or a signal is a **process failure**, judged apart.

**The base harness must be compatible.** The base arm runs the base's own `examples/ab`,
which must emit `ab-child/2`. A base whose harness predates it is **INCOMPATIBLE**, a named
failure. So `ab-child/2` lands on master first, in its own measurement-neutral PR,
**#336**, before the screen is adopted; every later merge base then has it. The base is never patched
(D087 (3)).

## 3. The PR tier

### T1: uninstrumented exact results and errors (A1)

**Arms** (`ab child ARM THREADS FILE`, with `AB_SHA256=1` and `AB_READ_BUF` set):

| arm | build | threads | env | runs on |
|---|---|--:|---|---|
| H-team | head | 3 | delta's selector (#334: `AB_STAGED_TRAVERSE=1`) | every fixture |
| B-team | base | 3 | none | every fixture |
| H-plain | head | 3 | none | every fixture |
| B-plain | base | 3 | none | every fixture |
| H-plain1 | head | 1 | none | error fixtures (another completion order) |
| B-plain1 | base | 1 | none | error fixtures |

**Read buffers:** valid fixtures at 4 MiB; error fixtures at **4 KiB and 4 MiB**.

**The fixtures,** made and checked on 2026-11-10 (the manifest has every digest and
value):

| id | what | made | checked |
|---|---|---|---|
| W | `wikidata.bz2` | — | oracle 209,700,196 B, hash `a8830caee5c511a6`, SHA-256 `3f075e30…` |
| C-random, -records, -runs, -pattern, -onebyte | `ab gen corpora 4` | input digests frozen | oracle out, hash and SHA-256 frozen |
| C-text | `ab gen corpora 4` | **per head**: built from the crate's own sources | its input and oracle records carry its digest each run |
| V-SS | W's blocks 3, 4, 6, 7 (T, S, S, T), one stream, combined CRC | Python, byte-identical to `walk_count extract` | classes TSST by `walk_count spans`; lbzip2 decodes it, 1,800,010 B |
| E-trunc-team | W truncated at byte 63,572 (mid block 2, T) | Python | `Truncated` at block 2; prefix = blocks 0–1, 900,013 B |
| E-crc-team | W, bit 684,063 (block 3's stored-CRC last bit) xor 1 | Python | `BlockCrc` at block 3; prefix 1,800,013 B |
| E-trunc-solo | W truncated at byte 131,799 (mid block 4, S between T 3 and T 5) | Python | `Truncated` at block 4; prefix 2,699,951 B |
| E-crc-solo | W, bit 1,448,263 (block 6's stored-CRC last bit; S between T 5 and T 7) | Python | `BlockCrc` at block 6; prefix 3,599,983 B |

**About the fixtures:**
- **Edits** are located from W's block map (`walk_count spans`), never by searching.
- **Every E fixture is rejected by lbzip2** (exit 1).
- **Expected kinds and blocks** follow from the edits. **mibz2 has not been run on the E
  fixtures:** the first screen run judges them.
- **Why V-SS:** W's 8 S→S pairs all cross a stream boundary (`w-classes.txt`), so V-SS
  supplies S→S within one stream. It is purpose-built, as the Researcher recommended,
  not W with a block dropped, and not concatenated streams.

**Admitted completion records** (`expected_runs`): 96 runs, each a wrapper with its kind,
run id, fixture, command, input SHA-256, env, executable SHA-256, exit status, signal,
timeout flag and record. Arm runs also carry arm, build, build SHA and read buffer.

| runs | count |
|---|--:|
| 8 valid fixtures × 4 arms | 32 |
| 4 error fixtures × 6 arms × 2 buffers | 48 |
| oracle decodes: one per valid fixture, and one prefix per error fixture (`ab child lbzip2 1`, `AB_SHA256=1`, by the head's harness) | 12 |
| lbzip2 rejections (`lbzip2 -dc`, one per error fixture) | 4 |

**What `outputs` binds and rejects:**
- **The set:** every expected run, exactly once; missing, duplicate, unexpected or empty
  sets are rejected.
- **The identities:**
  - the command, exactly;
  - the input's digest equals the fixture's (C-text: its per-head record);
  - the env is exactly `AB_SHA256`, `AB_READ_BUF` and the arm's selector;
  - the executable's SHA-256 is the build step's record of that build (ARTIFACTS), the same
    for every run of the build;
  - the build label and build SHA match the delta;
  - lbzip2's digest is recorded.
- **The process:** a timeout, a signal or a nonzero ab exit is a process failure. A
  rejection must exit nonzero, not by timeout or signal.
- **The record:** the full `ab-child/2` schema; a well-formed hash and SHA-256.
- **The outcomes:**
  - **The oracles** reproduce their frozen out, hash and SHA-256.
  - **A valid fixture** has empty `err`, **null `err_kind` and `err_block`**, and its
    oracle's length, hash and SHA-256. The SHA-256 gives byte-equality evidence.
  - **An error fixture** has its frozen kind and block, and **exactly its prefix oracle's
    output** (length, hash and SHA-256).
  - **The matched arms,** every buffer and thread count included, agree on (out, hash,
    SHA-256, kind, block).

**The prefix contract, from the Researcher's answer 2:**
- `parallel.rs` drains the current block and takes ready results in block order;
- the CRC is checked before a block is published;
- so, without cancellation or an earlier error, scheduling changes when output is
  available, not the complete prefix of preceding blocks;
- the read-buffer and thread-count variants exercise that.

**The negative controls** (`bench/test_team_screen.py`, 36 tests, all passing). Each
counterexample is a test that must fail:

| counterexample | source | at `25b778cd` | now |
|---|---|---|---|
| wrong build label | recheck | admitted | rejected |
| a different executable digest for every run | recheck | admitted | rejected |
| success records with `err_kind` BlockCrc and `err_block` 99 | recheck | admitted | rejected |
| malformed-input children exiting 124 | recheck | admitted | rejected (process failure) |
| four empty objects; one empty record; missing or duplicate run; matching failures on a valid input; agreeing on a wrong output; error without its block; wrong block; one arm's kind differs; wrong prefix; killed | first review | rejected | rejected |
| timeout flag; missing oracle completion; an oracle that does not reproduce; missing rejection; lbzip2 accepting; wrong command, input, selector or buffer; a base harness older than `ab-child/2`; record without `sha256` | recheck's other points | — | rejected |
| an expected decoder error with a normal exit | recheck: do not require nonzero | — | **admitted** (positive case) |

### T2: per-thread semantic probes (`walk-stats`): the next deliverable, not yet implemented

T2 uses the same instrument as the native diagnostic
(`2026-11-10-staged-engagement-diagnostic-draft.md` §2). It runs on the head's instrumented
build, and on the base's only once the instrument is on master.

**What it must validate, per D087 (2):**
- **Forbidden counters complete:** an entry counter in every walker variant, compiled in
  both, so a zero proves absence for that build.
- **Bindings:** exact build, input and arm bindings, as in T1.
- **Reconciliation:** each block's completed call records against the summary's
  independently counted walkers and membership, zero-work walkers included (diagnostic
  draft §3).
- **Roles and transitions:** the actual roster (one leader, two helpers), and the actual
  transitions in the per-block records, ordered by place.

**What it claims:** neither native balance nor performance. T1's uninstrumented A1 is
kept beside it.

## 4. The full tier, or a separate diagnostic: matched counted profiles

**What it runs:** callgrind on H-team and B-team on W, uninstrumented builds.

**When it is required:**
- for broader and diagnostic cases;
- for S2's matched whole-decoder Ir accounting when a default kernel changes (D087 (4));
- **until T2 exists, before admitting an affected candidate** (D087 (3)).

**How it is read:** `team_screen.py check`.
- **Each profile is matched to its own binary and source:** `--binary` and `--coop-src`
  are required.
- **Attribution:** a kernel that cannot be attributed is UNATTRIBUTED, never zero. The
  merge base is never patched to obtain a symbol.
- **A4 is diagnostic only:** Ir/D with D = 209,700,196, and Ir/M with M = 209,697,662.
- **`@9` is deferred** until an explicit team identity exists.

## 5. Limits, triggers, contract, price

**Limits:** the manifest's `limits_s`. A timeout is missing coverage, never a pass, never
retried silently.

**The trigger:**
- `src/**`, `Cargo.toml`, `Cargo.lock`;
- `examples/ab.rs`, `examples/walk_count.rs`;
- `tests/common/**`;
- `bench/team_screen.py`, `bench/team_gap_count.py`, `bench/team-screen/**`;
- the screen's workflow.

**The contract** is versioned by the manifest's `version`.

### The price (models)

**The Optimiser's equations hold.** Revision 1's rows summed to 220–300 s warm and 465–845
s cold. The merge delay is **max(0, Qs + Ts − B)**, with B about 473 s.

**T1 now runs 96 processes,** against revision 1's 36 plus the omitted oracles:
- **The fixtures:** made in Python (about 1 s, measured here), with no build.
- **The SHA-256** adds a software hash of each output: about +0.5–1 s per W run.

| part | PR tier, warm (s) | PR tier, cold (s) | full tier adds (s) |
|---|--:|--:|--:|
| checkout, toolchain, apt (lbzip2) | about 45 | about 45 | the Valgrind apt delta (unmeasured) |
| builds: head and base `ab`, plus the head with `walk-stats` (T2) | about 80 | 450–900 | none |
| fixtures: corpora, windows, `make_fixtures.py`, digest checks | about 25 | about 25 | none |
| T1: 80 arm runs, 12 oracles, 4 rejections (W about 1–3 s each, the rest 0.05–0.5 s) | 20–60 | 20–60 | none |
| T2: the instrumented H-team on the same fixtures | 5–15 | 5–15 | none |
| checker, artifact, cache save | about 15 | about 15 | none |
| callgrind H-team and B-team on W, then parsing | none | none | 80–160 |
| **total** | **about 190–240** | **about 560–1,060** | **80–160, plus apt and tails** |

All figures are models, not CI observations. With cold probability p, the PR tier's
expected time is about 215 + 600p s; p is unknown.

**Without T2,** the PR tier drops the third build and T2's runs: about 175–220 s warm and
about 520–770 s cold.

## 6. Open

1. **The `ab-child/2` prerequisite** is #336 (bench 3.6.0, a minor version), under the
   bench's usual neutrality check.
2. **T2:** the instrument's design is the native diagnostic's §2. Its implementation is the
   next deliverable (D087).
