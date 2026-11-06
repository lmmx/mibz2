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
