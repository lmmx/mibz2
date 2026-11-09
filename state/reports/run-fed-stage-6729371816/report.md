# #269 run-fed stage registration: REVISE before freezing

Assignment **6729371816**, dispatching **6729283573**. Reviewed head
**187c0ba517c3324aa28c5e7ccd2667f6c0572145**. Instruction pin
**a4f97714cc14366984015869d9d094d89488ac57**. No workflow dispatched, decoder executed,
timing collected, source implementation changed, or authority expanded.

**REVISE.** The SIMD emitter and matched-input diagnostic are worthwhile. Four matters
prevent consent to freeze: an unchecked safe API into unsafe fill; a baseline cache/buffer
mismatch and uncharged stage work; incomplete result-admission evidence; and causal/host
claims beyond the measurements. These are corrections to the proposed diagnostic, not a
reason to close line A on its instruction estimate.

## 1. Concrete safety finding; the diagnostic exemption is unsound

`src/run_stage.rs:101` exports safe `fill(recs, s, n, tt, k)`. Its AVX2 branch calls
`fill_avx2` without validating that `sort` succeeded for those records, scratch and n.
`Scratch.fo` is publicly mutable. At line 159 the unsafe implementation dereferences
`fo.add(0)` on the premise that checked tilings exist. Safe callers need not satisfy that
premise. On an AVX2 host, this uses no unsafe caller operation:

```rust
let mut tt = mibz2::bwt::TVector::new();
mibz2::run_stage::fill(
    &[], &mibz2::run_stage::Scratch::default(), 1,
    &mut tt, mibz2::Kernels::detect(),
);
```

The loop enters because `0 < 1` and reads an empty F-order vector through a raw pointer.
That is a concrete invalid-read path, derived from source, **not an executed Miri result**.
The scalar route can also manufacture a TVector with out-of-range links from a forged
public scratch, violating the invariant that downstream unchecked walks rely on.

The diagnostic's normal path calls `sort`, asserts true, and holds immutable borrows of
the same inputs during fill. That path has the missing premise. The public safe API does
not. Preserve the premise structurally: checked private state, validation, or an explicitly
unsafe internal interface with justified call sites. Do not merely add another prose
precondition to the safe public function. Include negative API/state tests and source-pinned
evidence reaching the corrected unsafe path before freezing its measured implementation.

**Guardian/Coordinator: B6 applies to the exposed unchecked fill/TVector construction
boundary.** Hold use/integration of that interface pending correction; continue remediation
and unrelated work. This is not a claim that the current valid Wikidata call crashed, nor
a STOP on run-fed research. The draft's blanket claim that independent unsafe review can
wait until production because table comparisons run first is not licensed by B5. Such
comparisons cannot repair UB incurred before they compare. This review supplies a concrete
adversarial objection, not a blanket safety certificate.

The differential, exhaustive, random, malformed-stream and two mutation checks are useful
reported evidence. Latest head CI includes successful Rust and Miri checks. The checked-in
Miri command is `cargo miri test --lib -- miri_`, without the explicit AVX2 flags quoted for
the local run; neither that green name nor this static read certifies AVX2-emitter coverage.
The draft itself excludes the emitter from Miri. No reached-path fuzz record was established
here. Record these limits without inventing a diagnostic-wide exemption.

## 2. Fair source loops; incomplete production-shaped measurement

**What agrees.** A and B consume the same scanned `Symbols`. The AVX2 MTF twins retain
the same list registers, `second`, STEPS, count updates and bounds/error checks; changed
record allocation, record indexing and stores are part of the intended intervention.
This is not scalar against SIMD. The checkpointed C kernel is separately measured and
correctly acknowledged as structurally different. Arm order rotates, each arm keeps its
own buffers, and each block is followed by walk/RLE1/CRC. Front, sort, fill and fallback
are separately timed over the same accepted-block partition.

**The baseline buffer mismatch matters precisely because cache conditions matter.**
`stage()` always walks into `x.walked`, including A, leaving `x.ll` separate. Production's
`Segmented::invert_in_place` (`src/bwt.rs:116–119`) writes the walk over `ll`. #266's ledger
explicitly depends on this reuse. Therefore A has an extra live/touched buffer and a
different state before the next MTF. Equal synthetic buffer policy across arms is not
the same as the incumbent's buffer policy. Reproduce the incumbent reuse in A, or limit
Q1 to this out-of-place harness and stop calling it the production one-worker stage.
Measure/charge the candidate's real additional record/sort storage rather than hiding it
by making the baseline use the candidate's buffer shape.

