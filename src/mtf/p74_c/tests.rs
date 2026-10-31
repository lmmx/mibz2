use super::*;

fn block(alpha: usize, syms: Vec<u16>, max_len: usize) -> Symbols {
    Symbols {
        crc: 0,
        orig_ptr: 0,
        seq_to_unseq: std::array::from_fn(|i| (255 - i) as u8),
        syms,
        eob: alpha as u16 + 1,
        max_len,
    }
}

fn avx2() -> bool {
    std::is_x86_feature_detected!("avx2")
}

fn run(mut n: usize, syms: &mut Vec<u16>) {
    while n != 0 {
        syms.push(((n - 1) % 2) as u16);
        n = (n - 1) / 2;
    }
}

// A separate list algorithm checks all 256 materialized entries, including unused ones.
fn list_before(b: &Symbols, consumed: usize) -> [u8; 256] {
    let mut list: Vec<u8> = (0..=255).collect();
    for &s in &b.syms[..consumed - 1] {
        if s > 1 {
            let v = list.remove(s as usize - 1);
            list.insert(0, v);
        }
    }
    list.try_into().unwrap()
}

fn check<const CP: usize>(b: &Symbols, force: Option<usize>, start: bool) -> TestControl {
    let (mut want, mut wc, mut wcp) = (Vec::new(), [0; 256], Vec::new());
    let wr = if CP == 0 {
        crate::mtf::mtf(b, &mut want, &mut wc, Kernels::detect())
    } else {
        crate::mtf::mtf_checkpoints(b, &mut want, &mut wc, &mut wcp, Kernels::detect())
    };
    let (mut out, mut counts, mut cps) = (vec![7; 11], [99; 256], vec![[99; 256]; 3]);
    let mut control = TestControl {
        force,
        ..Default::default()
    };
    // SAFETY: all callers check AVX2 and use scanner-valid symbol values.
    let got = unsafe { combined::<CP>(b, &mut out, &mut counts, &mut cps, start, &mut control) };
    assert_eq!(got, wr, "force={force:?}, CP={CP}, max={}", b.max_len);
    assert_eq!(out, want, "output, force={force:?}, CP={CP}");
    assert_eq!(counts, wc, "counts, force={force:?}, CP={CP}");
    assert_eq!(counts.iter().map(|&n| n as usize).sum::<usize>(), out.len());
    if CP == CHECKPOINT {
        // Includes partial checkpoint arrays on errors, not only successful results.
        assert_eq!(cps, wcp);
    }
    if CP != 0 && got.is_ok() {
        let mut cum = [0; 256];
        let mut expected = vec![cum];
        for chunk in out.chunks_exact(CP) {
            for &c in chunk {
                cum[c as usize] += 1;
            }
            expected.push(cum);
        }
        assert_eq!(cps, expected);
    }
    if let Some(at) = control.at {
        assert!(!start);
        assert!(b.syms[at - 1] >= 33 && b.syms[at - 1] < b.eob);
        assert_eq!(control.list.unwrap(), list_before(b, at));
        assert_eq!(
            control.fallbacks,
            b.syms[..at - 1].iter().filter(|&&s| s >= 33).count()
        );
        let (n, run, weight, next) = control.state.unwrap();
        assert_eq!((run, weight), (0, 1));
        assert!(n < b.max_len);
        if CP != 0 {
            assert_eq!(next, (n / CP + 1) * CP);
        }
        let mut prefix = block(b.eob as usize - 1, b.syms[..at - 1].to_vec(), b.max_len);
        prefix.seq_to_unseq = b.seq_to_unseq;
        prefix.syms.push(b.eob); // flush a run immediately before the trigger
        let mut ll = Vec::new();
        assert_eq!(
            crate::mtf::mtf_scalar(&prefix, &mut ll, &mut [0; 256]),
            Ok(n)
        );
        assert_eq!(&out[..n], &ll);
    } else if start {
        assert_eq!(control.list.unwrap(), std::array::from_fn(|i| i as u8));
    }
    control
}

fn all_boundaries<const CP: usize>(b: &Symbols) {
    check::<CP>(b, Some(usize::MAX), false);
    check::<CP>(b, None, true);
    for (i, &s) in b.syms.iter().enumerate() {
        if s >= 33 && s < b.eob {
            let c = check::<CP>(b, Some(i + 1), false);
            assert_eq!(c.at, Some(i + 1));
        }
    }
}

