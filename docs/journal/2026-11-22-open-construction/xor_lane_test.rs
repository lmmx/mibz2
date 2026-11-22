#[cfg(test)]
mod lane_audit {
    use super::*;
    #[test]
    fn eight_nonmonotone_ranks_in_an_unmarked_epoch() {
        let mut out = [0; BYTES];
        let spill = Spill::default();
        let low = [8u16, 5, 3, 1, 7, 9, 0, 6];
        for (d, v) in out.chunks_exact_mut(2).zip(low) {
            d.copy_from_slice(&v.to_le_bytes());
        }
        let mut r = Reader {
            out: &out,
            spill: &spill,
            in_out: 1,
            a: 0,
            id: 0,
            at: 0,
            read: 0,
            epoch: 1,
            cut: u32::MAX,
        };
        let pack = 7 << 17;
        let mut edges = vec![0; 65550];
        let mut expected = edges.clone();
        let mut before = 2;
        for (i, v) in low.iter().copied().enumerate() {
            let v = 65536 + u32::from(v);
            expected[16 + i] ^= v | pack;
            expected[v as usize] ^= (16 + i) as u32;
        }
        r.wire_xor(&mut edges, 16, 8, pack, 2, &mut before);
        assert!(edges == expected);
        assert_eq!(before, 2);
    }
}
