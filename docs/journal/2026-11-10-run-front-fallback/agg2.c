// As agg.c, for ONE function (argv[1] name:start:end) and the 4 KiB heap-heap class only:
// attributes each candidate (a load by the function whose bytes match a recent store's mod
// 4 KiB at a different address, both on the heap) to its (store PC, load PC) pair, with the
// two addresses of its first occurrence and the min/max of each side's addresses.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define NST 48
#define STACK_LO 0x1000000000ULL
#define NP 4096
typedef unsigned long long u64;
static u64 sa[NST], sip[NST]; static int ssz[NST], sp;
static struct { u64 sip, lip, n, s0, l0, smin, smax, lmin, lmax; } P[NP]; static int np;
static int overl(u64 a,int as,u64 b,int bs,u64 m){ for(int i=0;i<as;i++){ u64 x=(a+i)&(m-1); for(int j=0;j<bs;j++) if(((b+j)&(m-1))==x) return 1; } return 0; }
int main(int argc,char**argv){
  char *s=strdup(argv[1]); strtok(s,":"); u64 lo=strtoull(strtok(0,":"),0,16), hi=strtoull(strtok(0,":"),0,16);
  char line[256]; u64 ia=0; int in=0; u64 loads=0, cand=0;
  while(fgets(line,sizeof line,stdin)){
    char k; u64 a; int sz;
    if(line[0]=='I'){ if(sscanf(line+1," %llx,%d",&ia,&sz)==2) in=ia>=lo&&ia<hi; continue; }
    if(line[0]!=' ' || sscanf(line," %c %llx,%d",&k,&a,&sz)!=3) continue;
    if((k=='L'||k=='M') && in && a<STACK_LO){
      loads++;
      for(int i=0;i<NST;i++){ if(!ssz[i]) continue; u64 b=sa[i]; int z=ssz[i];
        if(b>=STACK_LO) continue; if(a<b+z && b<a+sz) continue;
        if(!overl(a,sz,b,z,4096)) continue;
        cand++; int j; for(j=0;j<np;j++) if(P[j].sip==sip[i]&&P[j].lip==ia) break;
        if(j==np && np<NP){ np++; P[j].sip=sip[i]; P[j].lip=ia; P[j].s0=b; P[j].l0=a; P[j].smin=P[j].smax=b; P[j].lmin=P[j].lmax=a; }
        if(j<np){ P[j].n++; if(b<P[j].smin)P[j].smin=b; if(b>P[j].smax)P[j].smax=b; if(a<P[j].lmin)P[j].lmin=a; if(a>P[j].lmax)P[j].lmax=a; }
        break; }
    }
    if(k=='S'||k=='M'){ sa[sp]=a; sip[sp]=ia; ssz[sp]=sz>64?64:sz; sp=(sp+1)%NST; }
  }
  printf("heap loads by %s: %llu, 4K heap-heap candidates: %llu\n", argv[1], loads, cand);
  for(int r=0;r<12;r++){ int b=-1; for(int j=0;j<np;j++) if(P[j].n && (b<0||P[j].n>P[b].n)) b=j; if(b<0) break;
    printf("  n=%llu store_pc=%llx load_pc=%llx | store %llx..%llx (span %llu) | load %llx..%llx (span %llu)\n",
      P[b].n,P[b].sip,P[b].lip,P[b].smin,P[b].smax,P[b].smax-P[b].smin,P[b].lmin,P[b].lmax,P[b].lmax-P[b].lmin); P[b].n=0; }
  return 0;
}
