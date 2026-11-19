// An aligned entry contains both input-to-output bank maps; lookup is by rank only.
#[repr(align(64))]
struct WideMasks([[[u8;16];4];1024]);
const fn make_wide() -> WideMasks {
    let mut table=[[[0x80;16];4];1024]; let mut i=0;
    while i<32 {let mut j=0; while j<32 {let mut k=0;while k<32 {
        let after_first=if k==0 {j} else if k<=j {k-1} else {k};
        let src=if after_first==0 {i} else if after_first<=i {after_first-1} else {after_first};
        table[32*i+j][2*(k/16)+src/16][k%16]=(src%16) as u8; k+=1;
    } j+=1;} i+=1;}
    WideMasks(table)
}
static WIDE:WideMasks=make_wide();
