// Retain all thin rows, the primary, and every sorted window boundary.
// Classify thick fragments by their original maximal run.
fn each_run(
    records: &[u16],
    p: u32,
    limit: u32,
    seal: &[u32],
    mut visit: impl FnMut(u8, u32, u32, bool),
) {
    let (mut i, mut pos, mut cut) = (0, 0u32, 0);
    while i < records.len() {
        let c = (records[i] >> 8) as u8;
        let mut end = pos;
        while i < records.len() && (records[i] >> 8) as u8 == c {
            end += u32::from(records[i] & 255);
            i += 1;
        }
        if end - pos <= limit {
            visit(c, pos, end - pos, true);
        } else {
            let mut x = pos;
            while x < end {
                while cut < seal.len() && seal[cut] < x {
                    cut += 1;
                }
                let mut stop = seal.get(cut).copied().unwrap_or(end).min(end);
                if p >= x {
                    stop = stop.min(p);
                }
                if stop > x {
                    visit(c, x, stop - x, false);
                }
                if stop == end {
                    break;
                }
                visit(c, stop, 1, true);
                x = stop + 1;
            }
        }
        pos = end;
    }
}
