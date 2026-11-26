use super::{Node,Piece};

// An old-column cursor supplies all positions. A second cursor visits the same
// old runs to discover the next insertion. Both move strictly forward.
pub(super) struct Virtual<'a> {
    pieces:&'a [Piece], nodes:&'a [Node], flags:&'a [u32], z:u32,
    cut:u32, at:u32, row:usize, used:u32,
    scan:usize, before_z:u32, next_event:Option<(u32,u32,u32)>,
}
impl<'a> Virtual<'a> {
    pub(super) fn new(pieces:&'a [Piece],nodes:&'a [Node],flags:&'a [u32],z:u32,cut:u32)->Self {
        let mut it=Self{pieces,nodes,flags,z,cut,at:0,row:0,used:0,scan:0,before_z:0,next_event:None};
        it.advance_event();
        it
    }
    fn advance_event(&mut self) {
        self.next_event=None;
        while let Some(p)=self.pieces.get(self.scan) {
            self.scan+=1;
            let w=self.nodes[p.id as usize].word;
            if w==self.z {self.before_z+=p.width;}
            else if self.flags[w as usize]!=0 {
                self.next_event=Some((self.cut+self.before_z,w,p.width));
                break;
            }
        }
    }
}
impl Iterator for Virtual<'_> {
    type Item=(u32,u32);
    fn next(&mut self)->Option<Self::Item> {
        let stop=if let Some((x,w,n))=self.next_event {
            if x==self.at {
                self.advance_event();
                return Some((w,n));
            }
            debug_assert!(x>self.at);
            x
        } else {u32::MAX};
        let p=self.pieces.get(self.row)?;
        let w=self.nodes[p.id as usize].word;
        let count=(p.width-self.used).min(stop-self.at);
        self.used+=count;
        if self.used==p.width {self.row+=1;self.used=0;}
        self.at+=count;
        Some((if self.flags[w as usize]!=0{self.z}else{w},count))
    }
}
