
#[test]
fn embedded_codes_and_upper_band() {
    for code in 0..=65535 {
        for (end,byte,source,target) in [(1,0,0,899999), (900000,255,899999,0), (65536,34,65535,65536)] {
            let mut r=Run::new::<3>(end,byte,(target as u32).wrapping_sub(source));
            r.set_prediction(code);
            assert_eq!((r.position::<3>(),r.byte(),r.map::<3>(source),r.prediction()),(end,byte,target,code));
            assert_eq!(r.retires::<3>(end),byte==QUOTE);
            assert!(r.retires::<3>(end+1));
        }
    }
    let n=131101usize;
    let alphabet=[QUOTE,b'x',QUOTE,b'a',QUOTE,b'Z',QUOTE,b'*'];
    let ll:Vec<_>=(0..n).map(|i|alphabet[i%alphabet.len()]).collect();
    let caps:Vec<_>=ll.iter().map(|&ch|(u16::from(ch)<<8)|1).collect();
    let mut psi:Vec<_>=(0..n).collect();
    psi.sort_by_key(|&i|ll[i]);
    let mut c=Dispatch::<2,2,3>::default();
    let mut got=Vec::new();
    let mut covered=0;
    for p in [0,1,65534,65535,65536,131068,131069,131070,131100] {
        let mut at=p;
        let want:Vec<_>=(0..n).map(|_|{at=psi[at];ll[at]}).collect();
        let took=c.attempt::<true>(&caps,n,p as u32,&mut got);
        assert_eq!(took,want.contains(&QUOTE));
        if took {
            assert_eq!(got,want);
            assert_eq!(c.runs.len(),n);
            covered+=1;
        }
    }
    assert!(covered>0);
    println!("all 65536 prediction codes preserve payloads and checked 9 marks spanning both 65535-run boundaries ({covered} covered)");
}
