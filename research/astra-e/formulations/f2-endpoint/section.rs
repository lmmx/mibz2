// FM backward extension. The pattern is short; each extra byte costs a scan of the input.
fn cylinder(caps: &[u16], pattern: &[u8]) -> Option<(u32, u32)> {
    let mut counts = [0u32; 256];
    for &rec in caps {
        counts[(rec >> 8) as usize] += u32::from(rec & 255);
    }
    let last = *pattern.last()? as usize;
    let base = starts(counts);
    let (mut lo, mut hi) = (base[last], base[last] + counts[last]);
    for &byte in pattern[..pattern.len() - 1].iter().rev() {
        if lo == hi {
            return None;
        }
        let (mut pos, mut before, mut within) = (0, 0, 0);
        for &rec in caps {
            let end = pos + u32::from(rec & 255);
            if (rec >> 8) as u8 == byte {
                before += lo.min(end).saturating_sub(pos);
                within += hi.min(end).saturating_sub(pos.max(lo));
            }
            pos = end;
            if pos >= hi {
                break;
            }
        }
        lo = base[byte as usize] + before;
        hi = lo + within;
    }
    (lo < hi).then_some((lo, hi))
}

// S = I(pattern) union {primary}. The extra singleton splits one return and
// makes the same construction cover a marked cycle containing no occurrence.
fn each_section_run(
    records: &[u16],
    p: u32,
    section: (u32, u32),
    mut visit: impl FnMut(u8, u32, u32, bool),
) {
    let mut cuts = [p, p + 1, section.0, section.1];
    cuts.sort_unstable();
    let mut next_cut = 0;
    let (mut pos, mut i) = (0, 0);
    while i < records.len() {
        let b = (records[i] >> 8) as u8;
        let mut end = pos;
        while i < records.len() && (records[i] >> 8) as u8 == b {
            end += u32::from(records[i] & 255);
            i += 1;
        }
        while pos < end {
            while next_cut < cuts.len() && cuts[next_cut] <= pos {
                next_cut += 1;
            }
            let stop = cuts.get(next_cut).copied().unwrap_or(end).min(end);
            visit(
                b,
                pos,
                stop - pos,
                pos == p || (section.0..section.1).contains(&pos),
            );
            pos = stop;
        }
    }
}
