//! CRC-32/BZIP2 (polynomial 0x04C11DB7, MSB first, init and final xor all ones): a byte
//! at a time by table (as bzip2 and lbzip2 take it), or sliced by 8 (portable)

pub const POLY: u32 = 0x04C1_1DB7;

const fn tables() -> [[u32; 256]; 8] {
    let mut t = [[0u32; 256]; 8];
    let mut i = 0;
    while i < 256 {
        let mut c = (i as u32) << 24;
        let mut k = 0;
        while k < 8 {
            c = if c & 0x8000_0000 != 0 { (c << 1) ^ POLY } else { c << 1 };
            k += 1;
        }
        t[0][i] = c;
        i += 1;
    }
    let mut j = 1;
    while j < 8 {
        let mut i = 0;
        while i < 256 {
            let p = t[j - 1][i];
            t[j][i] = (p << 8) ^ t[0][(p >> 24) as usize];
            i += 1;
        }
        j += 1;
    }
    t
}

pub static TABLES: [[u32; 256]; 8] = tables();
pub static TABLE: [u32; 256] = TABLES[0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Bytewise,
    Slice8,
    /// simd.rs: AVX2 RLE1, PCLMULQDQ CRC (dispatched at run time)
    Simd,
}

pub fn crc32_bzip2(data: &[u8], kind: Kind) -> u32 {
    !update(!0, data, kind)
}

pub fn update(mut s: u32, data: &[u8], kind: Kind) -> u32 {
    let t = &TABLES;
    let mut rest = data;
    if kind == Kind::Slice8 {
        let mut chunks = data.chunks_exact(8);
        for c in &mut chunks {
            let hi = s ^ u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
            s = t[7][(hi >> 24) as usize]
                ^ t[6][((hi >> 16) & 0xff) as usize]
                ^ t[5][((hi >> 8) & 0xff) as usize]
                ^ t[4][(hi & 0xff) as usize]
                ^ t[3][c[4] as usize]
                ^ t[2][c[5] as usize]
                ^ t[1][c[6] as usize]
                ^ t[0][c[7] as usize];
        }
        rest = chunks.remainder();
    }
    for &b in rest {
        s = (s << 8) ^ t[0][((s >> 24) as u8 ^ b) as usize];
    }
    s
}
