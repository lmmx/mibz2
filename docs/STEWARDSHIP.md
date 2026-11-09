# Stewardship of mibz2's production code and releases

This document records who looks after mibz2's production code and releases, what that role
may do without asking, and how a release is carried out and checked. It holds across research
campaigns: a campaign may fail, succeed and finish, or move on to other research, and this
role continues either way. Each campaign keeps its own rules in its own entry document.

## The decision

On 2026-11-04 the maintainer was asked whether the Coordinator's merge and release authority
should be permanent or end with the compact30 campaign. They chose:

> **Permanent steward.** I'm mibz2's standing production and release steward, and compact30 is
> the current campaign under that. A failed experiment stays out of production. A qualified
> improvement doesn't wait indefinitely after passing its gates. Other legitimate maintenance
> isn't held back because of compact30.

The steward is the Coordinator, which is the maintainer's Claude Code session. The role
extended the delegation in the compact30 research charter (§2, report 266) beyond that campaign.
compact30 ended on 2026-11-06. The agent team's standing rules are now `docs/team/RULES.md`
(row A6 is this role).

## The standing objective

The objective is a measurably faster, correct production decoder, properly released. The
steward:
- keeps a failed or unqualified experiment out of production;
- doesn't let a qualified, merged improvement sit unreleased indefinitely;
- doesn't hold other legitimate maintenance back because of a campaign.

## What the steward may do without asking

The steward may open, review and merge pull requests into `master`, and may release, under
these gates.

