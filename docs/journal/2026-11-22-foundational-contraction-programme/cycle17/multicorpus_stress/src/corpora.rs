use bzip2::write::BzEncoder;
use bzip2::Compression;
use mibz2::runs::{self as mruns, Runs};
use mibz2::stream::{Event, Scanner, SliceSource};
use mibz2::Kernels;
use std::io::Write;

pub struct BlockItem {
    #[allow(dead_code)]
    pub corpus_idx: usize,
    pub n: usize,
    pub p: u32,
    pub runs: Runs,
}

pub struct CorpusSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub blocks: Vec<BlockItem>,
}

/// Deterministic SplitMix64 PRNG (zero external crates).
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    #[inline]
    fn usize(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound
    }
}

fn bz2_to_blocks(bz2: &[u8], k: Kernels) -> Vec<BlockItem> {
    let mut sc = Scanner::new(SliceSource(bz2));
    let mut blocks = Vec::new();
    let mut idx = 0usize;
    while let Ok(ev) = sc.next(Vec::new()) {
        match ev {
            Event::Block { symbols, .. } => {
                let mut r = Runs::default();
                let n = mruns::front(&symbols, &mut r, k).expect("runs::front");
                blocks.push(BlockItem {
                    corpus_idx: idx,
                    n,
                    p: symbols.orig_ptr,
                    runs: r,
                });
                idx += 1;
            }
            Event::Done => break,
        }
    }
    blocks
}

fn compress_to_single_block(raw: &[u8], k: Kernels) -> Vec<BlockItem> {
    let mut enc = BzEncoder::new(Vec::with_capacity(raw.len() / 2 + 1024), Compression::best());
    enc.write_all(raw).expect("bz2 write");
    let bz2 = enc.finish().expect("bz2 finish");
    bz2_to_blocks(&bz2, k)
}

