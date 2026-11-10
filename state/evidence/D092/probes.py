import subprocess,json,re,math,statistics,hashlib
from pathlib import Path
p=Path(__file__).resolve().parent
def run(stores):
    trace=''.join(f'I  {pc:x},4\n S {addr:x},4\n' for pc,addr in stores)+'I  400080,4\n L 101000,16\n'
    r=subprocess.run([str(p/'agg3'),'f:400000:400100'],input=trace,text=True,capture_output=True,check=True)
    return {'trace':trace,'stdout':r.stdout}
out={}
out['repeated_named_pc']=run([(0x400000,0x3000),(0x400000,0x5000)])
assert 'D<-H=1' in out['repeated_named_pc']['stdout'] and '[store 400000 load 400080 n=2]' in out['repeated_named_pc']['stdout']
out['named_and_other_overlap']=run([(0x400000,0x3000),(0x400004,0x5000)])
assert 'D<-H=1' in out['named_and_other_overlap']['stdout']
out['six_pairs_top5']=run([(0x400000+4*i,0x3000+0x1000*i) for i in range(6)])
line=next(l for l in out['six_pairs_top5']['stdout'].splitlines() if 'mod4K' in l)
assert line.count('[store')==5
order='ABBA BAAB ABBA BAAB BAAB ABBA ABBA ABBA BAAB BAAB BAAB ABBA BAAB ABBA ABBA BAAB BAAB BAAB ABBA ABBA'.split()
out['order']={'ABBA':order.count('ABBA'),'BAAB':order.count('BAAB'),'slots':len(order)*4}
assert out['order']=={'ABBA':10,'BAAB':10,'slots':80}
def rule(v,l,u,req=.51):
    h=(u-l)/2
    if not v:return 'INVALID'
    if h>.10:return 'IMPRECISE'
    if u<req:return 'NEXT'
    if l>0:return 'CONTRACT_QUALIFICATION'
    return 'UNRESOLVED_NEXT'
out['decisions']={}
for name,v,l,u in [('invalid',False,.6,.7),('imprecise',True,.4,.8),('positive_below',True,.2,.39),('touch_budget',True,.4,.51),('supports_budget',True,.51,.6),('null',True,-.05,.05)]:
    out['decisions'][name]=rule(v,l,u)
out['s_limit']=.1*math.sqrt(20)/2.0930240544
out['rounding_ns_per_byte']=.0001*1e9/8000000
out['cancellation_interval_upper']=.05+2.0930240544*statistics.stdev([.04,.06]*10)/math.sqrt(20)
(p/'probes.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
