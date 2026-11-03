# Compact30: the v2 construction-cost harness's i9 dry run (reports 213–215)

**APPROVED: one untimed i9 dry run of the v2 harness. No performance capture.**
- **Report 213:** the guardian reviewed the v2 protocol, the analyser's G1/G2, the
  regression tests, the recipe, the machine-code identity and the VM evidence at
  `1dc2edf`, and approved one untimed dry run.
- **Report 214:** the researcher independently supported exactly that, and no more.
- **Report 215:** the guardian confirmed that no further review is needed.

**The approved command,** on the i9 in `/home/louis/dev/mibz2`, on `research/compact30-constructor`:

    just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run

The recipe builds the frozen harness at `fa21f33`, in its own pinned worktree
(`target/c30-construction-v2-pin`), with rustc 1.97.0. It checks the corpus hash first.

**Conditions:**
- The run stops on any failed precondition. **No automatic rerun and no code change.**
- The archive and analysis are kept whether the run succeeds or fails.
- The dry-run schedule, G3, G4 and G6–G10 must pass. Its timing records are **not
  performance evidence**.
- A successful dry run does **not** authorise the measured 20-round capture, which needs a
  separate decision.
- Also not authorised: constructor tuning, changes to the validity gates, walker changes,
  production integration, and any whole-decoder performance claim.

**Fixed as of this approval:**
- The revised G1/G2 rules are accepted as prospectively registered for v2. That includes
  the one-outlier-per-arm allowance. They **must stay unchanged once workstation measurement
  begins**.
- The economic thresholds stay as they were.
- v1's INVALID capture and STOP, and the traversal's VALID ADVANCE, are unchanged.

**The targeted Miri evidence** (69/69 cases exact) stays on its own branch,
`research/compact30-ctor2-miri` at `e57a225`, in
`docs/journal/2026-11-03-compact30-constructor-v2-miri.md` there. Neither the guardian nor
the researcher requires merging it into the frozen harness, and no further Miri work is
needed.

## The result: the i9 dry run passed

**Every check a dry run makes passed.** The i9 dry run ran on 2026-11-03 at 22:20 UTC and
exited with status 0:
- the dry-run schedule: S0–S10, plus S1_protocol;
- G3 machine-code identity, G4 allocation, G6 exact output, G7 round −1, G8 one-pass,
  G9 memory and G10 major faults.

Its timing records are not performance evidence. G1, G2, G5, every timing figure and the
decision were suppressed, as registered. No capture was run.

**The archive:** `c30-construction-v2-20261103T222051Z-dry-run.tar.gz`, 644,678 bytes, sha256
`65689a5800cd93b272b04d6c5a89822e561a5a10a8ab95aab20308cee4b02b27`. It is kept in
[2026-11-03-compact30-construction-v2-i9-dry-run/](2026-11-03-compact30-construction-v2-i9-dry-run/)
as received, and its contents are listed below.

**Checked here:**
- The checksum matches the console.
- All 33 manifest entries match their files.
- Replaying the pinned analyser (`bench/c30_construction.py` at `fa21f33`, unchanged at
  `1dc2edf`) on the archived records gives the same results as the archive. It was run with
  `--protocol v2 --identity-ok yes --rss rss-*.json`, under `python3 -I`, from a separate
  scripts directory. The JSON analysis is identical, and the console is identical to the
  archive's analysis section (`replay.console`).

