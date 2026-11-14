# Selective Cells in the public sequential decoder

Owner-directed production PR, based on master
`6dec95ba6aaf59d5134e5ab119fd52433e2032ee`. No merge, release, workstation
execution or change to an acceptance rule is part of this assignment.

## Change and source correspondence

The kernel is PR #380 at `a8a23c11607864cb431f87fff38169e10c41685f`.
Its `src/cells.rs` SHA-256 is
`3bbc63745068713b75bd6cbaac4217d374fbca6d1318a55c27bf9e788858aeee`.
This PR changes only three documentation lines in that file: the experimental,
off-by-default descriptions become calling-thread default descriptions. Every
executable line, test hook and safety assertion is identical. The two included
kernel test files are byte-identical. No writer, induction, admission, allocation,
MARK, rotation, repetition or fallback change is introduced.

| Public entry point | Proposed inverse BWT |
|---|---|
| `BzReader::new`, `BzReader::with_kernels` | Selective Cells, existing bounded Segmented fallback |
| `decode` | Selective Cells, same fallback |
| `decode_blocks` | Selective Cells on the calling thread, same fallback |
| `decode_with::<Segmented>` | Explicit Segmented reference |
| `Scratch` with omitted type parameter; `decode_block` using that scratch | Segmented, unchanged |
| `decode_parallel`, including `threads=1` | Existing parallel path, unchanged |
| `BzReader::parallel`, `parallel_with`, `parallel_planned`, including one worker | Existing parallel path, unchanged |
| `StreamDecoder` and cooperative/team plans | Existing implementations, unchanged |

Python `Reader` and `decompress` reach the new sequential default when their
resolved thread count is one. Resolution of `None`/zero and all multiworker paths
are unchanged. External users that parallelise calls to `decode_blocks` will
incur the new per-call Cells footprint; this API itself creates no workers.

`src/parallel.rs`, `src/coop.rs`, `src/run_fed.rs`, `src/python.rs`, and the standard
benchmark harness/workflow are byte-identical to the base. There is no generic
parameter added to `BzReader`, nor a layout change to its parallel state. The
sequential state was already boxed. Source identity does not guarantee identical
machine code layout or parallel timings; the ordinary PR benchmark must check them.

The imported forced-route fuzz/tests are from #380. Two existing service fixtures
also retain its reviewed `corpora(2)` correction: growing source text otherwise
crosses the generator's repetition threshold. Their four-block reach assertions
remain intact. No service implementation changes.

## Safety and correctness

The existing scanner, MTF, RLE1 and CRC checks remain in place. Only the kernel type
changes at the three calling-thread entry points. Cells state remains owned by its
scratch, including the Segmented fallback; no new sharing or pointer lifetime is
introduced. Refusals still invoke the original Segmented method, with the same
output/error behaviour, rather than starting an alternative fallback.

The safe Cells boundary validates the records, tiling and marked row before unsafe
induction. Density, initial-letter floor and per-cell budget checks are unchanged.
For record count R and budget B=floor(4n/25), kernel buffers are bounded by
48R+84B+4528+4G bytes (G=64 for debug/Miri, zero for release), excluding the existing
decoder/fallback/output buffers and allocator overhead. Reused state retains bounded
high-water buffers; mixed admitted/refused blocks can retain both Cells and fallback
storage. A bound is not a claim that RSS stays within +10%.

