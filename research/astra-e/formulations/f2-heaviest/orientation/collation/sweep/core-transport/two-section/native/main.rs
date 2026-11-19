fn name(arm:i32)->String {
    match arm {
        -2=>"compact_quote".into(),-1=>"compact_frequent".into(),8=>"original_h8".into(),24=>"physical_h8".into(),
        32..64=>format!("lift_core_h{}",arm-32),64.. =>format!("lift_stock_h{}",arm-64),_=>unreachable!(),
    }
}
fn small(){
    let mut cases=0;let mut disjoint=0;let mut fallback=0;let mut sub=Subject::default();
    for (alphabet,max_n) in [(2,7),(3,5)] {
        for n in 1..=max_n {for seed in 0..u32::pow(alphabet,n) {
            let mut z=seed;let caps=(0..n).map(|_|{let c=z%alphabet;z/=alphabet;((c<<8)|1) as u16}).collect();
            let mut x=Input::new(0,0,caps);let dense=Truth::new(&x);
            for lo in 0..n {for hi in lo+1..=n {for h in [0,1,4,8] {for p in 0..n {
                x.p=p;
                for force in [false,true] {
                    let r=sub.run(&x,(lo,hi),h+if force{32}else{64},true).unwrap();
                    dense.check_lift((lo,hi),h as usize,p,r.obj.as_ref().unwrap(),r.mark);
                    assert_eq!(sub.out,dense.output(p));
                    cases+=1;disjoint+=u32::from(r.mark.is_none());fallback+=u32::from(!r.second);
                }
            }}}}
        }}
    }
    println!("exact_marked_cases={cases} disjoint_cycles={disjoint} including_first_level_fallback={fallback}");
}
fn blocks(mode:&str,path:&str,reps:u32){
    let mut reader=BufReader::new(std::fs::File::open(path).unwrap());
    let arms=[-1,8,24,32,36,40,68,72];let mut blocks=0;
    if mode=="price"{println!("stream,rep,arm,setup_mark_ns,discover_ns,merge_ns,publish_ns,first_ns,second_index_shift_ns,second_discover_ns,second_merge_ns,second_emit_ns,total_ns,reserved_bytes");}
    else{println!("stream,arm,lo,hi,live_states,lf_pieces,forward_intersections,origin_bands,deleted_rows,prefix_bins,aliases,prefix_runs,core_runs,core_types,words,bytes_by_id,second_lookups,second_nodes,second_words,second_dict_bytes,second_selectors,second_merge_pieces,second_links,mark,skip,second,reserved_bytes");}
    while let Some(x)=Input::read(&mut reader) {
        let dense=Truth::new(&x);let expected=dense.output(x.p);
        let mut control=Subject::default();
        let q=control.run(&x,x.quote(),-2,true).unwrap();assert_eq!(control.out,expected);
        let u=highest(q.obj.as_ref().unwrap(),x.quote().0);
        let mut subjects:Vec<_>=(0..arms.len()).map(|_|Subject::default()).collect();
        for (&arm,sub) in arms.iter().zip(&mut subjects) {
            let r=sub.run(&x,u,arm,arm>=32).unwrap();
            assert_eq!(sub.out,expected);
            if arm>=32 {
                let h=if arm>=64{arm-64}else{arm-32};
                dense.check_lift(u,h as usize,x.p,r.obj.as_ref().unwrap(),r.mark);
            }
            if mode!="price"{
                let (mark,skip)=r.mark.unwrap_or((u32::MAX,0));
                let s=if arm>=32{r.metrics}else{[0;8]};
                let (aliases,cores)=if arm>=32{(sub.lifted.alias_count,sub.lifted.core_type_count)}else{(0,0)};
                let (k,dm)=r.obj.as_ref().map_or((0,0),|obj|(obj.0.len(),obj.0.iter().map(|w|w.1.len()).sum::<usize>()));
                let t=if arm<0{sub.named.information()}else if arm==8{sub.original.information()}
                      else if arm==24{sub.physical.information()}else{sub.lifted.information()};
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,name(arm),u.0,u.1,
                        s[0],s[1],s[2],s[3],s[4],s[5],aliases,s[6],s[7],cores,k,dm,t[0],t[1],t[2],t[3],t[4],t[5],t[6],mark,skip,r.second,r.cap);
            }
        }
        if mode=="price" {
            for rep in 0..reps {for pos in [0,1,7,2,6,3,5,4] {
                let j=(rep as usize+pos)%arms.len();
                let r=subjects[j].run(&x,u,arms[j],false).unwrap();
                assert!(r.second);assert_eq!(subjects[j].out,expected);
                println!("{},{},{},{},{},{},{},{},{},{},{},{},{},{}",x.index,rep,name(arms[j]),r.phases[0],r.phases[1],r.phases[2],
                         r.publish,r.first,r.inner[0],r.inner[1],r.inner[2],r.inner[3],r.total,r.cap);
            }}
        }
        blocks+=1;
    }
    eprintln!("verified {blocks} input blocks with capped h0,4,8 and stock h4,8");
}
// RSS
fn main(){
    let args=std::env::args().collect::<Vec<_>>();
    if args.len()==1{small();}
    else if args[1]=="rss"{rss(&args[2],args[3].parse().unwrap(),&args[4],&args[5]);}
    else{blocks(&args[1],&args[2],args.get(3).map_or(32,|s|s.parse().unwrap()));}
}