| item | value |
|---|---|
| source | `fa21f33`, no tracked changes; the recipe at `1dc2edf` |
| toolchain | rustc 1.97.0 (`2d8144b78`), LLVM 22.1.6, cargo 1.97.0 |
| tests in the pinned worktree | 94 (draft 4); 89 under v2 with 5 replaced, plus 30 v2 tests; 6 (listings) |
| machine | louis-3xs, i9-10900K, 20 CPUs online, 125.7 GiB |
| placement | workers on CPUs 1–4, W1 on the first |
| barrier test | 12,000 steps, 3 rounds × 2 arms, 4 workers: every check passed |
| records | 10,416 builds; 868 `check0`; 434 each of `onepass`, `counters` and `poisoncheck`; 2 calibration; preflight, meta and postflight |
| protocol, candidate | `c30-construction-v2`, `ctor2` |
| corpus | `wikidata-20261026.bz2`, sha256 `cf13f439…`: 434 blocks, 207,006,515 rows, 230 full blocks |
| allocation (W4) | rows 51,301,938 / 51,301,991 / 52,201,043 / 52,201,543; blocks 109/109/108/108; full blocks 57/57/58/58 |
| machine code (G3) | every listing diff is empty: P's `p_build` and `bwt::build`; C's `c2_build`, `Ctor2::build::<false>` (1,431 instructions), `W<false>::finish` (379) and `W<false>::refill`; the shared canonicaliser |
| poison coverage | poison pass 434/434 blocks; round −1 432/434 (the same as the VM dry run) |
| memory | estimate 1,855,391,770 B; peak VmHWM 1,770,512,384 B (0.954 of the estimate); about 130.8 GB available; no swap used; PSI 0 |
| faults | major 0 in measured builds and in round −1; minor by regime and arm: warm P 0, warm C 10, cold P 1,654, cold C 145 |
| RSS processes | P median 13.9 MB, C median 7.9 MB (three each) |

The harness binary's sha256 (`a55755db…`) differs from the VM build's (`ea2662dc…`). That is
expected, because the binary embeds machine-specific build paths. Binary identity for this
protocol rests on G3's comparison of the timed functions' machine code with the frozen
listings, and every one of those comparisons is identical.

**What follows.** The archive goes to the guardian for review. The measured 20-round capture
needs a separate decision. If authorised, it would be:

    just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 capture target/c30-construction-v2-20261103T222051Z-dry-run.tar.gz

That command is not authorised and has not been run.

## Report 213: the guardian's approval, as relayed

Kept as relayed. Link tracking parameters are removed; the words are unchanged.

Guardian decision: APPROVED — one untimed i9 dry run of the v2 construction-cost harness. No performance capture yet.

I've reviewed the pinned v2 protocol, the analyzer's G1/G2 implementation, the regression tests, the workstation recipe and the archived VM dry-run evidence. I found no blocking issue for this next step.

The targeted Miri work is also complete. All 69 additional cases passed, including the exact wide-offset boundaries, forced RAW, malformed inputs and allocation reuse. There is no reason to rerun Miri on your i9.

### Review outcome

| Area                       | Disposition               |
| -------------------------- | ------------------------- |
| V2 constructor correctness | Independent audit PASS    |
| Targeted Miri              | 69/69 passed              |
| V2 economic thresholds     | Unchanged                 |
| G1/G2 revisions            | Preregistered and tested  |
| Production machine code    | Matches reviewed listing  |
| V2 machine code            | Matches audited listing   |
| VM dry run                 | Passed                    |
| V1 historical result       | Unchanged: INVALID / STOP |
| i9 untimed dry run         | Authorized                |

The protocol's main remaining limitation is that the full 20-round statistical decision has only been exercised against synthetic records. That's expected: the dry run deliberately suppresses performance conclusions.

I accept the revised G2 rules as prospectively registered for v2, including the one-outlier-per-arm allowance. They must remain unchanged once workstation measurements begin.

### What to run

On your i9, in `/home/louis/dev/mibz2`, with the usual corpus available:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run
```

This command uses the frozen harness at `fa21f33` and its own pinned worktree. It performs the correctness, scheduling, identity, memory and evidence checks without producing an experimental performance verdict.

The run must stop on any failed precondition. Preserve the resulting archive and analysis, whether successful or unsuccessful. No automatic retry.

### Message for Claude

Guardian review of compact30 constructor v2, protocol and harness at `1dc2edf`:

**APPROVED: one untimed i9 dry run only.**

I reviewed the protocol, the analyzer's G1/G2 implementation, its regression tests, the pinned workstation recipe, machine-code identity and VM validation evidence. No blocking issue was identified for an i9 dry run.

The 69 additional targeted Miri cases passed on the VM. Archive this evidence on its separate research branch; no further i9 Miri work or merge into the frozen harness is required.

The approved command is:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run
```

