#include <algorithm>
#include <array>
#include <cassert>
#include <cmath>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <numeric>
#include <string>
#include <unordered_map>
#include <vector>
using u=uint32_t;
using U=uint64_t;
using V=std::vector<u>;
constexpr u NO=UINT32_MAX;
struct Band{u start,width,code,byte;};
struct Span{u label,width;};
struct Outputs {std::ofstream models,geometry,sections;};
void read(std::istream& in,void* p,size_t n){in.read((char*)p,n);assert(in.good());}
u get(std::istream& f){u x;read(f,&x,4);return x;}
void append(std::vector<Span>& v,u c,u w){if(!v.empty() && v.back().label==c)v.back().width+=w;else v.push_back({c,w});}
u runs(const V& v){u n=0,p=NO;for(u x:v){n+=x!=p;p=x;}return n;}
double entropy(const V& freq,u m){double v=0;for(u x:freq)if(x)v+=x*std::log2(double(m)/x);return v/m;}
struct Weights{
    std::vector<std::string> value;
    std::unordered_map<std::string,u> ids;
    u intern(const std::string& w){auto [it,n]=ids.try_emplace(w,value.size());if(n)value.push_back(w);return it->second;}
};
struct Block{
    u index,stream,n,m,mark,bits,nwords;V next,code,thin,sigma,inverse_sigma,conj,weight,shifted;
    V thin_spans;u cap_count,maximal_runs;std::vector<Band> band;Weights words;
    std::vector<uint8_t> reference;
    u spelled(u x)const{return weight[code[x]];}
};
bool next_block(Block& b,std::istream& edges,std::istream& words,std::istream& thin){
    std::array<u,8> h;
    edges.read((char*)h.data(),32);if(!edges){assert(edges.eof()&&edges.gcount()==0);return false;}
    auto [idx,stream,n,m,mark,bits,nwords,nb]=h;b.index=idx;b.n=n;b.m=m;b.mark=mark;b.bits=bits;b.nwords=nwords;b.stream=stream;
    V packed(m);read(edges,packed.data(),4*m);b.band.resize(nb);read(edges,b.band.data(),16*nb);
    assert(get(words)==idx&&get(words)==stream&&get(words)==n&&get(words)==m&&get(words)==4*m&&get(words)==nwords);
    for(u k=0;k<256;++k){assert(b.words.intern(std::string(1,(char)k))==k);b.weight.push_back(k);}
    V visits(nwords);
    for(u i=0;i<nwords;++i){get(words);visits[i]=get(words);u len=get(words);std::string w(len,0);read(words,w.data(),len);b.weight.push_back(b.words.intern(w));}
    assert(get(thin)==idx&&get(thin)==stream&&get(thin)==n&&get(thin)==m&&get(thin)==mark);
    b.cap_count=get(thin);b.maximal_runs=get(thin);u nr=get(thin);b.thin_spans.resize(nr);read(thin,b.thin_spans.data(),4*nr);
    for(u span:b.thin_spans)b.thin.insert(b.thin.end(),span>>8,span&255);
    assert(b.thin.size()==m);u mask=(1u<<bits)-1;V occurrence(nwords),first_frequency(256),seen(m);
    b.code.resize(m);b.next.resize(m);b.sigma.resize(m);b.inverse_sigma.resize(m);b.conj.resize(m);b.shifted.resize(m);
    for(u i=0;i<m;++i){
        u y=packed[i]&mask,k=packed[i]>>bits;assert(y<m&&seen[y]++==0);b.next[i]=y;b.code[i]=k;
        if(k>=256)occurrence[k-256]++;
        assert((uint8_t)b.words.value[b.weight[k]].back()==b.thin[y]);first_frequency[b.thin[i]]++;
    }
    assert(visits==occurrence);
    std::array<u,256> pos{};for(u j=1;j<256;++j)pos[j]=pos[j-1]+first_frequency[j-1];
    for(u i=0;i<m;++i){u rank=pos[b.thin[i]]++;b.sigma[i]=rank;b.inverse_sigma[rank]=i;}
    std::unordered_map<U,u> rotated;
    for(u i=0;i<m;++i){u old=b.inverse_sigma[i];b.conj[i]=b.sigma[b.next[old]];
        u word=b.spelled(old);U key=(U(word)<<8)|b.thin[old];auto [it,fresh]=rotated.try_emplace(key,0);
        if(fresh){std::string v(1,(char)b.thin[old]);const auto& w=b.words.value[word];v.append(w.data(),w.size()-1);it->second=b.words.intern(v);}
        b.shifted[i]=it->second;
    }
    u x=mark;while(b.reference.size()<n){const auto& w=b.words.value[b.spelled(x)];u amount=std::min<size_t>(w.size(),n-b.reference.size());
        b.reference.insert(b.reference.end(),w.begin(),w.begin()+amount);x=b.next[x];}
    return true;
}
V pair_ids(const V& labels,const V& tag){
    V ids;ids.reserve(labels.size());std::unordered_map<U,u> names;
    for(u i=0;i<labels.size();++i){const U key=(U(labels[i])<<8)|tag[i];auto [p,added]=names.try_emplace(key,names.size());ids.push_back(p->second);}
    return ids;
}
struct Obj{V base,width,label,source,incoming,back;};
Obj evaluate(Outputs& out,const Block& b,const char* name,const V& labels,const V& next,const V& wt,bool keep){
    u m=b.m;u alphabet=*std::max_element(labels.begin(),labels.end())+1;V mass(alphabet),pieces(alphabet),desc(alphabet),last(alphabet,NO);
    u intervals=0,additional=0;Obj o;
    if(keep)o.source.resize(m);
    for(u i=0;i<m;++i){u k=labels[i];mass[k]++;
        if(last[k]!=NO)desc[k]+=next[last[k]]>=next[i];
        last[k]=i;
        bool cut=!i || labels[i-1]!=k;bool order_break=!cut && next[i-1]>=next[i];
        if(cut){pieces[k]++;intervals++;}additional+=order_break;
        if(keep){
            if(cut || order_break){o.base.push_back(i);o.width.push_back(0);o.label.push_back(wt[i]);}
            assert(o.label.back()==wt[i]);o.source[i]=o.base.size()-1;o.width.back()++;
        }
    }
    u used=0,nonconvex=0,nonmonotone=0,convex_rows=0,unstable_rows=0,drops=0;
    for(u k=0;k<alphabet;++k)if(mass[k]){used++;if(pieces[k]>1){nonconvex++;convex_rows+=mass[k];}
        if(desc[k]){nonmonotone++;unstable_rows+=mass[k];drops+=desc[k];}}
    V incoming(m);for(u x=0;x<m;++x)incoming[next[x]]=labels[x];
    out.models<<b.index<<','<<name<<','<<m<<','<<used<<','<<nonconvex<<','<<convex_rows<<','<<nonmonotone<<','
        <<unstable_rows<<','<<drops<<','<<intervals<<','<<additional<<','<<intervals+additional<<','<<runs(incoming)<<','<<entropy(mass,m)<<'\n';
    if(keep){
        o.incoming.resize(m);o.back.resize(m);V seen(o.base.size());
        for(u i=0;i<m;++i)o.incoming[next[i]]=o.source[i];
        for(u i=0;i<m;++i){u id=o.incoming[i];o.back[i]=o.base[id]+seen[id]++;assert(next[o.back[i]]==i);}
        assert(seen==o.width);
    }
    return o;
}
void checked_output(const Block& b,const Obj& o,u mark,u skip){
    V forward(b.m);for(u i=0;i<b.m;++i)forward[o.back[i]]=i;
    size_t produced=0;u row=mark;
    while(produced<b.n){const auto& word=b.words.value[o.label[o.source[row]]];
        assert(skip<=word.size());u take=std::min<size_t>(word.size()-skip,b.n-produced);
        assert(std::equal(word.begin()+skip,word.begin()+skip+take,b.reference.begin()+produced,[](char a,uint8_t c){return uint8_t(a)==c;}));
        produced+=take;skip=0;row=forward[row];
    }
}
struct Range{u lo,hi,byte_depth;};
void section(Outputs& out,const Block& b,const Obj& o,const char* coord,const char* name,u begin,u end,u marked,u skip){
    if(begin==end)return;
    u g=o.base.size(),q=0,pieces=0,inc=0,visited=0,terminals=0,marksteps=0;
    U dictionary=0,markbytes=skip;
    V seen(g,NO),child(g),terminal(g);for(u i=0;i<g;++i)terminal[i]=o.base[i]>=begin && o.base[i]<end;
    std::vector<Range> stack{{begin,end,0}};
    while(q<stack.size()){
        Range r=stack[q];visited+=r.hi-r.lo;u prev=NO;
        for(u i=r.lo;i<r.hi;++i){u k=o.incoming[i];pieces+=k!=prev;prev=k;u x=o.back[i];bool first=seen[k]!=q;
            if(first){seen[k]=q;inc++;}
            if(terminal[k]){
                if(first){terminals++;dictionary+=r.byte_depth+b.words.value[o.label[k]].size();}
            }else if(first){
                child[k]=stack.size();stack.push_back({x,x+1,r.byte_depth+(u)b.words.value[o.label[k]].size()});
            }else{auto& parent=stack[child[k]];assert(parent.hi==x);parent.hi++;}
        }
        q++;
    }
    u x=marked;while(x<begin||x>=end){x=o.back[x];markbytes+=b.words.value[o.label[o.source[x]]].size();marksteps++;if(x==marked)break;}
    out.sections<<b.index<<','<<coord<<','<<name<<','<<begin<<','<<end-begin<<','<<q<<','<<pieces<<','<<inc<<','<<visited<<','<<terminals<<','
        <<dictionary<<','<<marksteps<<','<<markbytes<<','<<(x>=begin&&x<end)<<'\n';
}
void sections(Outputs& out,const Block& b,const Obj& o,const char* coord,u mark,u skip){
    u max=std::max_element(o.width.begin(),o.width.end())-o.width.begin();section(out,b,o,coord,"largest-piece",o.base[max],o.base[max]+o.width[max],mark,skip);
    std::array<u,256> freq{},base{};int prev=-1;
    for(u k=0;k<o.width.size();++k){int c=(uint8_t)b.words.value[o.label[k]][0];assert(c>=prev);if(c!=prev)base[c]=o.base[k];freq[c]+=o.width[k];prev=c;}
    u lead=std::max_element(freq.begin(),freq.end())-freq.begin();
    section(out,b,o,coord,"largest-first-byte",base[lead],base[lead]+freq[lead],mark,skip);
    section(out,b,o,coord,"quote-first-byte",base[34],base[34]+freq[34],mark,skip);
}
struct Acquired{u chunks=0,singles=0,in_runs=0;};
Acquired bwt_from_live_geometry(const Block& b,const Obj& expected,bool alias){
    u m=b.m,nb=b.band.size();std::vector<U> starts((m+63)/64);V prefix(starts.size()),order(nb),id(nb);
    for(auto a:b.band){assert(!(starts[a.start/64]&(U(1)<<(a.start%64))));starts[a.start/64]|=U(1)<<(a.start%64);}
    u p=0;for(u i=0;i<starts.size();++i){prefix[i]=p;p+=__builtin_popcountll(starts[i]);}assert(p==nb);
    for(u j=0;j<nb;++j){u x=b.band[j].start;u i=prefix[x/64]+__builtin_popcountll(starts[x/64]&((U(1)<<(x%64))-1));order[i]=j;}
    V base;u previous=NO,at=0;
    std::array<u,256> cursor{};cursor.fill(NO);std::array<u,256> used{};
    for(u j:order){auto s=b.band[j];assert(s.start==at);at+=s.width;u w=alias?b.weight[s.code]:s.code;
        if(w!=previous){base.push_back(s.start);previous=w;}id[j]=base.size()-1;
    }
    assert(base==expected.base&&at==m);for(u i=0;i<nb;++i){u c=b.band[i].byte;if(cursor[c]==NO)cursor[c]=i;}
    std::vector<Span> result;Acquired count;
    for(u run:b.thin_spans){u c=run&255,left=run>>8;
        while(left){u j=cursor[c];assert(j<nb&&b.band[j].byte==c);u w=std::min(left,b.band[j].width-used[c]);
            append(result,id[j],w);count.chunks++;count.singles+=w==1;
            used[c]+=w;if(used[c]==b.band[j].width){cursor[c]++;used[c]=0;}left-=w;
        }
    }
    at=0;for(auto s:result){for(u z=0;z<s.width;++z)assert(expected.incoming[at++]==s.label);}assert(at==m);
    count.in_runs=result.size();return count;
}
void process(Block& b,Outputs& out){
    u m=b.m;V spelling(m),leading(m),rotcode(m),rotspelling(m),rotbyte(m);
    for(u i=0;i<m;++i){spelling[i]=b.spelled(i);leading[i]=(uint8_t)b.words.value[spelling[i]][0];
        u x=b.inverse_sigma[i];rotcode[i]=b.code[x];rotspelling[i]=b.spelled(x);rotbyte[i]=b.thin[x];}
    auto numeric=evaluate(out,b,"physical-code",b.code,b.next,spelling,true);
    auto spelled=evaluate(out,b,"physical-spelling",spelling,b.next,spelling,true);
    evaluate(out,b,"physical-source-byte-spelling",pair_ids(spelling,b.thin),b.next,spelling,false);
    evaluate(out,b,"physical-first-byte",leading,b.next,spelling,false);
    evaluate(out,b,"conjugate-same-spelling",rotspelling,b.conj,rotspelling,false);
    evaluate(out,b,"conjugate-source-byte-code",pair_ids(rotcode,rotbyte),b.conj,b.shifted,false);
    auto shift=evaluate(out,b,"conjugate-rotated-spelling",b.shifted,b.conj,b.shifted,true);
    auto a=bwt_from_live_geometry(b,numeric,false);auto e=bwt_from_live_geometry(b,spelled,true);
    checked_output(b,spelled,b.mark,0);checked_output(b,shift,b.sigma[b.mark],1);
    sections(out,b,spelled,"physical",b.mark,0);sections(out,b,shift,"shifted",b.sigma[b.mark],1);
    out.geometry<<b.index<<','<<b.stream<<','<<b.n<<','<<m<<','<<b.cap_count<<','<<b.maximal_runs<<','<<b.thin_spans.size()<<','
        <<b.band.size()<<','<<numeric.base.size()<<','<<spelled.base.size()<<','<<shift.base.size()<<','<<a.chunks<<','<<a.singles<<','
        <<a.in_runs<<','<<e.in_runs<<','<<runs(shift.incoming)<<','<<entropy(spelled.width,m)<<','<<entropy(shift.width,m)<<'\n';
}
int main(int argc,char** argv){
    assert(argc==5);std::ifstream edge(argv[1],std::ios::binary),word(argv[2],std::ios::binary),thin(argv[3],std::ios::binary);
    std::string d=argv[4];Outputs out{std::ofstream(d+"/models.csv"),std::ofstream(d+"/geometry.csv"),std::ofstream(d+"/sections.csv")};
    out.models<<"block,model,rows,types,nonconvex_types,nonconvex_rows,nonmonotone_types,nonmonotone_rows,same_type_descents,source_runs,in_run_descents,minimal_convex_types,raw_incoming_runs,source_entropy\n";
    out.geometry<<"block,stream,n,m,cap_records,maximal_input_runs,thin_spans,bands,numeric_pieces,spelled_pieces,shift_pieces,stream_intersections,unit_intersections,numeric_incoming_runs,spelled_incoming_runs,shift_incoming_runs,physical_refined_entropy,shifted_refined_entropy\n";
    out.sections<<"block,coordinate,section,base,section_rows,queries,run_intersections,distinct_supports_incidence,visited_rows,second_word_types,second_dictionary_bytes,mark_steps,mark_skip,marked_hit\n";
    unsigned count=0;while(true){Block b;if(!next_block(b,edge,word,thin))break;process(b,out);count++;}
    assert(count==233&&word.peek()==std::char_traits<char>::eof()&&thin.peek()==std::char_traits<char>::eof());std::cerr<<count<<" true tables: all live acquired incoming records, conjugate/physical forward ranks and marked coded bytes matched.\n";
}
