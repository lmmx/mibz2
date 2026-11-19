"""Compare charged structure per tape to the protected ordinary-fold grammar."""
import csv
from pathlib import Path
import sys

here=Path(__file__).resolve().parent
def records(path):
    with path.open() as f:
        return {r['ordinal']:r for r in csv.DictReader(f)}
old=records(here.parent/'anchor-wikidata.csv')
cut=records(here/'wikidata.csv')
assert len(old)==len(cut)==10 and old.keys()==cut.keys()
writer=None
for ordinal,x in cut.items():
    y=old[ordinal]
    assert all(x[k]==y[k] for k in ('original_rows','original_skip','terminal_rows','terminal_ids','lap'))
    initial=int(x['created'])-int(x['joins'])
    row=dict(ordinal=ordinal,initial_types=initial,ordinary_rounds=y['rounds'],cut_rounds=x['rounds'],
             ordinary_joins=int(y['created'])-initial,cut_joins=x['joins'],
             ordinary_scanned_runs=y['scanned_runs'],cut_old_runs=x['old_runs'],
             cut_overlay_fragments=int(x['old_runs'])+int(x['slice_cuts']),cut_refined_runs=x['refined_runs'],
             cut_target_stream_runs=x['target_stream_runs'],ordinary_scanned_ids=y['scanned_ids'],
             cut_old_ids=x['old_ids'],cut_refined_ids=x['refined_ids'],
             cut_distinct_pairs=x['contacts'],cut_aliases_created=x['splits'],
             ordinary_grammar_visits=y['visits'],cut_grammar_visits=x['visits'],
             ordinary_literal_bytes=y['literal_bytes'],cut_literal_bytes=x['literal_bytes'],
             ordinary_copied_bytes=y['copied_bytes'],cut_copied_bytes=x['copied_bytes'],
             cut_peak_refined_runs=x['peak_refined_runs'],cut_peak_refined_ids=x['peak_refined_ids'])
    if writer is None:
        writer=csv.DictWriter(sys.stdout,row,lineterminator='\n')
        writer.writeheader()
    writer.writerow(row)
