# Independent source review: compact30 construction-cost harness

**Review target:** `lmmx/mibz2` at `e264eed7e1a1c2f1d456d40207ff302811df2eae` (collector, analysis and listing checker), with `.just/bench.just` and archival journal at `ce11857550e265bb375589593aee12ac702b696b`.  
**Construction baseline:** frozen `e003a327cce7a6228316c8c8070a6c61a4110ab7`, whose `examples/c30walk/ctor.rs` and `src/` are not changed by the new harness.  
**Scope:** code-path and validation audit, not a timing run. No i9 execution, no repository modification. The reviewed Rust and analysis were accessed via the connected GitHub source. The full raw VM dry-run JSONL was not committed; its reported analysis and manifest were inspected, but the archive was not independently replayed.

**Disposition: CONDITIONAL PASS for the measurement implementation, HOLD on i9 capture authorization until the gate issues below are adjudicated.**

## What is sound in the implementation

1. **Timing boundaries**: `cbuild.rs:103–119, 173–195, 592–623` wraps the public `mibz2::bwt::build` and `Ctor::build::<false>` calls. Each includes its table's validation, clear, capacity growth, scatter, and finalization. Corpus decoding, oracle construction, correctness checks and walks stay outside the windows. The C wrapper has a handful more instructions than P's tail jump; this is disclosed, measurable in the listing and unlikely to matter on full blocks, but should not be called a wrapper-free comparison.
2. **Correctness**: `cbuild.rs:627–739` validates C's record and RAW arrays byte-for-byte against oracle images after *every* build, and validates P's table against a reference, itself checked in round −1 against the counting-scatter definition. Both walkers are checked in round −1. The one-pass precondition is recomputed from the last column, rather than inferred from equality of outputs (`cbuild.rs:1052–1071`; `src/bwt.rs:253–318`).
3. **Schedule**: `cbuild.rs:788–931` uses pinned threads, a spinning start barrier, completion barrier and checks outside the measured window. `c30_construction.py:129–239` reconstructs the expected W1 and W4 snake assignments, record set, clock ordering, per-thread placement and the inter-step/pass ordering.
4. **Analysis**: `c30_construction.py:76–85, 401–520` computes per-round C/P ratios for the full corpus; takes the geometric mean and 95% t interval from the 20 paired rounds; decides only from the warm four-worker b4/e4 bounds. It uses the fixed 1.06714 and 1.02 thresholds. Cold, W1 and subsets are diagnostic, not decision arms.
5. **Reference listing check**: `c30_build_listing.py:46–195` checks the five pre-existing timed-path symbols against the reviewed pin's normalized disassembly and both new wrappers against recorded reference listings, plus symbol-call structure. This demonstrates normalized instruction-path stability, **not bitwise identity**; RIP-relative offsets and relocation addresses are intentionally normalized.
6. **Documented VM dry run**: its committed `c30-analysis.json` reports all stage-1 checks and G3, G4, G6–G10 true, with 10,416 build records, the correct 434-block corpus and zero major faults. VM estimated peak memory 1,855,326,234 bytes versus VmHWM 1,766,506,496. No construction performance figures were issued. Because the raw JSONL was not committed, this review cannot independently replay that record set.

## Finding 1 — A1: bypass of `TVector`'s type boundary (approval-required)

