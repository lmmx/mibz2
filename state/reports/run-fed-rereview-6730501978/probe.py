"""Untimed independent admission and physical-footprint checks at the reviewed head."""
import importlib.util
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'bench'))
import test_run_fed_leader as t
r = t.rfl
results = {}
for key, value in [('hash', []), ('out', {}), ('wall', float('inf')), ('cpu', False)]:
    records = t.clean()
    records[0][key] = value
    code, out, raw = t.run(records)
    assert code == 2 and 'INVALID' in out and 'Traceback' not in out
    results[f'malformed_{key}'] = {'exit': code, 'first_lines': out.splitlines()[:2], 'json_written': raw is not None}
for key in ['head', 'tree_clean', 'binary_sha256_before', 'corpus_sha256_after', 'rustc']:
    ident = t.identity()
    del ident[key]
    code, out, _ = t.run(t.clean(), ident=ident)
    assert code == 2 and 'INVALID' in out and 'Traceback' not in out
    results[f'missing_identity_{key}'] = {'exit': code, 'first_lines': out.splitlines()[:2]}
for name, factors in [('attributed', {r.CTL:1, r.PRIMARY:.94}), ('control_confound', {r.CTL:.816,r.PRIMARY:.945}), ('fast_secondary_only', {r.CTL:1,r.PRIMARY:1,r.SECONDARY[0]:.9})]:
    code, out, raw = t.run(t.clean(factors=factors))
    assert code == 0
    results[name] = {k:raw[k] for k in ['reading','attribution','fidelity']}
# Enumerate every contiguous cell partition up to 12 owned rows, at four offsets.
# Each cell may overstore into a later cell of its OWN range only. Final writes must
# cover precisely the owned rows and every cell must finish with its own intended value.
cases = 0
for offset in [0,1,7,16]:
    for n in range(1,13):
        for mask in range(1 << (n-1)):
            ends = [offset+i for i in range(1,n) if mask & (1<<(i-1))]+[offset+n]
            mem = {}; q = offset
            expected = {}
            for number, stop in enumerate(ends):
                expected.update({i:number for i in range(q,stop)})
                w = q
                while w < stop and w+8 <= offset+n:
                    mem.update({i:number for i in range(w,w+8)})
                    w += 8
                while w < stop:
                    mem[w] = number
                    w += 1
                q = stop
            assert mem == expected
            cases += 1
results['abstract_footprint'] = {'partitions_checked':cases,'max_rows':12,'offsets':[0,1,7,16], 'scope':'integer store model only; no Rust, AVX2 or Miri execution'}
results['common_origin'] = {'delay':20,'leader_duration':95,'build_end':100,'old_late':95>100,'new_late':20+95>100}
(ROOT/'probe.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results,indent=2))
