// Compare every physical meeting to exact destination runs computed by dense walking.
#define ASTRA_RANK_LIBRARY
#include "../ranks.cpp"
#include <stdexcept>
void test(U alphabet,U maxlen){
    N cases=0,occurrences=0,bands=0,joins=0,disconnected=0,logical=0;
    for(U n=1,total=alphabet;n<=maxlen;++n,total*=alphabet)for(U code=0;code<total;++code){
        std::vector<Run> runs;
        U z=code;
        for(U j=0;j<n;++j){
            U b=z%alphabet;z/=alphabet;
            if(runs.empty() || runs.back().byte!=b)runs.push_back({b,0});
            ++runs.back().len;
        }
        Oracle o(runs);Edges e=build(runs);
        for(U mask=1;mask<(1u<<n);++mask){
            auto s=section_mask(n,mask);BasicConstructor<TreeIndex> c(e);c.run(s);
            std::vector<Word> incoming(n);
            std::vector<U> from(n,nil);
            for(U x=0;x<n;++x)if(mask>>x&1){
                ++occurrences;
                U y=x;Word w;
                do{w.push_back(o.f[y]);y=o.psi[y];}while(!(mask>>y&1));
                assert(from[y]==nil);from[y]=x;incoming[y]=w;
            }
            std::vector<Band> hits(c.bands.begin(),c.bands.end());
            std::sort(hits.begin(),hits.end(),[](auto a,auto b){return a.dest<b.dest;});
            U g=0,prev=nil;
            for(auto [lo,hi]:s){
                if(prev!=nil && incoming[prev]==incoming[lo])++joins;
                for(U y=lo;y<hi;){
                    U start=y++;
                    while(y<hi && incoming[y]==incoming[start]){assert(from[y]==from[y-1]+1);++y;}
                    auto b=hits.at(g++);
                    assert(b.dest==start && b.lo==from[start] && b.hi-b.lo==y-start &&
                           c.word(b)==incoming[start]);
                }
                prev=hi-1;
            }
            for(U y=0,last=nil;y<n;++y)if(from[y]!=nil){
                if(last==nil || incoming[y]!=incoming[last])++logical;
                last=y;
            }
            assert(g==hits.size() && c.stats.coalesces==0);
            ++cases;bands+=g;disconnected+=s.size()>1;
        }
    }
    assert(bands==logical+joins);
    std::cout<<"alphabet="<<alphabet<<" max_n="<<maxlen<<" sections="<<cases<<" disconnected="<<disconnected
             <<" return_occurrences="<<occurrences<<" exact_meeting_bands="<<bands<<" projected_words_runs="<<logical
             <<" equal_labels_across_section_gaps="<<joins<<" dynamic_packet_coalescences=0\n";
}
int main(){test(2,8);test(3,6);}
