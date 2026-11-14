"""Untimed mathematical checks for PR383 adoption review; no decoder execution."""
import json
import math
import statistics

ratios = [math.exp(math.log(1.10) - .15 * math.sqrt(.9))] * 5
ratios += [math.exp(math.log(1.10) + .15 * math.sqrt(.9))] * 5
logs = list(map(math.log, ratios))
mean = statistics.mean(logs)
half = 2.2621571627409915 * statistics.stdev(logs) / math.sqrt(10)
interval = [math.exp(mean-half), math.exp(mean+half)]
assert interval[0] < 1.02 < 1.10 < interval[1]

word = (b'a'*9 + b'b'*9)*3 + b'c'
rotations = sorted(word[i:]+word[:i] for i in range(len(word)))
column = bytes(r[-1] for r in rotations)
fixtures = []
for multiple in (8, 10):
    expanded = b''.join(bytes([b])*multiple for b in column)
    runs = []
    for b in expanded:
        if runs and runs[-1][0] == b and runs[-1][1] < 127:
            runs[-1][1] += 1
        else:
            runs.append([b, 1])
    n, r = len(expanded), len(runs)
    assert r == 8
    fixtures.append(dict(multiple=multiple,n=n,records=r,budget=4*n//25,
                         sparse_refusal=64*r<n,density_refusal=10*r>=n,
                         runs=runs))
assert not fixtures[0]['sparse_refusal'] and fixtures[1]['sparse_refusal']
print(json.dumps(dict(counterexample=dict(ratios=ratios,estimate=math.exp(mean),
      ci95=interval,original_I2_pass=interval[0]<=1.02,
      original_I3_pass=interval[0]<=1.05),fixtures=fixtures),indent=2))
