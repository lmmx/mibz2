"""Compare original discovery-ID and independently canonical base-ID choices on frozen first roots."""
import csv,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from fold import Atlas,Column,Fold,incoming,independent,paths,read_input

def references(tapes):
    out=csv.writer(sys.stdout,lineterminator='\n')
    out.writerow(('stream','base','mass','succ','canonical','original'))
    for tape in paths(tapes):
        runs,_=read_input(tape)
        col=Column(runs);u=col.interval(b'"')
        root,words,minima,freq,_=incoming(col,u,Atlas(col,u,0))
        order=sorted(words,key=minima.get)
        fold=Fold(root,{k:x[1:]+b'"' for k,x in words.items()},order,freq)
        edges={fold.base[i]:fold.base[j] for i,j in fold.succ.items()}
        canon=independent(edges,{fold.base[k]:v for k,v in freq.items()})
        assert sum(freq[k] for k in fold.chosen)==sum(freq[k] for k in freq if fold.base[k] in canon)
        for k in order:
            out.writerow((int(tape.name.split('.')[0]),fold.base[k],freq[k],edges.get(fold.base[k],4294967295),
                          int(fold.base[k] in canon),int(k in fold.chosen)))
        sys.stdout.flush()

if __name__=='__main__':references(Path(sys.argv[1]))
