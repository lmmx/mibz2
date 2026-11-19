#[cfg(test)]
mod astra_direct_tests {
    use super::*;
    fn check(s: &[u8], capacity: usize, directory_words: usize) {
        let mut expected = Vec::new();
        let valid = rle::unrle_scalar(s,&mut expected,0).is_ok();
        let mut buf = Vec::with_capacity(capacity);
        buf.extend_from_slice(s);
        let mut dir = vec![0u16;directory_words];
        match repair(&mut buf,&mut dir) {
            Ok(true) => { assert!(valid); assert_eq!(buf,expected); }
            Ok(false) => assert_eq!(buf,s), // irrevocable fallback is still possible
            Err(_) => { assert!(!valid); assert_eq!(buf,s); }
        }
    }
    #[test]
    fn astra_overlaps_and_refusals() {
        if !Kernels::detect().avx2 {return;}
        // Alternating marker bytes keep tokens distinct. Enumerate left and right
        // moves, equality, zero-length spans and multiple changes of sign.
        for seq in 0..6usize.pow(5) {
            let mut k = seq;
            let mut s = Vec::new();
            for t in 0..5 {
                let b = (t+1) as u8;
                s.extend([b;4]);
                s.push([0,0,1,2,4,255][k%6]);
                if k%2 == 0 {s.extend([9,8,7]);}
                k /= 6;
            }
            check(&s,2048,64);
            check(&s,s.len(),64);
            check(&s,2048,4);
        }
        for n in 0..100 {
            let mut s:Vec<u8>=(0..n).map(|i|(i%11) as u8).collect();
            s.extend([222;4]);
            check(&s,2048,64);
        }
    }
}