#[test]
fn dispatch_boundaries() {
    for s in [0, 1, 16_383, 16_384, 16_385, 49_999, 50_000, 450_000] {
        for max in [0, 1, 2 * s, 2 * s + 1, 100_000, 900_000] {
            assert_eq!(bypass(s, max), s < 16_384 || 2 * s >= max);
        }
    }
    assert!(bypass(usize::MAX, usize::MAX));
    assert!(!bypass(usize::MAX / 2, usize::MAX));
}

#[test]
fn exact_guard_includes_trigger_and_run_digits() {
    if !avx2() {
        return;
    }
    for middle in [0, 1, 2] {
        // At token 128, F+1 == 127 == floor(128/32)+123: do NOT switch.
        // Excluding this trigger from S would switch one event too early.
        let mut syms = vec![33; 126];
        syms.extend([middle, 33, 33]);
        syms.resize(16_383, 2);
        syms.push(257);
        let b = block(256, syms, 900_000);
        assert!(!bypass(b.syms.len(), b.max_len));
        for c in [
            check::<0>(&b, None, false),
            check::<CHECKPOINT>(&b, None, false),
        ] {
            assert_eq!(c.at, Some(129));
            assert_eq!(c.fallbacks, 127);
        }
    }
    let mut syms = vec![33; 16_383];
    syms.push(257);
    let b = block(256, syms, 900_000);
    assert_eq!(check::<0>(&b, None, false).at, Some(127));
    assert_eq!(check::<CHECKPOINT>(&b, None, false).at, Some(127));
}

#[test]
fn exhaustive_small_streams_and_legal_boundaries() {
    if !avx2() {
        return;
    }
    // Exhaust every symbol string through length 3 for these alphabets. Lengths
    // include RUNA/RUNB; append EOB. Each string is checked at EVERY legal handoff,
    // and with a 4-byte interval so even small strings cross checkpoints.
    fn visit(alpha: usize, prefix: &mut Vec<u16>, left: usize) {
        let mut syms = prefix.clone();
        syms.push(alpha as u16 + 1);
        let b = block(alpha, syms, 64);
        all_boundaries::<0>(&b);
        all_boundaries::<4>(&b);
        if left > 0 {
            for s in 0..=alpha as u16 {
                prefix.push(s);
                visit(alpha, prefix, left - 1);
                prefix.pop();
            }
        }
    }
    for alpha in [1, 2, 3, 4, 5, 6, 31, 32, 33, 34] {
        visit(alpha, &mut Vec::new(), 3);
    }
    // Longer small streams: all 2^8 run-digit patterns surrounding every legal
    // boundary rank. This covers both sides of repeated handoffs and run flushes.
    for bits in 0..256 {
        for rank in [
            32, 33, 63, 64, 95, 96, 127, 128, 159, 160, 191, 192, 223, 224, 255,
        ] {
            let mut syms = Vec::new();
            for i in 0..8 {
                syms.extend([((bits >> i) & 1) as u16, rank + 1]);
            }
            syms.push(257);
            let b = block(256, syms, 64);
            all_boundaries::<0>(&b);
            all_boundaries::<4>(&b);
        }
    }
}

#[test]
fn all_alphabets_and_all_ranks() {
    if !avx2() {
        return;
    }
    for alpha in 1..=256 {
        let mut syms = Vec::new();
        // All ranks twice, reversing direction: the snapshot isn't the initial list.
        for rank in (1..alpha).chain((1..alpha).rev()) {
            syms.extend([0, rank as u16 + 1]);
        }
        syms.push(alpha as u16 + 1);
        let b = block(alpha, syms, 2048);
        all_boundaries::<0>(&b);
        all_boundaries::<4>(&b);
    }
}

