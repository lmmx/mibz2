use super::*;

fn oracle(ll: &[u8], primary: usize) -> (usize, Vec<u8>) {
    let mut psi: Vec<_> = (0..ll.len()).collect();
    psi.sort_by_key(|&i| ll[i]);
    let mut row = primary;
    let mut bytes = Vec::new();
    let mut period = 0;
    for j in 0..ll.len() {
        row = psi[row];
        bytes.push(ll[row]);
        if row == primary && period == 0 {
            period = j + 1;
        }
    }
    (period, bytes)
}

fn check(c: &mut Candidate, ll: &[u8], primary: usize) -> bool {
    let mut records = Vec::<u16>::new();
    for &byte in ll {
        if let Some(record) = records.last_mut() {
            if (*record >> 8) == u16::from(byte) && (*record & 255) < 127 {
                *record += 1;
                continue;
            }
        }
        records.push((u16::from(byte) << 8) | 1);
    }
    let (period, expected) = oracle(ll, primary);
    let mut output = vec![0xab; ll.len()];
    if c.attempt::<true>(&records, ll.len(), primary, &mut output) {
        assert_eq!(period, ll.len());
        assert_eq!(output, expected, "primary={primary}, column={ll:?}");
        true
    } else {
        false
    }
}

#[test]
fn all_small_marked_columns() {
    let mut c = Candidate::default();
    let mut total = 0;
    let mut own = 0;
    for (alphabet, upto) in [(2usize, 11), (3usize, 7)] {
        for n in 1..=upto {
            for mut k in 0..alphabet.pow(n as u32) {
                let ll = (0..n)
                    .map(|_| {
                        let byte = k % alphabet;
                        k /= alphabet;
                        byte as u8
                    })
                    .collect::<Vec<_>>();
                for primary in 0..ll.len() {
                    own += usize::from(check(&mut c, &ll, primary));
                    total += 1;
                }
            }
        }
    }
    assert!(own > 5000);
    println!(
        "{total} marked columns, {own} reconstructed by packed, {} declined",
        total - own
    );
}

#[test]
fn run_caps_and_all_byte_values() {
    let mut c = Candidate::default();
    let mut own = 0;
    for byte in 0..255 {
        let mut ll = vec![byte + 1; 1];
        ll.extend(std::iter::repeat_n(byte, 511));
        for p in [0, 1, 126, 127, 128, 129, 253, 254, 255, 256, 510, 511] {
            own += usize::from(check(&mut c, &ll, p));
        }
    }
    assert_eq!(own, 0);
    // The root period is full in every one of these left-shift examples.
    // A position-table capacity refusal is a valid dictionary guard.
    println!("large cap-boundary controls: {own} passed among 3060");
}

#[test]
fn long_caps_with_enough_position_space() {
    let mut data = vec![255; 383];
    data.extend(0..=255);
    let mut starts: Vec<_> = (0..data.len()).collect();
    starts.sort_by(|&a, &b| {
        data[a..]
            .iter()
            .chain(&data[..a])
            .cmp(data[b..].iter().chain(&data[..b]))
    });
    let ll: Vec<_> = starts
        .iter()
        .map(|&i| data[(i + data.len() - 1) % data.len()])
        .collect();
    let mut c = Candidate::default();
    for p in 0..ll.len() {
        assert!(check(&mut c, &ll, p));
    }
    println!(
        "{} successful cap-boundary rotations with all 256 byte values",
        ll.len()
    );
}

/// Record exact geometry, including failed admissions. Use the independent
/// shipped Segmented kernel to check each row of every accepted reconstruction.
#[test]
#[ignore]
fn corpus_geometry() {
    use mibz2::stream::{Event, Scanner, SliceSource};
    use std::io::Write;
    let file = std::fs::read(std::env::var("PACKED_CORPUS").unwrap()).unwrap();
    let mut writer = std::io::BufWriter::new(
        std::fs::File::create(std::env::var("PACKED_CSV").unwrap()).unwrap(),
    );
    writeln!(writer,"ordinal,stream_index,rows,primary,records,own,letters,cuts,rules,rule_capacity,top_capacity,output_capacity").unwrap();
    let mut scan = Scanner::new(SliceSource(&file));
    let mut r = Runs::default();
    let mut c = Candidate::default();
    let mut reference = Segmented::default();
    let (mut out, mut want) = (Vec::new(), Vec::new());
    let mut ordinal = 0;
    let mut count = 0;
    while let Ok(Event::Block { index, symbols, .. }) = scan.next(Vec::new()) {
        let k = Kernels::detect();
        let n = crate::runs::front(&symbols, &mut r, k).unwrap();
        if n < 65536 {
            continue;
        }
        let p = symbols.orig_ptr;
        let own = c.attempt::<false>(r.records(), n, p as usize, &mut out);
        if own {
            assert!(reference.invert_runs(&r, n, p, &mut want, k));
            assert_eq!(out, want);
            count += 1;
        }
        writeln!(
            writer,
            "{ordinal},{index},{n},{p},{},{},{},{},{},{},{},{}",
            r.records().len(),
            own as u8,
            if own { c.tops.len() } else { 0 },
            if own { c.cuts } else { 0 },
            if own { c.rules.len() } else { 0 },
            c.rules.capacity(),
            c.tops.capacity(),
            out.capacity()
        )
        .unwrap();
        ordinal += 1;
    }
    println!("{count}/{ordinal} large blocks reconstructed by packed; every accepted result equals Segmented");
    assert_eq!((ordinal, count), (233, 228));
}
