// Run with Node: node schedule-check.js
console.log((function(){
// Abstract differential check of one full-loop offset.
// Semantic tokens replace bytes/rows; this is not a Rust or pointer-provenance test.
function check(L, mask, available, off) {
  const marked = k => ((mask >> BigInt(k)) & 1n) !== 0n;
  function run(split) {
    const lanes=Array.from({length:L},(_,k)=>({seg:k,pos:0,base:100+k*20,p:100+k*20+off,done:false}));
    const writes=[], finishes=[], takes=[];
    let used=0, stop=-1;
    const step=k=>{lanes[k].pos++; writes.push([lanes[k].seg,"ordinary"]);};
    const hand=k=>{
      const l=lanes[k]; l.p=l.base+off; finishes.push([l.seg,"endpoint-"+l.seg]); l.done=true;
      if(used===available){stop=k;return false;}
      const seg=L+used++; takes.push([k,seg]); writes.push([seg,"first"]);
      lanes[k]={seg,pos:1,base:1-(off+1),p:1,done:false};return true;
    };
    if(split) {
      for(let k=0;k<L;k++)if(!marked(k))step(k);
      for(let k=0;k<L;k++)if(marked(k)&&!hand(k))break;
    } else {
      for(let k=0;k<L;k++){if(marked(k)){if(!hand(k))break;}else step(k);}
    }
    for(let j=0;j<L;j++) {
      if(j===stop)continue;
      const progressed=stop<0 || (split ? !(marked(j)&&j>stop) : j<stop);
      lanes[j].p=lanes[j].base+off+Number(progressed);
    }
    // On exhaustion the old tail must do the ordinary steps already performed
    // by the split loop. These commute with all remaining segment-local work.
    if(stop>=0&&!split)for(let j=stop+1;j<L;j++)if(!marked(j)){step(j);lanes[j].p++;}
    writes.sort((a,b)=>a[0]-b[0]||a[1].localeCompare(b[1]));
    return {lanes,writes,finishes,takes,used,stop};
  }
  const a=run(false),b=run(true);
  if(JSON.stringify(a)!==JSON.stringify(b))throw Error(JSON.stringify({L,mask:String(mask),available,off,a,b}));
}
let cases=0;
for(let L=1;L<=10;L++)for(let m=0;m<2**L;m++){
 let count=0;for(let k=0;k<L;k++)count+=(m>>k)&1;
 for(let q=0;q<=count+1;q++)for(const off of [0,1,7]){check(L,BigInt(m),q,off);cases++;}
}
for(const L of [31,32,63,64])for(const mask of [0n,1n,1n<<BigInt(L-1),(1n<<BigInt(L))-1n,((1n<<BigInt(L))-1n)&0xaaaaaaaaaaaaaaaan]){
 let count=0;for(let k=0;k<L;k++)count+=Number((mask>>BigInt(k))&1n);
 for(let q=0;q<=count+1;q++)for(const off of [0,1,7]){check(L,mask,q,off);cases++;}
}
return {cases,result:"all states match after commuting outstanding ordinary tail steps"};
})());
