"""Assemble the combined harness from the scanner trial's harness (ctor2su.rs at ba261ef),
adding the descriptor prototype's and v3's registered classes and identities."""
import re, sys
src = open('su.h').read()
d = open('d.h').read()
v3 = open('v3.h').read()

def between(text, start, end):
    i = text.index(start); j = text.index(end, i)
    return text[i:j]

h = src.replace('Ctor2su', 'Ctor2c').replace('Stats2su', 'Stats2c').replace('ctor2su-', 'ctor2c-')

# same_as_v2: add the descriptor's and v3's identities before its closing brace
anchor = '    assert!(s.bitmap_memsets <= s.blocks, "{what}: bitmap fills");\n}\n'
assert h.count(anchor) == 1
extra = '''    assert!(s.bitmap_memsets <= s.blocks, "{what}: bitmap fills");
    // the descriptor prototype's identities (its S62-F01 counters' partitions)
    assert_eq!(s.outer_iterations, s.tiles_nonuniform + s.runs, "{what}: outer iterations");
    assert_eq!(
        s.runs_end_fill + s.runs_end_complete,
        s.runs,
        "{what}: run endings"
    );
    assert_eq!(s.sweep_checks, 256 * s.blocks, "{what}: sweep checks");
    assert_eq!(
        s.init_descs,
        256 * s.blocks,
        "{what}: descriptors initialised"
    );
    assert!(s.fills_vector <= s.fills, "{what}: vector fills");
    assert!(
        s.tile_rows_second + s.tail_rows <= s.rows_scalar,
        "{what}: scalar rows by segment"
    );
    assert!(
        s.tile_refills_second + s.tail_refills_first + s.tail_refills_second <= s.refills,
        "{what}: refills by site"
    );
    assert!(s.refill_finishes <= s.refills, "{what}: refill finishes");
    assert!(s.pad_blocks <= s.blocks, "{what}: padded blocks");
    assert!(
        s.bitmap_reserves_single <= s.bitmap_reserves,
        "{what}: bitmap reserves"
    );
    // v3's identities: every eligible finish (v2-fast, 30 fields) took the leaf, and
    // nothing else did
    assert_eq!(s.leaf_fast, s.v2_fast_full, "{what}: leaf finishes");
    // v2's fast finishes are the leaf's and the same-epoch partial ones
    assert_eq!(
        s.fin_fast,
        s.leaf_fast + s.slow_ordinary_partial_same_epoch,
        "{what}: fast"
    );
    // every finish is the leaf's or the slow path's
    let finishes = s.fin_fast + s.fin_ordinary + s.fin_wide + s.fin_crossing + s.fin_forced;
    assert_eq!(
        finishes,
        s.leaf_fast + s.slow_entries,
        "{what}: finish total"
    );
    assert_eq!(
        s.slow_entries,
        s.slow_ordinary_old_epoch + s.slow_ordinary_partial_same_epoch + s.slow_raw_preset,
        "{what}: slow classes"
    );
    assert_eq!(
        s.slow_raw_preset,
        s.fin_crossing + s.fin_forced,
        "{what}: preset RAW"
    );
    assert!(s.final_finishes <= finishes, "{what}: final finishes");
}
'''
h = h.replace(anchor, extra)
h = h.replace('/// The prototype\'s counters against v2\'s counted build',
              '/// The combined build\'s counters against v2\'s counted build')

# the registered classes: the descriptor's desc_class and v3's selection, after scanner_class
dclass = between(d, '/// The prototype\'s registered classes (pre-registration §0.4), each runnable', '/// `ctor2d-synthetic --miri CLASS`')
dclass = dclass.replace('DESC_CLASSES', 'DESC_CLASSES').replace('Stats2d', 'Stats2c')
dclass = dclass.replace("/// The prototype's registered classes (pre-registration §0.4), each runnable on its own\n/// (`--miri CLASS`), all of them in the full and quick suites",
                        "/// The descriptor prototype's registered classes (its pre-registration §0.4), run as\n/// `descriptor:CLASS` under Miri, all of them in the full and quick suites")
