include!("geometry.rs");
include!("../../../edge-budget/gate.rs");

fn main() {
    // Same records, different primary. No unmarked certificate can resolve these thresholds.
    for (rows, heavy) in [(262_144, 1), (150_000, 5290)] {
        let mut records = Vec::new();
        for i in 0..rows { records.push(((i % 2) << 8 | 1) as u16); }
        for i in 0..heavy { records.push((((i % 2 + 2) << 8) | 17) as u16); }
        assert_eq!(unmarked_caps(&records), (rows, heavy));
        assert!(fits(&records, 0));
        assert!(!fits(&records, (rows + 1) as u32));
        println!("{rows} thin rows and {heavy} thick runs: primary 0 admits; primary {} rejects. Lower/upper charges {}/{} bytes.",
            rows+1, charge(rows,heavy), charge(rows+1,heavy+1));
    }
}
