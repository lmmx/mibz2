// The retained rank and three padded ranks always fit u32 on a bzip2 block.
#[inline]
fn append_rows<const MODE: u8>(queue: &mut Vec<u32>, row: u32, width: u32) {
    if MODE != 6 && width == 1 {
        queue.push(row);
        return;
    }
    if MODE == 5 {
        queue.extend(row..row + width);
        return;
    }

    let width = width as usize;
    let rounded = width.next_multiple_of(4);
    queue.reserve(rounded);
    let start = queue.len();
    // SAFETY: capacity covers the original length plus rounded slots.
    // Every published slot is initialized; up to three padded entries lie in
    // spare capacity. No references into the Vec survive the reserve or fill.
    unsafe {
        let dest = queue.as_mut_ptr().add(start);
        for offset in (0..rounded).step_by(4) {
            let first = row + offset as u32;
            dest.add(offset).cast::<[u32; 4]>().write([first, first + 1, first + 2, first + 3]);
        }
        queue.set_len(start + width);
    }
}