v3sel = between(v3, '/// The slow finish\'s preset-RAW test', 'fn synthetic(quick: bool) {')
# split v3's one selection into its four marked groups, each a class of the union
v3body = '''/// The slow finish's preset-RAW test (v3's discriminator)
fn is_preset_raw(raw: u32) -> bool {
    raw & ORD == 0
}

/// v3's targeted Miri selection (`ctor3-synthetic --miri`, 134 cases), as its four marked
/// groups, each runnable on its own as `v3:CLASS`: small blocks, the tag/RAW dispatch
/// counterexample (tiny, and a later epoch's partial portion), and the exact span cases
/// 3 → 61,440 (65,533, ordinary) and 2 → 61,440 (65,534, RAW)
const V3_CLASSES: [&str; 4] = ["small", "tag-tiny", "span", "tag-epoch1"];

fn v3_class(s: &mut Synth, class: &str) {
    let from = s.cases;
    match class {
        "small" => {
            let mut rng = Rng(0xc303);
            for n in 0..=40usize {
                for sigma in [1usize, 2, 3] {
                    let mut v = Vec::with_capacity(n);
                    while v.len() < n {
                        let len = [1, 2, 7, 16, 17, 30, 32, 33][rng.below(8)];
                        let b = (10 * rng.below(sigma)) as u8;
                        v.extend(std::iter::repeat(b).take(len.min(n - v.len())));
                    }
                    s.case(&format!("miri small n {n} sigma {sigma}"), &v, 0);
                }
            }
            for n in [1usize, 2, 31, 61] {
                s.case(&format!("miri small n {n} forced"), &vec![5u8; n], 1000);
            }
        }
        "tag-tiny" => {
            for n in 1..=4usize {
                let v: Vec<u8> = (0..n).map(|k| (k % 2) as u8 * 7).collect();
                s.case(&format!("tag dispatch tiny n {n}"), &v, 0);
            }
        }
        "span" => {
            for (label, first, want_wide) in [
                ("span 3 -> 61440 (65,533, ordinary)", 3usize, false),
                ("span 2 -> 61440 (65,534, RAW)", 2usize, true),
            ] {
                let mut ones: Vec<usize> = (0..29).map(|t| first + 2 * t).collect();
                ones.push(61_440);
                let st = s.case(label, &placed(61_440 + 200, &ones, 0), 0);
                assert_eq!(st.fin_wide, want_wide as u64, "{label}: wide groups");
            }
        }
        "tag-epoch1" => {
            let n = EPOCH + 300 + 1;
            let mut v = Vec::with_capacity(n);
            filler(&mut v, n);
            v[EPOCH + 10] = 9;
            let st = s.case("tag dispatch epoch 1 count 1", &v, 0);
            assert!(st.slow_ordinary_partial_same_epoch >= 1);
        }
        c => panic!("unknown v3 class {c}"),
    }
    s.mark(&format!("v3 class {class}"), from);
}

/// The union of the components' registered Miri classes, as `COMPONENT:CLASS`
fn union_classes() -> Vec<String> {
    let mut v: Vec<String> = SCANNER_CLASSES.iter().map(|c| format!("scanner:{c}")).collect();
    v.extend(DESC_CLASSES.iter().map(|c| format!("descriptor:{c}")));
    v.extend(V3_CLASSES.iter().map(|c| format!("v3:{c}")));
    v
}

fn run_class(s: &mut Synth, class: &str) {
    match class.split_once(':') {
        Some(("scanner", c)) => scanner_class(s, c),
        Some(("descriptor", c)) => desc_class(s, c),
        Some(("v3", c)) => v3_class(s, c),
        _ => panic!("unknown class {class}: COMPONENT:CLASS, COMPONENT one of scanner, descriptor, v3"),
    }
}

'''
# the scanner's synthetic_miri: generalised to the union
smiri = between(h, '/// `ctor2c-synthetic --miri CLASS`: one registered class, unpoisoned arenas', 'fn synthetic(quick: bool) {')
newmiri = '''/// `ctor2c-synthetic --miri CLASS`: one registered class of the union (`COMPONENT:CLASS`),
/// a component's classes (`COMPONENT`), or all of them (`all`); unpoisoned arenas
fn synthetic_miri(class: &str) {
    let mut s = Synth {
        a: Arenas::default(),
        st: Stats2c::default(),
        ck: Checked::default(),
        cases: 0,
        poisoned: !cfg!(miri),
        classes: Vec::new(),
    };
    let classes: Vec<String> = if class == "all" {
        union_classes()
    } else if !class.contains(':') {
        union_classes()
            .into_iter()
            .filter(|c| c.starts_with(&format!("{class}:")))
            .collect()
    } else {
        vec![class.to_string()]
    };
    assert!(!classes.is_empty(), "no class {class}");
    for c in &classes {
        run_class(&mut s, c);
    }
    println!(
        "ctor2c-synthetic --miri {class}: {} cases, {} array comparisons, {} walk outputs, {} restorations: every check exact; fresh {}, lookahead {}, held {}, no-full-tile runs {}, nonuniform {}, second segments {}, tails {}; scalar rows {}, fragment rows {}, runs {}, refills {}, crossing {}, forced {}, epochs {}; leaf {}, slow {} (old epoch {}, partial {}, preset RAW {}), wide {}",
        s.cases,
        s.ck.arrays,
        s.ck.walks,
        s.ck.restorations,
        s.st.fresh_pairs,
        s.st.lookahead_pairs,
        s.st.held_reuses,
        s.st.runs_no_full_tile,
        s.st.tiles_nonuniform,
        s.st.tiles_second_segment,
        s.st.tails,
        s.st.rows_scalar,
        s.st.rows_fragment,
        s.st.runs,
        s.st.refills,
        s.st.fin_crossing,
        s.st.fin_forced,
        s.st.epochs,
        s.st.leaf_fast,
        s.st.slow_entries,
        s.st.slow_ordinary_old_epoch,
        s.st.slow_ordinary_partial_same_epoch,
        s.st.slow_raw_preset,
        s.st.fin_wide
    );
}

'''
h = h.replace(smiri, dclass + v3body + newmiri)

