"""Untimed analytical witness, not execution of Rust or the decoder.

The run route leaves rle untouched; its zero capacity contradicts the ledger's
positive lower bound regardless of allocator growth. The ll example separately
uses the doubling model assumed by trace.rs's existing grown() tests.
"""
import json

def reserve_model(cap, request):
    return cap if request <= cap else max(request, 2 * cap, 8)

def violates(x, lo, hi):
    return x < lo * .999 or (hi is not None and x > hi * 1.001)

M = 900_000
rows = [600_000, 800_000]
ll = rle = 0
states = []
for n in rows:
    ll = reserve_model(ll, n)
    states.append(dict(rows=n, ll_capacity_model=ll, rle_capacity_exact=0))
assert rle == 0 and violates(rle, max(rows), 2 * max(rows) - 1)
assert ll == 1_200_000 and violates(ll, M, M)
reported = dict(ll=1_799_923, rle=38)
assert violates(reported['ll'], M, M)
assert violates(reported['rle'], M, 2 * M - 1)
print(json.dumps(dict(
    kind='analytical source counterexample; no Rust execution',
    states=states, old_ledger_ll=[M,M], old_ledger_rle=[max(rows),2*max(rows)-1],
    reported_team_capacities=reported,
    conclusion='rle lower bound contradicted without a growth assumption; ll contradiction also reproduced under the existing doubling model'
), indent=2))