#[test]
fn runs_adjacent_to_handoff_and_checkpoint_edges() {
    if !avx2() {
        return;
    }
    for prefix in [0, CHECKPOINT - 1, CHECKPOINT, CHECKPOINT + 1] {
        for before in [
            1,
            31,
            32,
            33,
            CHECKPOINT - 1,
            CHECKPOINT,
            CHECKPOINT + 1,
            3 * CHECKPOINT,
        ] {
            for after in [1, 32, 33, 2 * CHECKPOINT + 1] {
                let mut syms = vec![2; prefix];
                run(before, &mut syms);
                let at = syms.len() + 1;
                syms.push(256);
                run(after, &mut syms);
                syms.extend([33, 257]);
                let b = block(256, syms, 900_000);
                for c in [
                    check::<0>(&b, Some(at), false),
                    check::<CHECKPOINT>(&b, Some(at), false),
                ] {
                    assert_eq!(c.at, Some(at));
                    assert_eq!(c.state.unwrap().0, prefix + before);
                }
            }
        }
    }
}

#[test]
fn sampled_large_streams() {
    if !avx2() {
        return;
    }
    let mut seed = 0x74c0_1234_9876_abcd_u64;
    let mut random = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    for shape in 0..5 {
        let mut syms = Vec::new();
        let mut positions = Vec::new();
        for i in 0..100_000 {
            let r = random();
            if shape == 4 && i % 29 == 0 {
                run(63, &mut syms);
            }
            let rank = match shape {
                0 => 1 + r as usize % 255,
                1 => {
                    if i % 32 == 31 {
                        255
                    } else {
                        1
                    }
                }
                2 => {
                    if i < 50_000 {
                        1
                    } else {
                        32
                    }
                }
                3 => {
                    if i < 50_000 {
                        255
                    } else {
                        16
                    }
                }
                _ => [15, 16, 31, 32, 255][i % 5],
            };
            syms.push(rank as u16 + 1);
            if rank >= 32 {
                positions.push(syms.len());
            }
        }
        syms.push(257);
        let b = block(256, syms, 900_000);
        // Independent integer formulation of the live policy; includes runs in the
        // denominator, carries F across intervals, and checks never-switch streams.
        let mut high = 0;
        let expected = b.syms.iter().enumerate().find_map(|(i, &s)| {
            if s >= 33 && s < b.eob {
                high += 1;
                if high > 123 && 32 * (high - 123) > i + 1 {
                    return Some(i + 1);
                }
            }
            None
        });
        assert_eq!(check::<0>(&b, None, false).at, expected);
        assert_eq!(check::<CHECKPOINT>(&b, None, false).at, expected);
        let mut sampled = vec![positions[0], *positions.last().unwrap()];
        for _ in 0..64 {
            sampled.push(positions[random() as usize % positions.len()]);
        }
        for edge in (CHECKPOINT..100_000).step_by(CHECKPOINT) {
            let p = positions.partition_point(|&s| s < edge);
            sampled.extend(positions[p.saturating_sub(1)..(p + 2).min(positions.len())].iter());
        }
        sampled.sort_unstable();
        sampled.dedup();
        for at in sampled {
            assert_eq!(check::<0>(&b, Some(at), false).at, Some(at));
            assert_eq!(check::<CHECKPOINT>(&b, Some(at), false).at, Some(at));
        }
    }
}

#[test]
fn errors_and_truncated_symbols_preserve_partial_state() {
    if !avx2() {
        return;
    }
    let mut cases = vec![vec![], vec![0, 1], vec![33], vec![33, 0, 1], vec![257, 33]];
    for prefix in [0, 17, CHECKPOINT - 1, CHECKPOINT, CHECKPOINT + 1] {
        for len in [1, 32, 33, CHECKPOINT, 900_000, 2_097_151, 4_194_303] {
            let mut s = vec![2; prefix];
            s.push(33);
            run(len, &mut s);
            s.extend([256, 257]);
            cases.push(s);
        }
        let mut s = vec![2; prefix];
        s.push(33);
        s.extend([1; 22]); // RunTooLong, before flushing
        s.push(257);
        cases.push(s);
    }
    for syms in cases {
        for max in [
            0,
            1,
            18,
            CHECKPOINT - 1,
            CHECKPOINT,
            CHECKPOINT + 1,
            100_000,
            900_000,
        ] {
            let b = block(256, syms.clone(), max);
            let at = b.syms.iter().position(|&s| s == 33).map(|i| i + 1);
            for start in [false, true] {
                check::<0>(&b, at, start);
                check::<CHECKPOINT>(&b, at, start);
            }
        }
    }
}

