"""Independently validate every cached word and affine band against the checked gap construction."""
import csv
import itertools
import sys
from pathlib import Path
from check import rle
from memo import compare

policies=["last","wide","all"]
if len(sys.argv)==1:
    cases=covered=arrows=0
    for alphabet,limit in [(2,8),(3,6)]:
        for n in range(1,limit+1):
            for col in itertools.product(range(alphabet),repeat=n):
                for d in range(alphabet):
                    # The band table is independent of the marked row; any mark on the section suffices.
                    if d not in col: continue
                    p=col.index(d)
                    c,r=compare(rle(col),p,d,policies)
                    covered+=1
                    arrows+=c["tokens"]
                    cases+=len(policies)
    print(f"verified {cases} cache policy cases over {covered} sections and {arrows} return arrows")
else:
    root=Path(__file__).resolve().parents[4]
    sys.path.insert(0,str(root/"docs/journal/2026-11-18-astra-c-output-first"))
    from inputs import read_input,paths
    output=csv.writer(sys.stdout,lineterminator="\n")
    keys=["run_lookups","certificate_tests","hits","writes","stored","trie_nodes","dictionary_bytes"]
    output.writerow(["block","policy","runs","old_label_probes","reverse_probes",*keys])
    for path in paths(sys.argv[1]):
        runs,p=read_input(path)
        c,by=compare(runs,p,34,policies)
        for policy,s in by.items():
            output.writerow([path.name.split(".")[0],policy,len(runs),c["label_probes"],c["reverse_probes"],*[s[k] for k in keys]])
        sys.stdout.flush()
