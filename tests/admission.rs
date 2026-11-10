//! Which path a block takes (`runs::takes_runs`): the records' for a block of few symbols,
//! the column's for a block of high entropy, whatever its length.
//!
//! The bound alone (`2 * syms <= max_len`) sends a *short* block of high entropy by records,
//! since it has few symbols against its bound whatever they are: a small incompressible
//! file, or a stream's last block. The share of run symbols tells it apart.

mod common;

use mibz2::runs::takes_runs;
use mibz2::stream::{Event, Scanner, SliceSource};

/// Each block of `data` compressed at level 9: (its symbols' count, its bound, taken by
/// records)
fn blocks(data: &[u8]) -> Vec<(usize, usize, bool)> {
    let bz = common::compress(data, 9);
    let mut sc = Scanner::new(SliceSource(&bz));
    let mut syms = Vec::new();
    let mut out = Vec::new();
    while let Event::Block { symbols: b, .. } = sc.next(std::mem::take(&mut syms)).unwrap() {
        out.push((b.syms.len(), b.max_len, takes_runs(&b)));
        syms = b.syms;
    }
    out
}

/// Entity records as lines of JSON, in the shape of a Wikidata dump: the same keys and
/// property names in every line, ids and values that vary
fn entities(rng: &mut common::Rng, len: usize) -> Vec<u8> {
    let props = ["P31", "P279", "P17", "P131", "P625", "P18", "P373", "P646"];
    let mut out = Vec::with_capacity(len + 512);
    let mut id = 1u64;
    while out.len() < len {
        id += 1 + rng.below(40);
        let mut line = format!(
            "{{\"type\":\"item\",\"id\":\"Q{id}\",\"labels\":{{\"en\":{{\"language\":\"en\",\"value\":\"item {}\"}}}},\"claims\":{{",
            rng.below(1 << 20)
        );
        for (i, p) in props.iter().enumerate().take(2 + rng.below(6) as usize) {
            if i > 0 {
                line.push(',');
            }
            line.push_str(&format!(
                "\"{p}\":[{{\"mainsnak\":{{\"snaktype\":\"value\",\"property\":\"{p}\",\"datavalue\":{{\"value\":{{\"entity-type\":\"item\",\"numeric-id\":{},\"id\":\"Q{}\"}},\"type\":\"wikibase-entityid\"}},\"datatype\":\"wikibase-item\"}},\"type\":\"statement\",\"rank\":\"normal\"}}]",
                rng.below(1 << 24),
                rng.below(1 << 24)
            ));
        }
        line.push_str("}},\n");
        out.extend(line.bytes());
    }
    out
}

/// A short block of high entropy keeps the column, though the bound alone would admit it;
/// a full one keeps it as before
#[test]
fn a_short_high_entropy_block_takes_the_column() {
    let mut rng = common::Rng(0x5eed_0bad_cafe_f00d);
    for len in [1_000usize, 20_000, 200_000, 440_000] {
        let b = blocks(&rng.bytes(len));
        assert_eq!(b.len(), 1, "{len} bytes: one block");
        let (syms, max_len, by_records) = b[0];
        assert!(
            2 * syms <= max_len,
            "{len} bytes: the bound alone admits it"
        );
        assert!(!by_records, "{len} random bytes went by records");
    }
    for (syms, max_len, by_records) in blocks(&rng.bytes(2_000_000)) {
        assert!(
            !by_records,
            "a random block of {syms} symbols (bound {max_len}) went by records"
        );
    }
}

/// Text and entity records take the records' path, in full blocks and in short ones
#[test]
fn text_and_entity_records_take_the_records() {
    let mut rng = common::Rng(0x0dd_ba11_5eed);
    let text = common::corpora(1)
        .into_iter()
        .find(|c| c.0 == "text")
        .unwrap()
        .1;
    let json = entities(&mut rng, 3_000_000);
    for (name, data) in [("text", &text), ("entities", &json)] {
        let all = blocks(data);
        assert!(all.len() >= 3, "{name}: {} blocks", all.len());
        for (i, &(syms, max_len, by_records)) in all.iter().enumerate() {
            assert!(
                by_records,
                "{name}: block {i} ({syms} symbols, bound {max_len}) kept the column"
            );
        }
        for len in [5_000usize, 60_000, 300_000] {
            let short = blocks(&data[..len]);
            assert!(
                short.iter().all(|b| b.2),
                "{name}: its first {len} bytes kept the column"
            );
        }
    }
}

/// On the crate's corpora the rule decides as the bound alone decides: it changes nothing
/// for a block whose symbols are many against its bound, nor for one of long runs
#[test]
fn full_blocks_decide_as_the_bound_decides() {
    for (name, data) in common::corpora(1) {
        for (i, (syms, max_len, by_records)) in blocks(&data).into_iter().enumerate() {
            // (a stream's last block may be short: only the bound's refusals are held to,
            // and its admissions of blocks that are not high entropy)
            if 2 * syms > max_len {
                assert!(
                    !by_records,
                    "{name}: block {i} is past the bound and went by records"
                );
            } else if name != "random" && name != "records" {
                assert!(
                    by_records,
                    "{name}: block {i} ({syms} symbols) kept the column"
                );
            }
        }
    }
}
