# Scanner safety reconstruction and U1–U3 proof obligations

Evidence pin: `7f74e9f30e31af33b0d66fec3fe89af1758307d9`. Source-first reconstruction used `examples/c30walk/ctor2s.rs`, its diff against frozen v2, and raw logs before reading the checkpoint narrative or PR discussion. `source-first-note.txt` records that boundary. The current charter's safe-only narrowing was then checked in the supplied `ACTIVE-CHARTER.md`. This is an advisory evidence assessment; it grants no implementation or scope authority.

## Dispositions

| Item | Disposition | Meaning |
|---|---|---|
| Existing safe scanner, registered safety coverage | PASS on inspected evidence | No new safety counterexample found. This independently corroborates the existing checkpoint; it is not a novel finding. |
| U1/U2 source-range proof | CONDITIONAL | The proof holds with a dominating full-tile contract and exact endpoints; future implementation and targeted evidence are absent from this pin. |
| U1/U2 registered `n1=17` negative control | CONDITIONAL operation coverage | The combined mutant may correctly fail at U2; that does not separately exercise U1's parent-slice bound. |
| U3 phase induction | CONDITIONAL mathematical argument | All current call sites preserve `p<30`; `assert_unchecked` creates a new UB obligation and needs exact future source/evidence. |
| Future U1/U2/U3 implementation | Unreviewed | Current Miri logs cover safe code, not any future unchecked variant. No authorization is inferred. |

These dispositions do not overturn any already recorded gate result or claim that current safe code has UB. Discussion/status after this immutable pin belongs in the parent review's cursor, separately from the proof below.

## Current source, independently reconstructed

At `ctor2s.rs:877–933`, an epoch is bounded by `end=min(n,i+61440)` and its unread source is `rest=&ll[i..end]`. At each full-tile iteration let `j=end-rest.len()`. The induction is:

1. `rest` is exactly `ll[j..end]`; `0<=j<=end<=n`.
2. `split_first_chunk::<16>()` gives a full initialized tile iff `rest.len()>=16`, establishing `j+16<=end` without relying on destination counts.
3. If a previous lookahead failed, `held` contains the halves of that very first unread tile. The failed lookahead does not advance `r2`, and `rest=r2` preserves its position. Taking `held` consumes the cached observation, not the source tile.
4. A uniform run consumes the first tile and only subsequent matching full tiles. `rest.len()-r2.len()` is a positive multiple of 16, and equals its exact row count.
5. A nonuniform tile calls the inherited `tile(ll,j,16)` and then sets `rest=after`; the final tail has length `1..15` and is passed separately.

Thus each source row enters the inherited emitter once, in source order, and each epoch boundary remains an emitter boundary. Uniformity uses two equal qwords plus equality with an eight-bit rotation (`ctor2s.rs:900`), which is equivalent to all sixteen bytes being equal. Byte alignment does not add an obligation: `halves` uses safe byte arrays and `from_ne_bytes` (`:109–118`).

The new code does not weaken inherited output-pointer bounds. An append refills before a write at the portion end (`:584–595`); a run checks its entire remaining bucket allowance before emission (`:698–705`). Wrong histograms can terminate processing, but cannot invalidate the source-suffix invariant. Early failure leaves `rec` length zero, clears `raw`, and never publishes partially initialized records (`:750–756`, `:810–838`). The constructor initializes `p=0,y=0` and clears snapshots/bitmap on each successful validation path (`:775–805`), so stale reusable storage does not supply phase or held state.

This review does not reprove every inherited destination pointer operation from scratch. The changed scanner calls the same v2 operations under the same row partition; those existing obligations remain prerequisites. Frozen-file identity must separately be checked against the historical pins, not inferred merely from similar text.

## U1/U2: range, initialization, provenance and integer bounds

The proposed operations are checkpoint §7 (`2026-11-04-compact30-scanner-prototype.md:402–412`): a first unchecked source segment `[j,j+n1)` and a second `[j+n1,j+16)`, where `n1=min(16,30-p)`, only for a full nonuniform tile.

From the full-tile contract `j+16<=end<=ll.len()`, and `0<=n1<=16`, both ranges are ordered subranges of the original live immutable byte slice. All bytes are initialized because `ll` is a valid `&[u8]`. There is no alignment obligation for byte slices, no requirement for a padded allocation, and no permission to read beyond a slice merely because allocation capacity is greater. Empty second segments at `n1=16` are valid ranges; they must never be dereferenced as nonempty buffers. The source slice remains live, and the output writes are through independently borrowed output vectors; cached qwords are values, not dangling pointers.

At this source pin `n<=MAX_N=983010` (`:74–80`, `:752–756`), so `j+16`, `j+n1` and the epoch bound cannot overflow x86-64 `usize`. More directly, the mathematical endpoint inequalities themselves establish every accessed offset is no greater than `n`. This must be connected to the exact expressions in a future implementation; an unchecked subtraction is not justified merely by asserting a later endpoint bound.

U1/U2's memory-range proof does not need U3: `min(16, wrapping_sub(30,p))` is at most 16 even for out-of-domain phase, although such phase could break semantic labels. The current source's `p<30` proof avoids subtraction wrapping altogether. The two questions should stay separate; avoiding source bounds checks must not silently add a phase assumption or unsafe hint.

**Tail counterexample.** With `ll.len()=1,j=0,p=0`, the full-tile formula gives `n1=16`; applying U1 to the one-byte tail makes `[0,16)` invalid. Consequently the phrase "in `tile`" is insufficient as an implementation contract: that helper serves full tiles and tails (`:926`, `:932`). A future exact formulation must show a dominating `len==16` branch/full-tile type or a separate full-tile call path and keep the tail checked.

