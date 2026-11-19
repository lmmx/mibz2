#define main census_main
#include "census.cpp"
#undef main
struct Cursor {
    const std::vector<Span>& tape;
    size_t i=0;u consumed=0;
    u left()const{assert(i<tape.size());return tape[i].width-consumed;}
    u id()const{return tape[i].label;}
    void advance(u w){assert(w&&w<=left());consumed+=w;if(consumed==tape[i].width){consumed=0;i++;}}
};
void analyze_union(Block& b,std::istream& ms,std::ostream& counts,Outputs& outfile){
    assert(get(ms)==b.index&&get(ms)==b.m);
    const u src_intervals=get(ms),lf_intervals=get(ms),nm=get(ms);
    V masks(nm);read(ms,masks.data(),4*nm);
    u B=b.band.size();
    std::vector<U> mask((b.m+63)/64);V prefix(mask.size()),order(B);
    for(auto r:b.band)mask[r.start/64]|=U(1)<<(r.start%64);
    u size=0;for(u i=0;i<mask.size();i++){prefix[i]=size;size+=__builtin_popcountll(mask[i]);}assert(size==B);
    for(u j=0;j<B;j++){u x=b.band[j].start;order[prefix[x/64]+__builtin_popcountll(mask[x/64]&((U(1)<<(x%64))-1))]=j;}
    std::vector<Span> physical,bytes;std::array<u,256> freq{};
    for(u j:order)physical.push_back({j,b.band[j].width});
    for(u r:b.thin_spans)freq[r&255]+=r>>8;
    for(u c=0;c<256;c++)if(freq[c])bytes.push_back({c,freq[c]});
    std::vector<std::vector<Span>> high(B),low(256);
    std::unordered_map<u,u> prefixes;
    // The marked-coordinate embeddings below are only for transition and output auditing.
    V s_emb,k_emb,pm(b.m);u queue_base=0;
    for(auto z:b.band){for(u i=0;i<z.width;i++)pm[z.start+i]=queue_base+i;queue_base+=z.width;}
    Obj o;Cursor band{physical},byte{bytes};
    u source_chunks=0,source_single=0,overlap=0,rows=0;
    for(u v:masks){
        u flags=v&3,w=v>>2;
        while(w){
            u part=w,c=NO,j=NO;
            if(flags&1){part=std::min(part,band.left());j=band.id();}
            if(flags&2){part=std::min(part,byte.left());c=byte.id();}
            u weight=c;
            if(flags==1){
                u orig=b.weight[b.band[j].code];auto [it,inserted]=prefixes.try_emplace(orig,NO);
                if(inserted){
                    const auto& full=b.words.value[orig];assert(full.size()>1);
                    it->second=b.words.intern(full.substr(0,full.size()-1));
                }weight=it->second;
            }else if(flags==3){assert(c==b.weight[b.band[j].code]);overlap+=part;}
            if(o.label.empty()||o.label.back()!=weight){o.label.push_back(weight);o.width.push_back(0);o.base.push_back(rows);}
            u id=o.width.size()-1;o.width.back()+=part;
            if(flags==1)append(high[j],id,part);
            if(flags&2){append(low[c],id,part);byte.advance(part);}
            if(flags&1)band.advance(part);
            for(u i=0;i<part;i++){
                if(flags&1)s_emb.push_back(rows+i);
                if(flags&2)k_emb.push_back(rows+i);
            }
            rows+=part;w-=part;source_chunks++;source_single+=part==1;
        }
    }
    assert(s_emb.size()==b.m&&k_emb.size()==b.m&&rows==2*b.m-overlap);
    std::vector<Span> src_in,extra_in;
    u src_claims=0,extra_claims=0;
    std::array<u,256> idx{},used{};
    for(u v:b.thin_spans){
        u c=v&255,w=v>>8;
        while(w){
            const auto& r=low[c][idx[c]];u part=std::min(w,r.width-used[c]);append(src_in,r.label,part);
            src_claims++;w-=part;used[c]+=part;if(used[c]==r.width){used[c]=0;idx[c]++;}
        }
    }
    u queue_classes=0,modal_errors=0,single_queue_mass=0;
    for(u c=0;c<256;c++){
        assert(idx[c]==low[c].size());queue_classes+=low[c].size();
        u mode=0;for(auto r:low[c])mode=std::max(mode,r.width);
        modal_errors+=freq[c]-mode;
        if(low[c].size()==1)single_queue_mass+=freq[c];
    }
    for(const auto& v:high)for(auto r:v){append(extra_in,r.label,r.width);extra_claims++;}
    Cursor src{src_in},extra{extra_in};std::vector<Span> published;u merge_claims=0;
    for(u v:masks){
        Cursor& cur=(v&1)?src:extra;u w=v>>2;
        while(w){u part=std::min(w,cur.left());append(published,cur.id(),part);merge_claims++;w-=part;cur.advance(part);}
    }
    assert(src.i==src_in.size()&&extra.i==extra_in.size());
    // Independent ordinary decoding; the publisher above never uses the dense successor.
    V next(rows),seen(o.width.size());o.back.resize(rows);o.source.resize(rows);
    for(u k=0;k<o.width.size();k++)std::fill_n(o.source.begin()+o.base[k],o.width[k],k);
    for(auto r:published){
        for(u z=0;z<r.width;z++){
            u i=o.incoming.size(),k=r.label,x=o.base[k]+seen[k]++;
            o.incoming.push_back(k);o.back[i]=x;next[x]=i;
        }
    }
    assert(seen==o.width);
    for(u x=0;x<b.m;x++){
        u p=k_emb[pm[x]],h=s_emb[x],t=s_emb[b.next[x]];
        if(b.words.value[b.spelled(x)].size()==1)assert(p==h);
        else assert(p!=h&&next[h]==p);
        assert(next[p]==t);
    }
    u row=s_emb[b.mark];size_t out=0;
    while(out<b.n){
        const auto& w=b.words.value[o.label[o.source[row]]];assert(!w.empty());size_t take=std::min<size_t>(w.size(),b.n-out);
        assert(std::equal(w.begin(),w.begin()+take,b.reference.begin()+out,[](char a,uint8_t c){return uint8_t(a)==c;}));out+=take;row=next[row];
    }
    sections(outfile,b,o,"original-union",s_emb[b.mark],0);
    counts<<b.index<<','<<b.m<<','<<rows<<','<<src_intervals<<','<<lf_intervals<<','<<nm<<','<<overlap<<','<<source_chunks<<','<<source_single<<','
        <<o.width.size()<<','<<src_claims<<','<<src_in.size()<<','<<extra_claims<<','<<extra_in.size()<<','<<merge_claims<<','<<published.size()<<','<<queue_classes<<','<<modal_errors<<','<<single_queue_mass<<','<<entropy(o.width,rows)<<'\n';
}
int main(int argc,char** argv){
    assert(argc==6);std::ifstream edge(argv[1],std::ios::binary),word(argv[2],std::ios::binary),thin(argv[3],std::ios::binary),mask(argv[4],std::ios::binary);std::string dir=argv[5];
    std::ofstream counts(dir+"/union.csv");Outputs out;out.sections.open(dir+"/union-sections.csv");
    counts<<"block,m,union_rows,source_intervals,lf_intervals,merge_mask_spans,overlap_rows,source_claims,unit_source_claims,source_classes,base_in_claims,base_in_runs,extra_in_claims,extra_in_runs,publication_claims,published_runs,queue_projected_classes,exceptional_queue_rows,unfragmented_queue_rows,source_entropy\n";
    out.sections<<"block,coordinate,section,base,section_rows,queries,run_intersections,distinct_supports_incidence,visited_rows,second_word_types,second_dictionary_bytes,mark_steps,mark_skip,marked_hit\n";
    u n=0;while(true){Block b;if(!next_block(b,edge,word,thin))break;analyze_union(b,mask,counts,out);n++;}
    assert(n==233&&mask.peek()==std::char_traits<char>::eof());std::cerr<<n<<" exact positive unions; independent incoming publication reproduced each edge, each phase, and every marked byte.\n";
}
