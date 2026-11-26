// Included alongside Section/Part from shift-compact/projection.rs.
#[derive(Clone, Copy, Default, Debug)]
struct Owner {
    v: u32,
    base: u32,
    depth: u8,
    prefix: [u8; 8],
}

#[derive(Default)]
struct Atlas {
    forbid: Section,
    target: Section,
    owners: Vec<Owner>,
    root: (u32, u32),
    hidden_mark: Option<(u32, u32)>,
    intersections: u32,
    workspace_peak: usize,
}

#[derive(Clone, Copy, Default)]
struct AtlasSelect {
    // Input end in F order and the byte. Bzip2 blocks have at most 900,000 rows.
    end_byte: u32,
    delta: u32,
}

#[derive(Clone, Copy)]
struct AtlasPacket {
    base: u32,
    row: u32,
    width: u32,
    depth: u8,
    prefix: [u8; 8],
}

fn atlas_capacity<T>(items: &Vec<T>) -> usize {
    items.capacity() * std::mem::size_of::<T>()
}

impl Atlas {
    fn bytes(&self) -> usize {
        atlas_capacity(&self.forbid.parts)
            + atlas_capacity(&self.target.parts)
            + atlas_capacity(&self.owners)
    }

    fn prepare(caps: &[u16], freq: &[u32; 256], u: (u32, u32), h: usize, primary: u32) -> Self {
        let n: u32 = freq.iter().sum();
        assert!(h <= 8 && n <= 900_000 && u.0 < u.1 && u.1 <= n && primary < n);
        let mut atlas = Self::default();
        if h == 0 {
            atlas.root = u;
            atlas.target.push(u.0, u.1);
            atlas.owners.push(Owner::default());
            return atlas;
        }

        // A stable bucket scatter builds each original maximal L run exactly once.
        // Consecutive capped records of the same byte extend the current strip.
        let mut slot = [0usize; 256];
        let mut last = 256;
        for &rec in caps {
            let c = usize::from(rec >> 8);
            if c != last {
                slot[c] += 1;
                last = c;
            }
        }
        let mut nruns = 0;
        for count in &mut slot {
            let width = *count;
            *count = nruns;
            nruns += width;
        }
        let mut forward = vec![AtlasSelect::default(); nruns];
        let mut total = 0;
        let mut next = freq.map(|w| {
            let x = total;
            total += w;
            x
        });
        last = 256;
        let mut row = 0u32;
        for &rec in caps {
            let c = usize::from(rec >> 8);
            let w = u32::from(rec & 255);
            assert!((1..=127).contains(&w));
            let start = next[c];
            next[c] += w;
            if c == last {
                forward[slot[c] - 1].end_byte += w << 8;
            } else {
                forward[slot[c]] = AtlasSelect {
                    end_byte: (next[c] << 8) | c as u32,
                    delta: row.wrapping_sub(start),
                };
                slot[c] += 1;
                last = c;
            }
            row += w;
        }
        assert_eq!(row, n);

        let mut current = vec![AtlasPacket {
            base: 0,
            row: u.0,
            width: u.1 - u.0,
            depth: 0,
            prefix: [0; 8],
        }];
        let mut next_packets = Vec::new();
        let mut leaves = Vec::new();
        let mut hidden = Vec::new();
        for depth in 0..h {
            next_packets.clear();
            for p in &current {
                let mut x = p.row;
                let mut k = forward.partition_point(|s| s.end_byte >> 8 <= x);
                let limit = x + p.width;
                while x < limit {
                    let strip = forward[k];
                    let stop = limit.min(strip.end_byte >> 8);
                    let mut y = x.wrapping_add(strip.delta);
                    let end = y + stop - x;
                    atlas.intersections += 1;
                    while y < end {
                        let (to, short) = if y < u.0 {
                            (end.min(u.0), false)
                        } else if y < u.1 {
                            (end.min(u.1), true)
                        } else {
                            (end, false)
                        };
                        let w = to - y;
                        let base = p.base + x - p.row;
                        if short {
                            leaves.push(AtlasPacket {
                                base,
                                row: x,
                                width: w,
                                ..*p
                            });
                        } else {
                            let mut prefix = p.prefix;
                            prefix[depth] = strip.end_byte as u8;
                            let steps = depth as u8 + 1;
                            next_packets.push(AtlasPacket {
                                base,
                                row: y,
                                width: w,
                                depth: steps,
                                prefix,
                            });
                            hidden.push((y, to));
                            if y <= primary && primary < to {
                                debug_assert!(atlas.hidden_mark.is_none());
                                atlas.hidden_mark = Some((base + primary - y, u32::from(steps)));
                            }
                        }
                        x += w;
                        y = to;
                    }
                    k += 1;
                }
            }
            std::mem::swap(&mut current, &mut next_packets);
            if current.is_empty() {
                break;
            }
        }
        // All five scratch capacities are nondecreasing in aggregate until this point.
        atlas.workspace_peak = atlas_capacity(&forward)
            + atlas_capacity(&current)
            + atlas_capacity(&next_packets)
            + atlas_capacity(&leaves)
            + atlas_capacity(&hidden);
        drop(forward);
        drop(next_packets);
        if leaves.is_empty() {
            std::mem::swap(&mut leaves, &mut current);
        } else if !current.is_empty() {
            leaves.extend_from_slice(&current);
            atlas.workspace_peak = atlas
                .workspace_peak
                .max(atlas_capacity(&leaves) + atlas_capacity(&current) + atlas_capacity(&hidden));
        }
        drop(current);

        hidden.sort_unstable();
        for (lo, hi) in hidden {
            atlas.forbid.push(lo, hi);
        }
        atlas.root = (atlas.forbid.complement(u.0), atlas.forbid.complement(u.1));
        debug_assert_eq!(atlas.root.1 - atlas.root.0, u.1 - u.0);
        leaves.sort_unstable_by_key(|leaf| leaf.row);
        atlas.owners.reserve_exact(leaves.len());
        for leaf in leaves {
            atlas.owners.push(Owner {
                v: atlas.target.mass,
                base: leaf.base,
                depth: leaf.depth,
                prefix: leaf.prefix,
            });
            atlas.target.push(leaf.row, leaf.row + leaf.width);
        }
        assert_eq!(atlas.target.mass, u.1 - u.0);
        atlas
    }
}

