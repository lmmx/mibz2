from pathlib import Path
import csv,sys
out=Path(sys.argv[1])
with (out/'rss.csv').open('w') as f:
    writer=None
    for arm in (-2,-1,0,1,2,3):
        rows=list(csv.DictReader((out/f'rss-{arm}.csv').open()))
        assert len(rows)==228
        if writer is None:
            writer=csv.DictWriter(f,fieldnames=rows[0].keys(),lineterminator='\n');writer.writeheader()
        writer.writerows(rows)
        print(rows[0]['arm'],max(int(r['retained_bytes_with_output']) for r in rows),max(int(r['process_hwm_kib']) for r in rows))
