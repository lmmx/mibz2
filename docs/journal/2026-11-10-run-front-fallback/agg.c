// Reads valgrind lackey --trace-mem=yes output on stdin. For data accesses made by
// instructions inside the given function ranges (argv: name:start:end, absolute hex), counts:
// - accesses, and a histogram of their addresses mod 4 KiB (64-byte lines) per region;
// - candidate false store-to-load dependences: a load whose bytes match, modulo 4 KiB (and
//   modulo 1 KiB), bytes of one of the last NST stores (any function) at a different address.
// Region: stack if addr >= STACK_LO, else heap/other. Pairs are classed hh, ss, sh.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define NST 48
#define NF 16
#define STACK_LO 0x1000000000ULL
typedef unsigned long long u64;
static char *nm[NF]; static u64 lo[NF], hi[NF]; static int nf;
static u64 acc[NF][2], a4[NF][3], a1[NF][3], lines[NF][2][64];
static u64 sa[NST]; static int ss[NST], sp;
static int fn_of(u64 ia){ for(int i=0;i<nf;i++) if(ia>=lo[i]&&ia<hi[i]) return i; return -1; }
static int overl(u64 a,int as,u64 b,int bs,u64 m){ // byte ranges overlap modulo m
  for(int i=0;i<as;i++){ u64 x=(a+i)&(m-1); for(int j=0;j<bs;j++) if(((b+j)&(m-1))==x) return 1; } return 0; }
static int overl_real(u64 a,int as,u64 b,int bs){ return a<b+bs && b<a+as; }
int main(int argc,char**argv){
  for(int i=1;i<argc;i++){ char *s=strdup(argv[i]); nm[nf]=strtok(s,":"); lo[nf]=strtoull(strtok(0,":"),0,16); hi[nf]=strtoull(strtok(0,":"),0,16); nf++; }
  char line[256]; int cur=-1;
  while(fgets(line,sizeof line,stdin)){
    char k; u64 a; int sz;
    if(line[0]=='I'){ if(sscanf(line+1," %llx,%d",&a,&sz)==2) cur=fn_of(a); continue; }
    if(line[0]!=' ') continue;
    if(sscanf(line," %c %llx,%d",&k,&a,&sz)!=3) continue;
    int st = a>=STACK_LO;
    if(cur>=0){ acc[cur][st]++; lines[cur][st][(a&4095)>>6]++; }
    if((k=='L'||k=='M') && cur>=0){
      int h4=0,h1=0,c4=0,c1=0;
      for(int i=0;i<NST;i++){ if(!ss[i]) continue; u64 s=sa[i]; int z=ss[i];
        if(overl_real(a,sz,s,z)) continue;
        int cls = (s>=STACK_LO)+(st); // 0 hh,1 sh,2 ss
        if(!h4 && overl(a,sz,s,z,4096)){ h4=1; c4=cls; }
        if(!h1 && overl(a,sz,s,z,1024)){ h1=1; c1=cls; } }
      if(h4) a4[cur][c4]++; if(h1) a1[cur][c1]++;
    }
    if(k=='S'||k=='M'){ sa[sp]=a; ss[sp]=sz>64?64:sz; sp=(sp+1)%NST; }
  }
  for(int f=0;f<nf;f++){
    printf("%s accesses heap=%llu stack=%llu | alias4K hh=%llu sh=%llu ss=%llu | alias1K hh=%llu sh=%llu ss=%llu\n",
      nm[f],acc[f][0],acc[f][1],a4[f][0],a4[f][1],a4[f][2],a1[f][0],a1[f][1],a1[f][2]);
    for(int st=0;st<2;st++){ printf("  %s lines(mod4K,64B):", st?"stack":"heap"); for(int l=0;l<64;l++) if(lines[f][st][l]) printf(" %x:%llu",l*64,lines[f][st][l]); printf("\n"); }
  }
  return 0;
}
