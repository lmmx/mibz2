"""Recompute inversion time from archived occupancy summaries; runs no decoder."""
import json
import statistics as st
from pathlib import Path

out = []
for name in ['occupancy-t8-k2.json', 'occupancy-t12-k3.json', 'occupancy-t16-k4.json']:
    data = json.loads((Path(__file__).parent / name).read_text())
    for mode in ['timeline', 'full']:
        rows = []
        for run in data['runs']:
            if run['placement'] != 'spread' or run['mode'] != mode:
                continue
            kv = dict(word.split('=', 1) for word in run['line'].split() if '=' in word)
            assert kv['t.complete'] == 'yes'
            get = lambda k: float(kv['t.' + k])
            n = get('blocks')
            span = get('span_s')
            inv = get('q.tvec_mean') * span * 1000 / n
            dec = get('q.decode_ms_mean')
            check = get('w.mean_decoding') * span * 1000 / n
            assert abs(check - dec) < 0.00002, (check, dec)
            rows.append(dict(round=run['round'], decode_ms=dec, inversion_ms=inv,
                outside_inversion_ms=dec-inv, outside_fraction=1-inv/dec,
                span_s=span, mean_inversion_count=get('q.tvec_mean'),
                mean_decoding_count=get('w.mean_decoding')))
        out.append(dict(file=name,mode=mode,rounds=rows,
            median={key:st.median(r[key] for r in rows) for key in rows[0] if key!='round'},
            outside_fraction_range=[min(r['outside_fraction'] for r in rows),max(r['outside_fraction'] for r in rows)]))
print(json.dumps(out,indent=2))
