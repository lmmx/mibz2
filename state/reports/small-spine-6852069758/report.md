# PR379: all-small-spine B5 delta review

Assignment: PR162 comment 6852069758. Policy pin: a2da49638ac54d7763de1c167bd054f67ffdc763. Reviewed 2026-11-13.
Head: 835984e849bd4200c0e74b76fb67a336cd82ae2c.
Isolated base: be25366f46458e2f1dbb0bdac1f5fbc3366d0648.

## Disposition

The omitted-operation argument is sound for the existing Cells constructor contract; I find no new unsafe access or byte-correctness defect in this delta. **The independent source review is complete; B5 evidence closure remains PENDING**, principally candidate Miri completion and inspectable mutation results. This is not a merge, performance, default, memory, or release qualification. The observed Rust CI failure also needs Coordinator incident handling.

This conclusion is narrower than a fresh proof of the whole induction. It relies on the existing valid acyclic grammar, one marked child per marked rule, stored-byte representation for unmarked factors of length <=16, and original bounds/nonwrapping factor-placement invariants. It does not manufacture a MONODROMY disposition. The original F01 withdrawal (6790022921) remains intact.

Independence: I previously supplied the F01 adversarial witness/review and later withdrew that objection against corrected d4f0e8f. I also proposed adjacent output-order grammar emission in REVIEW6850440971. I did not implement this guard or its tests. That earlier general-emitter proposal is a different construction; no competing implementation is made here.

## Re-derived omission proof

Let S be the unmarked siblings encountered on the marked path from the returned word to its marked byte. Each iteration of the original origin walk visits exactly one of them. If a carries MARK, the sibling is b and the next word is a; otherwise the sibling is a and the next word is b. The candidate uses exactly these two cases, conjoining the unsigned low-word comparison <= SMALL (16). It does not clear MARK on the plain sibling, truncate a length to a byte, inspect only the final sibling, or infer eligibility from a corpus name.

The emission walk uses the same root, same immutable rule buffer, same child-selection condition and same next pointers. Nothing between walks mutates the grammar. Therefore all_small implies that every emission iteration takes copy_small, and none reaches pos.add, its dereference, or seed minimum. The zero-rule case is also covered: the conjunction is true, both walks terminate immediately, and the retained marked-byte store writes the period's byte.

Now consider the old writer with its freshly NIL-filled position vector. Under that implication it creates no seeds. The descending scan sees NIL in every slot and writes nothing; by induction over its iterations there can be no propagated position. The ascending scan likewise sees only NIL and emits nothing. The third marked walk tests precisely the same sibling lengths for >16, so emits no long recopy. Omitting all three is observationally equivalent for output. This is the actual omission proof; absence of a new pointer instruction alone would not justify it.

The candidate still forms self.pos.as_mut_ptr() for eligible blocks. An empty vector's pointer is not dereferenced or offset on this route; forming it alone does not require an allocated element. With retained non-NIL positions, the proof is deliberately different from saying the actual vector contains NIL: the old counterfactual would clear it, whereas the candidate leaves it stale and never accesses its elements. No subsequent small-block operation depends on its contents or length.

For a false guard, the original capacity/resize/fill code runs before the first position access. Thus long→small→long resets the entire active nr range before reuse, regardless of intervening small blocks or previously retained capacity. Moving initialization after the origin walk introduces no earlier position read: that walk accesses the rule buffer and scalar origin only. Allocation failure/timing can differ, as intended; successful byte/error behavior is the reviewed contract.

## Preserved operations and boundaries

A reproducible whitespace-normalized source comparison passes for position reset, emission/seed/marked-byte statements, descending and ascending passes, F01 recopy, repetition/truncation, and the production source preceding write_cells (ignoring the two added test-only fields). See source-check.json and check.py. This is a structural check combined with the control-flow proof, not an executable Rust proof.

- Both marked sides retain their original j and q updates. start=(j+SHIFT)%p and every destination computation are unchanged. Eligibility is not a phase exemption.
- Length16 is eligible; length17 is not. No additional copy_small call or destination is introduced. Its original overcopy allowance, arena padding and output n+32 resize remain necessary and unchanged.
- The marked-byte store, short-cycle period repetition, and final truncate(n) remain executed on both routes. Skipping the third walk cannot remove a short-factor write because that walk only copies long factors.
- On the long route the original seed minima, reverse propagation, forward expansion and final recopy remain in their original order. F01 repeated long siblings still require the final copy; the guard does not claim otherwise.
- try_invert still rejects invalid records, invalid primary/period and budget/floor/density failures before write_cells. No changed refusal path clears or writes output. Retained position state cannot alter admission or fallback, whose production code is unchanged.
- The all-small claim is about spine siblings, not every rule in the grammar. Long off-spine rules with no seed do not invalidate the omission proof; requiring global grammar shortness would be unnecessarily strong.

## Tests inspected, and what they establish

