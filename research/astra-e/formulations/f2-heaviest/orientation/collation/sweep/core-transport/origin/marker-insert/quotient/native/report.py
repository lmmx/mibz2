"""Paired fixed-horizon native price, exact structure, admission and memory checks."""
import csv
import sys
from collections import Counter, defaultdict
from math import exp, log, sqrt
from pathlib import Path
from statistics import mean, stdev

ARMS = ('named_a', 'named_b', 'sparse4', 'quotient4', 'quotient8', 'quotient12')
QUOTIENTS = ARMS[3:]
PHASES = ('setup_ns', 'discover_ns', 'merge_ns', 'publication_ns', 'second_index_ns',
          'second_discover_ns', 'second_merge_ns', 'second_emit_ns', 'total_ns')
EXTRA = ('common_first_ns', 'post_root_ns', 'second_total_ns')
METRICS = PHASES + EXTRA
T95_DF9 = 2.2621571628540993


def read(path):
    with path.open() as stream:
        return list(csv.DictReader(stream))


def prices(directory):
    rows = read(directory / 'phase.csv')
    assert len(rows) == 3600
    streams = {int(row['stream']) for row in rows}
    assert len(streams) == 10
    cells, slots, blocks = set(), Counter(), defaultdict(set)
    totals = defaultdict(int)
    for row in rows:
        c, rep, s, p, j = (int(row[k]) for k in ('cluster', 'rep', 'stream', 'position', 'block_position'))
        a = row['arm']
        assert a in ARMS and rep in range(60) and c == rep // 6
        assert p in range(6) and j in range(10) and (rep, s, a) not in cells
        cells.add((rep, s, a))
        slots[c, s, p, a] += 1
        blocks[rep, j].add(s)
        phases = {k:int(row[k]) for k in PHASES}
        phases['common_first_ns'] = sum(phases[k] for k in PHASES[:3])
        phases['post_root_ns'] = phases['total_ns'] - phases['common_first_ns']
        phases['second_total_ns'] = sum(phases[k] for k in PHASES[4:8])
        assert all(v >= 0 for v in phases.values())
        for k, v in phases.items():
            totals[rep, a, k] += v
    assert len(slots) == 3600 and set(slots.values()) == {1}
    assert len(blocks) == 600 and all(len(x) == 1 for x in blocks.values())
    pairs = [(a, b) for a in QUOTIENTS for b in ARMS[:3]]
    pairs += [('sparse4', 'named_a'), ('sparse4', 'named_b'), ('named_b', 'named_a')]
    with (directory / 'clusters.csv').open('w') as out:
        w = csv.writer(out, lineterminator='\n')
        w.writerow(('cluster', 'arm', 'control', 'metric', 'mean_log_ratio', 'ratio'))
        for a, b in pairs:
            for k in METRICS:
                if k == 'publication_ns' and b.startswith('named'):
                    continue
                logs = [log(totals[r,a,k] / totals[r,b,k]) for r in range(60)]
                clusters = [mean(logs[i:i+6]) for i in range(0,60,6)]
                for c, m in enumerate(clusters):
                    w.writerow((c, a, b, k, m, exp(m)))
                center = mean(clusters)
                half_width = T95_DF9 * stdev(clusters) / sqrt(len(clusters))
                print(f'{a}/{b} {k}: {exp(center):.5f} [{exp(center-half_width):.5f},{exp(center+half_width):.5f}]')
    for a in ARMS:
        print(a, 'mean ten-block ms', {k:round(mean(totals[i,a,k] for i in range(60)) / 1e6, 4) for k in METRICS})


def geometry(directory):
    for path, n in [('ten.csv', 10), ('admissions.csv', 228)]:
        rows = read(directory / path)
        assert len(rows) == n*len(ARMS) and len({(r['arm'],r['stream']) for r in rows}) == len(rows)
        for a in ARMS:
            rr = [r for r in rows if r['arm'] == a]
            assert len(rr) == n and all(r['second'] == 'true' for r in rr)
            keys = ('original_mass', 'selected', 'z', 'hazard_mass', 'hazard_runs', 'hazard_types',
                    'old_runs', 'new_runs', 'tokens', 'lookups', 'second_words', 'second_dictionary_bytes', 'second_links')
            sums = {k:sum(int(r[k]) for r in rr) for k in keys}
            if a in QUOTIENTS:
                assert all(int(r['mode']) == 2 for r in rr)
                assert all(int(r['tokens']) == int(r['original_mass']) + int(r['selected']) - int(r['z']) for r in rr)
                assert all(abs(int(r['old_runs'])-int(r['new_runs'])) <= 2*int(r['hazard_runs']) for r in rr)
                assert all(r['second_base'] == r['z_base'] for r in rr)
            print(path, a, sums, 'marked (old,Z,suffix)', dict(sorted(Counter(r['marked_kind'] for r in rr).items())),
                  'hazard-run range', (min(int(r['hazard_runs']) for r in rr), max(int(r['hazard_runs']) for r in rr)))
    rss = read(directory / 'rss.csv')
    assert len(rss) == 228*6 and len({(r['arm'], r['stream']) for r in rss}) == len(rss)
    for a in ARMS:
        rr = [r for r in rss if r['arm'] == a]
        assert len(rr) == 228
        print(a, 'max retained bytes', max(int(r['retained_bytes_with_output']) for r in rr),
              'process peak KiB', max(int(r['process_hwm_kib']) for r in rr))


if __name__ == '__main__':
    directory = Path(sys.argv[1])
    prices(directory)
    geometry(directory)
