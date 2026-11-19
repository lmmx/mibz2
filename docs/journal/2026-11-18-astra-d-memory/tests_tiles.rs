#[cfg(test)]
mod astra_tile_tests {
    use super::*;
    fn check(l:&[u8]) {
        let mut recs=Vec::new();
        let mut counts=[0;256];
        // Physical fragments need not have maximum length.
        for &c in l {recs.push((u16::from(c)<<8)|1); counts[c as usize]+=1;}
        let mut table=AstraTiles::default();
        let ends=table.build(&recs,&counts,l.len() as u32);
        let mut order: Vec<_>=(0..l.len()).collect();
        order.sort_by_key(|&i| l[i]);
        for (r,&i) in order.iter().enumerate() {
            assert_eq!(table.step(r as u32,&ends),(l[i],i as u32));
        }
    }
    #[test]
    fn astra_all_rows_and_cycles() {
        for n in 1..=8 {
            for mut k in 0..3usize.pow(n) {
                let l:Vec<_>=(0..n).map(|_|{let c=(k%3) as u8;k/=3;c}).collect();
                check(&l);
            }
        }
        for n in [127,128,129,255,256,257,4095] {
            check(&vec![255;n]);
            check(&(0..n).map(|i| ((i*73 + i/128)%256) as u8).collect::<Vec<_>>());
        }
    }
}
