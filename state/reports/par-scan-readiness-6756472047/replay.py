"""Execute the seven fetched tests without installing pytest; reproduce fixture behavior."""
import contextlib, importlib, json, pathlib, re, sys, tempfile, types
from unittest.mock import patch

@contextlib.contextmanager
def raises(cls, match):
    try:
        yield
    except cls as e:
        assert re.search(match,str(e)), (match,str(e))
    else:
        raise AssertionError('expected '+cls.__name__)

sys.modules['pytest']=types.SimpleNamespace(raises=raises, fixture=lambda **kw:lambda f:f)
results=[]
for module in ('test_par_scan_knee','test_par_scan_inputs'):
    m=importlib.import_module(module)
    for name in vars(m).copy():
        if not name.startswith('test_'):continue
        with tempfile.TemporaryDirectory() as td,contextlib.ExitStack() as stack:
            p=pathlib.Path(td)
            if hasattr(m,'_mkdirs'):m._mkdirs(p)
            if name=='test_level1_must_match':
                mp=types.SimpleNamespace(setattr=lambda obj,key,val:stack.enter_context(patch.object(obj,key,val)))
                getattr(m,name)(p,mp)
            else:getattr(m,name)(p)
        results.append(module+'.'+name)
print(json.dumps({'passed':results,'count':len(results),'pytest_installed':False},indent=2))
