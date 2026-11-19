from pathlib import Path
import subprocess,sys
d=Path(__file__).resolve().parent
p=Path(sys.argv[1])
subprocess.run([sys.executable,str(d/"merge.py"),str(p)],check=True)
s=p.read_text()
s=(d/"memo.inc.rs").read_text()+s[s.index("// Search monotonically"):];
s=s.replace("-> (usize,u64,u64,u64,usize,usize,usize)","-> (usize,u64,u64,u64,usize,usize,usize,u64,usize)")
s=s.replace("chunks,bwt_runs)","chunks,bwt_runs,visits.hits,visits.memo.capacity())")
s=s.replace("gaps,new_runs)=merge","gaps,new_runs,hits,capacity)=merge")
s=s.replace("original_runs},{new_runs}","original_runs},{new_runs},{hits},{capacity}")
s=s.replace("original_runs,new_runs\")","original_runs,new_runs,hits,memo_capacity\")")
p.write_text(s)
