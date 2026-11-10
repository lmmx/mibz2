"""Control-flow probe only: no unkillable process or decoder timing is created."""
import json, signal
from unittest.mock import patch
import bounded

class UnboundedWait(Exception): pass
class Child:
    pid = 123456
    def poll(self): return None
    def wait(self, timeout=None):
        if timeout is None: raise UnboundedWait('wait called without timeout after failed cleanup')
        raise AssertionError('unexpected bounded wait')

out = {}
with patch.object(bounded.subprocess, 'Popen', return_value=Child()), patch.object(bounded.signal, 'signal'), patch.object(bounded, 'stop_group', return_value=False):
    try: out['deadline_cleanup_failure'] = bounded.run(0, 0, ['synthetic'])
    except UnboundedWait as e: out['deadline_cleanup_failure'] = str(e)

class ExitedChild(Child):
    def wait(self, timeout=None): return -15
def interrupt(sig, handler):
    if sig == signal.SIGHUP: handler(sig, None)
with patch.object(bounded.subprocess, 'Popen', return_value=ExitedChild()), patch.object(bounded.signal, 'signal', side_effect=interrupt), patch.object(bounded, 'stop_group', return_value=False):
    out['signal_cleanup_failure'] = bounded.run(1, 0, ['synthetic'])
assert 'without timeout' in out['deadline_cleanup_failure']
assert out['signal_cleanup_failure'] == 143
print(json.dumps(out, indent=2))
