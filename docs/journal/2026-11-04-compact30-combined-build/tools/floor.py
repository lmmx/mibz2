#!/usr/bin/env python3
"""The combined build's floor and pass conditions (charter §6.3, D010) and v3's §7(b) and
§7(c), from this build's ledger and listings. Reads archives only.

    python3 floor.py LEDGER_JSON LEAF_LISTING BUILD_LISTING OVERLAP_JSON OUT_JSON

- O, the demonstrated overlap, at its lower bound:
  - S62-F02's B2 instruction at build +0x12b5 (`mov 0x8(%rsp),%r13`, the nonuniform full
    tile's `ll.len()` reload), executed S = 2,393,313 times: the registered known minimum;
  - this assembly's further demonstrated scanner × descriptor overlap (journal §6.2): the
    B2 epoch-entry reload `mov 0x38(%rsp),%rcx` (build +0x97d, class 3,575; +0x8b2, class
    79), absent from both isolated listings' classes, whose ledgers both credit removals
    there (overlap.py's table);
  - v3 against either: none demonstrable (no B2 finish class changes in either isolated
    profile; v3's isolated ledger credits nothing outside finish).
  F is reported with the registered minimum alone (the conservative floor) and with the
  further demonstrated overlap subtracted.
- §7(b): v3's contribution re-derived from the combined listing: the finish group's
  counter-supported class delta (B2's finish against the combined leaf + finish_slow, every
  class × its counted frequency), less v3's added costs outside it: active_tag's four
  instructions per epoch (located in the epoch class of the combined listing) and, charged
  to v3 at its least favourable end, the one added instruction at run's finish call site
  (class 508,988; the isolated v3 call sites equal B2's). Its lower end must be ≥ 50 M.
- §7(c): the combined leaf's instructions, entry to `ret`, ≤ 44 (v3's at dbf5139).
"""
import json, re, sys

ledger, leaf, build, overlap, out = sys.argv[1:6]
L = json.load(open(ledger))
rows = {(c['group'], c['frequency']): c for c in L['classes']}
assert all(c['agrees'] for c in L['classes']), 'a class is not counter-supported'
lower, upper = L['interval_removed']

# --- O and F ---------------------------------------------------------------------------
ov = {(r['group'], r['frequency']): r for r in json.load(open(overlap))}
s62 = 2_393_313
assert ov[('build', 2393313)]['both_credit']
further = 0
for f in (3575, 79):
    r = ov[('build', f)]
    assert r['both_credit'] and r['b2'] - r['scanner'] >= 1 and r['b2'] - r['descriptor'] >= 1
    further += f  # one shared B2 instruction (the 0x38(%rsp) reload) per execution
minimum = {'scanner': 60_000_000, 'descriptor': 40_000_000, 'v3': 50_000_000}
best_single = 84_060_421  # the scanner U2's isolated lower bound on B2 (D011)
F = {}
for name, comps in (('with_v3', ('scanner', 'descriptor', 'v3')), ('without_v3', ('scanner', 'descriptor'))):
    s = sum(minimum[c] for c in comps)
    F[name] = {'sum_of_minimums': s, 'F_registered_O': s - s62, 'F_further_O': s - s62 - further}

# --- §7(b) ----------------------------------------------------------------------------
finish_delta = sum(c['delta'] for c in L['classes'] if c['group'] == 'finish')
epochs = L['counted_record']['epochs']
text = open(build).read()
tag_ins = [l for l in text.splitlines() if re.search(r'\tshl\s+\$0x18,%r\w+d$|\tadd\s+\$0x7f000000,|\tor\s+\$0x80000000,', l)]
assert len(tag_ins) == 3, tag_ins
active_tag = 4 * epochs  # shl, add, or, and the store to W.active_tag
run_site = rows[('build', 508988)]
assert run_site['baseline'] == 5 and run_site['candidate'] == 6
run_site_charge = (run_site['candidate'] - run_site['baseline']) * run_site['frequency']
b7 = -finish_delta - active_tag - run_site_charge
leaf_n = sum(1 for l in open(leaf) if re.match(r'^\s+[0-9a-f]+:\t', l))
leaf_ret = [l for l in open(leaf) if re.match(r'^\s+[0-9a-f]+:\t', l)][-1].split('\t')[1].strip()
rec = L['counted_record']
model_gross = (81 - leaf_n) * rec['leaf_fast']

res = {
  'interval_removed': [lower, upper], 'O': {'s62_f02_known_minimum': s62, 'further_scanner_descriptor': further,
  'v3_cross': 0, 'lower_bound_registered': s62, 'lower_bound_with_further': s62 + further}, 'F': F,
  'pass_i_with_v3': lower >= F['with_v3']['F_registered_O'], 'margin_i_with_v3': lower - F['with_v3']['F_registered_O'],
  'pass_ii': lower > best_single, 'margin_ii': lower - best_single, 'best_single_isolated_lower': best_single,
  's7b': {'finish_group_delta': finish_delta, 'active_tag': active_tag, 'run_call_site_charge': run_site_charge,
          'contribution_lower': b7, 'minimum': 50_000_000, 'pass': b7 >= 50_000_000,
          'model_cross_check': {'gross_81_minus_leaf_times_leaf_fast': model_gross, 'leaf_fast': rec['leaf_fast'],
                                'slow_entries': rec['slow_entries']}},
  's7c': {'leaf_instructions_including_ret': leaf_n, 'last': leaf_ret, 'limit': 44, 'pass': leaf_n <= 44 and leaf_ret == 'ret'},
}
json.dump(res, open(out, 'w'), indent=1)
print(json.dumps(res, indent=1))
