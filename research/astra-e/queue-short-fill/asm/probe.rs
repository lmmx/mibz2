#[no_mangle]
pub fn extend_rows(q:&mut Vec<u32>,m:u32,w:u32) {q.extend(m..m+w);}
#[no_mangle]
pub fn first_single(q:&mut Vec<u32>,m:u32,w:u32) {
    if w==1 {q.push(m);} else {q.extend(m..m+w);}
}
#[no_mangle]
pub fn first_four(q:&mut Vec<u32>,m:u32,w:u32) {
    match w {
        1=>q.push(m),
        2=>q.extend_from_slice(&[m,m+1]),
        3=>q.extend_from_slice(&[m,m+1,m+2]),
        4=>q.extend_from_slice(&[m,m+1,m+2,m+3]),
        _=>q.extend(m..m+w),
    }
}
