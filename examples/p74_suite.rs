//! The frozen P74-C adversarial suite: seeded, reproducible, valid single-block .bz2
//! inputs whose distribution changes WITHIN a block. No timing.
//! `cargo run --release --example p74_suite -- DIR` writes DIR/<name>.bz2 and
//! DIR/intent.json (each block's design, measured symbol facts and the oracle's switch).
//!
//! L (the BWT's last column) is ordered by the following context: L's region for first
//! column byte c holds the bytes preceding each c. Two constructions use that:
//! - pair: low text (iid letters a..h, ranks <= 8) then pairs (r, 0xFF), r uniform over a
//!   pool. Region 0xFF, the last of L, holds the r's: the high-rank suffix, of exactly
//!   (number of 0xFF) bytes. Every other region gets letters or runs of 0xFF.
//! - keyed: low text then units (c_j, 0xFF, k1, k2, k3, k4), k = j in base 16, digit i
//!   in 0x10*(i-1)..; region 0xFF holds c_0, c_1, ... in key (= j) order. So the suffix
//!   is an exact script: lows alternate u = 0x40 / v = 0x41 (rank 1-2, one symbol each),
//!   events cycle a pool of H unused values (rank H-1 in a burst, H+1 between lows).
//!
//! Exact designs depend on S_0, F_0 (symbols and high events before the suffix), which
//! the suffix itself perturbs slightly; they are solved by a deterministic fixed-point
//! iteration on the ACTUAL compressed stream, and every property is re-checked on it.
//! The text never has a run of 4 equal bytes, so bzip2's RLE1 is the identity and
//! L's length is the text's length.
#[path = "../tests/common/mod.rs"]
mod common;
use common::Rng;
use mibz2::block::Symbols;
use mibz2::stream::{Event, Scanner, SliceSource};
use std::fmt::Write as _;

const SEP: u8 = 0xFF;
const U: u8 = 0x40;
const V: u8 = 0x41;
/// File i uses seed BASE_SEED + i (registered).
const BASE_SEED: u64 = 0x74c0_2026_1031_0100;
/// Edge files draw their pairs from seed ^ PAIR_STREAM.
const PAIR_STREAM: u64 = 0x5a5a_5a5a_5a5a_5a5a;

fn letters(rng: &mut Rng, n: usize) -> Vec<u8> {
    let mut t: Vec<u8> = Vec::with_capacity(n);
    while t.len() < n {
        let c = b'a' + rng.below(8) as u8;
        let l = t.len();
        if l >= 3 && t[l - 1] == c && t[l - 2] == c && t[l - 3] == c {
            continue;
        }
        t.push(c);
    }
    t
}

fn pairs(rng: &mut Rng, n: usize, pool: &[u8]) -> Vec<u8> {
    (0..n)
        .flat_map(|_| [pool[rng.below(pool.len() as u64) as usize], SEP])
        .collect()
}

fn keyed(script: &[u8]) -> Vec<u8> {
    assert!(script.len() <= 1 << 16);
    script
        .iter()
        .enumerate()
        .flat_map(|(j, &c)| {
            [
                c,
                SEP,
                (j >> 12) as u8,
                0x10 | (j >> 8 & 15) as u8,
                0x20 | (j >> 4 & 15) as u8,
                0x30 | (j & 15) as u8,
            ]
        })
        .collect()
}

/// H event values, unused elsewhere first; larger pools overlap prefix bytes, and 254
/// adds 0xFF itself (its extra region-0xFF entries sort at the region's end).
fn pool(h: usize) -> Vec<u8> {
    let order: Vec<u8> = (0x80..=0xFE)
        .chain(0x42..=0x60)
        .chain(0x69..=0x7F)
        .chain(0x61..=0x68)
        .chain(0x00..=0x3F)
        .chain([SEP])
        .collect();
    assert!(h <= order.len());
    order[..h].to_vec()
}

