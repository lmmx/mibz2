/// Count cap-127 records from production's literal/RUNB bitmaps. The literal bitmap has
/// no set bits at or after EOB and each bitmap has two trailing zero words.
pub fn capped_count(literal:&[u64],one:&[u64],end:usize)->Option<(usize,usize)>{
    let words=end/64+1;
    let mut caps=literal[..words].iter().map(|w|w.count_ones()as usize).sum::<usize>();
    let first=literal[..words].iter().enumerate().find_map(|(j,&w)|(w!=0).then(||j*64+w.trailing_zeros()as usize)).unwrap_or(end);
    if first>21{return None;}
    if first!=0 {
        let mask=(1u64<<first)-1;
        let opening=mask+(one[0]&mask);
        caps+=opening.div_ceil(127) as usize;
    }
    let mut exceptional=0;
    for j in 0..words{
        let start=64*j;
        let mut lit=u128::from(literal[j])|(u128::from(literal[j+1])<<64);
        if end-start<128{lit|=1u128<<(end-start);}
        let mut upcoming=lit>>1;
        // Set bit q if any of the seven following symbols is a literal.
        upcoming|=upcoming>>1;
        upcoming|=upcoming>>2;
        upcoming|=(lit>>5)|(lit>>6)|(lit>>7);
        let mut candidates=literal[j]&!(upcoming as u64);
        let bits=u128::from(one[j])|(u128::from(one[j+1])<<64);
        while candidates!=0{
            let bit=candidates.trailing_zeros();candidates&=candidates-1;
            let t=(lit>>(bit+1)).trailing_zeros();
            if t>21{return None;}
            let extra=(1u128<<t)-1+((bits>>(bit+1))&((1u128<<t)-1));
            caps+=(extra/127)as usize;exceptional+=1;
        }
    }
    Some((caps,exceptional))
}
