function analyze(widths, order, p=0) {
const R=widths.length,n=widths.reduce((a,b)=>a+b,0),dest=[],f=[],start=[],lab=[];
let y=0;for(const a of order){dest[a]=y;y+=widths[a];}
let x=0;for(let a=0;a<R;a++)for(let j=0;j<widths[a];j++,x++){
const t=(x-p+n)%n;f[t]=(dest[a]+j-p+n)%n;lab[t]=a;
}
start[0]=true;for(let i=1;i<n;i++)start[i]=lab[i]!==lab[i-1]||f[i]!==f[i-1]+1;
const initial=start.filter(Boolean).length;
let cur=Array.from({length:n},(_,i)=>i),next=f.slice(),peak=initial,seq=[initial],levels=0,roundWitness;
while(next[0]!==0){
const prev=[];for(const i of cur)prev[next[i]]=i;
const keep=cur.filter(i=>i<next[i]&&i<prev[i]), ranks=[];
for(let j=0;j<keep.length;j++)ranks[keep[j]]=j;
const newNext=[];let prevSrc=-2,prevDest=-2,prevLen=-1,k=0,groups=[];
for(const i of keep){
let j=f[i],len=1,seam=start[i];while(ranks[j]===undefined){seam||=start[j];j=f[j];len++;}
const d=ranks[j];newNext[i]=j;
if(seam||prevSrc+1!==i||prevDest+1!==d||prevLen!==len){k++;groups.push([i,d,len]);}
prevSrc=i;prevDest=d;prevLen=len;
}
seq.push(k);if(k>peak){peak=k;roundWitness=groups;}
if(k>3*initial-2)throw Error(JSON.stringify({widths,order,p,initial,seq}));
if(keep.length>cur.length/2||keep[0]!==0)throw Error("halving");
cur=keep;next=newNext;levels++;
}
return {initial,peak,seq,roundWitness,f};
}
function* perms(a,i=0){if(i===a.length){yield a.slice();return;}for(let k=i;k<a.length;k++){[a[i],a[k]]=[a[k],a[i]];yield* perms(a,i+1);[a[i],a[k]]=[a[k],a[i]];}}
function* comp(n,r,pre=[]){if(r===1){yield [...pre,n];return;}for(let x=1;x<=n-r+1;x++)yield*comp(n-x,r-1,[...pre,x]);}

let maxima = {}, cases = 0;
for (let R=2;R<=5;R++) {
  const orders=[...perms(Array.from({length:R},(_,i)=>i))];
  let largest=0;
  for (let n=R;n<=(R===5?16:23);n++) {
    for (const widths of comp(n,R)) for (const order of orders) {
      const result=analyze(widths,order);
      cases++;
      if (result.peak>largest) {
        largest=result.peak;
        maxima[R]={w:widths,o:order,...result};
      }
    }
  }
}
console.log(JSON.stringify({cases,maxima,grow:analyze([1,6,1,4],[3,0,2,1])},null,2));
