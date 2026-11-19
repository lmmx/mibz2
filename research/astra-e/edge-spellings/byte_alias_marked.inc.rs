fn main() {
    let (mut base,mut changed)=(old::Kernel::default(),alias::Kernel::default());
    let (mut wanted,mut got)=(Vec::new(),Vec::new());
    let (mut count,mut active,mut periodic,mut wide)=(0,0,0,0);
    let mut test=|column:&[u8],primary:usize| {
        let rec=records(column);
        let b=base.run::<true,true,false>(&rec,column.len(),primary as u32,16,&mut wanted);
        let c=changed.run::<true,true,false>(&rec,column.len(),primary as u32,16,&mut got);
        assert_eq!(got,reference(column,primary));
        assert_eq!(got,wanted);
        assert_eq!((b.retained,b.thick,b.bands),(c.retained,c.thick,c.bands));
        assert!(c.words<=b.words && c.table_bytes<=b.table_bytes);
        if c.words<b.words {
            active+=1;
            let mut a=[0;256];
            for &v in column {a[v as usize]+=1usize;}
            let mut z=0;
            for x in &mut a {let w=*x;*x=z;z+=w;}
            let mut f=vec![0;column.len()];
            for (i,&v) in column.iter().enumerate() {
                f[a[v as usize]]=i;a[v as usize]+=1;
            }
            let mut row=f[primary];
            let mut steps=1;
            while row!=primary {row=f[row];steps+=1;}
            periodic+=usize::from(steps<column.len());
        }
        count+=1;
        wide+=usize::from(b.thick>(u16::MAX as usize-3)/3);
    };
    let mut seed=0xcaba91u32;
    let mut step=|| {seed^=seed<<13;seed^=seed>>17;seed^=seed<<5;seed};
    for _ in 0..256 {
        let mut l=Vec::new();
        for _ in 0..128 {
            let width=[1,2,4,8,16,17,18,32,64,127,128][step() as usize % 11];
            l.extend(std::iter::repeat_n((step()%4) as u8,width));
        }
        for p in [0,l.len()-1,(step() as usize)%l.len()] {test(&l,p);}
    }
    for h in [21_844,21_845,100,21_846] {
        let mut l=vec![2];
        for j in 0..h {l.extend(std::iter::repeat_n((j&1) as u8,17));}
        test(&l,0);
    }
    assert!(active>0 && periodic>0 && wide==2);
    println!("{count} complete marked comparisons with the frozen decoder and dense stable-LF oracle; {active} reduced word codes, {periodic} active comparisons had a marked cycle shorter than n, {wide} used wide endpoint nodes.");
}
