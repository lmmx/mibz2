"""Run or replay the predeclared standalone CPU3 comparison. Two labels share edge.bin."""
import csv
from math import exp,log,sqrt
from pathlib import Path
from statistics import mean,median,stdev
import subprocess,sys

arms=["edge","edge_duplicate","inplace_separate_build","inplace_only_build","deferred"]
def order(r):
    rotated=arms[r%5:]+arms[:r%5]
    return list(reversed(rotated)) if r//5%2 else rotated

def run(bins,corpus,out):
    with out.open("x") as dest:
        write=csv.writer(dest,lineterminator="\n")
        write.writerow(["round","position","arm","cpu_ns","wall_ns","rss_kib","blocks","bytes","direct"])
        for r in range(24):
            for pos,arm in enumerate(order(r)):
                binary=bins/(("edge" if arm=="edge_duplicate" else arm)+".bin")
                v=subprocess.check_output(["nice","-n","10","taskset","-c","3",str(binary),str(corpus)],text=True).strip().split(",")
                assert len(v)==6 and v[3:5]==["862","419400392"]
                write.writerow([r,pos,arm,*v]);dest.flush()

def summary(path):
    with path.open() as s: rows=list(csv.DictReader(s))
    assert [(int(r["round"]),r["arm"])for r in rows]==[(i,x)for i in range(24)for x in order(i)]
    def values(a,k): return [int(r[k])for r in rows if r["arm"]==a]
    for k in ["cpu_ns","wall_ns","rss_kib"]:
        for a,b in [(arms[1],arms[0]),(arms[2],arms[0]),(arms[3],arms[0]),(arms[4],arms[0]),
                    (arms[3],arms[2]),(arms[4],arms[3])]:
            x,y=values(a,k),values(b,k)
            logs=[log(i/j)for i,j in zip(x,y,strict=True)]
            m=mean(logs);h=2.068658*stdev(logs)/sqrt(24)
            print(k,a,b,*[round(exp(n),6)for n in (m,m-h,m+h)],"medians",median(x),median(y))
if __name__=="__main__":
    if sys.argv[1]=="run":run(*map(Path,sys.argv[2:]))
    else:summary(Path(sys.argv[2]))
