import hashlib,json,os,subprocess,time
from pathlib import Path
root=Path('/workspace/mibz2'); out=root/'docs/journal/2026-10-31-p74-c-exploratory'
assert not (out/'raw.tsv').exists(), 'refusing a second run'
files=Path('/workspace/work/p74/experiment/files.txt').read_text().splitlines()
exe=root/'target/release/examples/p74_experiment'
def get(*args): return subprocess.check_output(args,text=True).strip()
def snapshot():
 return {name:(Path(path).read_text() if Path(path).exists() else "unavailable") for name,path in {'cpu_stat':'/sys/fs/cgroup/cpu.stat','quota':'/sys/fs/cgroup/cpu.max','proc_stat':'/proc/stat','cpu_pressure':'/proc/pressure/cpu'}.items()}
record={'code_commit':get('git','rev-parse','HEAD'),'working_tree_before':get('git','status','--porcelain'),'compiler':get('rustc','-Vv'),'cpu':get('lscpu','-J'),'topology':get('lscpu','-e=CPU,CORE,SOCKET,NODE'),'parent_affinity':sorted(os.sched_getaffinity(0)),'child_affinity':[0],'host':get('hostname'),'binary_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'guard_exception':'codex-epyc9v74-single-core-2026-10-31','frozen_manifest_sha256':hashlib.sha256((root/'docs/journal/2026-10-31-p74-c-validation/clean-preparation-manifest.json').read_bytes()).hexdigest(),'processes_before':get('ps','-eo','pid,ppid,comm,psr,pcpu')}
record['corpora']=[{'path':f,'sha256':hashlib.sha256(Path(f).read_bytes()).hexdigest()} for f in files]
cmd=['taskset','-c','0',str(exe),'--measure-approved',*files]; record['command']=cmd
record['before']=snapshot()
(out/'environment-before.json').write_text(json.dumps(record,indent=2)+'\n')
env=os.environ.copy(); env['P74_APPROVED_RUNNER']='codex-epyc9v74-cpu0-exploratory';env['P74_EXPLORATORY_HOST_EXCEPTION']=record['guard_exception']
start=time.monotonic_ns()
with (out/'raw.tsv').open('x') as stdout, (out/'stderr.txt').open('x') as stderr:
 p=subprocess.run(cmd,stdout=stdout,stderr=stderr,env=env)
record['wall_ns']=time.monotonic_ns()-start
record['after']=snapshot();record['exit_code']=p.returncode
record['processes_after']=get('ps','-eo','pid,ppid,comm,psr,pcpu')
(out/'environment.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({'exit_code':p.returncode,'wall_seconds':record['wall_ns']/1e9,'rows':sum(l.startswith('ROW\t') for l in (out/'raw.tsv').read_text().splitlines())}))
raise SystemExit(p.returncode)
