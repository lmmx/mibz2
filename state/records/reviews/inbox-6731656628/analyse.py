import csv,json,math,statistics as st,hashlib,pathlib
T=2.1447866879169273
def ci(v):
 m=st.mean(v);h=T*st.stdev(v)/math.sqrt(len(v));return [m,m-h,m+h]
def ratio(a,b):
 if any(x<=0 or y<=0 for x,y in zip(a,b)):return None
 return [math.exp(x) for x in ci([math.log(x/y) for x,y in zip(a,b)])]
result={}
for p in sorted(pathlib.Path('pair-raw').glob('raw-*.csv')):
 groups={}
 for x in csv.DictReader(p.open()):
  g=groups.setdefault(x['file'],{});r=g.setdefault(int(x['round']),{});assert x['arm'] not in r;r[x['arm']]=float(x['seconds']);r['out']=int(x['out_bytes']);r['rows']=int(x['paired_rows'])
 out={}
 for name,g in groups.items():
  assert sorted(g)==list(range(15))
  rs=[g[i] for i in range(15)];assert all(set(r)==set(['row','run','one','P','R','O','out','rows']) for r in rs)
  v=lambda a:[r[a] for r in rs]
  item={k:ratio(v(a),v(b)) for k,a,b in [('Q1','run','row'),('Q2','R','P'),('run_one','run','one')]}
  item['residuals']={name:ci([(r[a]-r[b])*3.3091e9/r['out'] for r in rs]) for name,a,b in [('R_minus_run','R','run'),('O_minus_row','O','row'),('P_minus_one','P','one')]}
  item['residual_gap']=ci([((r['R']-r['run'])-(r['P']-r['one']))*3.3091e9/r['out'] for r in rs])
  item['build_extra_per_row']=ci([(r['run']-r['one'])*3.3091e9/r['rows'] for r in rs]) if rs[0]['rows'] else None
  item['counts']={'rounds':15,'paired_rows':rs[0]['rows'],'out_bytes':rs[0]['out']}
  out[name]=item
 result[p.name]=out
pathlib.Path('pair-analysis.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result['raw-1.csv'],indent=2))