struct Facts {
    syms: usize,
    bypass: bool,
    l_bytes: usize,
    n_split: usize,
    s0: usize,
    f0: usize,
    /// inclusive rule: (consumed, output position, F before)
    switch: Option<(usize, usize, usize)>,
    /// draft rule (S_before), consumed position
    draft: Option<usize>,
    high_suffix: usize,
    rank_min: usize,
    rank_max: usize,
    rank_mode: usize,
    /// high events in the suffix per rank class 1..=7 (ranks 32j..32j+31)
    classes: [usize; 8],
    /// periodic boundary events with F + 1 == floor(S/32) + 123 (S % 32 == 0)
    at_equality: usize,
}

fn facts(b: &Symbols, n_split: usize) -> Facts {
    let bypass = b.syms.len() < 16_384 || 2 * b.syms.len() >= b.max_len;
    let (mut n, mut run, mut weight) = (0usize, 0usize, 1usize);
    let (mut s0, mut f0) = (None, 0);
    let (mut f, mut switch, mut draft) = (0usize, None, None);
    let (mut high_suffix, mut rank_min, mut rank_max, mut eq) = (0, usize::MAX, 0, 0);
    let mut hist = [0usize; 256];
    for (i, &s) in b.syms.iter().enumerate() {
        if s == b.eob {
            break;
        }
        if s <= 1 {
            run += (s as usize + 1) * weight;
            weight <<= 1;
            continue;
        }
        n += run;
        (run, weight) = (0, 1);
        if s0.is_none() && n >= n_split {
            s0 = Some(i);
            f0 = f;
        }
        if s >= 33 {
            let consumed = i + 1;
            if switch.is_none() && f + 1 > consumed / 32 + 123 {
                switch = Some((consumed, n, f));
            }
            if draft.is_none() && f + 1 > (consumed - 1) / 32 + 123 {
                draft = Some(consumed);
            }
            if consumed % 32 == 0 && f + 1 == consumed / 32 + 123 {
                eq += 1;
            }
            if s0.is_some() {
                high_suffix += 1;
                rank_min = rank_min.min(s as usize - 1);
                rank_max = rank_max.max(s as usize - 1);
                hist[s as usize - 1] += 1;
            }
            f += 1;
        }
        n += 1;
    }
    Facts {
        syms: b.syms.len(),
        bypass,
        l_bytes: n + run,
        n_split,
        s0: s0.unwrap_or(b.syms.len()),
        f0,
        switch: if bypass { None } else { switch },
        draft: if bypass { None } else { draft },
        high_suffix,
        rank_min: if high_suffix == 0 { 0 } else { rank_min },
        rank_max,
        rank_mode: (0..256).max_by_key(|&r| (hist[r], r)).unwrap(),
        classes: std::array::from_fn(|j| hist[32 * j..32 * j + 32].iter().sum()),
        at_equality: eq,
    }
}

fn encode(text: &[u8], level: u32) -> (Vec<u8>, Facts) {
    assert!(text
        .windows(4)
        .all(|w| !(w[0] == w[1] && w[1] == w[2] && w[2] == w[3])));
    assert!(
        text.len() <= 100_000 * level as usize - 19,
        "one block only"
    );
    let z = common::compress(text, level);
    let mut sc = Scanner::new(SliceSource(&z));
    let mut blocks = Vec::new();
    while let Event::Block { symbols, .. } = sc.next(Vec::new()).unwrap() {
        blocks.push(symbols);
    }
    assert_eq!(blocks.len(), 1);
    let n_split = text.len() - text.iter().filter(|&&c| c == SEP).count();
    let f = facts(&blocks[0], n_split);
    assert_eq!(f.l_bytes, text.len());
    (z, f)
}

