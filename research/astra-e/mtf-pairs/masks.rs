#[repr(align(64))]
struct PairMasks([[u8;16];256]);
const fn masks() -> PairMasks {
    let mut masks=[[0;16];256];
    let mut i=0;
    while i<16 {
        let mut j=0;
        while j<16 {
            let mut k=0;
            while k<16 {
                // Reverse the two positional maps to find the lane's original index.
                let after_first=if k==0 {j} else if k<=j {k-1} else {k};
                let old=if after_first==0 {i} else if after_first<=i {after_first-1} else {after_first};
                masks[16*i+j][k]=old as u8;
                k+=1;
            }
            j+=1;
        }
        i+=1;
    }
    PairMasks(masks)
}
static PAIRS:PairMasks=masks();
