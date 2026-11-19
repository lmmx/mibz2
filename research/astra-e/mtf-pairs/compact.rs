// Exactly one control byte per destination position; the complementary mask has bit 7 flipped.
#[repr(align(64))]
struct CompactMasks([[u8;32];1024]);
const fn compact_masks()->CompactMasks{
    let mut table=[[0;32];1024];let mut i=0;
    while i<32{let mut j=0;while j<32{let mut k=0;while k<32{
        let after_first=if k==0{j}else if k<=j{k-1}else{k};
        let old=if after_first==0{i}else if after_first<=i{after_first-1}else{after_first};
        table[32*i+j][k]=(old%16) as u8 | if old>=16{128}else{0};
        k+=1;
    }j+=1;}i+=1;}
    CompactMasks(table)
}
static COMPACT:CompactMasks=compact_masks();