**Location:** [`examples/c30walk/cbuild.rs:75–101`](https://github.com/lmmx/mibz2/blob/e264eed/examples/c30walk/cbuild.rs#L75-L101), used for P checks, P capacity reporting and one fault injection.

**Trigger:** any P check (`tv`) or the P injected-failure path (`tv_mut`).

**Mechanism:** `&TVector` / `&mut TVector` is cast through raw pointers and dereferenced as `&Vec<u32>` / `&mut Vec<u32>`. Rust `repr(Rust)` lacks a generic `repr(transparent)` layout/ABI contract. The implementation has one nonzero `Vec` field, and compile-time size/alignment equality strongly constrains its actual memory placement; the operation is *not* evidence of a demonstrated UB on the reviewed target. Nevertheless this relies on an internal representation not exposed by the library API, and the dynamic length test happens **after** constructing the reinterpreted reference.

**Consequence:** if the assumed interpretation ever ceases to hold, the check code could invoke undefined behavior or read invalid data, invalidating the run (even though the timed P builder itself is unchanged). It is not simply a reporting inconvenience.

**Recommendation:** require explicit guardian acceptance of A1 for this pinned research build, or make `TVector`'s layout contract explicit with `#[repr(transparent)]` on the one-field wrapper; alternatively, a read-only accessor plus capacity method. Any `src/` change entails a new frozen pin and revalidated disassembly. Avoid weakening exact P checks.

**Severity:** medium, approval-required; not proof of currently reachable UB.

## Finding 2 — G2 accepts an isolated badly staggered full-block step

**Location:** [`bench/c30_construction.py:401–421`](https://github.com/lmmx/mibz2/blob/e264eed/bench/c30_construction.py#L401-L421), [`434–436`](https://github.com/lmmx/mibz2/blob/e264eed/bench/c30_construction.py#L434-L436). The protocol states G2 as a **row-weighted measure per arm/regime/round across all-full steps**, but also says “in every all-full W4 step” (`docs/journal/2026-11-03-compact30-construction-protocol.md:346–350`). These formulations are not equivalent.

**Trigger:** one large start skew affecting a minority of all-full W4 steps, with other steps close to simultaneous.

**Consequence:** a serious lack of overlap on a full-block step may still satisfy G2, undermining the interpretation of that step as four-way contention and allowing a potentially distorted b4/e4 comparison to be accepted.

**Independent counterexample:** with 57 equally weighted full steps, one with start-skew fraction 1/3 and 56 with zero, the implemented weighted average is `(1/3)/57 = 0.005848`, **below 0.02**, even though one step has **33.3%** skew. The 61 analysis tests check uniform staggered starts, but do not cover a single outlier among many clean steps.

**Recommendation:** guardian must explicitly determine whether the registered criterion is truly an aggregate **average** or a per-step synchronization obligation. If the latter, add a worst-step or per-step violation check, test the outlier fixture, and amend the protocol *before* capturing. At minimum report both mean and maximum skew.

**Severity:** medium methodology/validation issue, **not** a claim of divergence from an intentionally aggregate protocol.

## Finding 3 — dry-run peak-RSS evidence can be bypassed outside the recipe

**Locations:** [`cbuild.rs:945–980, 996–1017`](https://github.com/lmmx/mibz2/blob/e264eed/examples/c30walk/cbuild.rs#L945-L1017); [`bench/c30_construction.py:278–302`](https://github.com/lmmx/mibz2/blob/e264eed/bench/c30_construction.py#L278-L302); [`.just/bench.just:346–356, 408–420`](https://github.com/lmmx/mibz2/blob/ce11857/.just/bench.just#L346-L420).

**Trigger:** call `c30walk cbuild ... --rounds 20 --cpus ...` without `--peak-rss`, then analyze with a positive `--identity-ok` flag, assuming the machine otherwise passes memory checks. The collector initializes `peak_rss = 0`; G9 checks `required >= estimate` and the margin, but never requires a nonzero peak argument or verifies its value. The official `just` *does* refuse a missing or nonpositive peak, but it accepts any positive number without proving it comes from a successful dry run on the same hardware and corpus.

**Consequence:** a capture can be marked formally valid without the prescribed actual-peak evidence. On the tested VM, actual peak was below estimate, so no underestimation was observed; the defect is in enforcement.

**Recommendation:** treat the registered dry-run result as an input artifact rather than a free-form number; verify its successful gates, corpus and pin, and machine/CPU identity, then require `required == max(estimate, verified_peak)` in capture validation. At minimum reject capture records with `peak_rss_arg <= 0`, missing field, or inconsistent `required`. Add tests of direct-binary bypass and a forged tiny positive argument.

**Severity:** medium validation gap; normal recipe path partially protected.

## Finding 4 — G7 poison flag overstates initialization coverage

**Locations:** [`cbuild.rs:421–430, 580–589, 672–694`](https://github.com/lmmx/mibz2/blob/e264eed/examples/c30walk/cbuild.rs#L421-L430) and [`cbuild.rs:672–694`](https://github.com/lmmx/mibz2/blob/e264eed/examples/c30walk/cbuild.rs#L672-L694).

**Trigger:** first round −1 C warm build (the fresh record vector has `capacity = 0`), or a block needing more than the existing capacity.

**Mechanism:** `poison` writes to **existing spare capacity** before the constructor runs. A fresh Vec has no spare capacity; a growing constructor replaces the allocation after poisoning. Nevertheless G7 always writes `"poisoned": true` for that block.

**Consequence:** the exact oracle comparison and walker checks remain effective, and the previously audited constructor has an independent initialization argument; but this G7 flag does **not** prove poisoning was applied to *every* built record.

**Recommendation:** report actual pre-build poisoned capacity/bytes and whether a growth allocation invalidated poisoning; don't assert universal poisoned coverage. If the guardian requires universal poison coverage, add a separate untimed correctness-only validation pass with preallocated poisoned capacity, rather than making measured builders artificially preallocate.

**Severity:** low–medium evidence/validation accuracy, no demonstrated bad output.

## Smaller issues and limitations

- **RSS file validation:** [`c30_construction.py:376–383`](https://github.com/lmmx/mibz2/blob/e264eed/bench/c30_construction.py#L376-L383) trusts supplied `rss` records' arm and growth, without checking three distinct P/C runs, matching corpus/CPUs, or `hwm_reset=true`. This affects only reported memory diagnostics, not the decision. The committed VM example does record successful high-watermark resets.
- **PSI and vmstat availability:** [`cbuild.rs:256–296`](https://github.com/lmmx/mibz2/blob/e264eed/examples/c30walk/cbuild.rs#L256-L296) treats unreadable/unparseable PSI identically to absent PSI, and defaults missing `pswpin/out` counters to zero. On a standard Linux machine these exist, but the memory gate should ideally distinguish “not present” from “could not read.”
- **Clock validation:** G5 accounts for estimated empty-window CPU overhead, but differential wrapper costs (C eight instructions versus P one branch) are included in the calls and not subtracted. This is transparent and likely negligible for the 230 full blocks. No timing-based claim has been drawn.
- **Cold allocation:** glibc may reuse freed pages; “cold” means fresh Rust arenas, not necessarily fresh physical mappings. Already disclosed and nondeciding.
- **Mixed step:** the full/short e4 diagnostic omits the one mixed step rather than reporting it separately, as the protocol originally proposed. This does not change whole-corpus b4/e4.
- **CPU topology:** the recipe chooses representatives from `thread_siblings_list`; if the lowest sibling is outside a restricted affinity set, pinning fails (rather than silently measuring the wrong CPU). Actual second-to-fifth core selection on the i9 remains untested.
- **Records:** six failure injections and 61+6 synthetic analysis/listing fixtures substantiate the VM test claims. The raw dry-run JSONL and complete tarball were not committed, so independent record-level replay is not possible from this repository snapshot. No Miri run of the new harness was reported; the earlier 942-case Miri result applies to the frozen constructor and excludes wide promotion.
- **Dry-run terminology:** even in `--dry-run`, `window()` obtains clock readings and records them; the analysis suppresses timing figures and any decision. Calling it “untimed” is operational shorthand for “no performance result consumed,” not absence of clock collection.

## Eight proposed amendments: disposition suggestions

| Amendment | Reviewer disposition |
|---|---|
| A1: `TVector` view | Explicit approval or safe layout/API contract required |
| A2: wrappers | Accept with both listings retained and overhead disclosed |
| A3: normalized code identity | Accept for this frozen build; do not call it bitwise equivalence |
| A4: extra memory/fault checks in dry run | Accept |
| A5: stronger P reference | Accept |
| A6: user-supplied peak | Repair enforcement/provenance before i9 capture |
| A7: omit mixed step from subset e4 | Accept only as a labelled diagnostic omission |
| A8: failure wording | Accept |

## Evidence and final authorization boundary

**Source-level review performed:** all 1,271 lines of `examples/c30walk/cbuild.rs`; all 558 lines of `bench/c30_construction.py`; all 199 lines of `bench/c30_build_listing.py`; the 61 analysis and six listing tests; `.just/bench.just` recipe; draft-3 protocol, 445-line journal; parent `examples/c30walk.rs`; production `src/bwt.rs` for P's semantics; archived dry-run analysis, console, listing metadata, RSS examples and manifest. The frozen constructor code was already independently audited, and the commit comparison shows no changes to it.

**Evidence not obtained:** actual i9 run (not authorised); actual 20-round capture; locally executing the pinned Rust binary; re-running the 61+6 tests using source files (archived console indicates they passed); independently replaying the full VM dry-run JSONL (not committed). The companion independent probes reproduce the G2/G9/G7 logical counterexamples without making measurements.

**Recommendation:** do not authorise the i9 capture yet. First resolve A1 explicitly and repair/clarify the G2/G9 validation obligations, keeping the exact constructor, walker, corpus and statistical thresholds unchanged. G7 coverage honesty and RSS-metadata checks should be corrected or explicitly qualified. Then freeze a harness pin and rerun its analysis tests, listing checks, failure injections and full untimed dry run on the chosen capture machine. No construction-timing conclusion is available or implied by this review.
