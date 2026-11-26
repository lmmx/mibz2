"""Read only the development block's exported first-token histogram to freeze the family."""
import json
from collections import Counter
from pathlib import Path
import sys
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[2]/"f2c"))
from second import read

n,p,skip,bases,words,col=read(Path(sys.argv[1])/"1.token")
freq=Counter()
for i,w in col: freq[i]+=w
top=sorted(range(len(words)),key=lambda i:(-freq[i],bases[i]))[:8]
print(json.dumps(dict(development_stream=1,fixed_parent_hex="22",reference_token_hex="3a22",
                     ordered_tokens=[dict(hex=words[i].hex(),count=freq[i],source_rank=bases[i]) for i in top]),
                 indent=2))
