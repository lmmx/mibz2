// agg3: candidate false store-to-load dependences, counted INDEPENDENTLY per class.
// Reads valgrind lackey --trace-mem=yes output (or a synthetic trace in the same format) on
// stdin. argv: name:start:end ... (absolute hex PC ranges of the functions whose loads count),
// optionally first "--image=lo:hi" (the binary's static image range; default 0x100000:0x1000000).
//
// For each load (L or M) by a listed function, and for each modulus m in {4096, 1024}: for
// each region class of the store (S stack >= 0x1000000000; D the static image; H anything
// else: heap and mmaps) it records whether ANY of the last NST stores (any function) of that
// class overlaps the load's bytes modulo m at a different real address. A load can count in
// several classes at once; no class consumes another (the defect of agg.c, which took the
// first match in ring order and recorded only its class: D086).
// Pairs: for each (function, load region, store region, modulus) the distinct (store PC,
// load PC) pairs among the matching stores, each counted once per load.
// These are trace candidates, not stalls: a store among the last NST observed need not be
// outstanding when a native load executes.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define NST 48
#define NF 16
#define STACK_LO 0x1000000000ULL
typedef unsigned long long u64;
static u64 IMG_LO=0x100000, IMG_HI=0x1000000;
static char *nm[NF]; static u64 lo[NF], hi[NF]; static int nf;
static u64 sa[NST], sip[NST]; static int ssz[NST], sp, filled;
static u64 loads[NF][3];            // [f][load region]
static u64 cnt[NF][3][3][2];        // [f][load region][store region][m: 0=4K 1=1K]
static u64 anyc[NF][2];
// pair table
#define HP (1<<20)
static struct { u64 key1, key2, n; } T[HP];
static int reg(u64 a){ return a>=STACK_LO?0: (a>=IMG_LO&&a<IMG_HI)?1:2; } // 0 S,1 D,2 H
static const char *RN="SDH";
static int fn_of(u64 ia){ for(int i=0;i<nf;i++) if(ia>=lo[i]&&ia<hi[i]) return i; return -1; }
static int overl(u64 a,int as,u64 b,int bs,u64 m){ for(int i=0;i<as;i++){ u64 x=(a+i)&(m-1); for(int j=0;j<bs;j++) if(((b+j)&(m-1))==x) return 1; } return 0; }
static void pair_add(int f,int lr,int sr,int mi,u64 spc,u64 lpc){
  u64 k1=((u64)f<<8)|(lr<<4)|(sr<<2)|mi, k2=spc*1000003ULL^lpc;
  u64 h=(k1*0x9e3779b97f4a7c15ULL^k2*0xc2b2ae3d27d4eb4fULL)&(HP-1);
  // exact: store spc and lpc separately in key2 high/low halves (PCs are < 2^32 under valgrind)
  u64 kk=(spc<<32)|(lpc&0xffffffffULL);
  for(;;){ if(!T[h].n){ T[h].key1=k1; T[h].key2=kk; T[h].n=1; return; }
           if(T[h].key1==k1 && T[h].key2==kk){ T[h].n++; return; } h=(h+1)&(HP-1); }
}
int main(int argc,char**argv){
  int a0=1;
  if(argc>1 && !strncmp(argv[1],"--image=",8)){ char*s=argv[1]+8; IMG_LO=strtoull(s,&s,16); IMG_HI=strtoull(s+1,0,16); a0=2; }
  for(int i=a0;i<argc;i++){ char *s=strdup(argv[i]); nm[nf]=strtok(s,":"); lo[nf]=strtoull(strtok(0,":"),0,16); hi[nf]=strtoull(strtok(0,":"),0,16); nf++; }
  char line[256]; int cur=-1; u64 ia=0;
  while(fgets(line,sizeof line,stdin)){
    char k; u64 a; int sz;
    if(line[0]=='I'){ if(sscanf(line+1," %llx,%d",&ia,&sz)==2) cur=fn_of(ia); continue; }
    if(line[0]!=' ' || sscanf(line," %c %llx,%d",&k,&a,&sz)!=3) continue;
    if((k=='L'||k=='M') && cur>=0){
      int lr=reg(a); loads[cur][lr]++;
      int hit[3][2]={{0}}; int any[2]={0,0};
      for(int i=0;i<(filled<NST?filled:NST);i++){ u64 b=sa[i]; int z=ssz[i];
        if(a<b+z && b<a+sz) continue;                    // a real overlap: a true dependence
        int sr=reg(b);
        for(int mi=0;mi<2;mi++){ u64 m=mi?1024:4096;
          if(overl(a,sz,b,z,m)){ hit[sr][mi]=1; any[mi]=1; pair_add(cur,lr,sr,mi,sip[i],ia); } } }
      for(int sr=0;sr<3;sr++) for(int mi=0;mi<2;mi++) cnt[cur][lr][sr][mi]+=hit[sr][mi];
      for(int mi=0;mi<2;mi++) anyc[cur][mi]+=any[mi];
    }
    if(k=='S'||k=='M'){ sa[sp]=a; sip[sp]=ia; ssz[sp]=sz>64?64:sz; sp=(sp+1)%NST; if(filled<NST) filled++; }
  }
  for(int f=0;f<nf;f++){
    printf("%s loads S=%llu D=%llu H=%llu | any4K=%llu any1K=%llu\n",nm[f],loads[f][0],loads[f][1],loads[f][2],anyc[f][0],anyc[f][1]);
    for(int mi=0;mi<2;mi++){ printf("  mod %s (load<-store):",mi?"1K":"4K");
      for(int lr=0;lr<3;lr++) for(int sr=0;sr<3;sr++) if(cnt[f][lr][sr][mi]) printf(" %c<-%c=%llu",RN[lr],RN[sr],cnt[f][lr][sr][mi]); printf("\n"); }
  }
  // top pairs per (f, lr, sr, m)
  for(int f=0;f<nf;f++) for(int mi=0;mi<2;mi++) for(int lr=0;lr<3;lr++) for(int sr=0;sr<3;sr++){
    u64 k1=((u64)f<<8)|(lr<<4)|(sr<<2)|mi; u64 tot=0; int any=0;
    for(int r=0;r<5;r++){ long best=-1; for(long h=0;h<HP;h++) if(T[h].n && T[h].key1==k1 && (best<0||T[h].n>T[best].n)) best=h;
      if(best<0) break; if(!any){ printf("PAIRS %s mod%s %c<-%c:",nm[f],mi?"1K":"4K",RN[lr],RN[sr]); any=1; }
      printf(" [store %llx load %llx n=%llu]",T[best].key2>>32,T[best].key2&0xffffffffULL,T[best].n); T[best].n=0; }
    if(any) printf("\n"); }
  return 0;
}
