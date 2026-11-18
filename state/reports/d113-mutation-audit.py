"""Demonstrate the submitted mutation scorer's false-positive exit classification."""
import os, pathlib, runpy, subprocess, tempfile
from unittest.mock import patch
root=pathlib.Path.cwd()
with tempfile.TemporaryDirectory(dir=root) as d:
    p=pathlib.Path(d);(p/'src').mkdir()
    (p/'src/cells_owned.rs').write_bytes((root/'cells_owned.rs').read_bytes())
    os.chdir(p)
    result=subprocess.CompletedProcess([],101,'','error: dependency download/build failed; no tests ran')
    with patch('subprocess.run',return_value=result):
        try:runpy.run_path(str(root/'d113-mutator.py'),run_name='__main__')
        except SystemExit as e:print('scorer exit:',e.code)
