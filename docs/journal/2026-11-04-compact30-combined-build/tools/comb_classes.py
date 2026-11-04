"""The combined build's changed classes and their counted-build expressions (comb_replay.py).

(group, frequency) -> [(counter expression, path), ...], one entry per path at that
frequency. Each was read from the listings (`ctor2c-*-false.asm` against B2's same-binary
listings) with the profile's per-instruction counts. A class is counter-supported only if
every one of its paths' expressions equals its frequency in the sequential counted record.
"""

# the unrolled first segment of `tile16`: row position k runs on every tile with n1 > k
# (n1 is even, 2..=16), and position k's refill detour on the refills at k
def n1_ge(j):
    return ' + '.join(f'tile_n1_{x}' for x in range(j, 17, 2))


CLASSES = {
  # --- build::<false> ---------------------------------------------------------------
  ('build', 29357924): [('rows_scalar - tile_rows_second - tail_rows', "B2's full tiles' first-segment row loop (unrolled in the combined build)")],
  ('build', 10544477): [('lookahead_pairs + runs_no_full_tile', 'uniform tiles U = J + Q (B2 tile test)')],
  ('build', 10541706): [('lookahead_pairs', 'lookahead pairs J')],
  ('build', 9933956): [('lookahead_pairs - held_reuses', 'matched lookaheads J - H')],
  ('build', 8935084): [('tile_rows_second', "full tiles' second-segment row loop")],
  ('build', 3003834): [('outer_iterations', 'outer scan iterations S + R')],
  ('build', 2396084): [('fresh_pairs', 'fresh pairs A')],
  ('build', 2393313): [('tiles_nonuniform', 'nonuniform full tiles S (overhead, and unrolled rows 0 and 1: n1 >= 2)')],
  ('build', 1116556): [('tiles_second_segment', 'second segments X')],
  ('build', 996634): [('tile_refills_first', "B2's refill detour from a tile's first segment (per position in the combined build)")],
  ('build', 801475): [('fills_vector', "fill's vector loop entry (m >= 8) and its alias check")],
  ('build', 610521): [('runs', 'runs R')],
  ('build', 610356): [('runs_lookahead', 'runs entering the lookahead loop (alignment nop executed)')],
  ('build', 607750): [('held_reuses', 'held reuses H')],
  ('build', 597255): [('runs_end_fill', 'runs ending after a fill (B2 return path)')],
  ('build', 508988): [('run_finishes', "finish calls from run's close")],
  ('build', 496004): [('run_opens', 'open from run')],
  ('build', 467712): [('128 * epochs', "B2's snapshot loop: 256 ranks, two per iteration (fixed trip)")],
  ('build', 302049): [('tile_refills_second', "refill detour from tile16's second segment")],
  ('build', 296405): [('complete_calls', "run's whole-group branch (complete)")],
  ('build', 230202): [('63 * epochs', 'the snapshot loop: ranks 4..256, four per iteration (fixed trip)')],
  ('build', 111104): [('crossing_scan', "the crossing pre-marking scan of the 256 counts"),
                      ('final_closes', 'the final close loop')],
  ('build', 55986): [('sweep_checks // 2 + blocks', 'final sweep loop head (two checks per iteration, plus the exit)')],
  ('build', 55552): [('sweep_checks // 2', 'final sweep loop body (two checks per iteration)')],
  ('build', 27776): [('init_descs // 4', 'descriptor initialisation and the cumulative-rank loop (four per iteration)')],
  ('build', 23559): [('final_finishes', 'finish from the final close loop')],
  ('build', 13266): [('runs_end_complete', 'runs ending after complete (B2 return path)')],
  ('build', 3654): [('epochs', 'epochs (snapshot, active_tag)')],
  ('build', 3575): [('epochs_tiled', 'epochs with a full tile (scan loop entry)')],
  ('build', 3424): [('epochs_untailed', "epochs with no tail (B2's epoch-loop back edge)")],
  ('build', 2606): [('runs_exhausted', 'runs whose lookahead exhausts the epoch')],
  ('build', 1769): [('tail_rows - tail_rows_second', "the tail's first-segment row loop")],
  ('build', 786): [('tail_refills_first', "refill detour from the tail's first segment")],
  ('build', 434): [('blocks', 'blocks')],
  ('build', 432): [('bitmap_in_place', 'bitmap resize without a reserve')],
  ('build', 233): [('pad_blocks', "the last group's padding (memset)")],
  ('build', 230): [('tails', 'tails'), ('bitmap_memsets', "bitmap resize's fill (>= 2 words)")],
  ('build', 218): [('tails_len_ge2', 'tail advance loop (unrolled by two) entry, len >= 2')],
  ('build', 165): [('runs_no_lookahead', 'runs with no lookahead tile')],
  ('build', 121): [('tails_even', 'tail advance loop exit, even len')],
  ('build', 106): [('tail_rows_second', "the tail's second-segment row loop")],
  ('build', 79): [('epochs_short', 'epochs with no full tile')],
  ('build', 22): [('tails_second_segment', 'tail second-segment entry and exit')],
  ('build', 12): [('tails_len1', 'a tail of length 1')],
  ('build', 9): [('tail_refills_second', "refill detour from the tail's second segment")],
  ('build', 8): [('raw_grows', 'raw arena regrowth')],
  ('build', 1): [('rec_grow_frees', 'rec regrowth freeing the old allocation'), ('snaps_reserves', 'snapshot reserve'),
                 ('bitmap_reserves_single', 'bitmap reserve of one word')],
  # --- refill ---------------------------------------------------------------------------
  ('refill', 1299478): [('refills', 'refill calls')],
  ('refill', 1262935): [('refill_finishes', 'refills that finish a portion')],
  # --- finish (B2's finish against the combined leaf + finish_slow) ----------------------
  ('finish', 1795482): [('leaf_fast + slow_entries', 'every finish: B2 entry and classification; the leaf prefix to the tag test')],
  ('finish', 1754362): [('fin_fast + fin_ordinary + fin_wide', "B2's ordinary path (u0, ul, span)")],
  ('finish', 1723413): [('fin_fast + fin_ordinary', "B2's ordinary, not wide (header)")],
  ('finish', 1723371): [('norm_full', "B2's full normalisation (30 fields)")],
  ('finish', 1647416): [('leaf_fast + slow_ordinary_partial_same_epoch', 'leaf: tag matched, the length test')],
  ('finish', 1647375): [('leaf_fast', 'leaf: header and five-chunk normalisation')],
  ('finish', 148107): [('slow_entries', 'finish_slow entry and classification')],
  ('finish', 106987): [('slow_ordinary_old_epoch + slow_ordinary_partial_same_epoch', "finish_slow's ordinary path")],
  ('finish', 76038): [('fin_ordinary + slow_ordinary_partial_same_epoch', "finish_slow's ordinary, not wide (header)")],
  ('finish', 75996): [('norm_full - leaf_fast', "finish_slow's full normalisation")],
  ('finish', 61): [('norm_partial_scalar', "the partial normalisation's scalar iterations")],
}


# tile16's first segment, unrolled by LLVM into 16 row positions (the do-while over the
# 16-byte window): rows 2j - 2 and 2j - 1 run on the tiles with n1 >= 2j (n1 is even), and
# each position k has its own refill detour. Their frequencies are the counted values on
# the primary corpus; each class's instructions were read at its positions' offsets
# (`movzbl 0xk(%r15)` ...) in ctor2c-build-false.asm.
for _j, _f in ((4, 2233908), (6, 2074155), (8, 1914734), (10, 1754798), (12, 1595099), (14, 1436198), (16, 1276757)):
    CLASSES[('build', _f)] = [(n1_ge(_j), f"tile16's unrolled first segment, rows {_j - 2} and {_j - 1} (n1 >= {_j})")]
for _k, _f in enumerate((81607, 81144, 76174, 75877, 70838, 70208, 65139, 65256, 59807, 59520, 54029, 53863, 48226, 48658, 43150, 43138)):
    CLASSES[('build', _f)] = [(f'tile_refills_first_at_{_k}', f"tile16's refill detour from unrolled row {_k}")]
