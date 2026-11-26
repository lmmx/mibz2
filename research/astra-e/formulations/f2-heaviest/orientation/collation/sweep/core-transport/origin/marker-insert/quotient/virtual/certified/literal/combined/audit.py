"""Reconcile complete-implicit outputs and physical record counts with held-out explicit data."""
from collections import Counter
from pathlib import Path
import csv
import hashlib
import json
import sys

def keyed(path):
    rows=list(csv.DictReader(path.open()))
    result={(int(r['stream']),int(r['h'])):r for r in rows}
    assert len(result)==len(rows)
    return result

def main(results,earlier):
    all_=keyed(results/'admissions.csv')
    held=keyed(earlier/'heldout.csv')
    manifest=list(csv.DictReader((earlier/'manifest.csv').open()))
    assert len(all_)==456 and len(held)==438
    expected={(int(r['admission']),h) for r in manifest for h in (4,12)}
    assert set(all_)==expected
    assert set(held)=={(int(r['admission']),h) for r in manifest if r['held_out']=='1' for h in (4,12)}
    assert all(all_[k]==v for k,v in held.items())
    assert all(r['actual_records']==r['literal_records'] and r['literal_in_root']=='true' for r in all_.values())
    plans={p:[int(r['actual_records']) for r in all_.values() if r['h']==str(p)] for p in (4,12)}
    out=dict(full_inputs=len(all_)//2,matched_prior_heldout_inputs=len(held)//2,
             stored_records_all={h:sum(vals) for h,vals in plans.items()},
             aggregate_sha256=hashlib.sha256((results/'admissions.csv').read_bytes()).hexdigest())
    return out
if __name__=='__main__':
    print(json.dumps(main(Path(sys.argv[1]),Path(sys.argv[2])),indent=2))
