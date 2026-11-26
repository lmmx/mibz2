// Width 1..254 occupies the low byte. Tag 0 precedes a full u16 and tag 255 a u32.
pub fn append(buf:&mut [u16], at:&mut usize, byte:u8, width:u32) {
    let tag = match width { 1..=254 => width as u16, 255..=65535 => 0, _ => 255 };
    assert_ne!(width,0);
    buf[*at] = u16::from(byte)<<8 | tag;
    *at += 1;
    if tag==0 || tag==255 {buf[*at]=width as u16;*at+=1;}
    if tag==255 {buf[*at]=(width>>16)as u16;*at+=1;}
}

pub fn take(buf:&[u16], at:&mut usize)->(u8,u32) {
    let h=buf[*at];*at+=1;
    let tag=h&255;
    let width=match tag {
        0 => {let w=buf[*at];*at+=1;u32::from(w)}
        255 => {let w=u32::from(buf[*at]) | u32::from(buf[*at+1])<<16;*at+=2;w}
        _ => u32::from(tag)
    };
    ((h>>8) as u8,width)
}

fn moved(list:&mut [u8], rank:usize)->u8{
    let byte=list[rank];list[..=rank].rotate_right(1);byte
}

pub struct Prefix {pub words:usize,pub runs:usize,pub rows:u64,pub consumed:usize,pub failed:bool}

// Validation failure or the requested interruption leaves an exact rollback point.
// Only the single symbols slice is read and written; no aliasing slices are held.
pub fn compact(buf:&mut [u16], alphabet:&[u8], stop:usize, max_len:u64)->Prefix{
    let mut list=[0u8;256];list[..alphabet.len()].copy_from_slice(alphabet);
    let mut state=Prefix{words:0,runs:0,rows:0,consumed:0,failed:false};
    let eob=alphabet.len() as u16+1;
    let mut i=0;
    while state.runs<stop && i<buf.len(){
        let s=buf[i];
        if s==eob && s>1 { return state; }
        let opening=state.runs==0 && s<2;
        let byte=if opening {list[0]} else{
            if s<2 || s>=eob {state.failed=true;return state;}
            i+=1; moved(&mut list[..alphabet.len()],usize::from(s-1))
        };
        let (mut t,mut u)=(0,0);
        while i<buf.len() && buf[i]<2 {
            if t==21 {state.failed=true;return state;}
            u|=u32::from(buf[i])<<t;i+=1;t+=1;
        }
        let width=(1u32<<t)+u-u32::from(opening);
        if state.rows+u64::from(width)>max_len || i==buf.len(){state.failed=true;return state;}
        // All original symbols in this span were read. Neither this prefix nor any
        // previous one can produce more packed words than its consumed symbols.
        append(&mut buf[..i],&mut state.words,byte,width);
        state.rows+=u64::from(width);state.runs+=1;state.consumed=i;
    }
    state
}

pub fn restore(buf:&mut [u16], words:usize, alphabet:&[u8])->usize{
    let mut at=0;
    let mut consumed=0;
    while at<words{
        let first=at==0;
        let(c,w)=take(&buf[..words],&mut at);
        let opening=first && c==alphabet[0];
        let v=w+u32::from(opening);
        consumed+=usize::from(!opening)+v.ilog2() as usize;
    }
    assert!(consumed>=words);
    let gap=consumed-words;
    buf.copy_within(0..words,gap);
    at=gap;
    let mut out=0;
    let mut list=[0u8;256];list[..alphabet.len()].copy_from_slice(alphabet);
    while at<consumed{
        let(c,w)=take(&buf[..consumed],&mut at);
        let opening=out==0 && c==alphabet[0];
        if !opening{
            let rank=list[..alphabet.len()].iter().position(|&b|b==c).unwrap();
            assert_ne!(rank,0);
            moved(&mut list[..alphabet.len()],rank);
            buf[out]=rank as u16+1;out+=1;
        }
        let v=w+u32::from(opening);
        let t=v.ilog2();
        for k in 0..t {buf[out]=((v>>k)&1)as u16;out+=1;}
        assert!(out<=at);
    }
    assert_eq!(out,consumed);consumed
}
