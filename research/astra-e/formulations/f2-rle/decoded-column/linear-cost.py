from pathlib import Path
from statistics import median,mean,stdev
from math import log,exp,sqrt
from collections import defaultdict
import csv
d=Path(__file__).resolve().parent
rows=list(csv.DictReader((d/"linear.csv").open()))
assert len(rows)==9*2*32
sums=defaultdict(int)
for z in rows:sums[(z["round"],z["arm"])]+=int(z["cpu_ns"])
pairs=[log(sums[(str(i),"sais")]/sums[(str(i),"rle")]) for i in range(1,33)]
m=mean(pairs);h=2.039513*stdev(pairs)/sqrt(32)
print("sparse suffix sorting over stock RLE CPU: %.4f [%.4f, %.4f]"%(exp(m),exp(m-h),exp(m+h)))
print("stream,median_sais_cpu_ns,median_rle_cpu_ns")
for block in dict.fromkeys(z["stream"] for z in rows):
    print(block,*[round(median(int(z["cpu_ns"]) for z in rows if z["stream"]==block and z["arm"]==a)) for a in ("sais","rle")],sep=",")
