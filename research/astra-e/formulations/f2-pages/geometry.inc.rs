impl<const WIDTH: u32> PageCandidate<WIDTH> {
    pub fn check_pages(&mut self, caps: &[u16], rows: u32) -> (usize, usize, usize) {
        assert!(self.prepare(caps, rows).is_some());
        let mut x = 0;
        let mut index = 0;
        let mut previous = None;
        let mut boundary_starts = 0;
        for &record in caps {
            let c = record >> 8;
            if previous != Some(c) {
                index += usize::from(x != 0);
                boundary_starts += usize::from(x % WIDTH == 0);
                assert_eq!(self.runs[index].start, x);
                previous = Some(c);
            }
            let end = x + u32::from(record & 255);
            while x < end {
                assert_eq!(self.locate(x), index, "WIDTH={WIDTH} row={x}");
                x += 1;
            }
        }
        assert_eq!(x, rows);
        let bytes = self.pages.packed.capacity() * 8 + self.pages.wide.capacity() * size_of::<WidePage>();
        assert_eq!(size_of::<WidePage>(), 16);
        (index+1, boundary_starts, bytes)
    }
}
