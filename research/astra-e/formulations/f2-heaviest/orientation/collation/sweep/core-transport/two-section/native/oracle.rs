impl Truth {
    // Verify the stable token matching on V followed by U. Equal byte payloads
    // can belong to different contiguous source buckets, including empty prefixes.
    fn check_lift(&self, u: (u32, u32), h: usize, p: u32, obj: &Obj, mark: Option<(u32, u32)>) {
        let n = self.l.len();
        let section = u.0 as usize..u.1 as usize;
        let m = section.len();
        assert!(m > 0 && h <= 8 && section.end <= n && (p as usize) < n);

        let mut physical_ordinal = vec![u32::MAX; n];
        let mut cuts = Vec::with_capacity(m);
        for source in section.clone() {
            let mut row = source;
            let mut depth = 0;
            while depth < h {
                let next = self.psi[row];
                if section.contains(&next) {
                    break;
                }
                row = next;
                depth += 1;
            }
            assert_eq!(physical_ordinal[row], u32::MAX, "overlapping source cuts");
            physical_ordinal[row] = 0;
            cuts.push((row, depth));
        }
        let mut physical = Vec::with_capacity(m);
        for (row, rank) in physical_ordinal.iter_mut().enumerate() {
            if *rank != u32::MAX {
                *rank = physical.len() as u32;
                physical.push(row);
            }
        }
        assert_eq!(physical.len(), m);

        let size = 2 * m;
        let mut label_at_base = vec![u32::MAX; size];
        let mut next_source = Vec::with_capacity(obj.0.len());
        for (id, (base, _)) in obj.0.iter().enumerate() {
            assert!((*base as usize) < size);
            assert_eq!(
                label_at_base[*base as usize],
                u32::MAX,
                "duplicate token base"
            );
            label_at_base[*base as usize] = id as u32;
            next_source.push(*base);
        }
        let mut used = vec![false; size];
        let mut dest = 0;
        let mut prev = None;
        for &(base, width) in &obj.1 {
            assert!(width > 0 && (base as usize) < size && width as usize <= size - dest);
            assert_ne!(prev, Some(base), "nonmaximal run in lifted column");
            prev = Some(base);
            let id = label_at_base[base as usize];
            assert_ne!(id, u32::MAX, "missing lifted token base");
            let payload = &obj.0[id as usize].1;
            for _ in 0..width {
                let source = next_source[id as usize] as usize;
                assert!(
                    source < size && !used[source],
                    "overlapping lifted source buckets"
                );
                used[source] = true;
                let truth_dest = if source < m {
                    // V -> U must take the complete nonempty core, even if the
                    // starting physical rank also represents a U phase state.
                    assert!(!payload.is_empty(), "empty V-to-U core");
                    let mut row = physical[source];
                    for (i, &byte) in payload.iter().enumerate() {
                        row = self.psi[row];
                        assert_eq!(self.l[row], byte, "wrong core byte");
                        assert_eq!(
                            section.contains(&row),
                            i + 1 == payload.len(),
                            "core must stop at first positive U hit"
                        );
                    }
                    m + row - section.start
                } else {
                    // U -> V uses exactly its confirmed, short-safe prefix.
                    let i = source - m;
                    let (cut, depth) = cuts[i];
                    assert_eq!(payload.len(), depth, "wrong lifted prefix length");
                    let mut row = section.start + i;
                    for &byte in payload {
                        row = self.psi[row];
                        assert_eq!(self.l[row], byte, "wrong prefix byte");
                    }
                    assert_eq!(row, cut);
                    physical_ordinal[cut] as usize
                };
                assert_eq!(truth_dest, dest, "wrong lifted partner for source {source}");
                dest += 1;
                next_source[id as usize] += 1;
            }
        }
        assert_eq!(dest, size);
        assert!(used.into_iter().all(|seen| seen));
        assert!(obj
            .0
            .iter()
            .zip(&next_source)
            .all(|((base, _), end)| base < end));

        let mut row = p as usize;
        let mut skip = 0;
        let expected = loop {
            if physical_ordinal[row] != u32::MAX {
                break Some((physical_ordinal[row], skip));
            }
            if section.contains(&row) {
                break Some(((m + row - section.start) as u32, skip));
            }
            row = self.lf[row];
            skip += 1;
            if row == p as usize {
                break None;
            }
        };
        assert_eq!(mark, expected, "wrong lifted mark phase or byte offset");
    }
}

#[cfg(test)]
mod lift_oracle_regressions {
    use super::{Obj, Truth};

    #[test]
    fn marks_on_both_sides_of_the_cut() {
        let truth = Truth {
            l: vec![3, 0, 1, 2],
            psi: vec![1, 2, 3, 0],
            lf: vec![3, 0, 1, 2],
        };
        let column = (vec![(0, vec![2, 3]), (1, vec![0, 1])], vec![(1, 1), (0, 1)]);
        for (p, mark) in [(0, (1, 0)), (1, (1, 1)), (2, (0, 0)), (3, (0, 1))] {
            truth.check_lift((0, 1), 2, p, &column, Some(mark));
        }
    }

    fn two_identity_cycles() -> (Truth, Obj) {
        let truth = Truth {
            l: vec![34, 34],
            psi: vec![0, 1],
            lf: vec![0, 1],
        };
        let column = (
            vec![(0, vec![34]), (2, vec![]), (3, vec![])],
            vec![(2, 1), (3, 1), (0, 2)],
        );
        (truth, column)
    }

    #[test]
    fn coincident_marks_select_positive_core_and_empty_prefix_aliases_are_legal() {
        let (truth, column) = two_identity_cycles();
        truth.check_lift((0, 2), 8, 1, &column, Some((1, 0)));
    }

    #[test]
    #[should_panic(expected = "wrong lifted partner")]
    fn exchanging_equal_empty_prefix_partners_is_detected() {
        let (truth, mut column) = two_identity_cycles();
        column.1.swap(0, 1);
        truth.check_lift((0, 2), 8, 1, &column, Some((1, 0)));
    }
}
