// Appended solely to the detached RSS library: reconstruct the genuine Runs fields cheaply.
impl Runs {
    pub fn census_original_caps(mut source: impl std::io::Read, size: usize) -> Self {
        use std::io::Read;
        let mut input = std::io::BufReader::new(&mut source);
        let mut runs = Self::default();
        runs.runs.reserve_exact(size);
        for _ in 0..size {
            let mut bytes = [0; 2];
            input.read_exact(&mut bytes).unwrap();
            let [width, symbol] = bytes;
            assert!(width > 0 && width <= 127);
            runs.runs.push(u16::from_le_bytes(bytes));
            runs.counts[symbol as usize] += u32::from(width);
            runs.per[symbol as usize] += 1;
        }
        let mut end = [0; 1];
        assert_eq!(input.read(&mut end).unwrap(), 0);
        runs
    }
}
