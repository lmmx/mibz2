"""Print all five rebuild hashes using the same compiler and target platform as the measured sources."""
import hashlib
import json
from pathlib import Path
import sys

base,published,bucket=map(Path,sys.argv[1:])
files={'base/run':base/'run','published/run':published/'run','published/audit':published/'audit',
       'bucket/run':bucket/'run','bucket/audit':bucket/'audit'}
print(json.dumps({k:hashlib.sha256(p.read_bytes()).hexdigest() for k,p in files.items()},indent=2))
