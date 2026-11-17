import pathlib,subprocess,json,math,statistics,importlib.util
root=pathlib.Path(__file__).resolve().parent
common=['--main','wikidata-x4','--guard','text','--rounds','10','--cores','10','--cpus','20']
args={
 'window':['--ref','mibz2','--cand','mibz2-occ-8-32','--cand','mibz2-occ-9-72','--cand','mibz2-occ-9-18','--judged','12,16,20','--hold-max-threads','0'],
 'low':['--ref','mibz2-occ-8-32','--cand','mibz2-occ-7-28','--cand','mibz2-occ-6-24','--context','mibz2','--judged','12,16,20','--hold-max-threads','0'],
 'cores':['--ref','mibz2','--cand','mibz2-occ-9-36','--cand','mibz2-occ-8-32','--cand','mibz2-occ-7-28','--cand','mibz2-occ-6-24','--judge-at','mibz2-occ-9-36=10','--judge-at','mibz2-occ-8-32=10','--judge-at','mibz2-occ-7-28=8,10','--judge-at','mibz2-occ-6-24=8','--judged','8,10','--hold-max-threads','9']}
for n,a in args.items():
 p=next((root/n).glob('run*'));source=root/f'source-{n}'/'bench'
 cmd=['python',str(source/'two_arm.py'),str(p/'judged.jsonl'),'--env',str(p/'env.json'),*common,*a]
 r=subprocess.run(cmd,text=True,capture_output=True);assert r.returncode==0,r.stderr+r.stdout
 assert r.stdout==(p/'verdict.txt').read_text(),n
 (root/f'{n}-replayed.txt').write_text(r.stdout)
 r=subprocess.run(['python',str(source/'test_two_arm.py')],text=True,capture_output=True)
 print(n,'exact replay',r.returncode,r.stdout[-1500:],r.stderr[-1000:]);assert r.returncode==0
print('All three verdicts byte-identical to archives.')
