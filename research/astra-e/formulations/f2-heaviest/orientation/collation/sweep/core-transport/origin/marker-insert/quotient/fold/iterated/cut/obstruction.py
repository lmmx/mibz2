"""A high-self family with one marked cycle, and another with isolated ballast."""
import csv
import sys
from cut import solve

writer=None
for n in (1,2,4,8,16,32,64,128):
    # One cycle: b is the protected anchor, the sole positive choice is T={a}.
    # Disjoint: a^n are fixed rows, unrelated to the two marked positions b,c.
    for kind,raw,words,mark,expected in [
        ("single",[(1,1),(0,n)],[b"a",b"bB"],n,b"bB"+n*b"a"),
        ("disjoint",[(0,n),(2,1),(1,1)],[b"a",b"bB",b"Ccc"],n,b"bBCcc"),
    ]:
        trace=[]
        horizon=sum(width*len(words[k]) for k,width in raw)
        result,count=solve(raw,words,mark,1,horizon,trace.append)
        repeated=expected*((1+horizon+len(expected)-1)//len(expected))
        assert result==repeated[1:1+horizon]
        assert count['rounds']==(n if kind=="single" else 1)
        assert count['joins']==count['rounds']
        assert count['terminal_rows']==(1 if kind=="single" else n+1)
        assert all(r['removed']==1 and r['nonself']==2 for r in trace)
        row=dict(family=kind,repeat=n,rounds=count['rounds'],joins=count['joins'],terminal=count['terminal_rows'],
                 first_self=trace[0]['self_mass'],first_rows=trace[0]['old_rows'],first_removed=1,
                 grammar_visits=count['visits'],grammar_peak_stack=count['stack_peak'])
        if writer is None:
            writer=csv.DictWriter(sys.stdout,row,lineterminator='\n')
            writer.writeheader()
        writer.writerow(row)
