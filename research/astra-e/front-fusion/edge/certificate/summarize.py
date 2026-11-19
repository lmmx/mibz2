"""Paired log-ratios for gate CPU, with independent whole-corpus gate attribution."""
import csv
import math
from pathlib import Path
import statistics

base = Path(__file__).resolve().parent
with (base / 'panel.csv').open() as source:
    rows = list(csv.DictReader(source))
assert len(rows) == 32 * 4
cpu = {}
for row in rows:
    r, pos, arm = (int(row[key]) for key in ('round', 'position', 'arm'))
    assert arm == (r % 4 + (pos if r // 4 % 2 == 0 else 4-pos)) % 4
    cpu[r, arm] = int(row['cpu_ns']) / int(row['passes'])
for arm, name in enumerate(('frozen', 'frozen clone', 'capped AVX2', 'symbol bitmap')):
    times = [cpu[i, arm] for i in range(32)]
    logs = [math.log(v/cpu[i, 0]) for i, v in enumerate(times)]
    mean = statistics.mean(logs)
    half = 2.039513 * statistics.stdev(logs) / math.sqrt(len(logs))
    print(name, 'median gate CPU ns / all five', round(statistics.median(times)),
          'ratio and paired 95% interval', *[round(math.exp(x), 5) for x in (mean, mean-half, mean+half)])
with (base / 'full.csv').open() as source:
    whole = list(csv.reader(source))
assert len(whole) == 3
shares = []
for _, run, total, gate, output in whole:
    assert int(output) == 209700196
    shares.append(int(gate)/int(total))
print('frozen gate fraction of complete serial CPU, input through checked CRC',
      *[round(s, 7) for s in shares], 'median', round(statistics.median(shares), 7))
