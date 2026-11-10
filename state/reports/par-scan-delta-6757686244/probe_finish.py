"""Independent control-flow checks; no unkillable processes."""
import sys,json,subprocess
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).parent/'bench'))
import bounded
class Child:
    pid=123456
    calls=[]
    def wait(self,timeout=None):
        self.calls.append(timeout)
        assert timeout is not None
        raise subprocess.TimeoutExpired('synthetic',timeout)
out={}
with patch.object(bounded,'members',return_value=[123457]):
    for code in [None,124,143]:
        c=Child();c.calls=[]
        out[str(code)]={'failed_cleanup':bounded.finish(c,c.pid,False,code),'wait_calls':list(c.calls)}
        assert out[str(code)]['failed_cleanup']==125 and c.calls==[]
        out[str(code)]['unreapable']=bounded.finish(c,c.pid,True,code)
        assert out[str(code)]['unreapable']==125 and c.calls==[10.0]
print(json.dumps(out,indent=2))
