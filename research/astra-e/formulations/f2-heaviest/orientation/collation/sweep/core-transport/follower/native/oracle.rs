impl Truth {
    // Derive the moved section and check the physical first return without using
    // the candidate's atlas, core labels, owner bands, or token permutation.
    fn check_physical(&self, u: (u32, u32), h: usize, p: u32, obj: &Obj, mark: Option<(u32, u32)>) {
        let n = self.l.len();
        let section = u.0 as usize..u.1 as usize;
        let m = section.len();
        assert!(m > 0 && h <= 8 && section.end <= n && (p as usize) < n);

        let mut ordinal = vec![u32::MAX; n];
        for source in section.clone() {
            let mut x = source;
            for _ in 0..h {
                let y = self.psi[x];
                if section.contains(&y) {
                    break;
                }
                x = y;
            }
            assert_eq!(ordinal[x], u32::MAX, "overlapping short-safe cuts");
            ordinal[x] = 0;
        }
        let mut physical = Vec::with_capacity(m);
        for (row, rank) in ordinal.iter_mut().enumerate() {
            if *rank != u32::MAX {
                *rank = physical.len() as u32;
                physical.push(row);
            }
        }
        assert_eq!(physical.len(), m);

        // Distinct contiguous source buckets may have the same physical payload.
        let mut word_by_base = vec![u32::MAX; m];
        let mut cursor = Vec::with_capacity(obj.0.len());
        for (id, (base, payload)) in obj.0.iter().enumerate() {
            assert!((*base as usize) < m && !payload.is_empty());
            assert_eq!(
                word_by_base[*base as usize],
                u32::MAX,
                "duplicate token base"
            );
            word_by_base[*base as usize] = id as u32;
            cursor.push(*base);
        }

        let mut dest = 0usize;
        let mut used_source = vec![false; m];
        let mut previous_base = None;
        for &(base, width) in &obj.1 {
            assert!(width > 0 && (base as usize) < m && width as usize <= m - dest);
            assert_ne!(previous_base, Some(base), "adjacent identical input runs");
            previous_base = Some(base);
            let id = word_by_base[base as usize];
            assert_ne!(id, u32::MAX, "undefined token base");
            let payload = &obj.0[id as usize].1;
            for _ in 0..width {
                let source = cursor[id as usize] as usize;
                assert!(
                    source < m && !used_source[source],
                    "overlapping source buckets"
                );
                used_source[source] = true;
                let mut row = physical[source];
                for (offset, &c) in payload.iter().enumerate() {
                    row = self.psi[row];
                    assert_eq!(self.l[row], c, "wrong byte from physical source {source}");
                    assert_eq!(
                        ordinal[row] != u32::MAX,
                        offset + 1 == payload.len(),
                        "physical word must stop on its first positive return"
                    );
                }
                assert_eq!(
                    row, physical[dest],
                    "wrong partner for physical source {source}"
                );
                dest += 1;
                cursor[id as usize] += 1;
            }
        }
        assert_eq!(dest, m);
        assert!(used_source.into_iter().all(|used| used));
        assert!(obj.0.iter().zip(&cursor).all(|((base, _), end)| base < end));

        let mut row = p as usize;
        let mut distance = 0;
        let expected = loop {
            if ordinal[row] != u32::MAX {
                break Some((ordinal[row], distance));
            }
            row = self.lf[row];
            distance += 1;
            if row == p as usize {
                break None;
            }
        };
        assert_eq!(
            mark, expected,
            "nearest inclusive physical section and byte offset"
        );
    }
}

#[cfg(test)]
mod physical_object_regressions {
    use super::{Obj, Truth};

    fn equal_cycles() -> (Truth, Obj) {
        let truth = Truth {
            l: vec![34, 34],
            psi: vec![0, 1],
            lf: vec![0, 1],
        };
        let separate = (vec![(0, vec![34]), (1, vec![34])], vec![(0, 1), (1, 1)]);
        (truth, separate)
    }

    #[test]
    fn equal_spelling_aliases_validate_by_source_bucket() {
        let (truth, obj) = equal_cycles();
        truth.check_physical((0, 2), 8, 1, &obj, Some((1, 0)));
    }

    #[test]
    #[should_panic(expected = "wrong partner")]
    fn swapped_equal_spelling_cycles_are_detected() {
        let (truth, mut obj) = equal_cycles();
        obj.1.swap(0, 1);
        truth.check_physical((0, 2), 8, 1, &obj, Some((1, 0)));
    }

    #[test]
    fn marked_prefix_and_cut_have_different_offsets() {
        let truth = Truth {
            l: vec![3, 0, 1, 2],
            psi: vec![1, 2, 3, 0],
            lf: vec![3, 0, 1, 2],
        };
        // U = {0} and V = {2}. Row 1 is three bytes after the previous cut.
        let obj = (vec![(0, vec![2, 3, 0, 1])], vec![(0, 1)]);
        truth.check_physical((0, 1), 2, 1, &obj, Some((0, 3)));
        truth.check_physical((0, 1), 2, 2, &obj, Some((0, 0)));
    }
}