Carry the existing [selective safety argument](https://github.com/lmmx/mibz2/blob/a8a23c11607864cb431f87fff38169e10c41685f/bench/cells-selective/SAFETY.md)
and [F01 writer correction](https://github.com/lmmx/mibz2/blob/a8a23c11607864cb431f87fff38169e10c41685f/docs/journal/monodromy/2026-11-11-cells-f01/HANDOFF.md).
[Researcher B5 closure 6857443323](https://github.com/lmmx/mibz2/pull/145#issuecomment-6857443323)
applies to that kernel. It is not self-certification of this public-path integration;
the Coordinator is asked to obtain the scoped independent integration review.

New tests exercise the actual public defaults against explicit Segmented and
libbzip2, including malformed CRCs/truncations, arbitrary columns/origins, periods,
admission/refusal witnesses, concatenated streams and retained-buffer reuse. The
new Miri test asserts actual successful Cells admission and short-cycle reach
through `BzReader`, alongside byte parity through `decode`. Inherited tests retain
the long-factor/F01 writer reach, selected-live-state comparison, 15/16/17-byte and
marked-word boundaries. CI requires discovery of these Miri tests and adds the
existing forced Cells fuzz mode. The ignored seven-corpus test is an untimed,
full-byte comparison of all three new public defaults and both reference decoders.

## Existing matched i9 evidence, not this PR's performance verdict

The archived 252-decode experiment used twelve paired rounds on seven families,
three common-tree complete-reader arms, hooks and sampler off. All observations
are retained. Ratios below are selective Cells / Segmented; intervals are the
registered paired log-t 95% intervals, not ratios of independent medians.

| Corpus | Wall ratio [95% CI] | CPU ratio [95% CI] | Peak RSS ratio [95% CI] |
|---|---|---|---|
| wikidata | 0.8630 [0.8319, 0.8952] | 0.8645 [0.8339, 0.8963] | 1.3107 [1.3077, 1.3138] |
| text | 0.9999 [0.9873, 1.0127] | 1.0014 [0.9898, 1.0133] | 1.0035 [1.0007, 1.0062] |
| pattern | 1.4352 [1.1964, 1.7217] | 1.3843 [1.1798, 1.6243] | 1.1511 [1.1467, 1.1556] |
| onebyte | 1.0392 [1.0274, 1.0511] | 1.0313 [1.0175, 1.0452] | 1.0781 [1.0766, 1.0797] |
| random | 0.9952 [0.9866, 1.0039] | 0.9959 [0.9881, 1.0038] | 0.9988 [0.9952, 1.0025] |
| records | 0.9862 [0.9701, 1.0026] | 0.9908 [0.9726, 1.0094] | 1.0006 [0.9976, 1.0035] |
| runs | 1.0180 [0.9980, 1.0384] | 1.0132 [0.9996, 1.0269] | 0.9994 [0.9972, 1.0017] |

Wikidata selective/original Cells wall is 0.9256 [0.8823, 0.9711]. Median Wikidata
wall is 0.48285s Segmented and 0.4121s selective; median RSS is 26,320 versus
34,524 KiB (maximum 26,400 versus 34,528 KiB). The approximately 8 MiB increment is
modest on the target desktop but material for each concurrent calling-thread
decoder and is not a universal memory-policy exemption. Pattern median RSS rises
16,000 to 18,496 KiB; its wall regression is **43.5%**, not hidden by the primary gain.
Onebyte budget-refuses and is not an induction win. Text has no established gain.

Evidence: bundle at `e04fe229e9d939840d50fea110c7b7c6f74811ef`,
`cells-selective-i9/20261113T233358053141311Z.tar.gz`, 60,935,434 bytes,
SHA-256 `330ff61ce5e2659c8f0a86837d26eab274d0eb1fbe4e5563d3f3824f92912fa7`.
[Coordinator validation](https://github.com/lmmx/mibz2/pull/145#issuecomment-6867899861)
and [D104 acceptance](https://github.com/lmmx/mibz2/pull/145#issuecomment-6867998245)
are bounded diagnostic evidence. The coarse high post-start activity estimate is
retained; throttle counters were unavailable, not measured zero.

## Integration decision still required

The standard PR CI benchmark evaluates the actual proposed `BzReader` default at
one thread and unchanged explicit parallel APIs at the existing thread counts.
Its source, corpora, estimator and gates are unchanged. CI success alone is not a
performance pass. Exact-head results and local logs are linked from the PR/REVIEW.
The earlier #380 default four-thread regression and unresolved cause remain part
of the record; this PR neither erases it nor claims to explain it through layout.

The [specific default-adoption gate](https://github.com/lmmx/mibz2/pull/145#issuecomment-6775049636)
requires every other corpus/thread count level and RSS within +10%, plus a
prospectively registered one-build i9 confirmation before release.
`PERFORMANCE-POLICY.md` additionally reserves secondary wall regressions over 10%
to Louis. The current instruction authorises preparing this PR, not changing those
rules. The narrow owner decision is whether to authorise a prospective adoption
exception for the documented single-thread pattern loss and Wikidata/pattern RSS
costs, keeping correctness, B5, primary/parallel performance and exact-head CI gates.
Any additional actual-default CI loss must be assessed explicitly too. The
Coordinator must record the applicable prospective rule through the established
review procedure; old failed verdicts stay failed. Actual public-default target
confirmation, if required for release, needs separate authorised registration/run.

No new workstation experiment, retry, ready tag, queue entry, merge or release was
performed. The overlapping attribution project was parked by Coordinator
[6868441092](https://github.com/lmmx/mibz2/pull/145#issuecomment-6868441092).