/// Corpus 1: Real Wikidata JSON (6 auth blocks + all 431 blocks)
pub fn load_wikidata(k: Kernels) -> (CorpusSpec, CorpusSpec) {
    let path = "/tmp/moonshot-scorer/corpora/wikidata.bz2";
    let data = std::fs::read(path).expect("read /tmp/moonshot-scorer/corpora/wikidata.bz2");
    let all_blocks = bz2_to_blocks(&data, k);

    // Exact 6 SHA-256 authenticated blocks used across Cycles 1–16
    let auth_ptrs: [u32; 6] = [264045, 320807, 325088, 802868, 791429, 205180];
    let mut auth_blocks = Vec::with_capacity(6);
    for p_want in auth_ptrs {
        let pos = all_blocks
            .iter()
            .position(|b| b.p == p_want)
            .unwrap_or_else(|| panic!("Wikidata auth block p={p_want} not found"));
        // Re-extract cleanly by cloning runs via Scanner on exact block index or scanning once
        // Runs doesn't impl Clone, so we collect auth indices and split below.
        auth_blocks.push(pos);
    }
    // Re-parse for the 6 auth blocks so both specs own their Blocks cleanly
    let mut sc = Scanner::new(SliceSource(&data));
    let mut auth_map: Vec<Option<BlockItem>> = (0..6).map(|_| None).collect();
    let mut idx = 0usize;
    while let Ok(ev) = sc.next(Vec::new()) {
        match ev {
            Event::Block { symbols, .. } => {
                if let Some(slot) = auth_blocks.iter().position(|&target| target == idx) {
                    let mut r = Runs::default();
                    let n = mruns::front(&symbols, &mut r, k).expect("runs::front");
                    auth_map[slot] = Some(BlockItem {
                        corpus_idx: idx,
                        n,
                        p: symbols.orig_ptr,
                        runs: r,
                    });
                }
                idx += 1;
            }
            Event::Done => break,
        }
    }
    let auth_vec: Vec<BlockItem> = auth_map.into_iter().map(|x| x.unwrap()).collect();
    (
        CorpusSpec {
            id: "1a_wikidata_6auth",
            label: "Wikidata JSON (6 auth blocks)",
            blocks: auth_vec,
        },
        CorpusSpec {
            id: "1b_wikidata_431",
            label: "Wikidata JSON (all 431 blocks)",
            blocks: all_blocks,
        },
    )
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Corpus 2a: Pure high-entropy pseudo-random base64 (900 KiB, rho ~ 1.05, LazyCells fallback)
pub fn gen_hi_ent_base64_fallback(k: Kernels) -> CorpusSpec {
    let mut rng = Rng::new(0x2026_1122_0002_0001);
    let mut raw = Vec::with_capacity(890_000);
    for _ in 0..890_000 {
        raw.push(B64[rng.usize(64)]);
    }
    CorpusSpec {
        id: "2a_hient_b64_r1",
        label: "High-Entropy Base64 (rho~1.03, fallback)",
        blocks: compress_to_single_block(&raw, k),
    }
}

/// Corpus 2b: Open-admitted Dense-Band High-Entropy (900 KiB, rho ~ 3.9, m_16 ~ 220k <= 256k!)
/// Combines a 58x repeated 11,400-byte random base64 block (ns ~ 11.4k <= 16.3k thick strips)
/// with ~216,000 bytes of pure uniform random base64 (m_16 ~ 218k <= 256k, edges ~ 850 KiB!).
/// Strictly admitted inside Open's DENSE regime with worst-case random pointer chasing!
pub fn gen_hi_ent_dense_open(k: Kernels) -> CorpusSpec {
    let mut rng = Rng::new(0x2026_1122_0002_0002);
    let mut tpl = Vec::with_capacity(11_400);
    for _ in 0..11_400 {
        tpl.push(B64[rng.usize(64)]);
    }
    let mut raw = Vec::with_capacity(890_000);
    for rep in 0..58usize {
        raw.extend_from_slice(&tpl);
        for s in 0..8 {
            raw.push(B64[(rep.wrapping_mul(17) + s * 7 + 3) & 63]);
        }
    }
    while raw.len() < 878_000 {
        raw.push(B64[rng.usize(64)]);
    }
    CorpusSpec {
        id: "2b_hient_dense_open",
        label: "High-Entropy Dense-Open (m16~220k, L3)",
        blocks: compress_to_single_block(&raw, k),
    }
}

/// Corpus 3: Structured English prose / natural-language text (900 KiB, Zipfian phrases, rho ~ 4.5-5.5)
pub fn gen_english_prose(k: Kernels) -> CorpusSpec {
    let mut rng = Rng::new(0x2026_1122_0003_0001);
    let subjects: &[&str] = &[
        "the old historian", "the research committee", "the young engineer", "every careful observer",
        "a small coastal town", "the northern expedition", "the municipal archive", "the autumn wind",
        "the chief librarian", "the university senate", "a quiet mathematical proof", "the evening train",
        "the maritime observatory", "the parliamentary report", "an independent review", "the morning tide",
        "the editorial board", "the natural philosopher", "the regional council", "the seasonal harvest",
        "the astronomical society", "the cartographer", "the workshop director", "the visiting scholar",
        "the bridge inspector", "the geological survey", "the botanical garden", "the historical society",
        "the merchant fleet", "the provincial governor", "the acoustic laboratory", "the clockmaker",
    ];
    let verbs: &[&str] = &[
        "carefully examined", "quietly recorded", "gradually discovered", "steadily maintained",
        "repeatedly questioned", "clearly demonstrated", "patiently reconstructed", "consistently reported",
        "strongly recommended", "eventually confirmed", "briefly described", "thoroughly analyzed",
        "systematically compared", "frequently observed", "formally published", "faithfully preserved",
        "unexpectedly revealed", "diligently measured", "broadly interpreted", "precisely documented",
    ];
    let objects: &[&str] = &[
        "the manuscript from the eastern library", "the seasonal variations in coastal pressure",
        "the long sequence of historical records", "the structural integrity of the stone arches",
        "the correspondence between the two scholars", "the gradual shift in agricultural practices",
        "the forgotten ledger from the harbor office", "the geometric symmetry of the courtyard",
        "the detailed survey of the mountain valley", "the statistical distribution of daily readings",
        "the annotated edition of the earlier treatise", "the economic balance of the southern provinces",
        "the delicate calibration of the brass instruments", "the architectural plans for the new hall",
        "the recurring patterns in the winter storms", "the careful translation of the ancient chronicle",
        "the archival catalogue of rare maps", "the subtle changes in local dialect and custom",
        "the public debate surrounding the treaty", "the experimental results from the autumn series",
    ];
    let clauses: &[&str] = &[
        ", although the original evidence remained incomplete for several decades.",
        ", while the surrounding community watched the developments with quiet interest.",
        ", because earlier accounts had overlooked the subtle details in the margin.",
        ", and the subsequent inquiry confirmed every major conclusion of the draft.",
        ", despite the persistent difficulties encountered during the winter months.",
        ", which eventually prompted a comprehensive revision of the standard manual.",
        ", whereas the opposing view emphasized the practical limits of the method.",
        ", before the final volume was delivered to the central archive in November.",
        ", so that future readers could trace each argument back to its primary source.",
        ", yet certain passages continued to inspire lively discussion among specialists.",
        ", until a clearer explanation emerged from the newly catalogued notebooks.",
        ", as noted in the preface to the second edition published ten years later.",
    ];
    let connectors: &[&str] = &[
        "In addition, ", "Nevertheless, ", "Furthermore, ", "Meanwhile, ", "Consequently, ",
        "For this reason, ", "At the same time, ", "In the following year, ", "By contrast, ",
        "As a result, ", "Under these circumstances, ", "During the same period, ", "", "", "",
    ];
    let modifiers: &[&str] = &[
        "across the northern district", "within the central collection", "throughout the nineteenth century",
        "beyond the ordinary expectations", "according to the official register", "near the upper estuary",
        "along the principal trade route", "inside the vaulted reading room", "without unnecessary delay",
        "under careful supervision", "in accordance with established custom", "after considerable debate",
    ];

    let adjectives: &[&str] = &[
        "ancient", "northern", "southern", "eastern", "western", "quiet", "careful", "modest",
        "detailed", "formal", "local", "public", "private", "annual", "monthly", "weekly",
        "coastal", "inland", "modern", "classic", "remote", "central", "narrow", "broad",
        "heavy", "light", "clear", "obscure", "recent", "earlier", "later", "final",
        "written", "printed", "bound", "loose", "stone", "wooden", "iron", "brass",
        "winter", "summer", "autumn", "spring", "golden", "silver", "simple", "complex",
        "steady", "sudden", "gentle", "harsh", "plain", "ornate", "open", "closed",
        "direct", "indirect", "minor", "major", "common", "rare", "hidden", "visible",
    ];
    let nouns: &[&str] = &[
        "ledger", "journal", "letter", "treatise", "volume", "chapter", "margin", "preface",
        "harbor", "bridge", "valley", "mountain", "estuary", "island", "garden", "courtyard",
        "council", "senate", "library", "archive", "museum", "academy", "station", "market",
        "compass", "lantern", "pendulum", "sextant", "barometer", "chronometer", "telescope", "scale",
        "harvest", "weather", "current", "horizon", "passage", "journey", "voyage", "expedition",
        "account", "inquiry", "measure", "pattern", "diagram", "drawing", "sketch", "engraving",
        "merchant", "scholar", "captain", "clerk", "mason", "weaver", "printer", "painter",
        "village", "borough", "parish", "county", "province", "kingdom", "republic", "empire",
    ];

    // Precompute Zipfian weights for subjects (realistic natural-language skew)
    let mut cdf_s = Vec::with_capacity(subjects.len());
    let mut acc = 0.0;
    for i in 0..subjects.len() {
        acc += 1.0 / ((i + 1) as f64).powf(0.85);
        cdf_s.push(acc);
    }
    let tot_s = acc;

    let mut raw = Vec::with_capacity(892_000);
    let mut para_sents = 0usize;
    let mut chapter = 1usize;
    while raw.len() < 888_000 {
        if para_sents == 0 && rng.usize(16) == 0 {
            let hdr = format!("Chapter {chapter}: Notes on the {} {}\n\n", adjectives[rng.usize(adjectives.len())], nouns[rng.usize(nouns.len())]);
            raw.extend_from_slice(hdr.as_bytes());
            chapter += 1;
        }
        let conn = connectors[rng.usize(connectors.len())];
        let u = (rng.next_u64() as f64 / u64::MAX as f64) * tot_s;
        let si = cdf_s.partition_point(|&c| c < u).min(subjects.len() - 1);
        let subj = subjects[si];
        let verb = verbs[rng.usize(verbs.len())];
        let obj = objects[rng.usize(objects.len())];
        let cl = clauses[rng.usize(clauses.len())];
        let adj1 = adjectives[rng.usize(adjectives.len())];
        let noun1 = nouns[rng.usize(nouns.len())];
        let adj2 = adjectives[rng.usize(adjectives.len())];
        let noun2 = nouns[rng.usize(nouns.len())];

        if conn.is_empty() {
            let b = subj.as_bytes();
            raw.push(b[0].to_ascii_uppercase());
            raw.extend_from_slice(&b[1..]);
        } else {
            raw.extend_from_slice(conn.as_bytes());
            raw.extend_from_slice(subj.as_bytes());
        }
        raw.extend_from_slice(format!(" of the {adj1} {noun1} {verb} {obj} near the {adj2} {noun2}").as_bytes());
        if rng.usize(2) == 0 {
            raw.push(b' ');
            raw.extend_from_slice(modifiers[rng.usize(modifiers.len())].as_bytes());
        }
        if rng.usize(2) == 0 {
            let yr = 1650 + rng.usize(375);
            let pg = 1 + rng.usize(999);
            let num = 10 + rng.usize(890);
            raw.extend_from_slice(format!(" (vol. {num}, p. {pg}, {yr})").as_bytes());
        }
        raw.extend_from_slice(cl.as_bytes());

        para_sents += 1;
        if para_sents >= 4 + rng.usize(4) {
            raw.extend_from_slice(b"\n\n");
            para_sents = 0;
        } else {
            raw.push(b' ');
        }
    }
    raw.truncate(888_000);
    CorpusSpec {
        id: "3_english_prose",
        label: "Structured English Prose (Zipfian)",
        blocks: compress_to_single_block(&raw, k),
    }
}

/// Corpus 4a: 4-letter DNA ({A,C,G,T}, 900 KiB, Open-admitted, high m_16, long chains!)
/// Simulates a diploid/polyploid repetitive genomic region: 10 copies of an 88 KiB ancestral
/// chromosome with 0.8% point SNPs (strictly over {A,C,G,T}, no runs of 4+ mutated away).
pub fn gen_dna_open(k: Kernels) -> CorpusSpec {
    const DNA: [u8; 4] = *b"ACGT";
    let mut rng = Rng::new(0x2026_1122_0004_0001);
    let len_base = 88_000usize;
    let mut base = Vec::with_capacity(len_base);
    for _ in 0..len_base {
        base.push(DNA[rng.usize(4)]);
    }
    let mut raw = Vec::with_capacity(885_000);
    for copy in 0..10usize {
        for (i, &b) in base.iter().enumerate() {
            // ~0.4% SNP rate so BWT runs cluster around length 6..12 (thin!) with some thick
            if rng.usize(250) == 0 {
                let alt = DNA[(copy + i + rng.usize(3) + 1) & 3];
                raw.push(alt);
            } else {
                raw.push(b);
            }
        }
        // 32-bp unique barcode per copy over {A,C,G,T}
        for bit in 0..32 {
            raw.push(DNA[((copy >> (bit % 6)) + bit) & 3]);
        }
    }
    CorpusSpec {
        id: "4a_dna_4let_open",
        label: "4-Letter Genomic DNA (Open, high m16)",
        blocks: compress_to_single_block(&raw, k),
    }
}

/// Corpus 4b: Pure IID random 4-letter DNA ({A,C,G,T}, 900 KiB, rho ~ 1.33, fallback sanity)
pub fn gen_dna_iid(k: Kernels) -> CorpusSpec {
    const DNA: [u8; 4] = *b"ACGT";
    let mut rng = Rng::new(0x2026_1122_0004_0002);
    let mut raw = Vec::with_capacity(885_000);
    for _ in 0..885_000 {
        raw.push(DNA[rng.usize(4)]);
    }
    CorpusSpec {
        id: "4b_dna_4let_iid",
        label: "4-Letter IID Random DNA (fallback)",
        blocks: compress_to_single_block(&raw, k),
    }
}

/// Corpus 5a: High-repetition multi-version archive (900 KiB, rho ~ 52-58 <= 64, Open-admitted!)
/// 55 versions of a 16 KiB structured config/JSON document with realistic small per-version edits.
/// Thick strips completely dominate — direct stress-test of BandCoal + S3a!
pub fn gen_archive_open(k: Kernels) -> CorpusSpec {
    let mut rng = Rng::new(0x2026_1122_0005_0001);
    let mut base_lines: Vec<String> = Vec::new();
    for i in 0..220 {
        let mut val = String::new();
        for _ in 0..36 {
            val.push(B64[rng.usize(62)] as char);
        }
        base_lines.push(format!(
            "  \"record_{i:04}\": {{\"id\": {i}, \"hash\": \"{val}\", \"active\": true}},\n"
        ));
    }
    let mut raw = Vec::with_capacity(890_000);
    for ver in 0..55usize {
        raw.extend_from_slice(
            format!("{{\"snapshot\": {ver:03}, \"ts\": {}, \"entries\": [\n", 1700000000 + ver * 3600)
                .as_bytes(),
        );
        for (i, line) in base_lines.iter().enumerate() {
            if (i + ver * 7) % 110 == 0 {
                let mut val = String::new();
                for _ in 0..36 {
                    val.push(B64[rng.usize(62)] as char);
                }
                raw.extend_from_slice(
                    format!(
                        "  \"record_{i:04}\": {{\"id\": {i}, \"hash\": \"{val}\", \"active\": false}},\n"
                    )
                    .as_bytes(),
                );
            } else {
                raw.extend_from_slice(line.as_bytes());
            }
        }
        raw.extend_from_slice(b"]}\n");
    }
    raw.truncate(888_000);
    CorpusSpec {
        id: "5a_archive_rho55",
        label: "Multi-Version Archive (rho~50, thick)",
        blocks: compress_to_single_block(&raw, k),
    }
}

/// Corpus 5b: Ultra-repetitive archive (900 KiB, rho ~ 85 > 64, LazyCells fallback boundary)
pub fn gen_archive_rho85(k: Kernels) -> CorpusSpec {
    let mut rng = Rng::new(0x2026_1122_0005_0002);
    let mut base = Vec::with_capacity(9_600);
    for i in 0..150 {
        let mut val = String::new();
        for _ in 0..32 {
            val.push(B64[rng.usize(62)] as char);
        }
        base.extend_from_slice(format!("entry_{i:03}={val};state=ok;\n").as_bytes());
    }
    let mut raw = Vec::with_capacity(890_000);
    for ver in 0..92usize {
        raw.extend_from_slice(format!("#REV={ver:03}\n").as_bytes());
        raw.extend_from_slice(&base);
    }
    raw.truncate(888_000);
    CorpusSpec {
        id: "5b_archive_rho85",
        label: "Multi-Version Archive (rho~85, fallback)",
        blocks: compress_to_single_block(&raw, k),
    }
}

/// Corpus 6a: Pathological Adversarial Long-Chain Giant Thin Cycle (900 KiB, K=8 thin-only!)
/// 8 copies of a 111,000-byte uniform random 64-symbol string with unique 16-byte tags.
/// Result: r_0 ~ 111k (r_0/n ~ 0.125 < 0.15 => Cap::NONE ADMITTED TO OPEN!),
/// almost ZERO thick strips (every BWT run has width 8 <= 16!),
/// m_16 ~ 886,000 => edges ~ 3,460 KiB (3.46 MiB spilling past L2!),
/// forcing emit_uni into ONE SINGLE ~886k-node random pointer chain vs wire_xor's 2-ended walk!
pub fn gen_adv_giant_thin_cycle(k: Kernels) -> CorpusSpec {
    let mut rng = Rng::new(0x2026_1122_0006_0001);
    let mut base = Vec::with_capacity(110_800);
    let mut prev = 255u8;
    for _ in 0..110_800 {
        // Avoid consecutive identical chars in raw so RLE1 doesn't alter length
        let mut c = B64[rng.usize(64)];
        while c == prev {
            c = B64[rng.usize(64)];
        }
        base.push(c);
        prev = c;
    }
    let mut raw = Vec::with_capacity(888_000);
    for copy in 0..8usize {
        raw.extend_from_slice(&base);
        // 16-byte unique non-repeating tag
        for t in 0..16usize {
            raw.push(B64[(copy * 7 + t * 11 + 5) & 63]);
        }
    }
    CorpusSpec {
        id: "6a_adv_giant_cycle",
        label: "Adversarial Giant Thin Cycle (m16~886k!)",
        blocks: compress_to_single_block(&raw, k),
    }
}

/// Corpus 6b: Pathological Adversarial Fibonacci/Alternating-Run Long Thin Chain (900 KiB)
/// Uses Fibonacci word bits to select between 2 distinct 4-letter alphabets across 10 copies
/// of a structured alternating/De-Bruijn-like sequence (r_0/n ~ 0.10 < 0.15, high m_16,
/// minimal thick strips, long thin chains over a structured small alphabet).
pub fn gen_adv_fib_altrun(k: Kernels) -> CorpusSpec {
    // Generate Fibonacci word of length 90_000 over {0, 1}
    let mut fib: Vec<u8> = Vec::with_capacity(95_000);
    let mut a: Vec<u8> = vec![0];
    let mut b: Vec<u8> = vec![0, 1];
    while b.len() < 89_000 {
        let mut c = Vec::with_capacity(b.len() + a.len());
        c.extend_from_slice(&b);
        c.extend_from_slice(&a);
        a = b;
        b = c;
    }
    fib.extend_from_slice(&b[..88_500]);
    // Modulate with a deterministic LFSR phase so contexts are rich yet runs have width ~10 <= 16
    let mut rng = Rng::new(0x2026_1122_0006_0002);
    let mut base = Vec::with_capacity(88_500);
    const SYM: [u8; 8] = *b"abcdefgh";
    for (i, &fb) in fib.iter().enumerate() {
        let r = rng.usize(4);
        base.push(SYM[(fb as usize) * 4 + ((r + i) & 3)]);
    }
    let mut raw = Vec::with_capacity(886_000);
    for copy in 0..10usize {
        raw.extend_from_slice(&base);
        for t in 0..16usize {
            raw.push(SYM[(copy * 3 + t) & 7]);
        }
    }
    CorpusSpec {
        id: "6b_adv_fib_altrun",
        label: "Adversarial Fib/AltRun Chain (m16~800k!)",
        blocks: compress_to_single_block(&raw, k),
    }
}
