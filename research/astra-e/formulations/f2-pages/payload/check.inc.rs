#[test]
fn packed_lf_and_boundaries() {
    assert_eq!(size_of::<Run>(),8);
    assert_eq!(align_of::<Run>(),8);
    let mut c=Candidate::default();
    for n in [1usize,31,64,65,358,900000] {
        for widths in [[1usize,17,33,67],[900000,1,1,1],[8189,127,128,25000],[1,1,1,1]] {
            let (mut left,mut i)=(n,0);
            let (mut caps,mut counts)=(Vec::new(),[0u32;256]);
            while left>0 {
                let b=[QUOTE,5,0,255][i%4];
                let len=widths[i%4].min(left);
                counts[b as usize]+=len as u32;
                i+=1;
                left-=len;
                let mut rem=len;
                while rem>0 {
                    let size=rem.min(127);
                    caps.push(u16::from(b)<<8|size as u16);
                    rem-=size;
                }
            }
            let mut cumulative=0;
            let mut lf=counts.map(|v| {let start=cumulative;cumulative+=v;start});
            c.prepare_counted(&caps,n as u32,&counts).unwrap();
            let mut x=0u32;
            for record in caps {
                let b=(record>>8) as u8;
                for _ in 0..record&255 {
                    let k=c.locate(x);
                    let r=c.runs[k];
                    assert_eq!(r.byte(),b);
                    assert_eq!(r.map(x),lf[b as usize]);
                    assert!(BOUNDARY_CONDITION);
                    lf[b as usize]+=1;
                    x+=1;
                }
            }
            assert_eq!(x,n as u32);
        }
    }
}
