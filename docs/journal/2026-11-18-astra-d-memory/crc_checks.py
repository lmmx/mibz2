"""Both independent CRC errors must reach their named public failure, on all arms."""
from pathlib import Path
import subprocess
import sys
import tempfile

envelope = Path(sys.argv[1]).read_bytes()
with tempfile.TemporaryDirectory() as temp:
    d = Path(temp)
    bits=f"{int.from_bytes(envelope,'big'):0{8*len(envelope)}b}"
    end=bits.rindex(f"{0x177245385090:048b}")
    for kind, bit in (("BlockCrc", 80), ("StreamCrc", end+48)):
        broken=bytearray(envelope)
        broken[bit//8] ^= 1<<(7-bit%8)
        (d/"input").write_bytes(broken)
        for exe in map(Path,sys.argv[2:]):
            result=subprocess.run([exe.resolve(),"input","output","inverse"],cwd=d,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            assert result.returncode and kind in result.stderr.decode(), (exe,kind,result.stderr)
            print(exe.name,kind)
