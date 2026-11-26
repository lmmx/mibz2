#[repr(C, align(8))]
#[derive(Clone,Copy,Default)]
struct Run {
    position_byte: u32,
    delta: u32,
}
impl Run {
    #[inline]
    fn new(pos:u32, byte:u8, delta:u32)->Self {
        Self {position_byte:(pos<<8) | u32::from(byte),delta}
    }
    #[inline]
    fn position(self)->u32 { self.position_byte >> 8 }
    #[inline]
    fn byte(self)->u8 { self.position_byte as u8 }
    #[inline]
    fn map(self, row:u32)->u32 { row.wrapping_add(self.delta) }
}