// The two readings of the guard, written independently of the kernel (plain division
// rather than `>> 5`, an index rather than pointer distance). Both return the switching
// token's position counted INCLUDING it, so the two are comparable. `back` = 0 is the
// implemented rule (S_incl), `back` = 1 the draft's §12 wording (S_before = S_incl - 1).
fn reading(b: &Symbols, back: usize) -> Option<usize> {
    let mut previous = 0usize;
    for (i, &s) in b.syms.iter().enumerate() {
        if s == b.eob {
            return None;
        }
        if s >= 33 {
            if previous + 1 > (i + 1 - back) / 32 + 123 {
                return Some(i + 1);
            }
            previous += 1;
        }
    }
    None
}

/// `len` symbols then EOB (alphabet 256): high-rank events (ranks 32..=254, varied) at
/// exactly the given 1-based consumed positions, low filler elsewhere: rank-1 tokens,
/// with a one-digit run (RUNA) at every 7th position, so S also counts run digits.
fn events_at(len: usize, events: &[usize]) -> Symbols {
    let mut syms: Vec<u16> = (1..=len).map(|p| if p % 7 == 0 { 0 } else { 2 }).collect();
    for &p in events {
        syms[p - 1] = 33 + (p * 37 % 222) as u16;
    }
    syms.push(257);
    let b = block(256, syms, 900_000);
    assert!(!bypass(b.syms.len(), b.max_len));
    b
}

/// The boundary-hugging stream: a burst of k0 + 122 events ending before 32*k0 (at the
/// front, feasible only for k0 = 4, or immediately before 32*k0), then one event at every S = 32k, k >= k0, so that
/// at each, F + 1 = k + 123 = floor(S_incl/32) + 123 exactly.
fn hugging(len: usize, k0: usize, front: bool) -> Vec<usize> {
    let burst = k0 + 122;
    let first = if front { 1 } else { 32 * k0 - burst };
    assert!(first >= 1 && first + burst <= 32 * k0);
    let mut events: Vec<usize> = (first..first + burst).collect();
    events.extend((k0..).map(|k| 32 * k).take_while(|&p| p <= len));
    events
}

#[test]
fn inclusive_trigger_boundary_counterexample() {
    if !avx2() {
        return;
    }
    for (k0, front) in [(4, true), (5, false), (50, false), (300, false)] {
        for continue_after in [false, true] {
            let mut events = hugging(20_000, k0, front);
            if !continue_after {
                events.retain(|&p| p <= 32 * k0);
            }
            let b = events_at(20_000, &events);
            // Implemented rule: F + 1 = k0 + 123 is not > floor(32k0/32) + 123.
            assert_eq!(reading(&b, 0), None, "k0={k0}");
            // Draft rule: F + 1 = k0 + 123 > floor((32k0 - 1)/32) + 123 = k0 + 122.
            assert_eq!(reading(&b, 1), Some(32 * k0), "k0={k0}");
            for c in [
                check::<0>(&b, None, false),
                check::<CHECKPOINT>(&b, None, false),
            ] {
                assert_eq!(c.at, None, "kernel switched, k0={k0}");
            }
            // The draft's switch point is itself a correct handoff of this kernel.
            for c in [
                check::<0>(&b, Some(32 * k0), false),
                check::<CHECKPOINT>(&b, Some(32 * k0), false),
            ] {
                assert_eq!((c.at, c.fallbacks), (Some(32 * k0), k0 + 122));
            }
        }
    }
}

#[test]
fn inclusive_trigger_perpetual_boundary_evasion() {
    if !avx2() {
        return;
    }
    // The longest stream a level-9 block can carry without the 2S >= N_max bypass.
    const LEN: usize = 449_998;
    for (k0, front) in [(4, true), (10, false), (1_000, false)] {
        let events = hugging(LEN, k0, front);
        let b = events_at(LEN, &events);
        // The construction: exactly at the boundary at every periodic event.
        let mut f = 0;
        for (i, &s) in b.syms.iter().enumerate() {
            if s >= 33 && s < b.eob {
                f += 1;
                if i + 1 >= 32 * k0 {
                    assert_eq!((i + 1) % 32, 0);
                    assert_eq!(f, (i + 1) / 32 + 123);
                }
            }
        }
        assert_eq!(f, k0 + 122 + LEN / 32 - k0 + 1);
        assert_eq!(reading(&b, 0), None);
        assert_eq!(reading(&b, 1), Some(32 * k0));
        for c in [
            check::<0>(&b, None, false),
            check::<CHECKPOINT>(&b, None, false),
        ] {
            assert_eq!(c.at, None, "kernel switched, k0={k0}");
        }
    }
}

