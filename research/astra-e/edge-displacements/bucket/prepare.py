"""Build the isolated scorer entry from the frozen edge decoder and queue transform."""
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile

from native import apply

here=Path(__file__).resolve().parent
root=here.parents[3]
out=here/'target'
out.mkdir(exist_ok=True)
predecessor='badb2aec3624bd65606c429b2e6f79e9d9d0050f'
files=['docs/journal/2026-11-18-open-return','research/astra-e/narrow-short',
       'research/astra-e/narrow-endpoint','research/astra-e/short-words',
       'research/astra-e/lookahead','research/astra-e/edge-budget']
with tempfile.TemporaryDirectory() as tmp:
    frozen=Path(tmp)
    archive=subprocess.Popen(['git','-C',root,'archive',predecessor,*files],stdout=subprocess.PIPE)
    subprocess.run(['tar','-x','-C',frozen],stdin=archive.stdout,check=True)
    assert archive.wait()==0
    d=frozen/'research/astra-e'
    subprocess.run([sys.executable,d/'edge-budget/prepare.py'],check=True)
    kernel=(d/'narrow-short/constructor_ahead.rs').read_text()
    assert hashlib.sha256(kernel.encode()).hexdigest()=='c0ccf8939b8cdf02ac4c8157c588d980afa378805fcb37498ca3cda0ccbc6d4f'
    for old,new in [('../short-words/endpoint.rs','wide.rs'),('../../../docs/journal/2026-11-18-open-return/segments.rs','segments.rs')]:
        needle=f'#[path = "{old}"]'
        assert kernel.count(needle)==1
        kernel=kernel.replace(needle,f'#[path = "{new}"]')
    (out/'kernel.rs').write_text(apply(kernel))
    for source,name in [(d/'short-words/endpoint.rs','wide.rs'),(d/'narrow-short/narrow.rs','narrow.rs'),
                        (frozen/'docs/journal/2026-11-18-open-return/segments.rs','segments.rs'),
                        (d/'edge-budget/gate.rs','gate.rs'),(d/'edge-budget/adapter.inc.rs','adapter.inc.rs')]:
        (out/name).write_bytes(source.read_bytes())
