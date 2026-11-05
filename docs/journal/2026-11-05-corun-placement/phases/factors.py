import re,sys,glob,statistics as st
rows=["scanner (headers, Huffman)","MTF + RLE2","T-vector build","walk, room (used)","RLE1","CRC","segment assembly","walk, room + assembly as one","walk, flagged (before #37)","walk, bitmap (before #4)","walk, plain","total"]
def parse(f):
    d={}
    for l in open(f):
        for r in rows:
            if l.strip().startswith(r):
                m=re.search(r"\s([0-9.]+) c/B",l[len(r)+2:]) or re.search(r"([0-9.]+) c/B",l)
                d[r]=float(m.group(1))
    return d
for dd in sys.argv[1:]:
    a=parse(dd+"/sweep-1.txt"); cs=[parse(f) for f in sorted(glob.glob(dd+"/sweep-4-cpu*.txt"))]
    print("==",dd, open(dd+"/sweep-1.txt").readline().split(";")[0])
    for r in rows:
        v=[c[r] for c in cs]; m=st.mean(v)
        print(f"  {r:32} alone {a[r]:6.2f}  4-copy {m:6.2f} [{min(v):.2f}-{max(v):.2f}]  factor {m/a[r]:.2f} [{min(v)/a[r]:.2f}-{max(v)/a[r]:.2f}]")
    # walk excl assembly
    we=a["walk, room (used)"]-a["segment assembly"]; wc=st.mean(c["walk, room (used)"]-c["segment assembly"] for c in cs)
    print(f"  walk excl assembly factor {wc/we:.2f}")
