---
description: "How mibz2 is released to crates.io and PyPI at one version: Deploy Release, Tag Release and Release Notes."
---

# Releasing

The crate and the Python package are released at one version, from one commit.

## Deploy Release

Run the **Deploy Release** workflow (`deploy.yml`) with a bump (patch, minor or major).
It runs `just release`, which:

1. bumps the version in `pyproject.toml` (`uv version --bump`) and sets the crate's in
   `Cargo.toml` to match;
2. writes the crate's changelog with release-plz;
3. commits `chore(release): bump 🐍 -> vX.Y.Z` and tags `py-X.Y.Z`;
4. pushes the commit and the tag, then starts **CI** on the tag (a tag pushed with the
   workflow's token starts no workflow by itself).

CI on a `py-*` tag builds the wheels and the sdist and publishes them to PyPI, and
publishes the crate to crates.io with release-plz (which tags `mibz2-vX.Y.Z` and makes the
GitHub release). Both use Trusted Publishing, in the `pypi` and `crates-io` environments.

## Out-of-band cases

- **Tag Release** (`tag.yml`): tags an existing commit `py-<version>` at the version its
  `pyproject.toml` and `Cargo.toml` already hold (no bump), and starts CI on the tag. Each
  registry is skipped where it already has the version.
- **Release Notes** (`release-notes.yml`): makes a version's GitHub release (`v<version>`)
  at a given commit if it does not exist, or updates its notes, from that version's
  section of `CHANGELOG.md`.

## Release notes: the evidence

Every version's section of `CHANGELOG.md` is the GitHub release's notes, so it carries the
release's evidence. release-plz writes a one-line entry per commit (and only for commits
that touch the crate's files). After the release, rewrite the section by hand:

1. **Dispatch the tag-to-tag bench.** Run **Benchmark** (`bench.yml`) from the Actions tab
   with *Use workflow from* the new tag and `base` the previous tag:

    ```sh
    gh workflow run bench.yml --ref vX.Y.Z -f base=vX.Y.W
    ```

    It is a paired A/B like a pull request's (the base's own harness binary as the arm
    `mibz2@base/ab`, the order alternating round to round), on one runner. Its figures are
    the run's summary and its artifact (`bench-pr-<sha>`: `result.json`, `comment.md`,
    `runs.jsonl`, `stages.jsonl`); nothing is recorded. The base needs `examples/ab.rs`,
    and the stage table needs `examples/phases.rs` (v0.1.2 on); v0.1.3 and v0.1.4 have run
    as bases this way. The comment's "the merge base with master" means the base given.

2. **Write the entry** to the template below, from that run alone, in a pull request.
   Never add up or chain the pull requests' figures: they were measured against other
   bases, often on other runner CPUs. A section holds no line starting `## [` (Release
   Notes ends the section there), so its headings are `###` and below.

3. **Re-run Release Notes** (`release-notes.yml`) once the pull request is merged, with the
   version and its release commit, so the GitHub release takes the section from master.

### The template

```markdown
## [X.Y.Z](https://github.com/lmmx/mibz2/compare/vX.Y.W...vX.Y.Z) - YYYY-MM-DD

One or two sentences: what changed for a user, and the release's own Wikidata figure at
1 thread and at the runner's cores, from the tag-to-tag run.

### Measured: vX.Y.Z against vX.Y.W

| corpus | threads | wall, vX.Y.W → vX.Y.Z (median) | wall change (95% interval) | peak RSS | same bytes |
|---|--:|--:|--:|--:|---|

Peak RSS, same bytes, and the stage table's rows for any stage a change moved (with the
total), as bullets.

<details><summary>Reproducing this run</summary>

The rerun command and the artifact's name; both tags' full SHAs; the runner's CPU, cores,
clock, kernel and image; rustc; the bench version, rounds and threads; the corpora's bytes
and full SHA-256s (`result.json`'s `corpora`).

</details>

### <!-- 3 -->Performance (one heading per group: Features, Bug Fixes, ...)

- **The change, in a few words** ([#N](https://github.com/lmmx/mibz2/pull/N))
  - *Mechanism:* one or two sentences.
  - *Measured, in its pull request:* its own paired figures, with the runner CPU.
  - *Trade-offs:* what got slower or bigger, and why it was kept.
  - *Evidence:* the bench comment, its runs, the journal entry.

### Downstream: wikidata-pq (if it exists; marked measured or modelled)
```

- **Decoder figures and downstream figures stay apart.** wikidata-pq's figures go under
  their own heading, each marked *measured* (a run, linked) or *modelled* (what the model
  assumes).
- **Measurement-only and docs-only changes** get one line each: what they add, and that
  the decoder's figures did not change.
- **Rejected experiments** go in the journal (`docs/journal/`), not the changelog.