**R_stage is a sum of timed regions, not the whole entry point.** `takes_runs` is computed
outside the timer; B's sort-result assertion and inter-phase dispatch are outside too.
A takes two clock brackets on accepted blocks, B three. On fallback blocks A and B execute
the same timed MTF/build body while the candidate's selector is excluded. Thus R_fb near
1 cannot certify that the real fallback costs zero or less than 2%. A whole-entry bracket
covering actual dispatch/selection/checks is needed for the whole-stage/fallback claim,
with attribution treated separately. Clock-bracket asymmetry should be quantified or
removed, particularly for the short synthetic cases. This does not authorise a change to
the registered estimand/repeat policy; describe any revised measurement before freezing.

**Decode-order is a controlled approximation, not identical cache contents.** First blocks
follow another arm/validation pass; later rounds reuse the same file and buffers. For
small corpora, their working sets can remain cached. Scanning is deliberately removed,
and its cache effects go with it. Keep these limits visible; no cache flush is demanded.

**Fallback attribution is currently wrong.** The decision scans each synthetic corpus's
whole R_stage, not its R_fb. Example: accepted blocks A=1, B=1.20, fallback A=B=1 gives
R_stage=1.10 and R_fb=1.00. It prints “fallback costs more than 2%” although all the loss
is on the selected record path. Retain the whole-corpus screen if that is the intended
rule, but name it correctly and report path counts/R_fb separately. The short-block
bound-blind selector is exactly where this distinction matters.

## 3. Result admission: different from #258, still incomplete

There is **no executable VALID verdict** in this diagnostic. It prints readings explicitly
UNJUDGED unless this is the registered run; the freeze/head/AVX2 checks are presently an
admission procedure, not a report verifier. Do not claim it inherits #258's JSON ingestion
bug: there is no external null hash or missing wall field to accept. It actually compares
table words, counts, maximal records and full first-file output before measurement, and
checks every block's CRC on timed passes. One binary supplies the baseline and candidate.
`Instant` supplies nonnegative durations; nonfinite log ratios yield None.

But None is then silently omitted from the six-corpus maximum (`filter_map`, lines 465–469).
If required synthetic intervals are absent, the remaining ones can pass; if all are absent,
the fold's initial worst=0 passes. The executable-transcription probe demonstrates this
decision predicate, **not a failed Rust run**. Require all six registered cases and all
required intervals to exist, with finite positive timing totals. Zero phase totals for
genuinely absent paths may be N/A; zero/missing required total-stage evidence may not pass.

Before freeze, specify an admission record containing full 40-character source identity,
exact rounds=15, AVX2 selection, primary affinity/host, corpus digests and sizes/block/path
counts, completed correctness status, and raw per-round arm/phase timings. The current
workflow prints only short HEAD, and only human-formatted medians/intervals survive—there
is no independent reconstruction of pairing/CI from those summaries. The binary accepts
any first file, not just the registered Wikidata SHA. Fixed workflow construction helps,
but record and verify the identity rather than relying on a filename. The text synthetic
corpus reads repository source directories at runtime, so its bytes are also source- and
workspace-dependent. Pin/hash them without changing their intended workload.

`wait` without individual PIDs in the contended workflow does not propagate a failed child.
A small shell-only reproduction returns success after `false & true & wait`. Collect each
exit status and mark partial contended results incomplete. They remain information-only;
that defect must not silently turn partial output into a complete stress result.

A correctness assertion is not an infrastructure void eligible for an unchanged-commit
blind retry: apply B6, investigate, repair and identify the new evidence/registration.
Transient build/runner failure is a different retry class. Keep the one-dispatch bound and
do not use local/CI outcomes to choose extra repeats.

## 4. Thresholds after two local rounds: preserve discovery/confirmation separation

Disclosure is good; “indicative” does not mean the author did not see the data. The local
two rounds are discovery data. Freeze all rules before new CI observations and keep those
rounds out of the new estimate. I do **not** recommend tightening 0.90 or 0.95 merely because
the local result looked pessimistic. That would be another data-dependent choice, not a
repair for the first one.

- Q1's pre-existing point threshold ≤0.90 with hi<1 is a reasonable *prototype screen*.
  It establishes evidence of some saving and a ten-percent point estimate, not a confident
  lower bound of ten-percent saving. Q1 lo>0.95 excludes ≥5% saving only for the tested
  implementation/workload/host condition, under its interval assumptions.
- The synthetic condition chosen after seeing wide intervals is only a point-estimate
  screen. A mean 1.01 with interval [0.80,1.30] passes; it is not evidence that no corpus
  regresses by more than 2%. Either retain it with that explicit limitation, or seek Louis's
  decision for a pre-specified upper-bound/multiplicity rule if noninferiority is required.
  I do not silently impose that C6 change. Missing cases must fail admission either way.
- Q2 and M1–M3 are information-only, discovery-derived hypotheses. Report estimates and
  intervals; do not let their post-hoc labels become proof of a mechanism or permission for
  a different statistical family. A useful lead can justify preparing a separate experiment.