**Changes that touch measured code** (`src/**`, `Cargo.toml`, `Cargo.lock`, the bench's inputs)
merge only when all of these hold:
- the tests and parity checks pass;
- CI is green on the head that will merge;
- the CI paired A/B bench (`.github/workflows/bench.yml`) shows an end-to-end improvement
  beyond noise, meaning its 95% interval excludes zero in the beneficial direction;
- no other figure regresses unacceptably.

**Target-gated changes** (D071; effective from the commit that merges this paragraph, for
registrations frozen after it, never for data collected before it). A change to measured code
whose new behaviour is selected only by a *target predicate*, a pure function of discovered
machine facts and the requested thread count, may merge by this route instead of the
beneficial CI interval above. This route is not evidence-equivalent to a CI gain: it accepts
a bounded slowdown off target (at most 2%) in exchange for a gain measured on target. Every
condition must be met; missing or invalid evidence fails.

1. **The predicate is frozen before data,** with its hardware rationale: which discovered facts
   it reads, why each is needed, and how an unknown or unreadable fact is handled (it must
   select the existing plan). A CI test evaluates it on every CI runner and finds it false,
   and asserts that the public dispatch there builds master's plan. Equal plans establish
   fallback selection only, not zero discovery, layout or shared-code cost; that cost is what
   condition 2 bounds.
2. **Off-target non-inferiority, on CI.** On the CI paired A/B bench, at every cell of the
   end-to-end table the registration names (each corpus × thread count × the metrics wall
   and CPU), the one-sided 95% upper bound of head over base is at most 1.02. Every cell must
   pass: an intersection-union test at level 5% for the claim "no named cell is worse by more
   than 2%". It is not simultaneous interval coverage, and it does not protect against
   retries; there are none (condition 6).
3. **On-target superiority, registered before data.** A frozen workstation A/B (RULES C5) on
   a machine where the predicate is true measures the exact candidate against master through
   the public entry point. At every registered budget where the predicate is true, wall speed
   (base over head) has a two-sided 95% interval whose lower bound exceeds 1.02, and CPU's
   one-sided 95% upper bound of head over base is at most 1.02. A Bonferroni correction
   applies when there is more than one such budget: the family and its level are stated in
   the registration.
4. **On-target no harm at the fallback budgets.** In the same run, at every registered
   neighbouring budget where the predicate is false, wall head over base has a one-sided upper
   bound at most 1.02, Bonferroni over the registered family of those budgets at a family
   level of 5%. Every one must be shown. NOT SHOWN fails qualification; it is not
   demonstrated harm.
5. **Coverage.** The adoption's reach is no wider than its evidence. Either the predicate is
   narrowed to the facts every qualifying machine shares, or the registration names the
   machines required (at least two distinct models) and the extrapolation to the rest of the
   class is argued in it and reviewed. Release notes state the scope but never substitute for
   this condition.
6. **The protocol is fixed in the registration:** rounds, the stopping rule (one attempt per
   registration), what makes a run invalid, and what follows an invalid run. No retry to
   pass. A later registration for the same change discloses every earlier attempt and its
   outcome and justifies its information value; all outcomes are retained.
7. **Tests, parity and green CI** hold on the head that merges, and the remaining conditions
   above (no other figure regressing unacceptably) apply as for any measured change.

**Before opening a PR that changes the default decoding kernel,** account for the whole decoder,
not only the part being optimised (RULES.md C4, S2).
- **Count the instructions** executed in `decode_block` on master and on the candidate, split
  into build, walk and the rest. Use matched toolchain, flags and input, and archive the
  profiles. Charge every induced cost: work, memory traffic, synchronisation and control.
- **An excess over master's count needs a stated mechanism,** written down first: what it
  buys (overlap, locality, less contention), its predicted effect at each thread count CI
  measures, and what falsifies it. Falsified, the PR is closed and written up.
- **CI's end-to-end bench decides the merge,** not the count alone.
- **Timing on the shared development machine** is never the evidence. A workstation run is
  evidence only under RULES.md C5.

PR #155 is why: its constructor saved 19.5% of its instructions, its walk added 31%, and CI
measured the decoder 7–19% slower.

**Docs-only and measurement-only changes** merge when CI is green and nothing measured
changed that wasn't meant to.

**A red check** is handled by `docs/team/CI-INCIDENTS.md`. It classifies the failure from its
log, re-runs only the failed job, paced to the outage, and records which checks did not run. It
does not relax the green-head rule above.

**For every merge:**
- merge only the head that CI measured;
- set `expectedHeadSha` to that head;
- a later commit that changes measured code needs a fresh bench.

## Release checkpoints

When accepted production changes have landed on `master`, the steward owns the release:
- Batch changes at sensible milestones. Don't release every research iteration.
- Never release unqualified experimental work.
- Don't leave accepted production improvements indefinitely unreleased.

The release runs through the repository's existing machinery.

1. **Deploy Release** (`.github/workflows/deploy.yml`) is dispatched by hand on `master` with
   `bump` set to patch, minor or major by semver. It runs `just release`
   (`.just/ship.just`), which:
   - bumps the Python and Rust versions together;
   - updates both lockfiles (`Cargo.lock` and `bench/alternatives/Cargo.lock`);
   - writes the crate's changelog with release-plz;
   - commits `chore(release): bump 🐍 -> vX.Y.Z`;
   - tags `py-X.Y.Z`;
   - pushes both the commit and the tag.
2. **The tag alone does not publish anything.** A tag pushed with the workflow's token starts
   no workflow. Deploy Release therefore starts `CI.yml` on the tag itself, by
   `workflow_dispatch`. Creating the tag is not the same as publishing.
3. **CI publishes:**
   - the wheels to PyPI (the `release` job, `pypi` environment);
   - the crate to crates.io through release-plz with Trusted Publishing (the `crates` job,
     `crates-io` environment).

   release-plz also tags `mibz2-vX.Y.Z` and creates the GitHub release.
4. **Verify completion.** All of these must be true before a release counts as done:
   - the CI run on `py-X.Y.Z` succeeded;
   - PyPI lists `mibz2 X.Y.Z`;
   - crates.io lists `mibz2 X.Y.Z`;
   - the GitHub release exists;
   - `CHANGELOG.md` has the version's entry.

   Record the shipped version and its release commit. If any step fails, find the cause and
   reconcile it. A partial publication must not be left unresolved.
5. **Downstream:** bump wikidata-pq's mibz2 pin in a separate PR. That PR follows wikidata-pq's
   own `CLAUDE.md` and must pass its own bench gate. A mibz2 win does not certify a downstream
   gain. A package release is not permission to publish anything to the Hugging Face Hub.

`.github/workflows/tag.yml` is an alternative, for tagging a commit that already carries the
new version. `just ship-rust-and-python` is the local recipe for the same release.

**Identity.** Deploy Release's release commit is made by the workflow's `github-actions[bot]`
identity, as for every prior release. That is the repository's release mechanism, not an
AI-attributed commit. Every other commit is the maintainer's: Louis Maddox
<louismmx@gmail.com>, with no AI attribution in commits, PR bodies, release notes or
metadata.

## What stays with the maintainer

These decisions still need the maintainer:
- relaxing a gate or an acceptance minimum;
- the bench's workload, statistical estimand, repeat policy and merge acceptance rule. Harness
  mechanics and diagnostic instrumentation that leave these unchanged are the steward's
  (RULES.md C6);
- a run on the i9 or another workstation. The maintainer authorises it by running a recipe
  frozen before the run, and that authorises that run only (RULES.md C5);
- new infrastructure, credentials or spending;
- widening the scope beyond mibz2 and its downstream pin bump.

A campaign's rules can reserve more for the maintainer. They cannot give the steward more than
this document and `docs/team/RULES.md` do without the maintainer's own words.

## Standing prohibitions

The steward never:
- reads from, writes to or calls the Hugging Face Hub;
- adds AI attribution;
- changes the allocator;
- weakens the exact checks;
- polls, or sets up cron jobs;
- force-pushes;
- treats silence as approval;
- merges an unbenched head.

## Campaigns

Each campaign has an entry document that maps its charter, roles, protocol, state and evidence.
Read it at the start of a session, after context compaction, and before any consequential
decision. Resolve it to a commit and record the revision that a decision used.

| Campaign | Entry document |
| --- | --- |
| compact30 (ended 2026-11-06, archived) | `docs/journal/compact30-channel/CAMPAIGN.md` (history: `docs/journal/2026-11-06-team-neutral.md`) |

There is no current campaign. The agent team's standing work runs under `docs/team/`.
