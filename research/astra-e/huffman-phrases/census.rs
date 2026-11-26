// Appended to the frozen stock block module in the observational binary only.
struct Counts {
    emitted: [u64;5],
    phase: [u64;3],
    table: [u64;5],
    slow: u64,
}
const MODES: [(&str,usize,bool,bool);5] = [
    ("pair",2,false,false),("first9-triple",3,false,false),
    ("first9-quad",4,false,false),("balanced-quad",4,true,false),("unbounded-quad",4,false,true),
];

fn phrase(ids:&[u16], lens:&[u8], limit:usize, balanced:bool, any:bool) -> usize {
    let mut bits=0;
    let mut best=1;
    for (i,&c) in ids.iter().take(limit).enumerate() {
        bits+=lens[c as usize] as usize;
        if bits>10 {break;}
        let k=i+1;
        let fits= match k {
            1|2 => true,
            3 if balanced => ids[..k].iter().all(|&v|v<128),
            4 if balanced => ids[..k].iter().all(|&v|v<32),
            3 => ids[1..k].iter().all(|&v|v<64),
            4 => ids[1..k].iter().all(|&v|v<16),
            _ => unreachable!(),
        };
        if any || fits {best=k;}
    }
    best
}

fn audit(symbols:&[u16], selectors:&[u8], lens:&[Vec<u8>], tables:&[Huff], eob:u16) {
    static BLOCK:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
    let block=BLOCK.fetch_add(1,std::sync::atomic::Ordering::Relaxed);
    for (mode,k,balanced,any) in MODES {
        let mut c=Counts{emitted:[0;5],phase:[0;3],table:[0;5],slow:0};
        for (group,t) in tables.iter().enumerate() {
            for prefix in 0..1024 {
                let mut used=0;
                let mut ids=[0u16;4];
                let mut count=0;
                while count<k && used<10 {
                    let key=(prefix<<used)&1023;
                    let e=t.lut[key];
                    if (e>>COUNT)&3==0 {break;}
                    let len = e>>LEN1&31;
                    if used+len>10 {break;}
                    let sym=((e>>SYM1)&511) as u16;
                    ids[count]=sym;
                    count+=1;
                    used+=len;
                    if sym==eob {break;}
                }
                let slot=if count==0 {0} else {phrase(&ids[..count],&lens[group],k,balanced,any)};
                c.table[slot]+=1;
            }
        }
        for (g,ids) in symbols.chunks(GROUP).enumerate() {
            let lengths=&lens[selectors[g] as usize];
            let mut pos=0;
            let mut step = |pos:&mut usize, phase:usize| {
                let word=if phase==2 {1} else {phrase(&ids[*pos..],lengths,k,balanced,any)};
                if lengths[ids[*pos]as usize]>10 {c.slow+=1;}
                c.emitted[word]+=1;
                c.phase[phase]+=1;
                *pos+=word;
            };
            while pos<ids.len() && GROUP-pos>=k*STEPS {
                for _ in 0..STEPS {
                    step(&mut pos,0);
                    if pos==ids.len() {break;}
                }
            }
            while pos<ids.len() && GROUP-pos>=k {step(&mut pos,1);}
            while pos<ids.len() {step(&mut pos,2);}
        }
        assert_eq!(symbols.len() as u64,(1..=4).map(|i|i as u64*c.emitted[i]).sum::<u64>());
        println!("{block},{mode},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",tables.len(),symbols.len(),c.phase[0],c.phase[1],c.phase[2],c.slow,c.emitted[1],c.emitted[2],c.emitted[3],c.emitted[4],c.table[0],c.table[1],c.table[2],c.table[3],c.table[4],symbols.len().div_ceil(50));
    }
}
