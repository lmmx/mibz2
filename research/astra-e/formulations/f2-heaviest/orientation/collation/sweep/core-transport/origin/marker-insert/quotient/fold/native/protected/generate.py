"""Freeze the one additional restriction and add an out-of-clock canonical second object."""
from hashlib import sha256
from pathlib import Path
import subprocess,sys

HERE=Path(__file__).resolve().parent
PARENT=HERE.parent

def subst(s,old,new):
    assert s.count(old)==1,old
    return s.replace(old,new)

def generate(out):
    subprocess.run([sys.executable,'-B',PARENT/'generate.py',out],check=True)
    p=out/'control.rs';s=p.read_text()
    assert sha256(s.encode()).hexdigest()==(HERE/'control.sha').read_text().strip()
    s=subst(s,'        self.choose();','''        let anchor = *self.order.iter().max_by_key(|&&k|
            (self.classes[k].mass,u32::MAX-self.classes[k].base)
        ).unwrap();
        for (i,c) in self.classes.iter_mut().enumerate(){
            if i==anchor || c.succ==anchor as u32 {c.succ=NONE;}
        }
        self.choose();''')
    p.write_text(s+'''
impl<const M:u8,const S:u8> Dispatch<M,S>{
    pub fn snapshot_second(&self,mark:Option<(u32,u32)>,ran:bool)->Option<(crate::Obj,(u32,u32))>{
        if !ran{return None;}
        Some(self.second.canonical(mark.unwrap(),&self.words))
    }
}
''')
    p=out/'second.rs'; p.write_text(p.read_text()+(HERE/'second-object.rs').read_text())
    p=out/'main.rs';s=p.read_text()
    s=subst(s,'    let mut selected=vec![false;words.len()];','''    let anchor=(0..words.len()).max_by_key(|&k|(seen[k],usize::MAX-k)).unwrap();
    let mut selected=vec![false;words.len()];''')
    s=subst(s,'let eligible=y!=k && rows.iter()', 'let eligible=y!=k && y!=anchor && k!=anchor && rows.iter()')
    s=subst(s,'    let mut w=Engine::default();let mut cnt=[0;5];','    let mut w=Engine::default();let mut named=Engine::default();let mut cnt=[0;5];')
    s=subst(s,'let r=w.run(&x,3,true);dense(&truth,&x,&r,true);','''let r=w.run(&x,3,true);dense(&truth,&x,&r,true);
                let a=named.run(&x,0,false);
                assert_eq!(named.fast.snapshot_second(a.mark,a.ran),w.fast.snapshot_second(r.mark,r.ran));''')
    s=subst(s,'let truth=Truth::new(&x);let expected=truth.output(x.p);\n        for (a,w)',
            '''let truth=Truth::new(&x);let expected=truth.output(x.p);let mut second=None;
        for (a,w)''')
    s=subst(s,'if a==3 && mode!="admissions"{dense(&truth,&x,&r,false);}', '''if a==3 && mode!="admissions"{dense(&truth,&x,&r,false);}
            if mode!="selections" && (a==0 || a==3){
                let object=w.fast.snapshot_second(r.mark,r.ran);
                if a==0{second=Some(object);}else{assert_eq!(second.as_ref().unwrap(),&object);}
            }''')
    s=subst(s,'fn cases(mode:',(HERE/'states.rs').read_text()+'\nfn cases(mode:')
    s=subst(s,'let a=named.run(&x,0,false);','''let a=named.run(&x,0,false);
                let doomed=r.selection.iter().filter(|r|r.3).map(|r|r.0).collect();
                let old=state_census(&truth.object(x.quote(),true),&doomed);
                let new=state_census(r.object.as_ref().unwrap(),&Default::default());
                assert_eq!(new[0],old[0]-old[1]);
                if a.ran{assert_eq!(a.info[0],old[0] as usize);}
                if r.ran{assert_eq!(r.info[0],new[0] as usize);}''')
    s=subst(s,'if mode=="selections"{println!','''if mode=="states"{println!("stream,old_states,fully_deleted_states,covered_deleted_rows,partially_deleted_state_rows,covered_rows,new_states,stock_new,all_eligible_upper");}
    else if mode=="selections"{println!''')
    s=subst(s,'if mode=="selections" && a!=3','if (mode=="selections" || mode=="states") && a!=3')
    s=subst(s,'if mode!="selections" && (a==0 || a==3)','if mode!="selections" && mode!="states" && (a==0 || a==3)')
    s=subst(s,'            if mode=="selections"{\n','''            if mode=="states"{
                let doomed=r.selection.iter().filter(|r|r.3).map(|r|r.0).collect();
                let original=truth.object(x.quote(),true);
                let old=state_census(&original,&doomed);
                let any=r.selection.iter().filter(|r|r.2!=u32::MAX).map(|r|r.0).collect();
                let upper=state_census(&original,&any);
                let new=state_census(r.object.as_ref().unwrap(),&Default::default());
                assert!(old[1]<=upper[1]);
                assert_eq!(old[0]-old[1],new[0]);
                if r.ran{assert_eq!(new[0] as usize,r.info[0]);}
                println!("{},{},{},{},{},{},{},{},{}",x.index,old[0],old[1],old[2],old[3],old[4],new[0],r.info[0],upper[1]);
            }else if mode=="selections"{
''')
    s=subst(s,'"geometry"|"admissions"|"selections"', '"geometry"|"admissions"|"selections"|"states"')
    p.write_text(s)

if __name__=='__main__':generate(Path(sys.argv[1]))
