import hashlib,json,math,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent
sys.path.insert(0,str(P/'bench'))
import walk_stats_check as wc

def trace(s):
 r=subprocess.run([str(P/'agg3'),'mtf_avx2:1000:2000'],input=s,text=True,capture_output=True,check=True)
 return r.stdout
duplicate='I 2000,4\n S 2000100,4\nI 2000,4\n S 2000100,4\nI 1100,4\n L 100100,16\n'
dup=trace(duplicate)
assert 'D<-H=1' in dup and '[store 2000 load 1100 n=2]' in dup
s=''
for j in range(6):
 s+='I 3000,4\n'+(' S 3000800,4\n'*48)
 s+=f'I {0x2000+j:x},4\n S 2000100,4\nI {0x1100+j:x},4\n L 100100,16\n'
top=trace(s)
line=next(x for x in top.splitlines() if x.startswith('PAIRS mtf_avx2 mod4K D<-H'))
assert line.count('[store ')==5 and 'D<-H=6' in top

def ledger(helper_bytes):
 rows=[]
 for t,b in [(0,1),(1,helper_bytes[0]),(2,helper_bytes[1])]:
  rows.append(dict(kind='call',team=0,block=0,thread=t,role='leader' if t==0 else 'helper',walker='traverse',takes=int(b>0),exhausts=1,bytes=b,tail=0,fragments=0,copied=0))
 for t in (1,2):rows.append(dict(kind='share',team=0,block=0,thread=t))
 rows.append(dict(kind='summary',team=0,block=0,rows=1+sum(helper_bytes),starts=1+sum(b>0 for b in helper_bytes),seg_bytes=1+sum(helper_bytes),entered=7,joined=True,built=6))
 rows.append(dict(kind='end',records=len(rows)))
 return rows
eng={}
(P/'classes.txt').write_text('T\n')
for name,hb in [('balanced',[1,1]),('unbalanced',[1,99]),('zero_work',[0,0])]:
 rows=ledger(hb);lines=[json.dumps(x) for x in rows]
 by,bad=wc.validate(lines,{0:'T'},2);pred=wc.predicates(by,'traverse',2)
 assert not bad and all(pred[k][0] for k in 'acd')
 f=P/(name+'.jsonl');f.write_text('\n'.join(lines)+'\n')
 r=subprocess.run([sys.executable,str(P/'bench/walk_stats_check.py'),str(f),'--walker','traverse','--helpers','2','--classes',str(P/'classes.txt')],text=True,capture_output=True)
 assert r.returncode==(0 if name=='balanced' else 1)
 eng[name]={'exit':r.returncode,'stdout':r.stdout,'predicates':pred}

order='ABBA BAAB ABBA BAAB BAAB ABBA ABBA ABBA BAAB BAAB BAAB ABBA BAAB ABBA ABBA BAAB BAAB BAAB ABBA ABBA'.split()
assert len(order)==20 and order.count('ABBA')==10
Q=.51;h=.10
allocation={'min_mean_if_Bge':Q-h,'min_L_if_Bge':Q-2*h,'min_mean_if_Bplus':Q+h,'Splus_redundant_when_valid_precise_Bge':Q-2*h>0}
out={'allocation_at_min_exposure_max_halfwidth':allocation,'trace_duplicate_store':dup,'trace_six_pairs_top5':top,'class_minus_named_pair_count_witness':-1,'engagement':eng,'source_sha256':{p:hashlib.sha256((P/p).read_bytes()).hexdigest() for p in ['steps.md','combined.md','agg3.c','bench/walk_stats_check.py','bench/test_walk_stats_check.py']},'native_decoder_executed':False}
print(json.dumps(out,indent=2))
