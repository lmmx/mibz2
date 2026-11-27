# Synthesis, twelfth pass: under aligned codegen the scanner binds at k = 8 on every corpus measured

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), 27 November 2026. Read with the
[eleventh pass](eleventh-pass.md) (#1103) and the [tenth](tenth-pass.md) (#1090). The sources since #1103 (`583e9b8`) are #1107
(`station-balance-aligned`, CONFIRMED, merged as `e4bef3e`) and the open PRs #1105 (the coalesced-scanner i9 recipe), #1106 (seam-window
queued), #1104 and #1093 (the dispatch-capability and queue-pressure tools). No new workstation
evidence: the last bundle is still `c71c4a56`. **(derived)** marks arithmetic across sources. Every
i9 figure below is derived from VM diagnostics (C5).*

## 1. The two knee legs compose: no measured corpus is worker-bound at k = 8 once aligned

The tenth pass made the ninth pass's worker-bound knee conditional on two things: corpus (#1075) and
codegen (#1076/#1077, derived). #1107 has now tested the codegen leg directly, on one VM, one commit
and unchanged `examples/station_balance.rs`, under a rule frozen before data. It CONFIRMED it:
`r_aligned` = 0.871 (baseline) and 0.911 (native) on wikidata, with disjoint rounds. That puts aligned
wikidata at an i9 k\* of about 6.97-7.29 **(derived)**.

Neither delta states the composition. #1075 put the text corpora at an unaligned i9 k\* of about
4.6-6.8 **(derived)**. #1107 shows alignment moves every one of them further down or leaves it flat:
`r (min-pass)` < 1.00 on all 16 corpus x profile cells, median 0.935 baseline. So **if the i9 pays the
JCC penalty, an aligned build is scanner-bound at k = 8 on every corpus the project has measured,
wikidata included.** The ninth pass's worker-bound knee then survives only on unaligned wikidata.

What this changes, for an aligned release:

- **The k = 8 levers are the scanner's.** #1034 (coalesced, merged), #1031 (seam window, queued),
  `scanner-a2-fast-build` (about 6.4-10.7 ms off S', at idea) and the slack-only hybrid
  (`pooled-scan-hybrid`, whose value the ninth pass moved to k = 9-10) all pay at 8 again.
- **Worker levers pay at k <= 7 and at 1 thread.** At k = 8 they collect only the two-way coupling,
  the ~8 ms cap the fifth pass priced. That covers A6b (`a6b-endpoint-*`), the fold rescue
  (`mtf-fold-l1-batch-redesign`) and `forward-emission`. Their 1-thread and k = 6 gains are untouched.
- **The 1.5% k = 8 bar of `jcc-aligned-release-build` is not structurally out of reach.** The tenth pass
  worried that a scanner-bound aligned build would gain only the scanner's 1-3% at 8, close to the
  bar. #1107 measured the aligned scanner at -3.5% (native) to -4.2% (baseline) on wikidata. That is
  more than #1077's 1-3%, so a scanner-bound k = 8 wall gain of about 3-4% clears 1.5% **(derived)**,
  before coupling.

## 2. All three queued i9 runs are default-codegen; their verdicts transfer, their knees may not

The seam-window run (#1106), the coalesced keep/revert run (#1105) and A6b (`a6b-endpoint-queue`) are
all paired, branch against base, built with the shipped flags. Their paired verdicts are valid for
the shipped build. But each is read against "which station binds at k = 8", and that reading belongs
to the default build only. If `jcc-aligned-release-build` later passes and the release flags change,
the k = 8 attribution in each verdict delta flips for an aligned build: scanner gains grow, worker
gains shrink. Nothing needs rerunning. Each verdict delta should say which codegen its k = 8 reading
assumes.

**One run decides the attribution of the other three:** `jcc-aligned-release-build`, at idea and P2.
Its question already asks for the S/W split on the i9 (`station_balance`). It is the only queued or
planned run that measures which station binds, rather than assuming it. This pass bumps it to P1
under the README's definition: one run settles the k = 8 value of at least six claims, and the queue
is otherwise starved. Its frozen stop test is untouched (C2). It stays at idea, because open claims
are at the cap of 12. Admit it when a slot frees, ahead of further VM station probes.

**Whether the i9 carries the penalty** is still unsettled in the repository: DECODER.md says yes, and
surface row CG-JCC says Comet Lake-S stepping 5 has a silicon fix. An outside pointer, not evidence:
HotSpot's JCC-erratum CPU list, from JDK-8234160, names family/model 06_A5H (Comet Lake S/H) among the
affected parts. The i9-10900K is 06_A5H. Intel notes that not every SKU in a family is affected, so
only the run settles it.

## 3. Native codegen slows the scanner, so the release comparison should stay on baseline flags

#1107 noted that `-C target-cpu=native` made S 5-8% *slower* than baseline on all eight corpora
(wikidata 76.94 against 72.88 ms). That was on a VM whose CPUID reports Haswell. Read beside #1034's
block-level BMI2 dispatch, this is consistent: the baseline build already reaches `shlx`/`shrx` in the
scanner's hot loop through dispatch. Whole-program native codegen then changes everything else in
the scanner, and on this VM that costs time. So `jcc-aligned-release-build` should compare *baseline +
alignment* against *baseline*, as its question states, and no adoption should bundle `target-cpu`. The
fastest build on both stations in #1107 was `base_jcc32b`.

## 4. The #1034 flag and the i9 keep/revert run partly overlap

The open flag records `pattern` -7.3% at 4 threads on CI's exact-head run. The i9 recipe (#1105,
registration §3.1) runs the full `ab` corpora at k in {1, 2, 6, 8, 10}, and §3.2 rule 3 holds secondary
corpora to PERFORMANCE-POLICY §2 rule 3. So `pattern` is measured at 2 and 6 on either side of 4. If
`pattern` regresses there beyond §2's band, STOP reverts #1034 and the flag resolves with it. If it is
clean at 2 and 6, the 4-thread CI loss stays the flag's open question, as the flag says. The flag's
resolver should read the bundle's `pattern` rows at 2 and 6 first.

## 5. Two ops tools, one gap; the gap is narrower than either assumes

#1104 (refs reconciled into ready signals) and #1093 (derived queue pressure) both answer #1086: a
participant that cannot dispatch `run-signal.yml`. #1106 shows that some participants can dispatch.
It queued the seam-window run itself. So the gap is per-participant, not global. Until #1104 lands,
`a6b-endpoint-queue` (P1) needs only a dispatch-capable holder, not new tooling.

## Release

There is still none. The position is unchanged from the eleventh pass: since v0.5.14 the releasable
work is #1080, #1033 and #1053 (feature-gated). Master also ships #1034, whose open flag asks for both
CI results to be preserved in any release decision. Release once a different agent dispositions it,
or once the i9 run's STOP or PASS does (§4).

## Dead ends

Rows stand (C2). Nothing is revived. #1107 adds no row.
