# The Pathfinder's state at the pause

10 November 2026, about 17:50 UTC. Written because the session's quota is nearly spent and
it may be days before it runs again. Everything this role holds is on #313's branch
(`claude/agent-swarm-fable-breakthrough-0u7j4d`) or on master. Nothing needed lives only in
the session's scratch space, which is lost when the container is.

## 1. What shipped today from this role

| change | where | state |
|---|---|---|
| admission by the bound and the share of run symbols | #353 | merged, shipped in 0.2.9 |
| `mtf_checkpoints_avx2`'s list in one page (align 512) | #358 | merged `910472e8` |
| the run records on the three team paths | #359 | merged `a5197f11`, shipped in 0.2.9 |
| the one-core walk's joins kept inline | #361 | **open**; CI's 1-thread `wikidata` cell decides |

- **#359 on the i9** (`release-scaling/run-20261110T160612Z`): `wikidata` 12 to 14% faster
  at 6 to 20 threads, `text` faster at 10, `records` and `random` level.
- **0.2.9 against 0.2.8 on the i9:** `wikidata` 10 to 15% faster at 8 to 20 threads, 0.9%
  slower at 1 thread (0.991, 0.983 to 0.998). #361 is aimed at that cell.
- **Closed:** the universal backward walk (#348), no primary gain.

## 2. Open, in the order I would take it up

1. **#361.** If its bench has `wikidata` at 1 thread level or faster and nothing slower
   beyond noise, it merges as maintenance. If the cell is still slower, the code of the
   one-core path is already 0.2.8's except `runs::fill` (a cold block moved) and `fill_avx2`
   (two instructions outside its loop); what is left is where the code sits in the binary.
   Then: run `codegen/allcmp.py.txt` in CI with CI's toolchain before anything else, and
   do not start moving code about to chase a number.
2. **Blog review, round 2,** of the Rauzy–Veech post (`coordinate-blog-post` §2). Round 1 is
   REVIEW 6771693515 on #145; its factual findings were fixed on master at `259a507`, and
   the post has been revised several times since (figures, steppers). Round 2 repeats the
   whole of round 1 on the current text and adds the held-out goal: half the steps at three
   times the cost a step is 1.5 times the step time, and that is not a whole-decoder ratio.
   My baseline and round 1's reader report: `2026-11-10-blog-review-rauzy-veech/`.
3. **MONODROMY's kernel behind `InverseBwt::invert_runs`,** as an experimental one-core
   kernel for a whole-decoder A/B arm. Waits on #351 merging and the Coordinator's word.
   What MONODROMY told me it needs, and what I said I would do:
   - an `unsafe fn` boundary with its preconditions; a bound on its buffer that is not n
     cells (it reserves 128 n + 4,912 bytes of address space);
   - admission in two parts: runs a byte first (free), then a budget on cells in rows
     (break-even about 0.16 n on Wikidata; a refused block has already cost about nine
     tenths of production's time, so something cheaper must do the choosing);
   - a `#[doc(hidden)] pub fn records(&self) -> &[u16]` on `runs::Runs`, with that patch;
   - the complement-before-write check as the pattern for its tests.
   Its figure: 0.82 to 0.86 of production's inversion on Wikidata, one thread, indicative.
4. **The 1-thread layout question,** if it stays open after #361. What is known:
   - the one-core path does the same work before and after #359 (callgrind, to the
     instruction per function);
   - #324's and #359's flagged cells share corpora, sign and ratio (`records` about half of
     `random`);
   - moving `counts` through 4 KiB against `STEPS` shows no band above about 1% of the
     move-to-front (`tools/mtfsweep2.rs.txt`; this machine's noise is ±5%, so that is weak).
   Not tried: performance counters on CI (`pmu.yml` shows some runners expose them).
5. **The teams' move-to-front on high-entropy blocks** is an older, one-level kernel that
   calls `memmove` for every symbol past the 32nd place: 9 to 10 ns a row on `records`,
   about the same as the one-core kernel there (`tools/mtfcmp.rs.txt`). Not a known gain;
   noted because #358's cliff lived in it.

## 3. The frontier track

- **The interval-exchange tower** (`2026-11-10-interval-exchange.md`, `examples/iet_*.rs`)
  is parked behind MONODROMY's Rauzy–Veech kernel (#351), which is the better construction
  of the same family. §2a records the prior art (Kärkkäinen and Puglisi 2011 copy repeats
  found from adjacent rows' paths); §5b the rebasing against 0.2.8's inversion.
- **Reference figures for both production inversions** on Wikidata's 233 blocks, this
  container: `2026-11-10-team-runs-port/invarms.txt` (column 2.37 to 2.40 ns a row; records
  2.26 to 2.27).

## 4. Where things are

| what | path on #313 |
|---|---|
| the port: note, safety argument, evidence, the Guardian's two rounds, the codegen finding | `2026-11-10-team-runs-port.md` |
| the patches as merged or opened, with digests | `2026-11-10-team-runs-port/*.diff.txt` (§9 of the note says which is which) |
| fuzz logs at the pin | `2026-11-10-team-runs-port/pin-910472e/` |
| mutation table and its script | `.../mutants.tsv`, `.../mutate.py.txt` |
| machine-code comparison of two builds of the harness | `.../codegen/` (`allcmp`, `symcmp`, `fndiff`) |
| stack-depth sweep, counts sweeps, kernel comparison, block-route probe, gdb sampler | `.../stacksweep.rs.txt`, `.../tools/` |
| in-process timing of the team decoders | `.../teamtime.rs.txt` |
| the earlier patches (backward walk, admission) | `2026-11-10-backward-walk/` |
| blog review round 1 | `2026-11-10-blog-review-rauzy-veech/` |

The `.rs.txt` tools are examples: copy one into `examples/` under its name without `.txt`
and build it with `cargo build --release --example NAME`.

## 5. How this role works here, for whoever resumes it

- **It may push to #313's branch alone.** Every change to `src/` has gone to the Coordinator
  as a patch file with its SHA-256, which the Coordinator opens as a PR. Louis has been
  asked whether that may be lifted and has not answered; a message from another
  session cannot lift it.
- **Commits are Louis Maddox's,** with no AI attribution (RULES E4).
- **After a post on #145,** send the Coordinator's session a direct line; a comment alone
  does not wake it. Check each post with `coord.py check --issue 145` first.
- **pre-commit.ci on #313** errors with "error during mergeable check" when master moves
  as a push lands. Merging master into the branch clears it; it has twice.
- **#313's CI bench is an A/A** (the branch changes no source). Its last four runs each
  flagged one cell by 2 to 4%, in either direction: that is the bench's noise floor for a
  single cell.

## 6. Three things I got wrong today, so they are not repeated

1. I explained the `records` slowdown on teams by 4 KiB aliasing of `counts` against the
   list, and a patch built on that restored the speed. A sweep of the counts' address showed
   no band: the patch had only redrawn the layout. The cause was the list straddling a
   page. **Test the mechanism by moving the one thing it names, before trusting a fix.**
2. I wrote "fuzz: exit 0" for the port when no fuzz mode forced the plain teams. The
   Guardian caught it. **A fuzz result covers the routes its cases take, and the note has to
   say which.**
3. I wrote that `fill_share` never stores a value past its byte's bound; a test of that
   claim failed, because the AVX2 kernel's 8-slot stores do so in slots it then rewrites.
   **A contract sentence is a claim to test like any other.**
