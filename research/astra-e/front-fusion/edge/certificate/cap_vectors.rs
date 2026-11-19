// Exercise overlapping vector loads and cap tails separately from the symbol-bit tests.
include!("geometry.rs");
include!("simd.rs");

fn main() {
    assert!(is_x86_feature_detected!("avx2"));
    let mut cases = 0;
    for prefix in 0..64 {
        for width in [1usize, 2, 15, 16, 17, 126, 127, 128, 142, 143, 144, 253, 254, 255, 5000] {
            for tail in 0..32 {
                let mut caps = Vec::new();
                for i in 0..prefix { caps.push(((i % 2) << 8 | 1) as u16); }
                let mut left = width;
                while left > 0 {
                    let piece = left.min(127);
                    caps.push((42 << 8) | piece as u16);
                    left -= piece;
                }
                for i in 0..tail { caps.push((((i % 2 + 3) << 8) | 2) as u16); }
                let expected = (prefix + 2 * tail + if width <= 16 {width} else {0}, usize::from(width > 16));
                assert_eq!(unsafe { unmarked_avx2(&caps) }, expected);
                assert_eq!(unmarked_caps(&caps), expected);
                cases += 1;
            }
        }
    }
    println!("{cases} AVX2 cap-window checks: 64 starts, 32 tail lengths, 15 widths including 16,127,143,254 and 5000");
}
