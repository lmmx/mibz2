#define ASTRA_TOKEN_LIBRARY
#include "writer.cpp"
extern "C" size_t token_capacity(const Object*);
extern "C" size_t full_capacity(const uint16_t*,size_t,const U*,U,U);

// Check every section source, every byte until its positive next return, and
// the occurrence-rank stable destination. Does not consult geometric witnesses.
N audit_token(const Object& o,const Oracle& dense,U lo){
    std::vector<U> source(o.bases,o.bases+o.k);U d=0;N edges=0;
    for(U i=0;i<o.r;++i){
        auto piece=o.root[i];auto w=o.words[piece.id];
        for(U j=0;j<piece.width;++j){
            U x=lo+source[piece.id]++;
            for(U k=0;k<w.width;++k){
                Byte want=o.dict[w.id+(k?k-1:w.width-1)];
                require(want==dense.f[x],"wrong token spelling at original row");
                require(!k || x-lo>=o.tokens,"premature section hit");
                x=dense.psi[x];
            }
            require(x==lo+d,"wrong positive first-return destination");++d;edges+=w.width;
        }
    }
    require(d==o.tokens && edges<=o.n,"incomplete arc cover");return edges;
}
int main(){
    static_assert(std::endian::native==std::endian::little);
    std::cout<<"stream,section,n,runs,tokens,hit_bands,words,root_runs,dict_bytes,per_band_spelling_bytes,source_mark,skip,second,geometry_allocated,assembly_additional_peak,output_object_allocated,rust_writer_reserved_with_output,full_compact_reserved_with_output,original_arc_bytes_checked\n";
    N cases=0,verified=0;
    while(true){
        std::array<U,4> head{};
        if(!std::cin.read(reinterpret_cast<char*>(head.data()),sizeof(head))){
            require(std::cin.eof() && std::cin.gcount()==0,"truncated header");break;
        }
        auto [stream,n,p,caps_n]=head;
        std::vector<uint16_t> caps(caps_n);
        require(bool(std::cin.read(reinterpret_cast<char*>(caps.data()),caps_n*2)),"truncated capped runs");
        std::array<U,256> freq{};
        std::vector<Run> runs;
        for(U v:caps){
            U b=v>>8,w=v&255;
            if(runs.empty() || runs.back().byte!=b)runs.push_back({b,0});
            freq[b]+=w;runs.back().len+=w;
        }
        Oracle independent(runs);auto expected=independent.tape(p);
        auto e=build(runs);
        auto baseline=full_capacity(caps.data(),caps.size(),freq.data(),n,p);
        U lo=std::accumulate(freq.begin(),freq.begin()+34,0u),width=freq[34];
        for(U depth=0;depth<2;++depth){
            Section section{{lo,lo+width}};
            SectionRanks ranks(section);
            BasicConstructor<PagedIndex> c(e);c.index_edges();c.run(section);
            auto base=mem.live;mem.peak=base;
            TokenObject data;
            require(rank_form(e,c,c.bands,ranks,n,p,true,true,data),"paid rank refused");
            auto o=data.view();auto obj=mem.live-base;auto peak=mem.peak-base;
            if(depth==0)compare(&o,caps.data(),caps.size(),freq.data(),p);
            U used=0;validate(consume(&o,&used),expected);require(used,"second refused");
            N old=0;for(auto b:c.bands)old+=c.groups[b.left].depth+1+c.groups[b.right].depth;
            auto traversed=audit_token(o,independent,lo);
            auto rust=token_capacity(&o);require(rust,"second refused capacity call");
            std::cout<<stream<<','<<(depth?"frequent":"quote")<<','<<n<<','<<runs.size()<<','<<width
                     <<','<<c.bands.size()<<','<<o.k<<','<<o.r<<','<<o.bytes<<','<<old<<','<<o.mark<<','<<o.skip
                     <<','<<used<<','<<base<<','<<peak<<','<<obj<<','<<rust<<','<<baseline<<','<<traversed<<'\n';
            ++cases;verified+=n;
            if(depth==0){
                std::vector<U> count(o.k);
                for(auto a:data.root)count[a.id]+=a.width;
                U best=0;
                for(U i=1;i<o.k;++i)if(count[i]>count[best] || (count[i]==count[best] && data.bases[i]<data.bases[best]))best=i;
                lo+=data.bases[best];width=count[best];
            }
        }
        std::cout.flush();
    }
    require(cases==456,"not 228 admissions");std::cerr<<cases<<" objects, "<<verified<<" marked bytes verified\n";
}
