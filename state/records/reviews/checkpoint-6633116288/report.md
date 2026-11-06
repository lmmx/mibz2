# Optimizer audit: checkpoint comment 6633116288

Date: 2026-11-04. Trigger: https://github.com/lmmx/mibz2/pull/145#issuecomment-6633116288
Publication branch: review/compact30-instruction-ledger. This is a checkpoint review, not the fresh evidence-only §6.2 review and not implementation authority.

## Pins and scope

- Descriptor #151: 29dd071f8787a28560a7a081fd7ac0ef13991466; docs/journal/2026-11-04-compact30-descriptor-prototype.md and its evidence directory.
- U2 #146: 49cb46b65070059af49e906fa8d98693b808bda2; docs/journal/2026-11-04-compact30-scanner-prototype.md §10 and scanner-u1u2-trial evidence directory.
- F1/F2: a1d110c79520aff10e4ce6eb5ad8fb5cc3bbe51b; scanner-prototype.md §8 and scanner-prototype-erratum.
- Queue/charter/protocol read at e5d86cc; ROLES.md and REVIEWER.md read on research/compact30-constructor. User instructions control narrower publication, no-compilation and commit identity rules.
- Frozen ctor2.rs blob 168c2bac9ad0d8da4d9f82788f009d32f419608e and ctor3.rs blob 01f91419904a605b721dc1742ba8087fe79f74cf match at all three checkpoint trees.

Inspected full archived build/refill/finish listings, final prototype source, release consoles, corpora records, all nine Miri class consoles for each prototype, all six MU diagnostics, M1/M2 logs, class tables and per-function/library tables. Replayed class arithmetic, library-table equality for U2, frozen blob identities and the C6 wrapper trace. The complete input, manifest, script and output accompany this report. No constructor, candidate, benchmark, timing or compiler was run. Raw compressed Callgrind profiles were not replayed; table verification is not independent verification of those raw profiles. Toolchains recorded in the evidence are rustc 1.97.0 / LLVM 22.1.6 and Miri nightly 1.101.0.

## Gate results

| Checkpoint | Gate 1 | Gate 2 | Gate 3 / qualification |
|---|---|---|---|
| descriptor | PASS archived coverage; all 9 classes exit 0, both mutants caught | partial miss: four scalar copies 10 instructions, but non-row overhead +18.38M exceeds 5M | Numerical interval replay PASS against 40M; formal method CONDITIONAL (F3); not §6.2 approval |
| U2 | PASS archived coverage; all 9 classes exit 0, MU1–MU3 report intended faults | partial miss: tile 37+4X still misses registered target; 12/12/12/11 row loops retained | Numerical interval replay PASS against 60M; formal method CONDITIONAL (F3); not §6.2 approval |
| F1/F2 correction | Prior archived safety result unchanged | prior partial miss unchanged | Original whole-constructor interval withdrawn; own-body 56,195,817 stays scoped, qualification REJECT |

### Descriptor safety and accounting

Source ctor2d.rs:392–444 updates hot wr/en and cold state at the same open/close/refill transitions. Reading an assembled descriptor does not introduce a second authoritative copy. Bounds derive from the byte/0..256 index and existing portion invariant; null/empty descriptors cannot be advanced as live portions. The archived synthetic suite covers RAW ordering, headers, padding/walker restoration, span boundaries, malformed counts, maximum size and poisoned reuse. Miri covers the registered nine classes, not an unclaimed exhaustive proof. M1 fails the full-portion assertion; M2 fails RAW equality at an epoch.

Class-table replay: -76,589,766 +20,487,195 +1,173,383 -3,279,857 = -58,209,045 own instructions. Conservative lower saving: 76,589,766 -20,487,195 -1,173,383 -378,882 = 54,550,306. Inclusive upper observation: 62,277,795. Width 7,727,489 consists exactly of 3,279,857 uncredited negative classes plus 4,447,632 uncredited B2 initialisation-copy instructions. This is not allocator sampling error. Shared library tables agree; changed descriptor initialization is explicitly charged. Own writes rise 5,004,156 (build -193,756; refill +5,197,912); inclusive +712,330 includes the removed copy. Do not substitute inclusive traffic for the own-code trade.

### U2 safety and accounting

Final ctor2su.rs:653–687 has U2 only, at line 666. n1=min(16, wrapping_sub(30,p)) is always <=16; the chunk contract proves i+16<=ll.len(), hence i+n1<=i+16. The unchecked slice's safety does not depend on a debug assertion establishing p<30. Correctness of the first-segment do-while does require p<30, maintained by advance and the scanner's chunk progression. It reads the checked 16-byte window; tails retain checked slicing. The exact allocation controls distinguish range-order failure (MU1), U1 endpoint (MU2 on registered variant a only), and U2 endpoint (MU3); both default and no-UB-check modes fail at the intended operation. Do not claim MU2 exercised an operation in final U2-only source.

