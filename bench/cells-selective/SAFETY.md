# Selective materialisation: changed unsafe-path argument

Implementer's argument, not independent B5 certification. Baseline corrected365
`d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2`; common integrated tree
`be25366f46458e2f1dbb0bdac1f5fbc3366d0648`. Existing constructor/grammar invariants
remain assumptions, including F01's necessary final marked-spine recopy.

The sole non-test production change selects the retained representation BEFORE
writing it. The exact expression `len <= SMALL as u64` includes MARK. Neither
length arithmetic, descriptors, top/rp advances, cell counting, slot updates nor
writer operations change. Test-only comparator/hooks are absent from release.

1. **Whole allocation initialized.** slots allocates vec![0;allocation], retains
   initialized storage across blocks, and resets the same canaries. Omitting writes
   cannot introduce uninitialized Rust bytes. No allocation/reservation/slack bounds
   change, including 16-byte readable source tails and 32-byte arena destination slack.
2. **Live word induction.** Initially a descriptor's one byte is initialized at its
   byte-table offset. Assume every existing descriptor's meaningful representation
   agrees with baseline. If `small`, both factors are unmarked, their positive lengths
   sum to<=16, and each is a literal. The SAME two copy16 operations execute in the SAME
   order. Each copy loads all16 bytes before storing, permitting the same overlap.
   The second destination begins at the first factor's exact length, overwriting its
   overcopied tail. Therefore the meaningful concatenation prefix is identical even
   when unused source/slack bytes differ. In the other case the SAME two descriptors
   are stored as an exact16-byte rule. This preserves ordered children and lengths.
3. **Discarded writes never establish a live representation.** In the short case,
   discarded stores would be at rp, the next unused rule position; rp does not advance.
   A later retained rule writes all16 bytes before publishing its descriptor. In the
   long/marked case, discarded copies would start at top and top+min(la,16), at or after
   the end of all live literals. top does not advance. A later short word overwrites
   every byte in its meaningful prefix as shown above. Removed writes cannot affect
   earlier live literal prefixes or earlier rules. Stale tails may change but are not
   semantic payload. Prefetch hints do not interpret those tails as live grammar.
4. **Bounds and phase unchanged.** Existing per-cell budget checks still precede all
   retained writes; region cursors advance identically. Marked words, including lengths
   2..16, remain rules. The writer receives identical root/length/rule_end, earlier-child
   rule order, origin mark, literals and rule descriptors. Its first-position propagation,
   overlapping bulk copies, F01 recopy, rotation, repetition, CRC/RLE1 and refusal fallback
   remain the original implementation. No new raw-pointer operation or alias is added.
5. **Reused allocation.** The same high-water buffer and pos policy remain. Tests poison
   original and candidate unused bytes differently before each block, then compare every
   selected payload/descriptor, written slot pair and frontier state. Large/short/large,
   marked,15/16/17, F01 n55/n550, period55 and arbitrary-cycle cases assert reach. Comparing
   entire scratch buffers would incorrectly require equality of intentionally dead bytes.

The test-only eager function is mechanically verified against365 after removing ONLY
its rename and identical snapshot calls. slots/write_cells/try_invert remain byte-exact
against365. The independent default/Plain/libbzip2 oracles supplement that shared-helper
comparison. The new mutations omit a retained literal copy, omit a retained rule child,
and incorrectly mask MARK in the predicate; they must compile and fail selected payload
or representation assertions. These are not three claims of output-byte equality kills.
Original F01/invariant mutations and forced-route fuzz remain additional coverage.

No unchanged-writer correctness assumption is replaced by an operation-count speed claim.
Codegen inspection and whole-reader measurements decide whether the branch is profitable;
independent review must assess the stated unsafe-path proof and its test evidence.
