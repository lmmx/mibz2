import os, shutil
from gen_c33 import clone_base, patch_wire, patch_dedup

ROOT = "/tmp/cks_bench"

def patch_wire16(dst):
    patch_wire(dst, unroll=8, with_q=False)
    outq_p = f"{dst}/outq.rs"
    with open(outq_p) as f:
        o = f.read()
    old_loop = """            let mut i = 0usize;
            while i + 8 <= span {"""
    new_loop = """            let mut i = 0usize;
            while i + 16 <= span {
                let b = &bytes[2 * i..2 * i + 32];
                let q0 = u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]);
                let q1 = u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]);
                let q2 = u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]);
                let q3 = u64::from_le_bytes([b[24], b[25], b[26], b[27], b[28], b[29], b[30], b[31]]);
                let v0 = hi | (q0 as u32 & 0xFFFF);
                let v1 = hi | ((q0 >> 16) as u32 & 0xFFFF);
                let v2 = hi | ((q0 >> 32) as u32 & 0xFFFF);
                let v3 = hi | ((q0 >> 48) as u32);
                let v4 = hi | (q1 as u32 & 0xFFFF);
                let v5 = hi | ((q1 >> 16) as u32 & 0xFFFF);
                let v6 = hi | ((q1 >> 32) as u32 & 0xFFFF);
                let v7 = hi | ((q1 >> 48) as u32);
                let v8 = hi | (q2 as u32 & 0xFFFF);
                let v9 = hi | ((q2 >> 16) as u32 & 0xFFFF);
                let v10 = hi | ((q2 >> 32) as u32 & 0xFFFF);
                let v11 = hi | ((q2 >> 48) as u32);
                let v12 = hi | (q3 as u32 & 0xFFFF);
                let v13 = hi | ((q3 >> 16) as u32 & 0xFFFF);
                let v14 = hi | ((q3 >> 32) as u32 & 0xFFFF);
                let v15 = hi | ((q3 >> 48) as u32);
                let u0 = dst_start + i;
                let eu = &mut edges[u0..u0 + 16];
                eu[0] ^= pack | v0;   eu[1] ^= pack | v1;   eu[2] ^= pack | v2;   eu[3] ^= pack | v3;
                eu[4] ^= pack | v4;   eu[5] ^= pack | v5;   eu[6] ^= pack | v6;   eu[7] ^= pack | v7;
                eu[8] ^= pack | v8;   eu[9] ^= pack | v9;   eu[10] ^= pack | v10; eu[11] ^= pack | v11;
                eu[12] ^= pack | v12; eu[13] ^= pack | v13; eu[14] ^= pack | v14; eu[15] ^= pack | v15;
                edges[v0 as usize] ^= u0 as u32;        edges[v1 as usize] ^= (u0 + 1) as u32;
                edges[v2 as usize] ^= (u0 + 2) as u32;  edges[v3 as usize] ^= (u0 + 3) as u32;
                edges[v4 as usize] ^= (u0 + 4) as u32;  edges[v5 as usize] ^= (u0 + 5) as u32;
                edges[v6 as usize] ^= (u0 + 6) as u32;  edges[v7 as usize] ^= (u0 + 7) as u32;
                edges[v8 as usize] ^= (u0 + 8) as u32;  edges[v9 as usize] ^= (u0 + 9) as u32;
                edges[v10 as usize] ^= (u0 + 10) as u32; edges[v11 as usize] ^= (u0 + 11) as u32;
                edges[v12 as usize] ^= (u0 + 12) as u32; edges[v13 as usize] ^= (u0 + 13) as u32;
                edges[v14 as usize] ^= (u0 + 14) as u32; edges[v15 as usize] ^= (u0 + 15) as u32;
                i += 16;
            }
            if i + 8 <= span {"""
    assert old_loop in o
    o = o.replace(old_loop, new_loop, 1)
    with open(outq_p, "w") as f:
        f.write(o)

d = clone_base("cand_c33_wire16"); patch_wire16(d)
d = clone_base("cand_c33_w8_d12"); patch_wire(d, unroll=8, with_q=False); patch_dedup(d, pair_bits=9, str_bits=11)
d = clone_base("cand_c33_w16_d12"); patch_wire16(d); patch_dedup(d, pair_bits=9, str_bits=11)
print("Generated Wire16, W8+D12, W16+D12!")