Class sum is -84,060,421; conservative subtraction is 88,402,421 -4,359,203 =84,043,218. The 17,203 width is exactly withheld negative classes. The 11 library call-site tables agree after removing addresses; caller instructions are included in build own counts. Own/inclusive reads fall 505,885 and writes rise 3,391,432. No economic inference follows.

### F1/F2 correction

Accept the correction's withdrawal and scope narrowing. At the correction pin §8 explicitly refuses a two-sample bound, reports the +1,840 snapshot memcpy delta, and lists library costs unresolved. New function tables reproduce own Ir 56,195,817, own Dr -3,389,475 and own Dw +1,995,539. Header-aware parser code handles positions/events and compressed positions; archives now contain Ir/Dr/Dw for two new runs. Historical run 2 is not reconstructed, as the correction acknowledges. This closes the misleading presentation in F1 and the supplied parser/archive deficiency in F2; it does not retroactively verify a whole-constructor net or recreate the missing old run.

## Findings and remedies

**F3, MEDIUM, formal-method condition:** descriptor §7.3 and U2 §10.4 explicitly use Callgrind-only frequencies for changed paths, despite charter §5.3b requiring counted-instantiation counters. Arithmetic least-favourable treatment is sensible but does not amend that requirement. Descriptor +1,173,383 and U2 +4,359,203 are charged, so no numerical gate crossing is demonstrated. Add the missing counted-only counters, prove the uncounted listing unchanged, and replay the path ledger. Until then retain numerical PASS separately from formal qualification; do not silently certify the coordinator's word “qualifies.”

**F4, MEDIUM, overlap unresolved:** the narrow descriptor-LEA and scanner-pair target sets are disjoint. The complete ledgers include more. Both tables credit S=2,393,313 at 38→37, and have negative classes with frequencies 3,575, 106 and 79. These are candidates requiring instruction correspondence, not proof of shared eliminated instructions. Thus “no intersection” cannot be inferred merely from target-site disjointness. Outer iterations (13→17 versus 13→11), refill caller detours and fill growth also interact. Map original B2 instructions once, attach frequencies, and keep integration changes separate. No proven numerical O discount is booked now; no isolated sums become a verified combined result.

**F5, LOW, reporting:** preserve +5,004,156 descriptor own writes and +3,391,432 U2 writes alongside instruction wins. These are data-access counts, not bytes or cycles. No timing conclusion is warranted.

## C6 against compiled descriptor refill

Non-independent proposal assessment: this thread originated C6. Original proof and boundary obligations remain in proposals/refill-boundary/proposal.md. This adds no authority or implementation.

The nonempty, successor-ordinary wrapper in descriptor listing is 69 instructions, including the finish call and excluding its body, versus B2's 62. The replay trace follows 0x94330–0x94442, skipping successor-RAW instructions 0x943f4/0x943f9/0x943fd. Old-portion crossing/wide effects remain inside the unchanged finish body. Relevant anchors:
- finish call 0x94362 must execute exactly once before replacing descriptor state;
- transient resets 0x94374, 0x9437c, 0x94385;
- rank division/address work 0x9439f–0x943e3;
- bitmap test 0x943e6–0x943f2;
- duplicated r0 stores 0x9436f and 0x94422.
There are five pushes and four pops plus stack adjustment; eliminating any is a compiler target, not a promised saving.

For the prior exposure floor E_min=1,084,913 and P=1,795,482, a proposed whole eligible wrapper cap C<=40 and failed-guard increment g<=5 imply incremental descriptor-relative saving >=(69-C)E_min-g(P-E_min)-H_D =27,909,632-H_D. This uses the prior counter-derived exposure floor, not a newly observed eligibility counter. B2-relative C6 stays 20,315,241-H. The difference 7*E_min=7,594,391 recovers descriptor wrapper overhead on those transitions; it is not an additional independent portfolio credit. Never add the two C6 targets, and never add C6 to a combined result already removing those instructions.

H_D must charge mode storage/propagation, retained pointer availability across finish, spills, callers, initial/empty/exhausted refills, failed guards, setup, and every other changed path, including interaction with U2. Retain natural-mode and nonempty guards; successor room b+30<=lim; same-allocation pointer provenance; old end+4 and old end+64 byte advances; current tag if later composed with v3; forced-mode fallback; no second finish after fallback. These are proposal obligations only.

No positive verified net while H_D is unresolved. Required falsifiers: full wrapper including guards exceeds cap, ineligible overhead exceeds cap, repeated finish, stale epoch tag, forced RAW bypass, boundary pointer violation, or output/RAW/walker mismatch. Fresh independent audit remains required for any implementation and §6.2.
