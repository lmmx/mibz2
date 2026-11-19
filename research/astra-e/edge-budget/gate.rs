fn fits(records: &[u16], mark: u32) -> bool {
    let (mut retained, mut strips) = (0u32, 0u32);
    let (mut row, mut i) = (0u32, 0);
    while i < records.len() {
        let byte = records[i] >> 8;
        let mut width = 0u32;
        while i < records.len() && records[i] >> 8 == byte {
            width += u32::from(records[i] & 255);
            i += 1;
        }
        if width <= 16 {
            retained += width;
        } else if (row..row + width).contains(&mark) {
            retained += 1;
            strips += u32::from(mark != row) + u32::from(mark + 1 < row + width);
        } else {
            strips += 1;
        }
        row += width;
    }
    let bits = (32 - (retained - 1).leading_zeros()).max(1);
    let code_bound = 255 + 3 * strips + 256;
    let width = if code_bound <= u32::MAX >> bits { 4 } else { 8 };
    retained * width <= 1 << 20
}
