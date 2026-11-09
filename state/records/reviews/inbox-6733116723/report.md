# Direct placement loses; runner dependence remains unidentified

Brief [6733116723](https://github.com/lmmx/mibz2/pull/160#issuecomment-6733116723). Measured head **aa409f6f2ebce9f41dc7902789f0ab8d6958b020**; result journal **c41ef38d7df2f596193c01d9adbb2f4893f1d217**. Governing specs **a4f97714cc14366984015869d9d094d89488ac57**.

**Keep C29 first.** B re-enters as a controlled runner/code-generation diagnostic, not a production lead ahead of C29. D's registered STOP stands. A compact stable run-index order is a bounded adjacent construction (C43), but it is still a counting sort and must pay dependent gathers. The current readings do not identify write streams, cursor reads or seeds as the cause individually.

## 1. Replayed evidence and the overlooked controls

Downloaded both complete artifacts and checked their GitHub SHA256:
- [#277 run42567752634](https://github.com/lmmx/mibz2/actions/runs/42567752634), artifact13651923399: **00b0e008bdd8a90108ccaa87e21467dd35d70df35e1b71b86e687078a64a3e94**.
- [#269 run42549324025](https://github.com/lmmx/mibz2/actions/runs/42549324025), artifact13645275344: **9be443df35b8aa51c08c4b9491557b0d077cb6b78a11167625e2f7195e88d539**.

Each run's pinned reporter reproduces its readings from `run-fed-1.json`; four contended copies were not pooled. Raw records, reporter outputs and an independent arithmetic replay accompany this report. The arithmetic retains the reporters' paired geometric mean,15 rounds and rounded t multiplier2.145. I did not reconstruct all synthetic inputs or rerun full admission, Rust, Miri, fuzz or timing. Thus **raw readings replayed**, not a new independent assertion of full VALID or safety.

| Pinned-copy ratio | Geometric mean [95% interval] |
|---|---|
| D/A, #277 | 1.052451 [1.050598,1.054308] |
| D/B, #277 | 1.078033 [1.072698,1.083395] |
| D place / B(sort+fill) | 1.204515 [1.202353,1.206682] |
| B/A, #277 | .976270 [.971281,.981284] |
| B/A, #269 | 1.159179 [1.138677,1.180050] |

Q1 STOP and D's synthetic-screen failure reproduce. D fallback's worst ratio1.006082 passes its screen. No reinterpretation of the registered result.

**B's exploratory Wikidata win is not a clean all-corpus win.** On this same AMD run, B/A is1.28423 on random,1.37493 on records,1.38307 on runs,1.06369 on text. Wikidata's B fallback/A fallback is1.021067 [1.014660,1.027515]. These B figures were not D's judged screens; they nevertheless defeat a claim that B is ready for production. Even the Wikidata point estimate does not meet the old5% minimum. Text also differs by checkout and must not be used as a matched cross-run workload.

**The declared baseline changed, but the measured kernels did not change at source level.**
- Master baselines a4f97714 and6145b3a9 have identical `src/mtf.rs` blobs27716447766a2514e9cd0da34fe617a7143f62a3 and `src/bwt.rs` blobs ed3c6e1a5bc068c587bdd2697446485f62d6b590.
- Both experiment heads fd60fffada25d475bbcc2b54165b96cf5fd7a92f and aa409f6f have identical entire `src/mtf.rs` blobs2610fa7ed1a7ee2c7653db47402bb7b7e9e3bb73 and `src/bwt.rs` blobs cb971c9f054492a5092264421166f4e6707fb5a8.
- In `src/run_stage.rs`, the old runtime prefix through B's build is unchanged; D and new tests are appended. Both logs report rustc1.99.1 (58cf8f9c3), AVX2 and CLMUL enabled.
- The harness changed from the old arms/control/store-reference arrangement to A/B/D, so code layout, inlining, allocation/cache history and measurement sequence remain confounded with host. Identical Rust is not identical machine code.

The master diff changes parallel/read paths, among other files; those are not A's timed `mtf::mtf -> bwt::build` path on already-scanned symbols. This narrows the baseline question substantially. It does not establish that CPU caused the reversal.

## 2. Price D's loss without naming an unmeasured cause

On the accepted Wikidata blocks: R=10,389,028 runs, N=167,407,936 BWT rows, C=12,676,540 cells, S=19,388 nonempty bucket instances across384 blocks. O=209,700,196 decoded output bytes is the phase denominator.

No estimate by rounded slots is needed: exact B vector stores=247,388,152/8=30,923,519; D has30,877,916 vectors plus84,059 scalar stores. Thus D issues **38,456 more source-level table-store operations**, despite writing fewer slots. That is only.124% more table stores; it cannot by itself justify attributing a20.45% phase regression to store volume. Compiler stores, spills, branches and instruction mix remain uncounted.

The paired phase gap's median is **.270698 nominal c/output B**, or **5.46398/run**, **4.47799/cell**, **2,927.86/nonempty bucket**. These are alternative ways of allocating one unexplained gap, not independent costs.

Three hypotheses survive, with different falsifiers:

| Term | Source evidence | Missing discriminator |
|---|---|---|
| Destination locality | D interleaves monotone bucket streams; B writes one ascending table stream | Actual reuse distances, line/RFO misses and exposed stalls;50.49 buckets/block is not50 simultaneous hardware streams |
| Cursor/control dependency | D loads a[c],end[c],cur[c], then dependent recs[l+1]/recs[l], with bounds checks, loop tests and state stores; B merges two sequential checked iterators | Native listing, dynamic checks/spills and critical dependencies; random-looking addresses do not imply misses |
| Seed searches | One narrowed binary search per nonempty bucket, before D's run loop | Separate counted probes and a bracket around existing seed setup, keeping full-entry cost included |

At n≤900,000, a conservative20 probes per seed is≤387,760 probes. Charging the entire gap to seeds would require at least **146.39 nominal cycle-equivalents/probe**, or2,928/seed, under this allocation. That makes an instruction-only seed explanation weak; cold dependent probes could still be material. Count actual probes and observe their bracket before discarding seeds. Conversely, per-run/cell dependence has millions of opportunities to consume5.46/run or4.48/cell.

All bucket state is about3KiB, and50 frontier cache lines would be about3.2KiB—not automatic cache-capacity exhaustion. Record cursors collectively advance O(R), but their interleaving can destroy the sequential merge's locality and add reload/control work. D's vector loop also tests bucket ownership, and its checked-record access differs from B's validated raw reads. “Equal stores, therefore scattered writes caused it” is not identified.

Minimum next attribution: count seeds, probes, cursor advances, cell/run transitions and actual source-store classes; inspect native loops for bounds checks and spills; retain the existing exact body as the timed control. An optional preparation control can materialise seeds before a matched placement bracket, but its cost stays in the full total and warm-record effects must be named. Do not subtract the preparation time and call the resulting warm path a production gain.

## 3. C43: compact stable run-index buckets

A precise version of “bucket-batched two-pass” exists, but **two grouping passes plus a fill** is the honest accounting:

1. Scan original L-order records. Validate rising starts, final n, lengths and counts; accumulate number of runs per byte and byte-length totals. Prefix run counts into bucket offsets.
2. Scan them again, scattering only the original run index j into its byte's bucket, stably. Store a u32 index, not B's [F-start,L-start] pair.
3. Consume index buckets in increasing byte order. For index j, load p=recs[j]>>8 and length=(recs[j+1]>>8)-p. Keep q as the running F position. Intersect [q,q+length) with a **single global L cursor**, emit T[q]=L[q]|Psi(q)<<8, and advance q. F buckets tile [0,n), so both output and the L-byte cursor advance globally.

Stable order within each byte preserves LF rank. Validation plus bucket-length agreement proves ownership before publishing the table. A malformed input must not expose partially initialised output. Global ascending fill can use B's slack/repair overstore argument if that exact allocation contract is preserved; it does not inherit D's need for per-bucket scalar tails. Any helper partition needs separate physical write ownership and B5 evidence.

Scratch ordering data becomes4(R+1) rather than8(R+1) bytes, approximately half. It is **still a stable counting sort**, not grouping for free. Source-record indices induce dependent record-pair loads; the cursor that supplies L[q] is sequential, but the p/length gathers are not globally sequential. They are monotone only within each byte. Prefetching future indices is a proposal whose instruction/occupancy cost must be charged. No linked-list-per-byte alternative is preferred: it replaces this lookahead-friendly index stream with a pointer-chasing recurrence.

### Instruction and miss budget

Let G be grouping/check/prefix/scatter cost, E the new fill minus B's1.009902 fill, and H other setup/ownership changes, all normalised to output bytes on this same evidence scale. To beat B's sort+fill **1.328894**:

`G + E + H < .318992 nominal c/output B.`

That is **6.43878/run** for all grouping plus incremental fill costs, or1.66461 total construction c/accepted BWT row. These are planning allowances, not measured instruction budgets. Relative to D, at least.270698/output B must be recovered just to reach B; future registration requires its own information/performance objective.

A useful sensitivity equation in per-run units is:

`g + u/rho + m*lambda + h < 6.43878.`

Here g is grouping cost, u incremental effective uops, rho achieved throughput only if issue binds, m extra exposed record-cache misses/run, lambda their effective unhidden penalty, h other effects. Do not add overlapped components twice; a full model uses binding resource maxima. With g=u=h=0, lambda20/50/100/200 permits at most.322/.129/.0644/.0322 additional exposed misses/run. Real grouping uses much of the allowance. These are hypothetical penalties in the same nominal-equivalent units, not asserted hardware latencies. One long-latency miss per run would exhaust the budget many times over.

There is no valid numeric retired-instruction cutoff without emitted code and a binding-resource account. The new ordering writes4R fewer bytes and reads a smaller order stream, but also reloads p/length pairs from recs. Logical metadata traffic can merely move rather than fall. B's entire sort is only.319; halving its metadata bytes cannot claim halving its elapsed cost.

A genuinely bounded alternative that rescans L for each of sigma buckets costs O(sigma*R) record inspections. At50.5 buckets it replaces one cheap grouping with roughly50 scans; a batch ofb buckets still needs ceil(sigma/b) scans or extra deferred records. A fixed-capacity tile cannot flush globally F-ordered output without preserving older pending runs or rescanning. C43 is therefore the concrete low-complexity candidate, subject to the6.44/run account, after the runner control.

## 4. One diagnostic separating source, build context and runner

**Information objective:** establish whether the B/A reversal survives an identical frozen executable on both observed host families, and quantify any complete-revision/build-context effect separately. No production qualification, new workload selection or retrospective relaxation.

### Frozen design

- Exact old baseline a4f97714cc14366984015869d9d094d89488ac57; exact new baseline6145b3a985302bb2b6629b12c8c0602426547544; B runtime from fd60fffada25d475bbcc2b54165b96cf5fd7a92f. Freeze adapter, harness, dependency lock, rustc1.99.1, target/features, release/LTO settings and binary SHA256. No per-host native compilation for the main comparison.
- One common harness exposes A_old, A_new and B with identical data preparation, allocation policy and dispatch wrappers. **A_old and A_new kernel sources are identical here.** Record that negative control explicitly; do not pretend duplicate symbols are distinct algorithms. If linker folding makes one address, report it. A null control difference is not evidence about whole-decoder revisions.
- If the question includes the *whole revision's code generation*, freeze two builds of the same harness, one in each revision, each containing its A and the same pinned B. Run **both builds on both hosts**, yielding A_old/B_old and A_new/B_new. This extra B control is necessary: comparing two revision binaries' A against only one B would confound layout/context with baseline. The three algorithm labels alone do not solve this.
- Both families use exact same corpus bytes. Pin one synthetic text artifact across all arms/hosts; never regenerate it separately from each checkout. Keep the approved scale/corpora, thread grid, paired geometric ratio, repeat/multiplicity policy and merge rule intact. Any proposed difference needs C6 approval before freeze.15 rounds and the judged pinned copy follow these diagnostics; contended copies remain separate.
- Within each host, pair arms by round and balance the frozen order; preserve decode-order walk/RLE1/CRC after each stage and exact actual-path counts. Match buffers and reset semantics. Archive symbol/code bytes and wrapper addresses to detect code-generation differences.
- Entry B/A is the main algorithm contrast. Front, B sort+fill versus A build, fill alone, fallback and accepted-block ratios are attribution. Report time in seconds/ns per output byte and accepted BWT row; do not compare nominal-GHz-scaled “cycles” across different CPUs as hardware cycles.
- Host/build interaction: compute within-host log(B/A_old), log(B/A_new), log(A_new/A_old). Compare these contrasts across hosts, alongside source and executable identity. Rounds are paired within a host, **not across independent VMs**. One VM per family identifies a difference between those environments, not a population-level CPU-family causal effect. No invented interaction significance or extra repetitions outside C6; a formal replicated-family design needs its frozen repeat/multiplicity decision.

### Dispatch reality

Current sweep.yml uses `runs-on: ubuntu-latest`. GitHub's [standard-runner specifications](https://docs.github.com/en/actions/reference/runners/github-hosted-runners) list OS/architecture/resources for this label, not a choice between EPYC9V74 and Xeon6973P-C. Pinning ubuntu-24.04 pins the image family, not that CPU model. A same-dispatch matrix cannot promise one of each.

Preferred single dispatch: two **already authorised** runner labels/groups whose hardware inventory actually guarantees the required models, serialize heavy work, and reject a mismatching runtime CPUID/model before measurements. No new machine, credentials or spending is authorised by this design.

If only the existing standard pool is available, freeze a finite recruitment plan before launching. Each job records hardware first; accept only the named eligible stratum, never based on readings. If a family is absent, report **that comparison missing**; do not poll/retry until a favourable family/result arrives. A runner-pinned pair or later approved recruitment can complete it. This is a hardware-availability limitation, not permission to invent labels or launch new infrastructure. The Coordinator owns registration/CI, Louis any workstation run.

### Which phase moved?

The old/new archival association is:

| Quantity | Old Xeon run | New EPYC run |
|---|---:|---:|
| B front / A front, paired ratio | 1.18465 | .940795 |
| B fill / A build, paired ratio | .942810 | .744284 |
| B sort / A build, paired ratio | .224064 | .235586 |
| A front, median ns/output B | .30720 | .29372 |
| B front, median ns/output B | .35896 | .27622 |
| A build, median ns/output B | .34092 | .36746 |
| B fill, median ns/output B | .31705 | .27341 |
| B sort, median ns/output B | .07546 | .08636 |

Both **relative front** and **relative fill/build** improve across these runs; sort does not become relatively cheaper. A build actually takes more nanoseconds per output byte in the newer run. This is more informative than the change in nominal c/B, but it is still host+harness/build-context association. No PMU was exposed. The answer to “which moves with CPU?” remains unmeasured until the crossed control; current evidence cannot allocate the effect uniquely.

## 5. Ranking and next move

1. **C29 production integration/evidence remains first.** It targets established front work and retains production's room consumer. Existing matched controls support it; this single-worker B comparison does not rank its critical-worker benefit against a pipelined C29 intervention.
2. **Crossed runner/build diagnostic above**, ahead of further D optimisation. High information value: identical kernel source, a large ratio reversal, and large same-body fallback regressions require resolving code generation/history before trusting a hardware-specific construction policy.
3. **Existing matched LS/direct-F transfer** retains its frozen question and primary/secondary declarations. Read its target-machine evidence under C5; do not silently swap B or rewrite old verdicts.
4. **C43 compact-index order**, only if counted/native cost fits the6.44/run allowance. Compare against B, not merely the losing D. B is the locality/control baseline; its2.37% Wikidata lead is too small and its regressions too large to qualify integration.
5. C37/C38 and content-copy directions remain lower priority. No new result here closes their prior gaps.

Expected D to remove sort cost; observed it adds.2707/output B over sort+fill and loses Q1. Gap closed: no production gain; a loss was localised to the compound placement phase, not causally decomposed. New understanding: the hot source is unchanged between the runner observations, and B's favourable Wikidata ratio conceals poor screens. Best next information comes from controlled code/host crossing; the adjacent construction must preserve locality without spending the saved ordering budget on gathers.

OBSERVER: replace “the construction evidently depends on CPU” with “B/A reversed across host+harness contexts; identical hot Rust sources narrow the baseline confound, but CPU causality is not isolated.” Replace “equal stores proves scattered writes caused D's loss” with the compound placement attribution above. Historical STOPs remain unchanged.

No implementation, compilation, timing, safety certification, merge, workload/acceptance change or new authority. Migration head/digest verified; early receipt and progress committed; routing audit has only five previously resolved historical findings. The unpinned PROTOCOL§11 references supply no additional authority.