#[cfg(test)]
mod atlas_contract {
    use super::*;

    fn caps(last: &[u8], cap: usize) -> (Vec<u16>, [u32; 256]) {
        let mut records = Vec::new();
        let mut freq = [0u32; 256];
        let mut i = 0;
        while i < last.len() {
            let c = last[i];
            let mut end = i + 1;
            while end < last.len() && last[end] == c && end - i < cap {
                end += 1;
            }
            records.push((u16::from(c) << 8) | (end - i) as u16);
            freq[c as usize] += (end - i) as u32;
            i = end;
        }
        (records, freq)
    }

    fn check(last: &[u8], a: usize, b: usize, h: usize, cap: usize, every_mark: bool) -> usize {
        let n = last.len();
        let mut psi: Vec<_> = (0..n).collect();
        psi.sort_by_key(|&i| last[i]);
        let (record, freq) = caps(last, cap);
        let mut hidden = vec![None; n];
        let mut source_at = vec![None; n];
        for s in a..b {
            let mut x = s;
            let mut prefix = Vec::new();
            while prefix.len() < h && !(a..b).contains(&psi[x]) {
                x = psi[x];
                prefix.push(last[x]);
                assert!(hidden[x]
                    .replace(((s - a) as u32, prefix.len() as u32))
                    .is_none());
            }
            assert!(source_at[x].replace((s, prefix)).is_none());
        }
        // Split row-level packets by actual consecutive translation and hit decisions.
        let mut packets = vec![(a..b).collect::<Vec<_>>()];
        let mut visits = 0;
        let mut finished = 0;
        for _ in 0..h {
            let mut next = Vec::new();
            for packet in packets {
                let mut previous = None;
                for x in packet {
                    let y = psi[x];
                    let short = (a..b).contains(&y);
                    let descriptor = (last[y], y as i32 - x as i32);
                    let key = (descriptor, short);
                    if previous.map(|(d, _)| d) != Some(descriptor) {
                        visits += 1;
                    }
                    if previous != Some(key) {
                        if short {
                            finished += 1;
                        } else {
                            next.push(Vec::new());
                        }
                        previous = Some(key);
                    }
                    if !short {
                        next.last_mut().unwrap().push(y);
                    }
                }
            }
            packets = next;
        }
        let bands = finished + packets.len();
        let marks = if every_mark {
            (0..n).collect::<Vec<_>>()
        } else {
            vec![0, n / 2, n - 1]
        };
        for &mark in &marks {
            let result = Atlas::prepare(&record, &freq, (a as u32, b as u32), h, mark as u32);
            assert_eq!(result.hidden_mark, hidden[mark]);
            assert_eq!(result.intersections, visits);
            assert_eq!(result.owners.len(), bands);
            assert_eq!(result.target.mass, (b - a) as u32);
            if h == 0 {
                assert_eq!(result.workspace_peak, 0);
                assert_eq!(result.root, (a as u32, b as u32));
            }
            let mut ordinal = 0;
            for x in 0..n {
                assert_eq!(result.forbid.rank(x as u32).is_some(), hidden[x].is_some());
                if let Some((source, word)) = &source_at[x] {
                    assert_eq!(result.target.rank(x as u32), Some(ordinal));
                    assert_eq!(result.target.row(ordinal), x as u32);
                    let k = result.owners.partition_point(|o| o.v <= ordinal) - 1;
                    let o = result.owners[k];
                    assert_eq!(o.base + ordinal - o.v, (*source - a) as u32);
                    assert_eq!(&o.prefix[..usize::from(o.depth)], word);
                    ordinal += 1;
                } else {
                    assert_eq!(result.target.rank(x as u32), None);
                }
            }
            assert_eq!(
                result.root,
                (
                    result.forbid.complement(a as u32),
                    result.forbid.complement(b as u32)
                )
            );
            assert_eq!(result.root.1 - result.root.0, (b - a) as u32);
            assert!(result.bytes() >= result.owners.len() * std::mem::size_of::<Owner>());
        }
        marks.len()
    }

    #[test]
    fn exhaustive_stable_columns() {
        let mut marks = 0;
        let mut cases = 0;
        for n in 1..=7 {
            for bits in 0..1usize << n {
                let last: Vec<_> = (0..n).map(|i| ((bits >> i) & 1) as u8).collect();
                for a in 0..n {
                    for b in a + 1..=n {
                        for h in [0, 1, 3, 8] {
                            marks += check(&last, a, b, h, 2, true);
                            cases += 1;
                        }
                    }
                }
            }
        }
        println!("{cases} column/interval/depth combinations; {marks} verified marked atlases");
    }

    #[test]
    fn cap_boundaries_and_short_returns() {
        let last = [vec![34; 257], vec![0; 130], vec![35; 259], vec![34; 127]].concat();
        let n = last.len();
        for cap in [1, 8, 126, 127] {
            for (a, b) in [(0, n), (1, 2), (120, 260), (260, 510)] {
                for h in [0, 1, 8] {
                    check(&last, a, b, h, cap, false);
                }
            }
        }
    }
}