/// The suffix script of a boundary design, given the prefix's S_0 and F_0. A burst
/// brings F to `delta` events below the boundary at S = 32k*; events then follow
/// every `period` symbols. `perturb`: (m, j) adds one event at S_m + j (j > 0), or
/// moves S_m's event one symbol earlier (j = -1). Returns the script and S_m list.
fn hug_script(
    s0: usize,
    f0: usize,
    delta: usize,
    period: usize,
    perturb: Option<(usize, isize)>,
    units: usize,
    h: usize,
) -> (Vec<u8>, Vec<usize>) {
    // The lead-in u, v are first uses of values absent from the prefix: two high events.
    let lead = 2;
    let f0 = f0 + lead;
    let k = (1..)
        .find(|&k: &usize| {
            if k + 122 < delta + f0 {
                return false;
            }
            let b = k + 122 - delta - f0;
            let end = s0 + lead + b;
            end < 32 * k && k + 122 - delta <= end / 32 + 123
        })
        .unwrap();
    let burst = k + 122 - delta - f0;
    // The burst's last two slots are lows: after the burst u and v sit behind the pool,
    // so these two first lows are high-rank events too.
    let mut events: Vec<usize> = (0..burst - 2).map(|i| s0 + lead + i + 1).collect();
    let periodic: Vec<usize> = (0..)
        .map(|m| 32 * k + period * m)
        .take_while(|&s| s <= s0 + units)
        .collect();
    events.extend(&periodic);
    if let Some((m, j)) = perturb {
        let at = periodic[m];
        if j < 0 {
            *events.iter_mut().find(|e| **e == at).unwrap() -= 1;
        } else {
            events.push(at + j as usize);
        }
    }
    events.sort_unstable();
    let pool = pool(h);
    let (mut script, mut e, mut low) = (Vec::with_capacity(units), 0, false);
    let mut next = events.iter().peekable();
    for t in 0..units {
        if next.peek() == Some(&&(s0 + t + 1)) {
            next.next();
            script.push(pool[e % h]);
            e += 1;
        } else {
            script.push(if low { V } else { U });
            low = !low;
        }
    }
    (script, periodic)
}

struct Out {
    name: String,
    json: String,
}

fn record(dir: &str, name: &str, seed: u64, design: &str, level: u32, z: &[u8], f: &Facts) -> Out {
    std::fs::write(format!("{dir}/{name}.bz2"), z).unwrap();
    let sw = |o: Option<(usize, usize, usize)>| match o {
        Some((c, n, p)) => format!("{{\"consumed\":{c},\"output\":{n},\"f_before\":{p}}}"),
        None => "null".into(),
    };
    let mut j = String::new();
    write!(
        j,
        "{{\"name\":\"{name}\",\"seed\":\"{seed:#018x}\",\"level\":{level},\"design\":{design},\
         \"symbols\":{},\"bypass\":{},\"l_bytes\":{},\"suffix_output_start\":{},\
         \"suffix_symbol_start\":{},\"high_before_suffix\":{},\"high_in_suffix\":{},\
         \"suffix_rank_min\":{},\"suffix_rank_max\":{},\"suffix_rank_mode\":{},\
         \"suffix_high_by_class\":{:?},\"boundary_equalities\":{},\
         \"oracle_switch\":{},\"draft_rule_switch\":{}}}",
        f.syms,
        f.bypass,
        f.l_bytes,
        f.n_split,
        f.s0,
        f.f0,
        f.high_suffix,
        f.rank_min,
        f.rank_max,
        f.rank_mode,
        &f.classes[1..],
        f.at_equality,
        sw(f.switch),
        f.draft.map_or("null".into(), |c| c.to_string()),
    )
    .unwrap();
    eprintln!(
        "{name}: S={} bypass={} switch={:?}",
        f.syms, f.bypass, f.switch
    );
    Out {
        name: name.into(),
        json: j,
    }
}

/// Smallest-change search for a letter count giving exactly `target` symbols.
fn exact_s(
    target: usize,
    mut n: usize,
    text: impl Fn(usize) -> Vec<u8>,
    level: u32,
) -> Option<usize> {
    // Newton steps, then a deterministic outward scan n0, n0+1, n0-1, n0+2, ...
    for step in 0..20 {
        let s = encode(&text(n), level).1.syms;
        if std::env::var("P74_SUITE_DEBUG").is_ok() {
            eprintln!("  search n={n} S={s}");
        }
        if s == target {
            return Some(n);
        }
        let d = target as isize - s as isize;
        let _ = step;
        n = (n as isize + d) as usize;
    }
    for r in 1..64isize {
        for m in [n as isize + r, n as isize - r] {
            if encode(&text(m as usize), level).1.syms == target {
                return Some(m as usize);
            }
        }
    }
    None
}

