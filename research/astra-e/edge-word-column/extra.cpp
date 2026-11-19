#define main census_main
#include "census.cpp"
#undef main
#include <queue>
#include <iomanip>
#include <sstream>
struct ExtraOut{std::ofstream alt,partners,sections,literals,convex;};
struct Fixed{
    u cases=0,rows=0,unique_blocks=0,unique_rows=0;
    bool ambiguous=false,observed=false;
    std::string partner;
};
std::unordered_map<std::string,Fixed> corpus;
struct Opt{u removed;u eligible;u eligible_mass;u conflict_edges;};
Opt optimum(const V& g,const V& mass){
    size_t n=g.size();V indegree(n),take=mass,skip(n),queue;
    u eligible=0,sum=0,edges=0;
    for(u k=0;k<n;++k)if(g[k]!=NO){assert(k!=g[k]);eligible++;sum+=mass[k];if(g[g[k]]!=NO){indegree[g[k]]++;edges++;}}
    for(u k=0;k<n;++k)if(g[k]!=NO && !indegree[k])queue.push_back(k);
    u opt=0;
    for(u i=0;i<queue.size();++i){u k=queue[i],v=g[k];if(g[v]==NO){opt+=std::max(take[k],skip[k]);continue;}
        take[v]+=skip[k];skip[v]+=std::max(take[k],skip[k]);if(!--indegree[v])queue.push_back(v);}
    for(u start=0;start<n;++start)if(indegree[start]){
        std::vector<u> cycle;for(u k=start;indegree[k];k=g[k]){cycle.push_back(k);indegree[k]=0;}
        long best=0;
        for(u first=0;first<=1;++first){long on=first?take[start]:-1000000000L,off=first?-1000000000L:skip[start];
            for(size_t i=1;i<cycle.size();++i){u k=cycle[i];long old=off;off=std::max(on,off)+skip[k];on=old+take[k];}
            best=std::max(best,first?off:std::max(off,on));}
        opt+=(u)best;
    }
    return{opt,eligible,sum,edges};
}
std::string hexword(const std::string& a){
    static const char* digits="0123456789abcdef";std::string out;for(uint8_t v:a){out.push_back(digits[v>>4]);out.push_back(digits[v&15]);}return out;
}
void pair_eval(ExtraOut& f,const Block& b,const Obj& o,const V& next,const char* name,bool by_word){
    const u n=next.size();V owner(n);u k=by_word?b.words.value.size():o.base.size();V mass(k),g(k,NO),targets(k),best(k);
    for(u x=0;x<n;++x){owner[x]=by_word?o.label[o.source[x]]:o.source[x];mass[owner[x]]++;}
    std::unordered_map<U,u> pairs;for(u x=0;x<n;++x)pairs[(U(owner[x])<<32)|owner[next[x]]]++;
    double cond=0;for(auto [key,w]:pairs){u a=key>>32,z=(u)key;targets[a]++;g[a]=z;best[a]=std::max(best[a],w);cond+=w*std::log2(double(mass[a])/w);}
    if(by_word)for(u i=0;i<k;i++)if(mass[i]){
        const auto& word=b.words.value[i];auto& r=corpus[word];r.rows+=mass[i];r.cases++;
        if(targets[i]!=1){r.ambiguous=true;continue;}
        const auto& target=b.words.value[g[i]];r.unique_blocks++;r.unique_rows+=mass[i];
        if(r.observed&&r.partner!=target)r.ambiguous=true;
        if(!r.observed){r.observed=true;r.partner=target;}
    }
    u distinct=0,ambiguous=0,dominant=0,oneuse=0;for(u i=0;i<k;i++)if(mass[i]){distinct++;ambiguous+=targets[i]>1?mass[i]:0;
        dominant+=best[i];if(targets[i]!=1||g[i]==i)g[i]=NO;if(mass[i]==1)oneuse++;}
    if(!by_word){
        for(u i=0;i<k;i++){u a=o.base[i],z=owner[next[a]],last=owner[next[a+o.width[i]-1]];
            u from_endpoints=z==last&&z!=i?z:NO;assert(from_endpoints==g[i]);}
    }
    u anchor=std::max_element(mass.begin(),mass.end())-mass.begin();
    if(by_word)for(u i=0;i<256;i++)if(i<mass.size()&&mass[i]){
        f.literals<<b.index<<','<<i<<','<<mass[i]<<','<<targets[i]<<','<<best[i]<<',';
        if(targets[i]==1){u dest=owner[next[std::find(owner.begin(),owner.end(),i)-owner.begin()]];f.literals<<hexword(b.words.value[dest]);}f.literals<<'\n';}
    for(u protect=0;protect<=1;protect++){
        V z=g;if(protect)for(u i=0;i<k;i++)if(i==anchor||z[i]==anchor)z[i]=NO;
        Opt v=optimum(z,mass);
        f.partners<<b.index<<','<<name<<','<<protect<<','<<n<<','<<distinct<<','<<oneuse<<','<<pairs.size()<<','<<ambiguous<<','<<dominant<<','
            <<v.eligible<<','<<v.eligible_mass<<','<<v.conflict_edges<<','<<v.removed<<','<<mass[anchor]<<','<<cond/n<<'\n';
    }
}
void alternate(Block& b,ExtraOut& f){
    const u m=b.m,B=b.band.size();
    V kbase(B),piece(B),pmap(m);
    std::vector<U> mask((m+63)/64);V rank(mask.size()),sorted(B);
    u end=0;for(u j=0;j<B;j++){kbase[j]=end;end+=b.band[j].width;u l=b.band[j].start;mask[l>>6]|=U(1)<<(l&63);}
    assert(end==m);end=0;for(u i=0;i<mask.size();++i){rank[i]=end;end+=__builtin_popcountll(mask[i]);}
    for(u j=0;j<B;j++){u x=b.band[j].start;u r=rank[x>>6]+__builtin_popcountll(mask[x>>6]&((U(1)<<(x&63))-1));sorted[r]=j;}
    Obj o;u empty=b.words.intern("");u last=NO,prev_end=0;
    for(u j:sorted){auto t=b.band[j];const auto& w=b.words.value[b.weight[t.code]];u k=b.words.intern(w.substr(0,w.size()-1));
        if(k!=last || kbase[j]<prev_end){o.base.push_back(t.start);o.label.push_back(k);o.width.push_back(0);}
        o.width.back()+=t.width;piece[j]=o.base.size()-1;last=k;prev_end=kbase[j]+t.width;
        for(u i=0;i<t.width;i++)pmap[t.start+i]=kbase[j]+i;
    }
    u xclasses=o.base.size();std::array<u,256> freq{},byte_id{};for(u span:b.thin_spans)freq[span&255]+=span>>8;
    u at=m;for(u c=0;c<256;++c)if(freq[c]){byte_id[c]=o.base.size();o.base.push_back(at);o.label.push_back(c);o.width.push_back(freq[c]);at+=freq[c];}
    assert(at==2*m);
    std::vector<Span> incoming;for(u span:b.thin_spans)append(incoming,byte_id[span&255],span>>8);
    u low_runs=incoming.size();for(u j=0;j<B;++j)append(incoming,piece[j],b.band[j].width);
    u high_runs=incoming.size()-low_runs;
    for(auto r:incoming)o.incoming.insert(o.incoming.end(),r.width,r.label);
    o.source.resize(2*m);o.back.resize(2*m);V next(2*m),seen(o.width.size());
    for(u k=0;k<o.width.size();k++)std::fill_n(o.source.begin()+o.base[k],o.width[k],k);
    for(u i=0;i<2*m;++i){u k=o.incoming[i],j=o.base[k]+seen[k]++;o.back[i]=j;next[j]=i;}
    assert(seen==o.width);
    for(u x=0;x<m;x++){assert(next[x]==m+pmap[x]);assert(next[next[x]]==b.next[x]);}
    u row=b.mark;size_t out=0;while(out<b.n){
        const auto& w=b.words.value[o.label[o.source[row]]];size_t t=std::min<size_t>(w.size(),b.n-out);
        assert(std::equal(w.begin(),w.begin()+t,b.reference.begin()+out,[](char a,uint8_t c){return uint8_t(a)==c;}));
        out+=t;row=next[row];
    }
    u zero_ids=0,zero_rows=0;for(u i=0;i<xclasses;++i)if(o.label[i]==empty){zero_ids++;zero_rows+=o.width[i];}
    f.alt<<b.index<<','<<m<<','<<o.width.size()<<','<<xclasses<<','<<zero_ids<<','<<zero_rows<<','<<low_runs<<','<<high_runs<<','<<B<<','
        <<entropy(o.width,2*m)<<'\n';
    // Reuse the exact interval traversal; its output stream is moved temporarily.
    Outputs tmp;tmp.sections=std::move(f.sections);
    u a=std::max_element(o.width.begin(),o.width.begin()+xclasses)-o.width.begin();
    section(tmp,b,o,"alternating","largest-X-piece",o.base[a],o.base[a]+o.width[a],b.mark,0);
    a=std::max_element(o.width.begin()+xclasses,o.width.end())-o.width.begin();
    section(tmp,b,o,"alternating","largest-queue-byte",o.base[a],o.base[a]+o.width[a],b.mark,0);
    if(freq[34]){a=byte_id[34];section(tmp,b,o,"alternating","quote-queue-byte",o.base[a],o.base[a]+o.width[a],b.mark,0);}
    f.sections=std::move(tmp.sections);
    pair_eval(f,b,o,next,"alternating-piece",false);
}
void validate_optimum(){
    for(u n=1;n<=5;n++){
        u cases=1;for(u j=0;j<n;j++)cases*=n;
        for(u input=0;input<cases;input++){
            V g(n),mass(n);u digits=input;
            for(u i=0;i<n;i++){
                u c=digits%n;digits/=n;g[i]=c==i?NO:c;mass[i]=1+(input+3*i)%7;
            }
            u brute=0;for(u picked=0;picked<(1u<<n);picked++){
                bool ok=true;u sum=0;for(u i=0;i<n;i++)if((picked>>i)&1){
                    if(g[i]==NO||((picked>>g[i])&1)){ok=false;break;}sum+=mass[i];
                }
                if(ok)brute=std::max(brute,sum);
            }assert(optimum(g,mass).removed==brute);
        }
    }
}
void fixed_words(const std::string& dir){
    std::vector<std::string> keys;for(auto& [key,r]:corpus)if(!r.ambiguous&&r.observed&&key!=r.partner)keys.push_back(key);
    std::sort(keys.begin(),keys.end(),[](auto& a,auto& b){
        if(corpus.at(a).rows!=corpus.at(b).rows)return corpus.at(a).rows>corpus.at(b).rows;
        return a<b;
    });
    std::ofstream f(dir+"/fixed-partners.csv");
    f<<"min_blocks,eligible_words,total_eligible_rows,position,source_hex,target_hex,blocks,rows\n";
    for(u cap:{1u,2u,12u,117u,233u}){
        u n=0,sum=0;for(auto& key:keys)if(corpus.at(key).cases>=cap){n++;sum+=corpus.at(key).rows;}
        u rank=0;for(auto& key:keys){const auto& r=corpus.at(key);if(r.cases<cap)continue;
            f<<cap<<','<<n<<','<<sum<<','<<++rank<<','<<hexword(key)<<','<<hexword(r.partner)<<','<<r.cases<<','<<r.rows<<'\n';if(rank==32)break;
        }if(!rank)f<<cap<<",0,0,0,,,0,0\n";
    }
}
int main(int argc,char** argv){
    validate_optimum();assert(argc==5);std::ifstream edge(argv[1],std::ios::binary),word(argv[2],std::ios::binary),thin(argv[3],std::ios::binary);std::string d=argv[4];
    ExtraOut f{std::ofstream(d+"/alternating.csv"),std::ofstream(d+"/partners.csv"),std::ofstream(d+"/alternate-sections.csv"),std::ofstream(d+"/literal-partners.csv"),std::ofstream(d+"/conjugate.csv")};
    f.alt<<"block,m,classes,xclasses,zero_classes,zero_rows,low_byte_runs,high_ID_runs,bands,source_entropy\n";
    f.partners<<"block,model,protect_heaviest,rows,types,singletons,transition_types,ambiguous_rows,modal_partner_hits,eligible_types,eligible_mass,conflict_edges,max_nonadjacent_mass,heaviest_mass,conditional_entropy\n";
    f.sections<<"block,coordinate,section,base,section_rows,queries,run_intersections,distinct_supports_incidence,visited_rows,second_word_types,second_dictionary_bytes,mark_steps,mark_skip,marked_hit\n";
    f.literals<<"block,literal,frequency,partners,modal_frequency,single_successor_hex\n";
    f.convex<<"block,m,types,refined_runs,raw_word_runs,singleton_refined_types\n";
    Outputs sink;unsigned n=0;while(true){Block b;if(!next_block(b,edge,word,thin))break;
        V words(b.m),rot(b.m);for(u i=0;i<b.m;i++){words[i]=b.spelled(i);rot[i]=b.spelled(b.inverse_sigma[i]);}
        auto physical=evaluate(sink,b,"physical",words,b.next,words,true);
        pair_eval(f,b,physical,b.next,"physical-piece",false);pair_eval(f,b,physical,b.next,"physical-equivalent-spelling",true);
        auto conjug=evaluate(sink,b,"conjugate",rot,b.conj,rot,true);checked_output(b,conjug,b.sigma[b.mark],0);
        V raw(b.m);for(u j=0;j<b.m;j++)raw[b.conj[j]]=rot[j];
        f.convex<<b.index<<','<<b.m<<','<<conjug.base.size()<<','<<runs(conjug.incoming)<<','<<runs(raw)<<','<<std::count_if(conjug.width.begin(),conjug.width.end(),[](u w){return w==1;})<<'\n';
        alternate(b,f);n++;
    }
    fixed_words(d);assert(n==233);std::cerr<<n<<" exact alternating and conjugate marked objects, every augmented transition and byte, each endpoint partner, and compatibility optima recorded.\n";
}
