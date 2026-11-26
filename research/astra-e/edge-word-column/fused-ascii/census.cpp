#define main original_census_main
#include "../census.cpp"
#undef main
struct RangeTag{u start,end,code,ordinal;};
void verify(const Block& b){
    const u m=b.m,B=b.band.size(),first=256+b.nwords;
    const bool fits=uint64_t(first)+128*128-1<=(UINT32_MAX>>b.bits);
    V next(m),code(m),previous_next=b.next,previous_code=b.code,seq(B),start(B),q(m);
    std::iota(seq.begin(),seq.end(),0);std::sort(seq.begin(),seq.end(),[&](u a,u z){return b.band[a].start<b.band[z].start;});
    std::array<u,256> pos{};for(u c:b.thin)pos[c]++;
    u at=0;for(auto& c:pos){u n=c;c=at;at+=n;}
    for(u x=0;x<m;x++)q[pos[b.thin[x]]++]=x;
    at=0;for(u j=0;j<B;j++){start[j]=at;at+=b.band[j].width;}
    std::vector<RangeTag> ranges;
    u literal_bands=0,orig_runs=0,eligible=0,paired=0,legacy_gathers=0,legacy_writes=0,groups=0,pairgroups=0;
    for(u j:seq){auto v=b.band[j];u k=v.code<128?v.code:NO;
        if(k!=NO)literal_bands++;
        if(!ranges.empty()&&ranges.back().code==k&&(k==NO||ranges.back().ordinal+ranges.back().end-ranges.back().start==start[j]))
            ranges.back().end+=v.width;
        else ranges.push_back({v.start,v.start+v.width,k,start[j]});
    }
    V positions;for(auto v:ranges)positions.push_back(v.start);
    for(u j=0;j<B;j++){
        auto v=b.band[j];u at=start[j],limit=at+v.width;
        for(u z=0;z<v.width;z++){next[v.start+z]=q[at+z];code[v.start+z]=v.code;}
        if(!fits||v.code>=128)continue;
        eligible+=v.width;
        while(at<limit){
            const u y=q[at],id=std::upper_bound(positions.begin(),positions.end(),y)-positions.begin()-1;
            const auto& g=ranges[id];assert(y<g.end);groups++;pairgroups+=g.code!=NO;
            do{u mid=q[at],x=v.start+at-start[j];
                if(g.code!=NO&&mid!=b.mark){
                    u k=g.ordinal+mid-g.start;assert(q[k]==b.next[mid]);next[x]=q[k];code[x]=first+v.code*128+g.code;paired++;
                }at++;
            }while(at<limit&&q[at]<g.end);
        }
    }
    if(fits)for(u x=0;x<m;x++){
        u y=previous_next[x];if(previous_code[x]>=128||y==b.mark)continue;legacy_gathers++;
        if(previous_code[y]<128){previous_code[x]=first+previous_code[x]*128+previous_code[y];previous_next[x]=previous_next[y];legacy_writes++;}
    }
    for(u x=0;x<m;x++){
        u target=b.next[x];bool shortcut=fits&&b.code[x]<128&&b.code[target]<128&&target!=b.mark;
        assert(next[x]==(shortcut?b.next[target]:target));assert(code[x]==(shortcut?first+128*b.code[x]+b.code[target]:b.code[x]));
    }
    auto decode=[&](const V& ds,const V& symbol){
        size_t n=0;u x=b.mark,hops=0;
        do{
            u k=symbol[x];if(k>=first){u z=k-first;assert(b.reference[n++]==z/128&&b.reference[n++]==z%128);}
            else{const auto& word=b.words.value[b.weight[k]];for(uint8_t c:word)assert(b.reference[n++]==c);}
            x=ds[x];hops++;
        }while(x!=b.mark);assert(n>0&&n<=b.n);
        for(u i=n;i<b.n;i++)assert(b.reference[i]==b.reference[i%n]);
        return hops;
    };
    for(u i=0;i<m;i++)orig_runs+=i==0||b.code[i]!=b.code[i-1];
    std::cout<<b.index<<','<<b.stream<<','<<b.n<<','<<m<<','<<fits<<','<<b.bits<<','<<b.nwords<<','<<B<<','<<literal_bands<<','<<orig_runs<<','<<ranges.size()<<','<<eligible<<','
       <<groups<<','<<pairgroups<<','<<paired<<','<<legacy_gathers<<','<<legacy_writes<<','<<decode(b.next,b.code)<<','<<decode(previous_next,previous_code)<<','<<decode(next,code)<<'\n';
}
int main(int argc,char** argv){
    assert(argc==4);std::ifstream edge(argv[1],std::ios::binary),word(argv[2],std::ios::binary),thin(argv[3],std::ios::binary);
    std::cout<<"block,stream,n,m,fits,bits,words,bands,ascii_bands,physical_label_runs,partner_groups,eligible_source_rows,partner_claims,dual_ascii_claims,fused_pair_gathers,old_pair_lookups,old_pair_rewrites,old_hops,legacy_hops,fused_hops\n";
    u n=0;while(true){Block b;if(!next_block(b,edge,word,thin))break;verify(b);n++;}
    assert(n==233);std::cerr<<n<<" verified all original and directly composed row/code transitions; all three marked traversals matched each coded byte.\n";
}