The new boundary/reuse witness uses columns [1,0^16] and [1,0^17], row1, asserts maximum plain lengths16/17 and opposite guard outcomes, and confirms a fresh eligible call leaves position capacity zero. Its helper poisons output with the complement of the oracle bytes and checks equality before reach assertions. It retains positions from a long block, runs the small boundary plus bbaaa and a one-symbol short-cycle case, checks zero seed/scan/recopy counters and unchanged positions/capacity, then requires long passes and F01 recopy on another long witness.

Existing tests cover arbitrary binary columns with every origin through n10, exact-fit/refusal reuse, malformed records, rotated long witnesses, and production-admitted repeated F01 with repetition and smaller-buffer reuse. Source inspection is not a claim to have executed Rust here: rustc, cargo and just are absent.

The guard's two marked-side cases are proved directly above. The new test does not separately report a marked-side coverage matrix. A guard-false-positive mutation (force eligibility or omit one side's comparison) would be a useful focused extra diagnostic, but I do not invent it as an already-run test or replace the existing B5 obligations with it.

The new mutation driver removes the actual short-copy or F01-copy statement while retaining counters, compiles first, then expects a test equality failure. The named tests place output equality before reach checks, so the intended target is substantive. The driver's substring assertion alone is not a raw-log witness: review the actual assertion location and both baseline/mutant results before crediting kills. Candidate execution records for this driver and the original mutation suite were not available in the inspected candidate tree, PR conversation or current producer cursor.

## Observed validation, not attributed claims

GitHub check records are attached to835984e. The actual Rust and fuzz jobs checked out synthetic PR merge e6cc258914337330f8e51cf5eef5a76530156ca1. I fetched both Git commit objects: both trees equal c7f091b918756460174f9ba0269afa4707c150ef. Thus these are the reviewed source tree, not an unbound green status.

Inspected Rust job131868260462 logs show all8 Cells unit tests passing, including the new boundary/reuse witness, and all5 tests/cells.rs differential tests passing. The full library result is107 passed/1 ignored. **The whole Rust job fails**: tests/service.rs a_block_crc_in_the_middle:182 and injected_panics:264 both fail starts.len()>=4 (8 pass/2 fail in that binary; exit101). Format and Clippy pass. Later harness steps, including Experimental cells harness tests, are skipped. The observed failure is a fixture-size precondition, not a demonstrated Cells byte mismatch; I have not reproduced or classified its underlying cause and do not waive it.

Inspected fuzz-smoke job131868260522 logs show the30-second cells command, seed1,26021 cases, production admitted2580 and forced admitted13663, with successful job conclusion. This corroborates actual forced-route reach independently of the author's separate seed42 claim; it does not report a guard-side reach histogram. Other fuzz modes also ran but are not a substitute for this Cells evidence.

Miri job131868261279 had passed its test-selection step and was still executing Miri at observation. Its incomplete log endpoint returned unavailable. No Miri PASS is inferred. Benchmark job131868260723 was also running; no timing/RSS conclusion follows.

Author reports8 native/5 differential/6 Miri witnesses,27380 cases seed42, two killed copy mutants and a continuing original mutation suite. Native/differential counts are independently corroborated by the separate CI logs above. The local Miri, seed42 and mutation claims remain attributed until their records are archived and inspected. Retain the reported initial wrong boundary fixture assertion, read-only Miri cache failure, and missing-libpython3.12 all-features link attempt alongside successes; none is silently rewritten into PASS.

## Narrow closure and next value

To close the remaining safety evidence gap, archive exact-source candidate Miri and mutation logs/results (including original suite coverage and baseline outcomes), preserving failed attempts; inspect every relevant completed witness and byte-equality kill. Complete/classify ordinary CI separately, including the two service-test preconditions and skipped harness steps. No new broad design review is demanded for unchanged F01; a source change requires a targeted delta review of that change.

Expected mechanism: remove unused position initialization and scans for all-small blocks. What happened here: static equivalence checks pass and actual CI native/differential/forced-fuzz coverage is available; candidate Miri/mutation evidence closure and full CI are unfinished. No measured portion of the reported performance gap is credited. The prior22 pattern/0-of196 Wikidata eligibility screen is a workload observation, not a correctness theorem or guaranteed gain. Guard comparisons/ANDs are paid on long blocks too; retained small-block capacity means zero fresh allocation is not a peak-RSS saving claim. Highest next value is completing the existing evidence and frozen whole-reader comparison, not launching a dispatcher or another writer.

The unchanged standing default-arm gate and separately reported experimental seven-input comparison remain distinct. PR365 DEFER/slice-one integration dependencies, MONODROMY's pending mathematical ownership and Pathfinder architecture remain. No workstation timing, gate change, default adoption or merge is authorized by this review.

## Record integrity

Receipt0c707f0247883951e72c334a134a04837ef701de precedes research. Migration manifest digest and frozen legacy head were verified. Pinned coord.py audit on fresh connector-fed comments reports five resolved historical misroutes plus historical researcher LOG6757065549 (unresolved in this older pinned script; already recorded by prior reviews). No new routing issue was observed. route sends REVIEW to145. The known coordinator-only FROM assumption in pinned verify will be recorded explicitly, with exact-body/author/issue and role-aware routing verification after publication. The persistent task remains untouched.