Use the pinned recipe and harness at `fa21f33`, preserving constructor v2 at `8beffe2` and all existing validity and economic thresholds.

Archive the console, analysis, manifest, checksums, machine identity, listing checks, memory evidence and any failure record. Verify that the dry-run schedule, G3, G4 and G6–G10 pass. Do not interpret its timing records as performance evidence.

If any condition fails, stop without an automatic rerun or code change.

**Not authorized:** the 20-round measured capture, constructor tuning, changing validity gates, modifying the walker, production integration or any whole-decoder performance claim.

Return the i9 dry-run archive, checksum and analysis for Guardian review. The measured capture requires a separate decision.

Constructor v1's INVALID/STOP and compact30 traversal's VALID ADVANCE remain unchanged.

We're now one successful workstation preflight away from deciding whether to measure v2's construction cost. The central economic question—whether the much shorter append path outweighs deferred normalization—is still unanswered, but the correctness work is in good shape.

## Report 214: the researcher's assessment, as relayed

Kept as relayed. The paragraph breaks and headings the relay lost are restored, and link
tracking parameters are removed. The words are unchanged.

This is a meaningful step forward for v2. I checked the new GitHub records, including the targeted Miri logs, the v2 protocol and the archived VM analysis.

My assessment: the evidence supports sending the v2 harness to the guardian for approval of an i9 dry run. It does not yet justify authorizing a performance capture.

### What has improved

The targeted Miri tests address the most important coverage gaps from my earlier audit. All 69 additional cases completed without reported undefined behaviour, including:

* The exact 65,533/65,534 ordinary-to-RAW boundary.
* Forced RAW construction and malformed-count rejection.
* Pending portions crossing epoch boundaries.
* Reusing allocations after shrinking and growing blocks.
* A wide group spanning multiple epochs.

Together with the earlier 473 completed cases, this materially strengthens the memory-safety evidence. It remains targeted testing, not exhaustive verification.