**Negative-control coverage.** For an interior tile, `ll.len()=32,j=0,n1=17`, U1's `[0,17)` is valid relative to `ll`; it merely consumes one byte from the next tile. U2's `[17,16)` is reversed, or `16-n1` underflows in a length expression. A Miri failure there tests U2/range ordering, not U1. The original journal promises a combined mutant failure, not explicitly a failure at U1, so this example does not refute that promise. It shows why a combined failure must not be reported as separate coverage of each operation's bound. An inherited emitter rejection in the first segment can also prevent execution reaching U2. Separate controls strengthen the proposed validation; this is not a current-code memory-safety defect.

Concrete falsifiable controls, as recommendations for any separately authorized design:

- U1: a nonuniform final full tile whose endpoint equals `ll.len()`; change its first range endpoint to `ll.len()+1`. A one-tile input of length 16 at `p=0` isolates this operation.
- U2: `ll.len()=32,j=16,p=16,n1=14`, with a uniform first tile and nonuniform final tile. Preserve first segment `[16,30)` and change second `[30,32)` to `[30,33)`. This isolates a nonempty second segment and crosses the actual parent-slice endpoint.
- Retain positive cases for empty second segments, odd allocation offsets, held transitions, epochs and all tail lengths. Require a diagnostic identifying the intended operation, not merely a nonzero process exit or oracle mismatch.

These suggested controls are a research/validation contribution. The author should not serve as the fresh evidence-only formal reviewer of a future implementation that adopts them without disclosing this contribution.

## U3: complete phase induction, including the omitted bulk case

The current `advance(k)` at `ctor2s.rs:341–350` uses one subtraction, not a general modulus:

`p' = p+k` if `p+k<30`, otherwise `p'=p+k-30`.

The sufficient uniform preconditions `0<=p<30` and `0<=k<=30` ensure `p+k<=59`, preserving `0<=p'<30`. More generally the single subtraction works when `p+k<60`; `k<=30` is sufficient for every valid p, not necessary for each individual p. Some bound cannot be omitted: `p=29,k=31` produces `p'=30` after one subtraction. This is a counterexample to a weakened proof, not a reachable current execution.

The current exhaustive call-site argument is:

- `tile` advances 16 for a full tile (`:622–623`), and 1 per tail row (`:624–626`).
- `fill` advances `m` (`:644`), where the only caller `run` chooses `k=min(room,m)` (`:712–716`). `room=(end-write)/2` is bounded by the open portion's capacity, at most 30 (`open`, `:390–410`; appends maintain monotone pointers within it). Thus `fill` receives at most 30.
- `complete` emits whole groups of 30 rows and changes `y` by 32 per group but deliberately leaves `p` unchanged (`:653–692`). Because `30*whole mod 30=0`, this preserves phase. It is a distinct source-position change and should be named explicitly in the induction.
- Each build starts `p=0` (`:804`). A source epoch is 61440 rows, divisible by 30, so an epoch boundary preserves/reset-independent phase zero. Snapshots, descriptor refill, finishing, RAW canonicalization and rejection do not assign `p`.

Therefore U3's premise is true for all current reachable `tile` entries, including tails and valid prefixes of malformed histograms, assuming the inherited portion-pointer invariant. `assert_unchecked(self.p<30)` is nevertheless a new unsafe proof obligation, not a runtime check: if a later change violates any premise, the behavior is undefined. The exact future statement, call graph and mutation evidence need their own pin. The existing safe scanner's Miri evidence does not execute this hypothetical intrinsic.

## Reproducible evidence inventory

Run `python3 replay_safety.py PATH_TO_PINNED_CHECKOUT` to regenerate `results.json`. The script reads only saved evidence and exercises a finite abstract tile-partition model; it neither implements nor runs a constructor.

- Eight saved Miri classes complete with `exit 0`: 105 cases, 198 array comparisons, 297 walker outputs, 198 restorations. Toolchain is the archived `1.101.0-nightly (16d1ffac1 2026-10-26)`; it is distinct from the release assembly toolchain.
- `malformed`'s zero reported array/walker counts are intentional: source `:1586–1612` checks rejection and both vectors empty directly for both instantiations. Its statistics are not added to the successful-case accumulator. Zero scanner counters in that log do not establish that no scanner prefix executed.
- The 44 corpora JSONL records total 6052 block/configuration evaluations, 12104 array comparisons and 18156 walker checks. The 200 adversarial records total 528 block/configuration evaluations, 1056 arrays and 1584 walks. These totals span four configurations and must not be confused with unique physical blocks or the primary-corpus performance denominator.
- Full synthetic console records 54029 cases, including 30 changing-size/reuse cases, 4002 malformed/size cases, span endpoints 65533/65534 and 31302 multi-epoch portions. It reports 99966 array comparisons, 149949 walks, 99966 restorations, exit 0.
- M1 and M2 are detected as valid-input rejection with exit 101. The artifact supports detection of their intended semantic faults; it does not claim that these old mutants test U1/U2 or undefined behavior.
- The finite scanner model checks 436880 tile/epoch/tail configurations against an independently grouped reference partition. The `advance` model checks all 930 pairs `p in 0..29,k in 0..30`. These are bounded checks supporting the written inductions, not universal proof by test or new Miri evidence.

No benchmark, perf counter, constructor compilation, new runtime capture, source change or experimental-protocol change was performed.
