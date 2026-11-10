"""Untimed liveness probe of the recipe's exact deadline construction; clean up our processes."""
import json, os, pathlib, signal, subprocess, sys, tempfile, time

def live(pid):
    try:
        return pathlib.Path(f'/proc/{pid}/stat').read_text().split()[2] != 'Z'
    except FileNotFoundError:
        return False

results=[]
with tempfile.TemporaryDirectory() as td:
    for ignores in (False, True):
        path=pathlib.Path(td)/str(ignores)
        child='import signal,time,os,pathlib; '
        if ignores:
            child+='signal.signal(signal.SIGTERM, signal.SIG_IGN); '
        child+='pathlib.Path(os.environ["PIDFILE"]).write_text(str(os.getpid())); time.sleep(30)'
        script='"$PYTHON" -c "$CHILD" & wait'
        env=dict(os.environ, PYTHON=sys.executable, CHILD=child, PIDFILE=str(path))
        p=subprocess.Popen(['setsid','--wait','timeout','--kill-after=1','1','bash','-c',script],env=env)
        pid=None
        try:
            status=p.wait(timeout=5)
            pid=int(path.read_text())
            # Give timeout's advertised escalation interval time to expire.
            time.sleep(1.2)
            results.append(dict(ignores_term=ignores,exit=status,grandchild_alive=live(pid)))
        finally:
            if pid is not None and live(pid):
                os.kill(pid,signal.SIGKILL)
            if p.poll() is None:
                p.kill();p.wait()
print(json.dumps(results,indent=2))
