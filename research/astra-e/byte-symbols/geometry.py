import csv
from pathlib import Path
from statistics import median

base = Path(__file__).resolve().parent
blocks = [(int(r['syms']),int(r['rows']),int(r['eob']),int(r['stock_capacity']))
          for r in csv.DictReader((base/'blocks.csv').open())]
for name, panel in [('all',blocks),('large',[b for b in blocks if b[1] >= 65536]),
                    ('small',[b for b in blocks if b[1] < 65536])]:
    sizes = [b[0] for b in panel]
    print(f'{name} blocks={len(panel)} rows={sum(b[1] for b in panel)} '
          f'symbols={sum(sizes)} max_eob={max(b[2] for b in panel)} '
          f'min_eob={min(b[2] for b in panel)} fits={sum(b[2]<=255 for b in panel)}')
print('W,slots,logical_bytes_saved_whole_corpus,max_contiguous_payload_saved,top_K_payload_saved_upper_bound,capacity_bytes_saved_if_all_slots_acquired')
for w in [1,2,4,7,8,9,10]:
    k = 4*w
    saved = [n if e<=255 else 0 for n,_,e,_ in blocks]
    print(w,k,sum(saved),max(sum(saved[i:i+k]) for i in range(len(saved)-k+1)),
          sum(sorted(saved,reverse=True)[:k]),k*min(b[3] for b in blocks),sep=',')
