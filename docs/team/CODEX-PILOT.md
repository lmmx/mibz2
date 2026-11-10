# Codex production delegate — manual pilot specification

**Status:** Owner-authorised Codex implementation delegate. This document and its routing changes are a **proposal until the setup PR merges**. Once merged, Codex is a registered but **manually launched** role: `coord.py` can route and audit its inbox, but nothing automatically wakes Codex. It has no Coordinator authority.

**Mailbox:** [issue #363](https://github.com/lmmx/mibz2/issues/363), routed by `coord.py` after merge; human-launched execution only. **Relay:** [CODEX-RELAY-PILOT.md](CODEX-RELAY-PILOT.md). **Assignment:** the first production slice of MONODROMY's cells inverse-BWT kernel.

## Position in the team

Codex is an **implementation delegate** for one scoped production change, not a successor or stand-in to MONODROMY, the Pathfinder, or the Coordinator. The model that runs it does not determine a role's identity or authority.

- **Louis:** directs the pilot and authorises its scope. His workstation access, spending, infrastructure changes and role/rule changes remain his.
- **Existing Coordinator:** retains priority, merge, release and trade-off decisions according to the live rules. Louis may appoint ChatGPT Work to **operate that existing role** during an outage, using the explicit [failover procedure](COORDINATOR-FAILOVER.md); Codex never inherits those powers.
- **MONODROMY:** owns the frontier mathematical research programme, proofs, hypotheses, and original cells construction. Codex may report errors or counterexamples, not silently rewrite the research record or speak as MONODROMY.
- **Pathfinder:** owns architectural and product-performance leadership. Codex may point to implementation issues without assuming that identity.
- **Independent reviewers:** B5 soundness and memory-safety checks are a hard condition for new unsafe code. Codex cannot certify its own work as independently reviewed.
- **ChatGPT Work:** may serve as the acting operator of the **existing** Coordinator role only following Louis's explicit activation and an attributed #145 handoff under [COORDINATOR-FAILOVER.md](COORDINATOR-FAILOVER.md), not silently.

If the research design conflicts with verified code or correctness, stop that approach, record the smallest counterexample, propose a narrow correction, and flag it for review. Do not broaden the mission into a new invention programme.

## One assignment: cells kernel, slice one

Read the **current master** versions of OWNERSHIP.md, RULES.md, PROTOCOL.md, PERFORMANCE-POLICY.md, `CLAUDE.md`, and `docs/journal/2026-11-10-coordinator-handoff.md`. Then read these at their actual GitHub refs (PR #362 may or may not have merged):

- `docs/journal/monodromy/2026-11-10-cells-production/DESIGN.md`
- `docs/journal/monodromy/2026-11-10-cells-production/HANDOFF.md`
- `examples/rauzy.rs` and `docs/journal/monodromy/2026-11-10-rauzy-veech.md`
- [#145: final production gate](https://github.com/lmmx/mibz2/pull/145#issuecomment-6775049636)

Build a **separate `InverseBwt` type**, retaining `Segmented` as the default. At the existing `invert_runs` seam, attempt cells on admitted run-record blocks; refused blocks must use the existing fallback with identical bytes and errors. The admission is `r/n` → `r₂−1` lower bound → cell-count budget. Keep internal buffers bounded, type state behind a pointer, and isolate unsafe operations with written invariants and checks. Add a selectable third `examples/ab.rs` arm so CI can actually measure the otherwise-off-by-default new type.

**Not in scope:** changing the default type, team paths or scheduling threshold, the wikidata-pq cap, release tags/dependencies, workstation queue, MONODROMY's original branch, mathematical publications, or altering acceptance gates after measurement.

### Evidence that must travel with the PR

- Byte-identical output and the same error behaviour on corrupt input, compared with production and libbzip2.
- A written table of every unsafe operation, its bounds/invariants, and how each is established.
- Forced-route fuzz mode with asserted reach; Miri tests, including canaries between buffer regions; mutation checks for the admission thresholds; differential edge-case and two-run tests.
- Neutral CI measurements on all **default** arms and an explicitly labelled **experimental** third arm. Do not claim the 18% inversion-stage gain predicts an equal whole-decoder gain.
- Peak RSS and refusal cost. The first PR must not make any existing default corpus slower beyond noise. An independent B5 adversarial review is required before merge.
- Build and test commands actually run, the exact head SHA, failures, and unverified claims.

The [i9 cells-vs-production log](https://github.com/lmmx/mibz2/pull/145#issuecomment-6775038009) gives a *stage-level* baseline: cells/records-path inversion `0.815, 0.827, 0.818` across three repeats; the i9 team threshold result separately says keep six threads. Neither authorises a default change.

## Deliverable and stop rule

1. Start from the current master on a **fresh, isolated branch**, not PR #362's branch. Keep the change small; open a reviewable implementation PR.
2. Record findings and checkpoints on **that implementation PR**; link it to [the manual inbox #363](https://github.com/lmmx/mibz2/issues/363). A status message or green CI is not a merge decision.
3. Leave **slice one off by default**. Do not merge, release, spend/dispatch hardware jobs, edit **another** role's cursor/state branch, or post messages masquerading as the Coordinator/Guardian/MONODROMY. After a genuinely completed routed assignment, update only `team-state/codex:state/cursor.json` and report `FROM: codex`, `TYPE: REVIEW` on #145 as the relay guide specifies.
4. If a blocker is genuine, preserve a reproducible example, specific files/lines, and a safe partial PR rather than substituting speculative code. Mark open items as open.
5. The **second PR** to flip the default is a separate, later decision under the Coordinator's gate in #145 comment 6775049636, including an i9 one-build A/B and performance/RSS constraints.

Before doing anything, check the current repository state; this guide is a starting handoff, not a guarantee that no later decision superseded it.
