"""Execute the source's test functions using stdlib fixture substitutes (pytest unavailable)."""
import importlib.util, inspect, tempfile, pathlib, signal, contextlib, json
from unittest.mock import patch

results=[]
for filename in ['test_bounded.py','test_par_scan_finish.py']:
    spec=importlib.util.spec_from_file_location(filename[:-3],pathlib.Path(__file__).parent/'bench'/filename)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    for name,fn in vars(module).items():
        if not name.startswith('test_'):continue
        old={s:signal.getsignal(s) for s in [signal.SIGTERM,signal.SIGINT,signal.SIGHUP]}
        try:
            with tempfile.TemporaryDirectory() as d, contextlib.ExitStack() as stack:
                class Monkey:
                    def setattr(self,obj,key,value,raising=True):
                        stack.enter_context(patch.object(obj,key,value,create=not raising))
                fixtures={'tmp_path':pathlib.Path(d),'monkeypatch':Monkey()}
                fn(**{k:fixtures[k] for k in inspect.signature(fn).parameters})
                results.append({'test':name,'result':'PASS'})
        finally:
            for s,h in old.items():signal.signal(s,h)
print(json.dumps(results,indent=2))
