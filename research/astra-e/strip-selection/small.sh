#!/usr/bin/env bash
set -euo pipefail
d=research/astra-e/strip-selection
out=${1:?new directory}
mkdir "$out"
out=$(cd "$out" && pwd)
python3 "$d/prepare.py" "$out"
cat >> "$out/main.rs" <<'RS'
#[test]
fn forced_primary_and_swaps() {
    let mut kernels:[changed::Kernel;4]=Default::default();
    let mut old=stock::Kernel::default();
    let(mut checked,mut swaps)=(0,0);
    for i in 0..24u32 {
        let mut caps=Vec::new();let mut l=vec![];
        for j in 0..(16+i) {
            let c=((j*17+j/4+i*7)%7) as u8;
            let len=[16,17,16,18,1,22,17,19,16,17,30][(i as usize+3*j as usize)%11];
            l.resize(l.len()+len,c);
            for _ in 0..len {caps.push(u16::from(c)<<8|1);}
        }
        let mut psi:Vec<_>=(0..l.len()).collect();psi.sort_by_key(|&p|l[p]);
        for mark in (0..l.len()).step_by(5) {
            let mut p=mark;
            let want:Vec<_>=(0..l.len()).map(|_|{p=psi[p];l[p]}).collect();
            let mut out=vec![];
            let baseline=old.run::<true,true,false>(&caps,l.len(),mark as u32,16,&mut out);
            assert_eq!(out,want);
            for(mode,c) in kernels.iter_mut().enumerate() {
                let (s,_,q)=match mode {
                    0=>c.run_selected::<1>(&caps,l.len(),mark as u32,&mut out),1=>c.run_selected::<2>(&caps,l.len(),mark as u32,&mut out),
                    2=>c.run_selected::<3>(&caps,l.len(),mark as u32,&mut out),_=>c.run_selected::<4>(&caps,l.len(),mark as u32,&mut out)
                };
                assert_eq!(out,want);
                assert_eq!((s.retained,s.thick),(baseline.retained+q,baseline.thick));
                checked+=1;swaps+=usize::from(q>0);
            }
        }
    }
    println!("{checked} exact marked-policy checks, {swaps} with active swaps");assert!(swaps>0);
}
RS
deps=${PAGE_DEPS:-${CARGO_TARGET_DIR:-target}/release/deps}
rustc --test --edition=2021 -C opt-level=3 -C codegen-units=1 "$out/main.rs" --extern mibz2="$deps/libmibz2.rlib" -L "dependency=$deps" -o "$out/check"
"$out/check" forced_primary --nocapture