fn main() {
    let dir = std::env::args().nth(1).expect("usage: p74_suite DIR");
    std::fs::create_dir_all(&dir).unwrap();
    let mut outs = Vec::new();
    let mut i = 0u64;
    let mut seed = || {
        i += 1;
        BASE_SEED + i
    };
    let all: Vec<u8> = (0x00..=0xFE).collect();

    // 1. Low-rank prefix -> high-rank suffix; split q of ~400k symbols; 256-symbol alphabet.
    for q in [5, 25, 50, 75, 90] {
        let sd = seed();
        let mut rng = Rng(sd);
        let mut t = letters(&mut rng, q * 4_000);
        t.extend(pairs(&mut rng, (100 - q) * 4_000, &all));
        let (z, f) = encode(&t, 9);
        let d = format!("{{\"family\":\"split\",\"q_percent\":{q},\"letters\":{},\"pairs\":{},\"pool\":255,\"expect\":\"switch in suffix\"}}", q * 4_000, (100 - q) * 4_000);
        outs.push(record(&dir, &format!("split-q{q:02}"), sd, &d, 9, &z, &f));
    }

    // 2. Boundary designs (keyed): (name, delta, period, perturb, expectation).
    type Hug = (
        &'static str,
        usize,
        usize,
        Option<(usize, isize)>,
        &'static str,
    );
    let hugs: [Hug; 5] = [
        (
            "hug-eq-p32",
            0,
            32,
            None,
            "no switch; draft rule switches at 32k*",
        ),
        (
            "hug-eq-p32-extra",
            0,
            32,
            Some((1_000, 17)),
            "switch at S_1000 + 17",
        ),
        (
            "hug-eq-p32-early",
            0,
            32,
            Some((1_000, -1)),
            "switch at S_1000 - 1",
        ),
        ("hug-under-p33", 0, 33, None, "no switch"),
        ("hug-over-p31", 4, 31, None, "switch after the slack erodes"),
    ];
    for (name, delta, period, perturb, expect) in hugs {
        let sd = seed();
        let units = 60_000;
        let h = 64;
        // Pre-declared alternates: letter counts 20000 + a, a = 0..8, in order.
        let mut done = None;
        'alt: for a in 0..8 {
            let prefix = letters(&mut Rng(sd), 20_000 + a);
            let (mut s0, mut f0) = (20_000 + a + 4 * units / 3, 0);
            for _ in 0..30 {
                let (script, periodic) = hug_script(s0, f0, delta, period, perturb, units, h);
                let mut t = prefix.clone();
                t.extend(keyed(&script));
                let (z, f) = encode(&t, 9);
                if (f.s0, f.f0) == (s0, f0) {
                    done = Some((z, f, a, periodic));
                    break 'alt;
                }
                (s0, f0) = (f.s0, f.f0);
            }
        }
        let (z, f, a, periodic) = done.expect("no fixed point");
        let want = perturb.map(|(m, j)| (periodic[m] as isize + j) as usize);
        let kstar = periodic[0];
        if std::env::var("P74_SUITE_DEBUG").is_ok() {
            eprintln!("{name}: s0={} f0={} k*={kstar} sw={:?} high_suffix={} eq={} n_periodic={} ranks {}..{}", f.s0, f.f0, f.switch, f.high_suffix, f.at_equality, periodic.len(), f.rank_min, f.rank_max);
        }
        match name {
            "hug-eq-p32" => {
                assert_eq!(f.switch, None);
                assert_eq!(f.draft, Some(kstar));
                assert_eq!(f.at_equality, periodic.len());
            }
            "hug-under-p33" => assert_eq!(f.switch, None),
            "hug-over-p31" => assert!(f.switch.is_some()),
            _ => assert_eq!(f.switch.map(|s| s.0), want),
        }
        let d = format!("{{\"family\":\"boundary\",\"letters\":{},\"units\":{units},\"pool\":{h},\"delta\":{delta},\"period\":{period},\"perturb\":{},\"first_boundary_S\":{kstar},\"periodic_events\":{},\"expect\":\"{expect}\"}}", 20_000 + a, perturb.map_or("null".into(), |(m, j)| format!("[{m},{j}]")), periodic.len());
        outs.push(record(&dir, name, sd, &d, 9, &z, &f));
    }

    // 3. Rank classes (keyed): a burst-only suffix cycling H values, steady rank H - 1;
    //    and rank 255 (254 values incl. 0xFF, one low between events: rank H + 1).
    for h in [33, 64, 65, 128, 129, 192, 193, 225, 253, 254] {
        let sd = seed();
        let mut t = letters(&mut Rng(sd), 20_000);
        let p = pool(h);
        let script: Vec<u8> = if h == 254 {
            [U, V]
                .into_iter()
                .chain((0..8_000).map(|e| {
                    if e % 2 == 0 {
                        p[e / 2 % h]
                    } else if e % 4 == 1 {
                        U
                    } else {
                        V
                    }
                }))
                .collect()
        } else {
            [U, V]
                .into_iter()
                .chain((0..8_000).map(|e| p[e % h]))
                .collect()
        };
        t.extend(keyed(&script));
        let (z, f) = encode(&t, 9);
        let rank = if h == 254 { 255 } else { h - 1 };
        let d = format!("{{\"family\":\"rank-class\",\"letters\":20000,\"units\":{},\"pool\":{h},\"steady_rank\":{rank},\"rank_class\":{},\"expect\":\"switch in suffix\"}}", script.len(), rank / 32);
        outs.push(record(&dir, &format!("rank-{rank:03}"), sd, &d, 9, &z, &f));
    }

    // 4. Short blocks around S0 = 16384 and the 2S >= N_max edges (level 1 and 9).
    for (target, level, npairs, expect) in [
        (16_383, 9, 3_000, "bypass (S < 16384)"),
        (16_384, 9, 3_000, "switch"),
        (16_385, 9, 3_000, "switch"),
        (49_999, 1, 30_000, "switch"),
        (50_000, 1, 30_000, "bypass (2S >= N_max)"),
        (449_999, 9, 200_000, "switch"),
        (450_000, 9, 200_000, "bypass (2S >= N_max)"),
    ] {
        let sd = seed();
        // Separate streams: the pairs don't change with the letter count. Pre-declared
        // alternates if no letter count gives S exactly: npairs + 1, + 2, ... + 7.
        let (n, npairs) = (npairs..npairs + 8)
            .find_map(|npairs| {
                let text = |n: usize| {
                    let mut t = letters(&mut Rng(sd), n);
                    t.extend(pairs(&mut Rng(sd ^ PAIR_STREAM), npairs, &all));
                    t
                };
                exact_s(target, target.saturating_sub(npairs), text, level).map(|n| (n, npairs))
            })
            .expect("no exact S");
        let text = |n: usize| {
            let mut t = letters(&mut Rng(sd), n);
            t.extend(pairs(&mut Rng(sd ^ PAIR_STREAM), npairs, &all));
            t
        };
        let (z, f) = encode(&text(n), level);
        assert_eq!(f.syms, target);
        let d = format!("{{\"family\":\"edge\",\"target_symbols\":{target},\"letters\":{n},\"pairs\":{npairs},\"pool\":255,\"expect\":\"{expect}\"}}");
        outs.push(record(
            &dir,
            &format!("edge-l{level}-s{target}"),
            sd,
            &d,
            level,
            &z,
            &f,
        ));
    }

    let mut j = String::from("{\"generator\":\"examples/p74_suite.rs\",\"base_seed\":\"");
    writeln!(j, "{BASE_SEED:#018x}\",\"files\":[").unwrap();
    let body: Vec<String> = outs.iter().map(|o| o.json.clone()).collect();
    j.push_str(&body.join(",\n"));
    j.push_str("\n]}\n");
    std::fs::write(format!("{dir}/intent.json"), j).unwrap();
    let names: Vec<&str> = outs.iter().map(|o| o.name.as_str()).collect();
    std::fs::write(
        format!("{dir}/files.txt"),
        names
            .iter()
            .map(|n| format!("{dir}/{n}.bz2\n"))
            .collect::<String>(),
    )
    .unwrap();
}
