#[cfg(test)]
mod stream_checks {
    use super::*;
    #[test]
    fn separated_runs_and_trailing_pages() {
        let mut c=Candidate::default();
        for n in [63usize,64,65,351,899999] {
            for widths in [[1usize,64,65,25000], [4096,8192,33,17], [20,1,1,1],[1,1,1,1],[1500000,1,1,1]] {
                let (mut left,mut i)=(n,0);
                let (mut caps,mut counts)=(Vec::new(),[0u32;256]);
                while left>0 {
                    let byte=[QUOTE,5,0,255][i%4];
                    let len=widths[i%4].min(left);
                    counts[byte as usize]+=len as u32;
                    left-=len;
                    i+=1;
                    let mut part=len;
                    while part>0 {
                        let k=part.min(127);
                        caps.push((u16::from(byte)<<8)|k as u16);
                        part-=k;
                    }
                }
                let reference=c.prepare(&caps,n as u32);
                let expected_bases=c.page_first.clone();
                let expected_masks=c.starts.clone();
                assert_eq!(reference,c.prepare_counted(&caps,n as u32,&counts));
                assert_eq!(c.starts,expected_masks);
                assert_eq!(c.page_first,expected_bases);
            }
        }
    }
}
