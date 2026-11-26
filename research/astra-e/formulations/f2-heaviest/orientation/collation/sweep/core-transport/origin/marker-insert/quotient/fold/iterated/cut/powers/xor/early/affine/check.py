"""Differentially run the original pinned pair_screen inner loop with supplied colors."""
from array import array
from collections import Counter
import csv
import itertools
import json
from pathlib import Path
import subprocess
import sys
import textwrap
from constructor import initial,step

PIN="9cd25610131b66924eaa75813d0f649db79f354f"
DIR="docs/journal/2026-11-18-switch-elimination"
old_source=subprocess.check_output(["git","show",f"{PIN}:{DIR}/pair_screen.py"],text=True)
old={"__name__":"old_oracle"}
exec(compile(old_source,"old_pair_screen","exec"),old)
start=old_source.index("        by_image = sorted")
end=old_source.index("        print(generation",start)
body=textwrap.dedent(old_source[start:end])
body="\n".join(l for l in body.splitlines() if not l.startswith("colours ="))
tail="return bands, new_mark, dict(intersections=intersections, raw=len(pieces), removed=removed, rows=rows)"
exec("def supplied(bands, mark, colours, rules):\n"+textwrap.indent(body+"\n"+tail,"    "),old)

def check(bands,image,mark,cols,rules,reference_rules):
    ref,expected_mark,h=old["supplied"](bands,mark,cols,reference_rules)
    result,ix,m,c=step(bands,image,mark,cols,rules)
    assert m==expected_mark and result==ref and ix==sorted(range(len(ref)),key=lambda i:ref[i][1])
    assert c["removed"]==h["removed"] and c["rows"]==h["rows"] and rules==reference_rules
    assert c["raw"]<=h["raw"]
    return result,ix,m,c|{"full_intersections":h["intersections"],"full_raw":h["raw"]}

def word(value,rules,inv,cache):
    if value not in cache:
        if value<256:cache[value]=bytes([value])
        else:
            x,y=inv[value];cache[value]=word(x,rules,inv,cache)+word(y,rules,inv,cache)
    return cache[value]

def decode(bands,mark,rules,horizon):
    n=sum(w for _,_,w,_ in bands)
    nxt=array("I",[0])*n;labels=array("I",[0])*n
    for x,y,w,v in bands:
        nxt[y:y+w]=array("I",range(x,x+w));labels[y:y+w]=array("I",[v])*w
    cache={};inv={v:k for k,v in rules.items()};out=bytearray()
    while len(out)<horizon:
        out+=word(labels[mark],rules,inv,cache);mark=nxt[mark]
    return bytes(out[:horizon])

def tiny():
    checks=0
    for k in range(1,5):
        for w in itertools.product((1,2),repeat=k):
            x=[0]
            for n in w:x.append(x[-1]+n)
            for image in itertools.permutations(range(k)):
                targets=[0]*k;y=0
                for e in image:targets[e]=y;y+=w[e]
                bands=[(x[e],targets[e],w[e],e)for e in range(k)]
                for cs in itertools.product((0,1),repeat=k):
                    for mark in range(y):
                        rules={};b,_,m,c=check(bands,image,mark,cs,rules,{})
                        assert decode(b,m,rules,y)==decode(bands,mark,{},y)
                        checks+=1
    print(json.dumps({"arbitrary_interval_tilings_colorings_marks":checks}))

def panel(inputs,out):
    historical=list(csv.DictReader(subprocess.check_output(["git","show",f"{PIN}:{DIR}/evidence/pair-screen.csv"],text=True).splitlines()))
    with (out/"panel.csv").open("w") as f:
        writer=csv.DictWriter(f,["tape","round","old_bands","old_rows","old_mark","new_mark","comparisons","hits","gaps","raw","removed","rows","bands","full_intersections","full_raw","rules","checked_output"],lineterminator="\n")
        writer.writeheader()
        for path in sorted(inputs.glob("0*/input.txt")):
            lines=path.read_text().splitlines();mark=int(lines[0]);runs=[tuple(map(int,l.split())) for l in lines[1:]]
            bands,image=initial(runs);rules={};refs={};want=(path.parent/"cut.bin").read_bytes()
            horizon=len(want);size=horizon
            assert decode(bands,mark,rules,horizon)==want
            for generation in range(64):
                if len(bands)<2 or size<64:break
                cols={b[3]:old["colour"](b[3],generation) for b in bands}
                before=len(bands);prior=mark
                bands,image,mark,c=check(bands,image,mark,cols,rules,refs);size=c["rows"]
                test=generation in (0,1) or size<64 or len(bands)<2
                if test:assert decode(bands,mark,rules,horizon)==want
                if path.parent.name=="0000":
                    h=historical[generation]
                    assert [int(h[k])for k in ("intersections","removed","rows","bands","pair_rules_to_date")]==[c["full_intersections"],c["removed"],size,len(bands),len(rules)]
                writer.writerow(dict(tape=path.parent.name,round=generation,old_bands=before,
                    old_rows=size+c["removed"],old_mark=prior,new_mark=mark,rules=len(rules),checked_output=int(test))|c)
            assert size<64 or len(bands)<2
            if path.parent.name=="0000":assert generation==len(historical)
            f.flush()
if __name__=="__main__":
    if sys.argv[1]=="tiny":tiny()
    else:panel(Path(sys.argv[1]),Path(sys.argv[2]))
