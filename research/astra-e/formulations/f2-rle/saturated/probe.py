"""Extend the independent dense oracle to count the least saturated return section."""
from pathlib import Path
import sys
here=Path(__file__).resolve().parent
base=here.parent
s=(base/'probe.rs').read_text()
s=s.replace('"section.rs"',f'"{base/"section.rs"}"')
i=s.index('    text.truncate(n);')
s=s[:i]+'''    let mut kept=vec![false;n]; kept[p]=true;
    let mut at=0;
    while at<n {
        let mut end=at+1;
        while end<n && ll[end]==ll[at] {end+=1;}
        if end-at<=16 {kept[at..end].fill(true);}
        at=end;
    }
    let mut lf=vec![0;n];for x in 0..n{lf[psi[x]]=x;}
    let (mut probes,mut off)=(0,0);
    let mut owners=vec![false;n];
    for &start in &h.danger {
        let mut y=start as usize;
        while !kept[y] {
            y=lf[y];probes+=1;
            if y==start as usize{off+=1;break;}
        }
        if kept[y]{owners[y]=true;}
    }
    let mut sat=vec![false;n];
    let mut longest=0;
    for s in 0..n {
        if !owners[s]{continue;}
        let mut y=s;let mut len=0;
        loop {sat[y]=true;len+=1;y=psi[y];if kept[y]{break;}}
        longest=longest.max(len);
    }
    x=p;
    let mut travelled=0;
    for i in 0..n {
        assert!(!hits[i].1 || sat[x]);
        travelled+=usize::from(sat[x]);
        hits[i].1=sat[x]; x=psi[x];
    }
    if n>65536 {
        println!("{p},{},{},{},{},{probes},{off},{longest}",h.danger.len(),owners.iter().filter(|&&b|b).count(),sat.iter().filter(|&&b|b).count(),travelled);
    }
'''+s[i:]
i=s.index('    println!("stream,n,admitted')
j=s.index('    while let Ok(',i)
s=s[:i]+'''    println!("primary,danger,owners,saturated_rows,marked_saturated,rewind_steps,off_section,longest_owned_word");
'''+s[j:]
i=s.rindex('        println!(')
j=s.index('\n    }',i)
s=s[:i]+s[j:]
Path(sys.argv[1]).write_text(s)
