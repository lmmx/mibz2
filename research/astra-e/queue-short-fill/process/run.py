"""Run the official W1 decoder, alternating two pairs of same-binary labels."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import random
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('original', type=Path)
parser.add_argument('singleton', type=Path)
parser.add_argument('corpus', type=Path)
parser.add_argument('out', type=Path)
a = parser.parse_args()
a.out.mkdir(parents=True, exist_ok=True)
assert os.sched_getaffinity(0) == {1}
assert os.getpriority(os.PRIO_PROCESS, 0) == 15
paths = [a.original.resolve(), a.singleton.resolve(), a.corpus.resolve()]
digest = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
assert digest[str(paths[2])] == '82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915'

def run(label):
    exe = paths[label // 2]
    cmd = [str(exe), 'run', '--arm', 'candidate', '--workers', '1', '--file', str(paths[2]), '--repeat', '1']
    row = json.loads(subprocess.check_output(cmd, text=True))
    assert row['workers'] == 1 and row['arm'] == 'candidate' and row['out_bytes'] == 209700196
    assert all(row[k] > 0 for k in ('wall_s', 'cpu_s', 'peak_rss_kb'))
    return row

design = [(0,1,3,2), (1,2,0,3), (2,3,1,0), (3,0,2,1)]
rng = random.Random(926443)
order = []
for group in range(8):
    labels = list(range(4))
    rng.shuffle(labels)
    order.extend([[labels[i] for i in line] for line in design])
manifest = {'binaries_and_corpus': digest, 'info': [json.loads(subprocess.check_output([p, 'info'])) for p in paths[:2]],
            'labels': ['originalA','originalB','singletonA','singletonB'], 'order': order,
            'affinity': sorted(os.sched_getaffinity(0)), 'nice': os.getpriority(os.PRIO_PROCESS, 0),
            'cgroup_quota': Path('/sys/fs/cgroup/cpu.max').read_text().strip(),
            'utc_started': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
            'co_runners_at_start': subprocess.check_output(['ps','-eo','psr,ni,pcpu,comm','--sort=-pcpu'],text=True).splitlines()[:15]}
with (a.out / 'manifest.json').open('x') as f:
    json.dump(manifest, f, indent=2)
with (a.out / 'warmup.json').open('x') as f:
    json.dump([run(0), run(2)], f, indent=2)
with (a.out / 'runs.jsonl').open('x', buffering=1) as f:
    for r, seq in enumerate(order):
        for pos, label in enumerate(seq):
            time.sleep(0.1)
            row = run(label)
            print(json.dumps(dict(round=r + 1, position=pos, label=label, **row)), file=f)