#[test]
fn inclusive_trigger_one_event_past_boundary() {
    if !avx2() {
        return;
    }
    const LEN: usize = 100_000;
    for (k0, front) in [(4, true), (10, false)] {
        for m in [k0 + 1, 100, 2_000] {
            // An extra event inside (32m, 32m + 32): F + 1 = m + 124 > m + 123.
            for j in [1, 17, 31] {
                let mut events = hugging(LEN, k0, front);
                events.push(32 * m + j);
                let b = events_at(LEN, &events);
                let want = Some(32 * m + j);
                assert_eq!(reading(&b, 0), want);
                for c in [
                    check::<0>(&b, None, false),
                    check::<CHECKPOINT>(&b, None, false),
                ] {
                    assert_eq!((c.at, c.fallbacks), (want, m + 123), "m={m} j={j}");
                }
            }
            // The event at 32m moved one symbol earlier: F + 1 = m + 123 > m + 122.
            let mut events = hugging(LEN, k0, front);
            let i = events.iter().position(|&p| p == 32 * m).unwrap();
            events[i] -= 1;
            let b = events_at(LEN, &events);
            let want = Some(32 * m - 1);
            assert_eq!(reading(&b, 0), want);
            for c in [
                check::<0>(&b, None, false),
                check::<CHECKPOINT>(&b, None, false),
            ] {
                assert_eq!((c.at, c.fallbacks), (want, m + 122), "m={m} early");
            }
        }
    }
}

/// Untimed audit through the existing actual handoff hook, absent from release code.
#[test]
#[ignore = "requires explicit P74_AUDIT_FILES list; correctness only"]
fn audit_manifest_corpora() {
    use crate::stream::{Event, Scanner, SliceSource};
    assert!(avx2());
    let list = std::fs::read_to_string(std::env::var("P74_AUDIT_FILES").unwrap()).unwrap();
    for (file, path) in list.lines().enumerate() {
        let z = std::fs::read(path).unwrap();
        let mut sc = Scanner::new(SliceSource(&z));
        let mut block = 0;
        while let Event::Block { symbols: b, .. } = sc.next(Vec::new()).unwrap() {
            let bypassed = bypass(b.syms.len(), b.max_len);
            let mut explicit = 0usize;
            let mut high = 0usize;
            for &s in &b.syms {
                if s == b.eob {
                    break;
                }
                if s > 1 {
                    explicit += 1;
                }
                if s >= 33 {
                    high += 1;
                }
            }
            // floor((i+1)/32), deliberately independent spelling of the guard.
            let mut f = 0usize;
            let want = b.syms.iter().enumerate().find_map(|(i, &s)| {
                if s >= 33 && s < b.eob {
                    f += 1;
                    if !bypassed && f > 123 && 32 * (f - 123) > i + 1 {
                        return Some(i + 1);
                    }
                }
                None
            });
            // Audit checks the actual frame's observed handoff.
            for cp in [0, CHECKPOINT] {
                let c = if cp == 0 {
                    check::<0>(&b, None, bypassed)
                } else {
                    check::<CHECKPOINT>(&b, None, bypassed)
                };
                assert_eq!(c.at, want);
                let state = c.state.unwrap_or((0, 0, 1, 0));
                println!(
                    "AUDIT\t{file}\t{block}\t{cp}\t{}\t{explicit}\t{high}\t{}\t{}\t{}\t{}\t{}",
                    b.syms.len(),
                    usize::from(bypassed),
                    c.at.unwrap_or(0),
                    state.0,
                    c.fallbacks,
                    b.syms.iter().filter(|&&s| s <= 1).count()
                );
            }
            block += 1;
        }
    }
}

// tests/mtf_p74.rs, ported to the combined kernel
mod gate1;
