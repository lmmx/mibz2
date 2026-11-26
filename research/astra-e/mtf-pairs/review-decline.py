"""Encode the exact dense-decline bitstream and recover its output independently with libbzip2."""
import bz2
import hashlib
import sys
from pathlib import Path

raw=Path(sys.argv[1]).read_bytes()
crc=int(sys.argv[2],16)
def bits(x,n):
    assert 0<=x<1<<n
    return format(x,'0'+str(n)+'b')

# Two used bytes, two complete four-symbol tables, 5,601 selectors selecting group zero.
prefix=bits(0x314159265359,48)+bits(crc,32)+'0'+bits(0,24)+bits(0x8000,16)+bits(0xc000,16)
groups=bits(2,3)+bits(5601,15)+'0'*5601+(bits(2,5)+'0'*4)*2
body='1000'*140000+'11'
payload=prefix+groups+body+bits(0x177245385090,48)+bits(crc,32)
payload+='0'*(-len(payload)%8)
stream=b'BZh9'+int(payload,2).to_bytes(len(payload)//8,'big')
Path(sys.argv[3]).write_bytes(stream)
assert bz2.decompress(stream)==raw
print('libbzip2=PASS input_bytes=%d input_sha256=%s decoded_sha256=%s' % (
    len(stream),hashlib.sha256(stream).hexdigest(),hashlib.sha256(raw).hexdigest()))
