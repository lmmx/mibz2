// Dense first-return geometry oracle. This is an offline accounting program.
#include "../color.hpp"
#include <array>
#include <fstream>
#include <iostream>
#include <string>
#include <unordered_map>
using induction::U;
using Byte=uint8_t;
using Bytes=std::vector<Byte>;
using Rows=std::vector<U>;
using lf_color::Graph;
using lf_color::Map;

Bytes read(const char* path) {
    std::ifstream f(path,std::ios::binary);
    assert(f); return Bytes(std::istreambuf_iterator<char>(f),{});
}
struct Input {
    Bytes L,raw;
    Rows lf;
    std::vector<induction::Run> runs;
};
Input make(const char* caps, const char* raw) {
    Input s; s.raw=read(raw); auto in=read(caps); assert(!(in.size()%2));
    for(size_t i=0;i<in.size();i+=2) {
        assert(in[i]&&in[i]<=127);
        s.L.insert(s.L.end(),in[i],in[i+1]);
    }
    U n=s.L.size(); assert(n==s.raw.size()); s.lf.resize(n);
    std::array<U,256> freq{},next{};
    for(Byte a:s.L) ++freq[a];
    for(U a=1;a<256;++a) next[a]=next[a-1]+freq[a-1];
    for(U x=0;x<n;++x) {
        Byte a=s.L[x]; s.lf[x]=next[a]++;
        if(x==0||s.L[x-1]!=a) s.runs.push_back({x,x+1,s.lf[x],a});
        else ++s.runs.back().end;
    }
    return s;
}
void audit(const Input& s, U mark, U block, const char* arm, const Bytes& removed,
           const Graph& g, const Map* map) {
    const U n=s.L.size(); Rows ranks(n+1,0),alive;
    U removed_spans=0,kept_spans=0,source_atoms=1,image_atoms=1,joint_atoms=1,adjacent=0;
    for(U x=0;x<n;++x) {
        if(map) assert(removed[x]==map->removed(x)&&map->rank(x)==ranks[x]);
        if(!removed[x]) { if(map) assert(map->select(alive.size())==x); alive.push_back(x); }
        ranks[x+1]=ranks[x]+!removed[x]; adjacent+=removed[x]&&removed[s.lf[x]];
        if(x==0||removed[x]!=removed[x-1]) ++(removed[x]?removed_spans:kept_spans);
        if(x) {
            bool l=s.L[x]!=s.L[x-1], a=removed[x]!=removed[x-1];
            bool b=removed[s.lf[x]]!=removed[s.lf[x-1]];
            source_atoms+=l||a; image_atoms+=l||b; joint_atoms+=l||a||b;
        }
    }
    if(map) assert(!adjacent&&map->rank(n)==alive.size()&&map->kept==alive.size());
    assert(!removed[mark]); Rows successor(alive.size()),spell(alive.size()),visits(alive.size(),0);
    std::unordered_map<std::string,U> table; std::vector<std::string> words;
    U total=0,max_width=0,short1=0,short2=0,dict_bytes=0,affine=0,bands=0,spelling_runs=0;
    for(U k=0;k<alive.size();++k) {
        U x=alive[k]; std::string word;
        do{word.push_back(char(s.L[x])); x=s.lf[x];}while(removed[x]);
        U target=ranks[x]; ++visits[target]; successor[k]=target;
        const U len=word.size(); total+=len; max_width=std::max(max_width,len);
        short1+=(len==1); short2+=(len==2);
        if(map) {
            auto v=map->next(k);
            assert(v.target==target&&v.length==len&&v.first==Byte(word[0]));
            assert(v.second==(len==2?Byte(word[1]):0));
        }
        auto [p,inserted]=table.emplace(word,words.size());
        if(inserted){words.push_back(word);dict_bytes+=len;} spell[k]=p->second;
        bool boundary=!k||successor[k]!=successor[k-1]+1;
        bool different=!k||spell[k]!=spell[k-1];
        affine+=boundary; bands+=boundary||different; spelling_runs+=different;
    }
    assert(total==n&&std::all_of(visits.begin(),visits.end(),[](U h){return h==1;}));
    U k=ranks[mark],pos=n,lap=0;
    do{
        for(unsigned char a:words[spell[k]]) {assert(pos&&s.raw[--pos]==a);}
        k=successor[k];++lap;
    }while(k!=ranks[mark]);
    assert(pos==0&&lap==alive.size());
    size_t aux=0,live=0,capacity=0;
    if(map) {
        aux=sizeof(Graph)+sizeof(Map);
        for(const auto& row:g.edges)aux+=row.capacity()*sizeof(lf_color::Bin);
        live=g.tiles.size()*sizeof(lf_color::Tile)+map->data.size()*sizeof(lf_color::Piece);
        capacity=g.tiles.capacity()*sizeof(lf_color::Tile)+map->data.capacity()*sizeof(lf_color::Piece)+aux;
    }
    std::cout<<block<<','<<arm<<','<<n<<','<<mark<<','<<ranks[mark]<<','<<s.runs.size()<<','<<g.tiles.size()
        <<','<<alive.size()<<','<<(n-alive.size())<<','<<kept_spans<<','<<removed_spans<<','<<source_atoms<<','<<image_atoms
        <<','<<joint_atoms<<','<<adjacent<<','<<affine<<','<<bands<<','<<spelling_runs<<','<<words.size()
        <<','<<dict_bytes<<','<<short1<<','<<short2<<','<<max_width<<','<<total<<','<<live<<','<<capacity<<'\n';
}
int main(int argc,char** argv) {
    assert(argc==6);
    U mark=std::stoul(argv[3]),block=std::stoul(argv[4]);
    auto s=make(argv[1],argv[2]); auto g=lf_color::graph(s.runs);
    auto types=read(argv[5]); U n=s.L.size(); assert(types.size()==n);
    for(const char* name:{"color-greedy","type-greedy","type-matching","mod2"}) {
        Bytes rm(n); std::string arm=name;
        if(arm=="color-greedy") {
            auto phases=lf_color::choose(g);
            Map m(g,phases,mark); Bytes c(n);
            for(U x=0;x<n;++x) {
                U y=s.lf[x];
                if(x!=y){U j=std::countr_zero(x>y?x-y:y-x);c[x]=((x>>j)&1u)^phases[j];}
            }
            U flip=1^c[mark];
            for(U x=0;x<n;++x) rm[x]=s.lf[x]!=x&&c[x]==flip&&c[s.lf[x]]!=flip;
            audit(s,mark,block,name,rm,g,&m);
        } else {
            for(U x=0;x<n;++x) rm[x]=arm=="mod2"?(x%2!=mark%2):((types[x]>>(arm=="type-matching"))&1u);
            audit(s,mark,block,name,rm,g,nullptr);
        }
    }
}