# synthetic(): after the scanner's classes, the descriptor's classes and v3's suite classes
v3suite = between(v3, '    // v3: the tag/RAW dispatch (the audit\'s counterexample).', '    if !quick {\n        // 8. RAW discovery order')
anchor = '''    for c in SCANNER_CLASSES {
        scanner_class(&mut s, c);
    }
'''
assert h.count(anchor) == 1
h = h.replace(anchor, anchor + '''
    // the descriptor prototype's registered classes (its pre-registration §0.4)
    for c in DESC_CLASSES {
        desc_class(&mut s, c);
    }

''' + v3suite)
h = h.replace("    // the prototype's registered scanner classes (pre-registration §0.4)\n",
              "    // the scanner trial's registered classes (its pre-registration §0.4)\n")

# main: count modes for the combined build
cnt = between(h, '        // the ledger\'s cross-check: each block of each file built once', '        "synthetic" if args.first()')
dcounted = between(d, '        // S62-F01: the counted build alone, in the `--only d` process', '        "count" if args.first().map(String::as_str) == Some("--only")')
dcounted = dcounted.replace('Ctor2d', 'Ctor2c').replace('Stats2d', 'Stats2c').replace('c2d', 'c2c').replace('`--only d`', '`--only c`')
newcnt = cnt.replace('"su" => assert!(c2s.build', '"c" => assert!(c2s.build').replace('--only {x}: v2 or su', '--only {x}: v2 or c')
newcnt = newcnt.replace('this prototype\'s uncounted build', 'the combined uncounted build')
h = h.replace(cnt, dcounted + newcnt)
h = h.replace('/// `ctor2c-check`, `ctor2c-adversarial`, `ctor2c-synthetic`', '/// `ctor2c-check`, `ctor2c-adversarial`, `ctor2c-synthetic`, `ctor2c-count`')
open('harness.rs', 'w').write(h)
print(len(h.splitlines()))
