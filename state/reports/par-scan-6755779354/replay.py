"""Untimed reporter checks at eed6cc7705c329ed685c00411cf2330e11b95752.
Place beside par_scan_knee.py and test_par_scan_knee.py from that commit.
"""
import json
import tempfile
from pathlib import Path
import test_par_scan_knee as t

for test in (t.test_readings, t.test_figures, t.test_admission):
    with tempfile.TemporaryDirectory() as d:
        test(Path(d))
with tempfile.TemporaryDirectory() as d:
    r,v=t.run(Path(d),t.grid(mutate=lambda rows:[x.update(bytes=1) for x in rows]))
    assert r.returncode==0
    result={'existing_tests':'3 passed','uniform_wrong_size':{'exit':r.returncode,'reading':v['reading']},
            'native_rust_execution':False}
print(json.dumps(result,indent=2))
