# cells-live-path-gap: prospective preparation freeze

Dispatch363/6819644553; preparation starts2026-11-12 09:43UTC, stops by11:43UTC.
Question: on the same live single-thread reader, where is bounded Cells' inversion
advantage lost, and can its eligible gain pay its construction/fallback/reader costs?
Decision: use the fixed experiment below, report one cost reconciliation and one
next action; instrumentation changing the off-build conclusion or failing phase
reconciliation invalidates quantitative attribution. Retain descriptive counts.
No production qualification, gate change, workstation execution or readiness.

## Fixed programme (no candidate observations yet)

Wikidata, pattern, onebyte in that order. One thread, one allowed logical CPU
selected explicitly and pinned for every child; record affinity/topology. Four
separate release binaries: baseline/Cells × hooks compiled off/on. Exact corrected
production source d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2, default BzReader and
unchanged benchmark CellsReader. Rust1.99.1 x86_64-unknown-linux-gnu, release fat LTO,
one codegen unit, no RUSTFLAGS. Original 4MiB drain and original ab streaming hash.
SHA256/full byte comparison is untimed only. Fresh process each decode; no warmup,
within-block repeats, minima or adaptive extension. Settle0.25s before each measured
child. Ten paired rounds per corpus (120 measured decodes total). Round index0..9:
even rounds off first, odd on first; baseline first iff round mod4 is0 or3,
otherwise Cells first. Thus both binary orderings and off/on order are balanced5:5.
Exactly9 prechecks: trace-on baseline and Cells full-output checks + independent
Python/libbzip2 oracle once per corpus. Every measured hash/length matches precheck;
all blocks/streams still execute original CRC/error checks. No forced admission.

Inputs: frozen generator tests/common/mod.rs and ab gen at corrected source,
scale4, bzip2 compression level9. onebyte20,000,000 `a` bytes; pattern19,600,000
bytes (`0123456` repeated2,800,000). Compressed SHA256 respectively
22924f5b3187645f97653ae4923df14d2597816c2e57f2c7228bc844119c4a9e and
0f07e2a274b9189677c0f22fd3fc8d15f17324b1f49e651a321239321d4e1407.
Wikidata manifest bench/wikidata.json, windows at bench-data
 a0692452b67ec3a8833e1fc4adf58a4f09cd9e6a;
compressed11,854,933 bytes SHA256
82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915;
decoded209,700,196 bytes SHA256
3f075e30f4f8c73414150655afbf57caa2647c7927e743fc0635b4912d65cdaf.
No corpus replay during preparation. Existing367 reconstruction/oracle retained.

Raw per-process wall/CPU/RSS, exact binary/compiler/source/input digests retained.
Ratios Cells/baseline and on/off, paired by complete decode, geometric mean and
95% t interval on log ratios (df9, t=2.2621571628540993); never treat blocks as
independent observations. Faster if entire interval<1, slower if>1, otherwise
unresolved. If on/off yields a different Cells/baseline category, no quantitative
phase attribution. Inclusive parent spans and exclusive children must reconcile
exactly; remaining whole-reader/drain work and process-minus-internal time remain
charged. Clock count, record growth, requested allocation/capacity and copy bytes
are descriptive source accounting, not physical traffic. Unavailable counters are
explicitly missing, not zero. Scanner/front, order, induction, writer, fallback,
finish/CRC and inversion inclusive are measured on actual code; never phases.rs.

## Source/evidence boundary and warm comparison

Based on367 a08277e1c36e276cfea7b3116d99ea5fd93f0e84; historical dependency
57f51bbc0ca62ac3ee1cc148a1010842eea6222a and failed legacy evidence are preserved,
not requalified. Source hooks are generated into a separate ignored tree from
corrected365, with exact pre/post SHA manifests. Original unsafe operations and
F01 long-factor tests remain unchanged. Independent harness review still needed.
365 DEFER,371 REVISE METADATA, stopped treap, deferred summaries and372's absence
of backend-screen clearance stand. Reference Optimiser6782313693/report
ceda2ae2ab60f289d9e3240381112205f6eebae4 and372
0596240ef7b61a23c3c4135c910d569f9fbeb8a4.
Original archives: i9 d85b2074099f027d048aa7a057da10d25f2b70bb,
SHA256552e26aaf3b41636afdbd5a195ad5f0ce1c66aed945ec2911a508057baed705c;
CI artifact13802703606 SHA256
c36061ac6a7e8ac2991acbd82e2935fd20f9a3d075e102bd42c67b46cb575141.

This programme cannot decide warm port versus research directly. Separate proposed
budget, NOT admitted or implemented here: one frozen Wikidata scan to capture the
same records, primary and n>=65536 blocks, then ten balanced paired whole-file
passes of production d4f0e8 and research c79ff7d384072e5e95eda7e513227f2aa06ff3d5,
one untimed warm inversion per eligible block per arm then one observed inversion
per block per pass, no minima. That is20 measured inversion-file passes plus20
untimed warm passes, and one extraction, not additional live-reader decodes.
Both consume the identical prebuilt records in decode order, same host/compiler/
affinity/buffer-reuse policy, with admission/refusal charged for the bounded port;
research remains unbounded and separately labelled. Matching records does not
match internal layouts. Exact adapter correctness, final budget and conditioning
require separate Coordinator disposition before admission. Do not execute367's
superseded three-round recipe. Warm port loses -> price port/writer; warm wins but
live inversion does not -> conditioning; live inversion wins but total does not ->
remaining stages/reader. Without approved warm evidence, leave those branches open.

## Preparation exit

Safe hooks, reproducible four-build harness, fixed plan and synthetic correctness/
error/fallback/failure-path tests only. No real corpus census/timing. Any new unsafe
or changed invariant stops that portion. Deliver immutable full or partial work
within2h, exact logs/digests and limitations. Coordinator must reconcile results,
arrange independent harness review, decide separate integration and check the
frozen recipe; exact-head green CI and PROTOCOL9 queue/ready signal are subsequent
requirements. This document grants none of those permissions.
