
#[cfg(all(test,target_arch="x86_64"))]
#[test]
fn mixed_full_lanes() {
    if !(std::is_x86_feature_detected!("avx2") && std::is_x86_feature_detected!("popcnt")) {return}
    fn check<const RANK:u8>() {
        let mut c=Dispatch::<2,2,RANK>::default();
        let mut seed=0xb1729cd3_40973ad5u64;
        for p in 0..128 {
            seed^=seed<<13;
            seed^=seed>>7;
            seed^=seed<<17;
            c.starts.push(match p%8 { 0=>0, 1=>u64::MAX, 2=>1<<63, 3=>1, _=>seed });
            c.page_first.push(133+p);
        }
        for p in 0..128u32 {
            for offset in 0..64 {
                let xs=std::array::from_fn(|i|64*((p+37*i as u32)%128) + (offset+19*i as u32)%64);
                let want=xs.map(|x| {
                    let page=(x/64) as usize;
                    let rank=(0..=x%64).filter(|bit|c.starts[page] & (1<<bit) != 0).count();
                    c.page_first[page] as usize+rank
                });
                // Both features were established by the outer test.
                assert_eq!(unsafe {c.locate8(&xs)}, want);
                assert_eq!(xs.map(|x|c.locate(x)),want);
            }
        }
    }
    check::<2>();
    check::<3>();
    println!("131072 independently enumerated full-lane prefixes across two AVX2 methods");
}
