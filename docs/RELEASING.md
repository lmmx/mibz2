# Releasing mibz2

The release runbook. The coordinator follows it step by step for every release. It is
referenced from `CLAUDE.md`, so every session reads it before releasing.

The maintainer's standing instruction is that the coordinator releases mibz2 itself, without
asking first, whenever merged work is worth releasing. Nobody needs to prompt it.

## 1. Is there something worth releasing?

After every merge to master, not only when asked, look at what has merged since the last tag:

```sh
git log <last tag>..master -- src Cargo.toml
```

Worth releasing:
- **A performance gain** shown under its PR's registered rule: the CI bench, or a registered
  workstation run admitted under RULES C5.
- **A change that is definitely committed to:** a fix users need, or an API or packaging
  change.

Docs, journal, bench-harness or blog changes alone are not.

## 2. When

- **Release while the maintainer is around,** not overnight or when they are away from the keyboard. A broken
  release needs someone to hear of it.
- **If work becomes releasable while they are away,** release it when they are next around, and say so.

## 3. Before

1. **Master is green:** Test, Fingerprint and the site.
2. **One paired bench run of the last tag against master:** a `workflow_dispatch` of the Benchmark
   workflow on master, with `base=<the last tag's commit SHA>`. Give it the SHA, never the
   tag name.
   - It checks that nothing got worse. It is not a gate to re-run until the result looks good.
   - Name its CPU. It is in the run's `env.json` artifact and in its step summary.

## 4. The release

1. **Dispatch Deploy Release** with `bump`: patch, or minor for a public API change. It bumps the crate
   and the Python package together, tags `vX.Y.Z` and `py-X.Y.Z`, and starts CI.yml. CI.yml
   publishes the crate (release-plz, Trusted Publishing) and the PyPI wheels.
2. **Verify the publication:**
   - crates.io and PyPI are at the new version, with every wheel and the sdist;
   - docs.rs has built it.
3. **Fix any publishing failure** with a follow-up patch release, as #209 did.

## 5. Release scaling: the new release against the last, on the workstation

The History page's Release scaling view shows each release against the one before it on a
real many-core machine (`bench/release-scaling/`, `docs/generated/release-scaling.md`). It
shows what CI's 4-CPU runners can't, such as the decoder's knee past 8 threads (the
three-team policy came from it). Every release gets one.

1. **As soon as the tag exists, ask Louis to run, on the i9:**

   ```sh
   cd ~/dev/mibz2 && git checkout master && git pull
   just submit bench-release-scaling
   ```

   - With no arguments, the recipe compares the current release (the newest `vX.Y.Z` tag)
     against the last one (the tag before it). Never write version numbers into the command.
   - It is his run, under RULES C5. The agents never time on the workstation. Its figures inform; they don't decide.
2. **When the submission lands on #145:**
   - fetch the bundle from the `evidence` branch at the commit the post names, and check its SHA-256;
   - copy its `recorded/` directory to `bench/release-scaling/<date>-<machine>-<release>/`;
   - commit that to master. The site rebuilds the History page from it.
   - **check the live History page shows it,** after the site deploys: its hero's Release
     scaling view must open on the new run, with the run named by its releases ("0.2.9 against
     0.2.8"). Look in a fresh browser profile and in one that has seen the page before, since
     the view remembers a reader's last choice. A run that is recorded but not shown first is
     not done (10 November 2026: the hero stayed on 0.2.4 against 0.2.1 after 0.2.9's run).
3. **If the run can't be had** (the i9 is away, or Louis declines), say so in the report. The release
   doesn't wait for it.

## 6. After

1. **A CHANGELOG PR** with the release's notes. #210 is the model. It gives:
   - what changed;
   - the paired run's table and its CPU;
   - the stages;
   - the cost;
   - the caveats;
   - the release-scaling figures once they are in.
2. **The downstream bump:** mibz2 in lmmx/wikidata-pq's `reshape/Cargo.toml`, in a PR. It merges
   under that repository's bench rules.
3. **Report the release:**
   - to the maintainer, in one short message;
   - in the channel log, as a LOG on #145;
   - to the Observer, as a CHECKPOINT.