The archived [v2 VM dry-run analysis](https://github.com/lmmx/mibz2/blob/1dc2edf/docs/journal/2026-11-03-compact30-construction-v2-harness/vm-dry-run/c30-analysis.json) also reports every applicable schedule and non-timing gate passing. The full 434-block corpus was checked, with complete coverage in the separate poison-validation pass.

### Two things matter for the guardian

First, G1 and G2 have changed deliberately. V2 gates production's warm four-worker stability, and G2 permits one isolated synchronization outlier per arm, bounded by a 10% maximum and a 2% aggregate threshold.

These are prospective rules for v2 only. I verified that the protocol explicitly preserves v1's INVALID outcome and STOP decision. That's essential experimental discipline.

Second, the performance analysis hasn't been exercised on real v2 capture data. Its decision logic has synthetic regression coverage, but the dry run suppresses the actual timing figures. This is an acknowledged limitation, not evidence that the analysis is defective.

### Recommended next decision

I'd support the guardian reviewing the frozen harness at `fa21f33` and, if satisfied, authorizing one untimed i9 dry run, using the proposed command:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run
```

That authorization should be limited to the dry run and archiving its evidence. No automatic capture afterward.

The Miri branch can remain separate; it does not need merging merely to preserve its evidence.

Most importantly, the shorter v2 scalar loop is now a credible, correctly implemented architectural improvement—but not yet a demonstrated speed improvement. That is exactly the distinction the next construction-cost experiment needs to resolve.

## Report 215: the guardian's confirmation, as relayed

The researcher and Guardian are aligned. No further review is needed before the i9 dry run.

The researcher's recommendation was to authorize one untimed workstation preflight. I already granted exactly that authorization in my previous response, after reviewing the frozen harness and protocol.

I also rechecked the VM analysis on GitHub at `1dc2edf`. Every applicable schedule and non-timing gate passed. The performance decision was correctly suppressed.

### Current authorization

| Activity                          | Status         |
| --------------------------------- | -------------- |
| Constructor v2 independent audit  | PASS           |
| Additional targeted Miri tests    | 69/69 passed   |
| VM harness validation             | PASS           |
| One untimed i9 dry run            | APPROVED       |
| Measured 20-round i9 capture      | Not authorized |
| Constructor tuning or integration | Not authorized |

### Next action

The approved command remains:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 dry-run
```

Claude should archive its results and return the archive checksum and analysis for Guardian review. A successful dry run does not automatically authorize the measured capture.

The researcher's latest message adds useful independent confirmation, but it doesn't change the plan. There's no need for another Astra consultation, further Miri testing or a new round of protocol revisions.

We can proceed with the already-approved dry run.

## Report 216: the guardian reviews the dry run and authorises one measured capture, as relayed

Kept as relayed. What report 216 decides:
- The dry run's required checks passed. The guardian verified this independently, from the
  archive.
- **One measured 20-round v2 capture is authorised**, using the frozen harness and the
  verified dry-run archive:

      just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 capture target/c30-construction-v2-20261103T222051Z-dry-run.tar.gz

- The machine stays otherwise idle: no Miri, no other benchmark, no CPU-intensive work.
- Whatever formal result the protocol produces (PASS, FAIL, INCONCLUSIVE or INVALID) is
  kept as it is. The thresholds don't change, and there is no automatic rerun.
- A PASS would lead only to a separately approved production-shaped decoder evaluation, not
  to integration.

Guardian review: the i9 dry run has passed its required checks. I independently inspected your uploaded archive, not just the terminal output.

**Archive integrity verified — 33/33 files**

The archive SHA-256 matches the reported value:

`65689a5800cd93b272b04d6c5a89822e561a5a10a8ab95aab20308cee4b02b27`

The evidence confirms:

| Check                             | Result                         |
| --------------------------------- | ------------------------------ |
| Frozen harness `fa21f33`          | Matched                        |
| Rust 1.97.0 / LLVM 22.1.6         | Matched                        |
| CPUs 1, 2, 3, 4                   | Confirmed                      |
| Scheduled construction operations | 10,416                         |
| Exact output checks               | 10,416/10,416 passed           |
| Separate poison pass              | 434/434 blocks covered         |
| Schedule checks                   | 15/15 passed                   |
| Applicable validity gates         | 7/7 passed                     |
| Machine-code identity             | All 7 listed functions matched |
| Major page faults                 | Zero                           |

The memory checks also passed, with approximately 1.77 GB peak resident memory against a 1.86 GB estimate.

These are preflight results, not evidence that v2 is faster. The dry run correctly suppressed all timing-based conclusions.

### Guardian decision: authorize one measured v2 capture

The workstation preflight has satisfied the registered prerequisites. I authorize one 20-round i9 construction-cost capture, using the frozen harness and the verified dry-run archive.

From `~/dev/mibz2`, run:

```
just bench-c30-construction-v2 target/walk-decoder/work/corpora 20 capture target/c30-construction-v2-20261103T222051Z-dry-run.tar.gz
```

No additional Git fetch is necessary if your working tree remains as it was for the successful dry run. The recipe pins its own harness.

Keep the machine otherwise idle. Don't run Miri, another benchmark or CPU-intensive work concurrently. Let the registered checks determine the outcome.

This capture has four possible formal results: PASS, FAIL, INCONCLUSIVE or INVALID. We will preserve whichever result the protocol produces, without modifying the thresholds or automatically rerunning it.

When it finishes, send me the terminal output and the generated capture archive. I'll verify the evidence and assess whether v2 meets both construction-cost limits:

- Four-worker CPU cost: b4 upper 95% CI ≤ 1.06714
- Four-worker completion time: e4 upper 95% CI ≤ 1.02

If those pass, the next step would be a separately approved production-shaped decoder evaluation—not immediate integration.

The v1 STOP remains unchanged. The compact30 traversal VALID ADVANCE remains unchanged. This experiment answers whether constructor v2 makes that traversal improvement economically viable.
