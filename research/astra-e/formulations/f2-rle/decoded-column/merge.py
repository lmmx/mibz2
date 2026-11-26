"""Compile the same original and decoded dense oracle around the sparse order screen."""
from pathlib import Path
import sys
d=Path(__file__).resolve().parent
s=(d/"probe.rs").read_text()
insert='''    let (exceptional,comparisons,visited,anchors,longest,gaps,new_runs)=merge(&text,&corrected,&row,&mapped,&vulnerable,&groups,&new_rank);
    let original_runs=1+l.windows(2).filter(|s|s[0]!=s[1]).count();
    println!("{index},{n},{exceptional},{comparisons},{visited},{anchors},{longest},{gaps},{original_runs},{new_runs}");
'''
a=s.index("    let v = vulnerable.iter()")
b=s.index("\n}",a)
s=s[:a]+insert+s[b:]
a=s.index('    println!(\n        "stream,') if '    println!(\n        "stream,' in s else s.index('    println!("stream,')
b=s.index(";",a)
s=s[:a]+'    println!("stream,n,exception_rows,comparisons,byte_pairs,anchor_exits,longest_comparison,insertion_gaps,original_runs,new_runs")'+s[b:]
Path(sys.argv[1]).write_text((d/"merge.inc.rs").read_text()+s)
