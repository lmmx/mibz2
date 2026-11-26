from pathlib import Path
import subprocess,sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,here.parent/'crc-phrases/prepare.py',out],check=True)
key=out/'frozen/research/astra-e/narrow-short/constructor_ahead.rs'
with key.open('a') as f:f.write('\n'+(here/'census.inc').read_text())
(out/'main.rs').write_text(f'#![allow(dead_code)]\n#[path="{key}"] mod kernel;\n'+(here/'main.rs').read_text())
