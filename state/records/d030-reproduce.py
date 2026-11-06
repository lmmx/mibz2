import json, pathlib, tempfile, subprocess, sys
import test_team_size as t
P='mibz2-team4@12=3x4,mibz2-team3@10=3x3'
t.FAST={('mibz2-team4',12):.14,('mibz2-team3',10):.14}
results={}
for case in ['clean','duplicate control round','missing cell','two output lengths','failed run','duplicate new primary','wrong new helpers','cpu cap','rss cap']:
 rows=t.grid(case)
 target=next(r for r in rows if (r['arm'],r['threads'],r['round'])==('mibz2-team4',12,3))
 if case=='duplicate new primary':rows.append(target.copy())
 if case=='wrong new helpers':target['thread_cpu']=t.thread_cpu(3,3)
 if case in ['cpu cap','rss cap']:
  for r in rows:
   if (r['arm'],r['threads'])==('mibz2-team4',12):r['cpu' if case=='cpu cap' else 'rss_kb']=2 if case=='cpu cap' else 100000
 with tempfile.TemporaryDirectory() as tmp:
  p=pathlib.Path(tmp);(p/'r.jsonl').write_text(''.join(json.dumps(r)+'\n' for r in rows))
  subprocess.run([sys.executable,str(pathlib.Path(__file__).with_name('team_size.py')),'report',str(p/'r.jsonl'),'--primary',P,'--out',str(p/'out')],check=True,capture_output=True)
  v=json.loads((p/'out.json').read_text())['verdict'];results[case]=v
  ps=v['primaries']; flags=[x['passes'] for x in ps.values()]
  if case=='clean':assert all(flags)
  elif case in ['wrong new helpers']:assert flags==[False,True]
  elif case in ['cpu cap','rss cap']:assert all(flags) and not ps['mibz2-team4@12']['within_adoption_limits']
  else:assert not any(flags)
pathlib.Path(__file__).with_name('results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps({k:{'passes':{c:p['passes'] for c,p in v['primaries'].items()},'limits':{c:p.get('within_adoption_limits') for c,p in v['primaries'].items()},'control_flagged':v['control_flagged']} for k,v in results.items()},indent=2))