- “STOP, INCONCLUSIVE” is an operational pause, not evidence of no useful effect. Preserve
  the uncertainty when recording the line's status.

## 5. The proposed mechanism readings do not identify the mechanism

M1 is a pre-touch intervention, not a verified cold-versus-resident comparison. A's table
may already be resident after the preceding walk; D's writes also disturb caches and
write buffers. R_warm≈1 can mean both are warm, so it cannot establish that write misses
never bind. A difference does not uniquely attribute its cause to write misses either.

M2's measured floor is a store-only reference, not a universal lower bound. Production
overlaps stores with other work and has a different write order. B also writes overlapping
vectors at cell ends: for lengths l_i, it emits `8*sum(ceil(l_i/8))` u32 stores, between n
and `n+7c`, despite a final table of only n words. Three one-row cells emit 24 slots for a
three-word table. Equal final 4n-byte tables do not imply equal store work or traffic.
With the old census c/n≈0.088, the general upper bound is about 1.616n stored slots; this is
a bound, not a measurement of the new stage. The fat-LTO release should also demonstrate
that floor stores survive optimisation: the next build overwrites them, and no observer
requires their values. I have not inspected this binary's assembly; elimination is a risk,
not an observed fact.

M3's printed “cycles a cell” is `median wall seconds * GHZ * 1e9 / cells`, **not measured
PMU cycles**. GHZ is one `/proc/cpuinfo` sample in the workflow (default 2.8 otherwise).
Frequency changes and time off-CPU break a load-latency interpretation. PMU cycle counts
are printed separately, with coverage. Use those when valid, or label the figure nominal
frequency-scaled time and leave the hardware mechanism unresolved. Similar cycles/cell to
a load latency does not prove the iterator chain binds; require a dependency-changing
contrast or a demonstrated critical path for that stronger claim. M1≥.95 plus M2<.80
therefore does not logically falsify all write-traffic-bound explanations.

The inherited instruction prior needs the same care. #266 explicitly labels 2.6 versus
2.5 instructions/row a source-shape model, not compiled counts. Their ratio is 0.9615, and
the uncertainty is unspecified. It cannot rule out an instruction win, much less a cycle
win. Same table size rules out shrinking that table, not improvements in store ordering,
front traffic or cache behaviour. Keep the model as motivation for measurement, not a
closed theorem resurrected as an admission gate.

## 6. What CI can answer; when to prepare an i9 run

A corrected, frozen CI run answers whether this one-thread stage is cheaper on the actual
recorded runner and corpus. `ubuntu-latest` does not guarantee a Xeon model; inspect the
actual CPU. Many independent copies stress shared resources but are not the pipelined
team. Q2 times only a leader's serial front, not its overlapped team critical path. Neither
condition proves whole-decoder improvement or the effect on Louis's i9.

**Recommended conditional next step:** after safety and measurement corrections, take the
bounded CI screen. If Q1 is FEASIBLE or INCONCLUSIVE, or Q2 offers a meaningful lead, prepare
a separate frozen i9 diagnostic with matched production buffer lifetime and an explicit
target question, then offer the usual C5 route for Louis to run. No run is authorised here.
If both CI leads are convincingly negative, pausing this implementation can be a rational
compute decision; record “negative on this runner,” not “line A disproved on i9.” If the
question must be answered empirically **on i9 even after negative CI**, an i9 run is still
needed. Conditioning confirmation on positive CI is economical, not a proof of cross-host
transportability. Production qualification remains a separate end-to-end comparison.

Expected: pessimistic instruction model and local stage ratios about 1.0–1.1. Observed in
this review: source-level safety/admission/fairness counterexamples, no new timing. The
performance gap remains unmeasured on the target. Learned: the present diagnostic does not
yet separate its proposed mechanisms or reproduce the incumbent buffer state. Its next
value is a corrected, bounded stage screen, not another model-based global rejection.

## Evidence and limits

Read the registration and #266 census at the reviewed head, diagnostic, PMU support,
run_stage, MTF/BWT source, test corpora and MTF tests, Cargo configuration and sweep/Miri
workflows; test/lib diffs; current PR/check metadata; the pinned common/role instructions,
inboxes/channel and migration manifest. Latest checks: 23 success, three skipped
(Release, Publish crate, deploy-branch). No rustc/cargo was available in this executor;
no native tests, Miri, disassembly or timing were run here. Reported local tests remain
attributed; successful CI names do not expand their reached-path coverage.

`probes.py`/`probes.json` retain arithmetic and a faithful transcription of the relevant
decision predicate; they are not the Rust binary's output. `evidence.json` lists source
and check identities. Pinned `coord.py audit` found only five resolved historical routing
tripwires; route returned #145. Findings go on #145 for Coordinator action, including the
concrete fill-boundary objection for the Guardian. Persistent task unchanged.
