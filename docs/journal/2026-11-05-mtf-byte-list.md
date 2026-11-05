# MTF's list as bytes: registration and instruction count

Registered before any timing of this branch. The decision rule may be amended only in a commit
made before the first bench comment on this PR, and each amendment says what changed and why.

## The change

`mtf_avx2`'s move-to-front list held sequence indices. Every symbol and every RUNA/RUNB run
mapped the front through `seq_to_unseq`: a load on the serial chain from one symbol to the next.
The list is now initialised as `seq_to_unseq` itself, so it holds the bytes. A move permutes
both lists alike: `list_unseq[p] = seq_to_unseq[list_seq[p]]` at every step, so the front is the
byte, with no lookup. The output is the same for every input, corrupt ones included: an index
past the alphabet maps to the same entry of `seq_to_unseq` as before. The scalar `mtf`, the
checkpointed MTF and every check are unchanged.

## Measured: whole-decoder instructions

Inclusive Ir of the driver's `measured`, with the driver, toolchain and input of
`2026-11-05-deferred-flag-walk/`. The base arm is #170's head, 55dff0a, whose source is master
e7c30a86's. Profiles are in `cg/`.

| block | master e7c30a86 | this branch | change | output |
|---|--:|--:|--:|---|
| 0 | 6974 | 6881 | -93 | identical |
| 1 | 19857878 | 19726151 | -131727 | identical |
| 41 | 19207725 | 19092330 | -115395 | identical |
| 97 | 7418 | 7321 | -97 | identical |
| 233 | 19659043 | 19520924 | -138119 | identical |
| 400 | 19220433 | 19101050 | -119383 | identical |
| 472 | 8192 | 8069 | -123 | identical |

## The decision rule

The same as `2026-11-05-mtf-short-runs.md`, which #170 passed under:
- **correctness:** CI green, Miri included, and the same bytes;
- **instructions:** at or below master's (met);
- **benefit:** `wikidata` at 1 or 4 threads, CPU or wall faster beyond noise in one run, and
  not slower in the other;
- **no harm:** at most three runs.

It is closed as not shown if no run shows a gain beyond noise. About 0.6% of decode
instructions is near the bench's power; the MTF stage's own figure is reported as diagnostic.
