# Archived structural replay and critical-block priority

The archive replay independently reconstructs the handoff's frequencies and
strengthens the [direct-refill proposal](proposals/refill-boundary/proposal.md).
It creates no constructor and makes no new measurements. Both construction
captures remain INVALID; these are structural counts and descriptive exposures.

Reproduce with:

```sh
python docs/journal/compact30-research/replay_structure.py --repo .
```

The script checks the archive's Git blob and the JSONL's manifest length/SHA-256
before reading its 434 per-block counter records. Exact output, including every
block's lower bound, is in [structural-replay.json](structural-replay.json).
Archive blob: `c0e5ddbac2e60eba0d6e3123ad8b22d7b0fce7fd`; SHA-256:
`ebdb74549b39f83f07ae1f7d1c943916bac2ecb0be85868799aa30fe5e3d8af9`.
JSONL SHA-256: `f25325eb2339b1a1385d598e8dd5a8a6d5e37c110c207ddcff18bae53f2e172b`.
The archive manifest pins source `fa21f33b372f7534e7610d93710e06c3af506ccc`;
the frozen B2 source/listing identities are separately recorded in the index.

| Reconstructed frequency | Result |
|---|---:|
| Blocks / rows | 434 / 207,006,515 |
| Large blocks (n≥899,000) | 230 |
| Full tiles T / uniform U / nonuniform S | 12,937,790 / 10,544,477 / 2,393,313 |
| Tail rows | 1,875 |
| Q upper bound Σ min(epochsᵢ,runsᵢ) | 3,450 |
| Eligible fast-full finishes, B2 bounds | 1,647,374–1,647,407 |
| Nonempty scalar refills, lower bound | 1,122,673 |
| Direct-refill eligible transitions, new lower bound | **1,084,913** |

For block i let Pᵢ be all finished portions. Each run closes at most one nonempty
prefix, and the final sweep closes at most 256. Hence
Nᵢ=max(0,Pᵢ−runsᵢ−256) bounds nonempty scalar refills below. On a successful
natural build a refill has a successor below the bucket limit. Its successor is
ineligible for a full interior shortcut only if it crosses a bucket boundary or
is the global terminal partial record. There can be at most one of the latter,
and only when nᵢ mod 30 is nonzero. This covers partial **wide** records too;
`norm_partial` alone does not.

Therefore Eᵢ=max(0,Nᵢ−fin_crossingᵢ−[nᵢ mod 30≠0]) is conservative.
Summing gives 1,084,913, stronger than the aggregate fallback 1,081,119. Every
positive bound belongs to one of the 230 large blocks. This does not claim an
exact eligibility count; future counted-only instrumentation must supply that.

At the proposal's illustrative full-wrapper cap C≤40 and failed-guard cap g≤5,
the conditional budget is 27E−5P−H = **20,315,241−H** instructions. H includes all
remaining changed callers, setup, fallback, spills and shifted work and is
unbounded before compilation; thus verified net credit is zero. The transient
null reset exposes **26,037,912 logical store bytes**, before replacement costs.

## Relevance to four-worker critical blocks

The replay retains all 20 archived warm rounds, including the unstable round.
Across 1,140 all-full candidate steps, the completion-critical worker is chosen
by latest **absolute end**, consistent with completion=max(end)−min(start).
Longest worker duration is also computed separately; it is not substituted for
the critical endpoint.

| Structural exposure | All assigned workers, mean | Latest-ending worker, mean | Ratio |
|---|---:|---:|---:|
| Direct-refill eligibility lower bound | 4,719.82 | 5,144.56 | 1.0900 |
| Scalar rows | 166,529.17 | 181,263.84 | 1.0885 |
| Fast finishes (includes partials) | 7,164.23 | 7,829.84 | 1.0929 |

This supplies a concrete priority reason for the proposal: its conservative
exposure is larger on those archived critical workers. It does not establish
that an instruction removed shortens their execution, that the same worker stays
critical, or that a future capture will pass. Register pressure, memory
dependencies, guards and shared resource effects remain unresolved.
