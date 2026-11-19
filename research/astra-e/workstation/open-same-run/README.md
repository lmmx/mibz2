# Scalar-cap versus ungated Open on the i9

Scalar-cap was faster than ungated Open at every tested worker count under the Coordinator's preregistered rule. Its wall-time advantage was 4.86% at seven workers and 2.34% at ten. Its peak RSS was lower at eight, nine and ten workers. The seven-worker RSS interval includes one. Against LazyCells, the four wall ratios were 0.87446, 0.88060, 0.87529 and 0.87669. Scalar used less peak memory at every worker count, while every wall interval remained above the campaign target of 0.80.

Sixteen same-round pairs at each W produced the following direct geometric means and two-sided 95% Student intervals.

| W | Scalar / Open wall | Scalar / Open peak RSS | Scalar / Open median peak ratio | Scalar / Open maximum peak KiB |
|---:|---|---|---:|---|
| 7 | 0.95145 [0.94935, 0.95355] | 1.00168 [0.99637, 1.00702] | 1.00537 | 194,396 / 195,124 |
| 8 | 0.96052 [0.95357, 0.96752] | 0.99369 [0.98921, 0.99819] | 0.99419 | 204,440 / 206,744 |
| 9 | 0.96907 [0.95516, 0.98319] | 0.98500 [0.97860, 0.99143] | 0.98150 | 213,616 / 216,152 |
| 10 | 0.97660 [0.95447, 0.99925] | 0.98436 [0.97850, 0.99026] | 0.98589 | 221,564 / 225,996 |

| W | Scalar / Lazy wall [95% interval] | Scalar / opend wall [95% interval] | Open / Lazy wall | Scalar / Lazy paired RSS upper | Scalar / Lazy maximum peak KiB |
|---:|---|---|---:|---:|---|
| 7 | 0.87446 [0.87065, 0.87830] | 0.93339 [0.93113, 0.93566] | 0.91909 | 0.71357 | 194,396 / 298,256 |
| 8 | 0.88060 [0.87182, 0.88946] | 0.93357 [0.92466, 0.94256] | 0.91679 | 0.68685 | 204,440 / 338,460 |
| 9 | 0.87529 [0.86585, 0.88484] | 0.94211 [0.93315, 0.95116] | 0.90323 | 0.66610 | 213,616 / 339,676 |
| 10 | 0.87669 [0.86062, 0.89306] | 0.93843 [0.91920, 0.95806] | 0.89770 | 0.67686 | 221,564 / 357,164 |

All eight historical reproducibility comparisons stayed within the frozen ±0.03 absolute tolerance. The earlier scalar/Lazy values were 0.870, 0.880, 0.876 and 0.859. The registered common Open/Lazy reference was 0.919. This comparison isolates ungated Open and supplies separate same-run results for deployment-gated opend. B5’s adoption decision is separate. No instruction, IPC, cache or DRAM counters were measured. [Coordinator registration](https://github.com/lmmx/mibz2/pull/145#issuecomment-7003825682) · [Archived registration](registration.json) · [Earlier scalar](https://github.com/lmmx/mibz2/pull/145#issuecomment-7000695815) · [Earlier Open](https://github.com/lmmx/mibz2/pull/145#issuecomment-6997132553)

## Provenance and scope

Both ready tags pointed to recipe `9f867e69e5e7fe33aaae569af94a447eb38cbe26`, whose addition over scalar recipe `cafcab59` changes only the two Just entries. The Coordinator queued dry then measured in master `f6cfcb04` via #624. Both bundles record recipe `9f867e69`, live master `f6cfcb04`, release `530ed404`, scorer path revision `75297a1d`, selected tree `75f7b032`, printed scorer v1 at `81585172`, Rust 1.99.1 and all 20 logical CPUs of an i9-10900K. Each fresh process used its ordinary worker interface with W workers and 4W slots. Each timed process decoded four concatenated Wikidata copies to 838,800,784 bytes, with two seconds between processes. [Recipe](https://github.com/lmmx/mibz2/commit/9f867e69e5e7fe33aaae569af94a447eb38cbe26) · [Queue decision](https://github.com/lmmx/mibz2/pull/624)

The dry archive was published at `f0c26c1b` with SHA-256 `9e043b67e9e30c8c76237e29f3dd36e9415081e4c40c5703479a1b43ab8f1550`. It passed 338 candidate cases and 98 quick cases for each control. It also records 98 extra candidate cases independently at each W7–W10. Its worker-specific files cover the candidate only. All four arms passed the general gates again in the measured archive. The independently rebuilt executable SHA-256 differs between dry and measured, as recorded in those two bundles. Both selected the same fresh scorer sources and frozen candidate entry. [Dry publication](https://github.com/lmmx/mibz2/tree/f0c26c1b202bf9eeec76528e43731a4a8647c1ea/astra-e) · [Original dry archive](dry.tar.gz)

The measured archive was published at `f71f153b` with SHA-256 `083288e4788798e784c39e5a47f95405f1c94b384485fe4c46aaa575c471c30c`. Independent recalculation checks all 256 rows in the rotated forward/reverse roster, decoded byte counts, sample sizes, full output and reference checksums, all twelve official wall and RSS contrast cells, median ratios and maxima, and each stretch decision. It uses a separate df15 t critical value and reproduces the reported numerical fields within 2×10⁻¹². The CSVs include all six arm pairs, absolute median wall and peak RSS, largest observed process peaks and diagnostic CPU ratios. [Measured publication](https://github.com/lmmx/mibz2/tree/f71f153baaae2d28f0a61acc25a2129612eac9cc/astra-e) · [Original measured archive](measured.tar.gz) · [Contrast table](contrasts.csv) · [Absolute arm statistics](arms.csv) · [Decisions](decision.json)

## Reproduce

Run from the repository root. The audit reads pinned Git objects and both local copies of the upstream bundles. It rewrites the two CSV tables deterministically and prints the decision JSON.

```sh
git fetch --no-tags origin refs/heads/evidence refs/heads/bench/scalar-vs-open master
d=research/astra-e/workstation/open-same-run
python3 -B "$d/audit.py" | diff - "$d/decision.json"
git diff --exit-code -- "$d/contrasts.csv" "$d/arms.csv"
```
