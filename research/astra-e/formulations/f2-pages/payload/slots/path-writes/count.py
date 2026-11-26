from pathlib import Path
import subprocess,sys
here=Path(__file__).resolve().parent
out=Path(sys.argv[1]).resolve()
subprocess.run([sys.executable,str(here/'harness.py'),str(out)],check=True)
candidate=(out/'slots.rs').read_text()
candidate=candidate.replace('    fallback: Segmented,','    path_account: [u64;3],\n    fallback: Segmented,')
anchor='                let byte=$byte;'
assert candidate.count(anchor)==1
candidate=candidate.replace(anchor,anchor+'''
                let open=if PATH==3 {cursor[lane]!=limit[lane]} else {spare};
                self.path_account[usize::from(open)] += 1;
                let growing=if PATH==3 {cursor[lane]==limit[lane]} else {paths[lane].len()==paths[lane].capacity()};
                self.path_account[2] += u64::from(growing);''')
candidate+='\nimpl<const A:u8,const B:u8,const C:u8> Dispatch<A,B,C>{pub fn account(&mut self)->[u64;3]{std::mem::take(&mut self.path_account)}}'
(out/'slots.rs').write_text(candidate)
focus=(out/'focus.rs').read_text().replace('            let cap=old.capacities();','''            for (name,acc) in [("frame",p32.account()),("slack",p40.account()),("cursor",p44.account())] {
                println!("{index},{name},{},{},{}",acc[0],acc[1],acc[2]);
            }
            let cap=old.capacities();''')
focus='\n'.join(l for l in focus.splitlines() if 'if full {println!("block,' not in l and 'println!("{index},{n},' not in l)
(out/'focus.rs').write_text(focus)
