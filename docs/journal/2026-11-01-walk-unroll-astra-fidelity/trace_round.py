import re,sys,collections
# trace.py ASM START_HEX: follow the hot path of one round from START (fall through `js`
# to the cold stubs; follow jmp/jns/jne to their targets) until START comes round again
ins=[]
for l in open(sys.argv[1]):
    m=re.match(r"\s+([0-9a-f]+):\t(\S+)\s*(.*)",l)
    if m: ins.append((int(m.group(1),16),m.group(2),m.group(3).split("#")[0].strip()))
addr={a:i for i,(a,_,_) in enumerate(ins)}
start=int(sys.argv[2],16); i=addr[start]; path=[]
cold={"js"}; follow={"jmp","jns"}
while True:
    a,m,o=ins[i]; path.append(ins[i])
    if len(path)>2000: sys.exit("no loop")
    if m in follow or (m in ("jne","jb") and len(path)>5):
        t=int(o.split()[0],16)
        if m in follow or t==start:
            if t==start: break
            i=addr[t]; continue
    i+=1
    if ins[i][0]==start: break
c=collections.Counter()
for a,m,o in path:
    c["ins"]+=1
    dst,_,src=o.partition(",")
    if m=="prefetcht0": c["pf"]+=1; continue
    if m=="lea" or m=="nop" or m.startswith("cs") : 
        if m!="lea": c["ins"]-=1
        continue
    if "PTR" in dst and m.startswith("mov"): c["st"]+=1
    if "PTR" in src or ("PTR" in dst and m in ("cmp","test","add","sub")): c["ld"]+=1
    if m.startswith("j"): c["jcc" if m!="jmp" else "jmp"]+=1
    if "[rsp" in o: c["stack"]+=1
tl=sum(1 for a,m,o in path if m=="mov" and re.search(r"DWORD PTR \[r\w+\+r\w+\*4\]",o))
# macro-ops: fuse test/cmp + jcc
mops=c["ins"]-sum(1 for x,y in zip(path,path[1:]) if x[1] in("test","cmp","inc","add","sub","and","dec") and y[1].startswith("j") and y[1]!="jmp")
print(sys.argv[1], "round: %d instructions, %d macro-ops, %d T loads; loads %d stores %d prefetch %d jcc %d jmp %d"%(c["ins"],mops,tl,c["ld"],c["st"],c["pf"],c["jcc"],c["jmp"]))
print("per lane-step: ins %.2f mops %.2f ld %.2f st %.2f pf %.2f br %.2f"%(c["ins"]/32,mops/32,c["ld"]/32,c["st"]/32,c["pf"]/32,(c["jcc"]+c["jmp"])/32))
steplike=re.compile(r"^(eax|ecx|r14d),")
for a,m,o in path:
    ordinary = (m=="mov" and (re.match(r"(eax|r14d),DWORD PTR \[rsp\+0x[0-9a-f]+\]$",o) or re.match(r"eax,DWORD PTR \[r\w+\+r\w+\*4\]",o) or o=="ecx,eax" or re.match(r"DWORD PTR \[rsp\+0x[0-9a-f]+\],ecx$",o) or re.match(r"rcx,QWORD PTR \[rsp\+0x[0-9a-f]+\]$",o) or re.match(r"BYTE PTR \[r\w+\+r\w+\*1\],al",o))) or m in("test","shr","prefetcht0","js")
    if not ordinary: print("   extra %x %s %s"%(a,m,o[:50]))
