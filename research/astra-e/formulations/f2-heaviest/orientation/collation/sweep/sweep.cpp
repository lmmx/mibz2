#include "substring.hpp"
#include <array>
#include <chrono>
#include <fstream>
#include <iostream>
#include <map>
#include <set>
#include <string>

using Bytes=std::vector<uint8_t>;
struct Key {int start,length;};
struct Price {
    int words,bytes,nodes;
    bool operator==(const Price&) const=default;
};
struct Less {
    const ExactSubstrings* index;
    bool operator()(Key x,Key y) const {
        int cap=std::min(x.length,y.length);
        if(index->common(x.start,y.start,cap)==cap)return x.length<y.length;
        return index->ordered(x.start,y.start);
    }
};

class Dictionary {
    const ExactSubstrings& index;
    std::map<Key,int,Less> keys;
    using Iter=decltype(keys)::iterator;
    int d=0,overlap=0;

    int between(Iter left,Iter right){
        if(left==keys.end() || right==keys.end())return 0;
        Key a=left->first,b=right->first;
        int same=index.common(a.start,b.start,std::min(a.length,b.length));
        return same-(same==a.length);
    }
public:
    explicit Dictionary(const ExactSubstrings& idx):index(idx),keys(Less{&idx}){}
    void change(Key k,int delta){
        auto it=keys.lower_bound(k);
        if(it!=keys.end() && !keys.key_comp()(k,it->first)){
            it->second+=delta;
            if(it->second>0)return;
            assert(it->second==0);
            auto prev=it==keys.begin()?keys.end():std::prev(it),next=std::next(it);
            overlap+=between(prev,next)-between(prev,it)-between(it,next);
            d-=k.length;keys.erase(it);
            return;
        }
        assert(delta==1);
        auto prev=it==keys.begin()?keys.end():std::prev(it);
        int old=between(prev,it);
        auto added=keys.emplace_hint(it,k,1);
        overlap+=between(prev,added)+between(added,it)-old;d+=k.length;
    }
    Price price() const {
        int count=keys.size();
        return {count,d,(count>0)+d-count-overlap};
    }
};

struct Orbits {
    struct Location {int component,offset;};
    struct Cycle {int reverse_base,rows;};
    std::vector<Location> loc;
    std::vector<Cycle> cycles;
    Bytes reversed;
    Orbits(const std::vector<int>& psi,const Bytes& byte):loc(psi.size(),{-1,-1}){
        for(int root=0;root<int(psi.size());++root)if(loc[root].component==-1){
            int cid=cycles.size(),x=root;Bytes orbit;
            do {
                assert(loc[x].component==-1);
                loc[x]={cid,int(orbit.size())};orbit.push_back(byte[x]);x=psi[x];
            }while(x!=root);
            cycles.push_back({int(reversed.size()),int(orbit.size())});
            for(int copy=0;copy<2;++copy)reversed.insert(reversed.end(),orbit.rbegin(),orbit.rend());
        }
    }
    Key arc(int cid,int begin,int end) const {
        auto [base,size]=cycles[cid];
        int length=end-begin;if(length<=0)length+=size;
        return {base+(end?size-end:0),length};
    }
};

class Sweep {
    const Orbits& orbits;
    Dictionary dict;
    std::vector<std::set<int>> active;
    void arc(int c,int a,int b,int delta){dict.change(orbits.arc(c,a,b),delta);}
public:
    Sweep(const Orbits& o,const ExactSubstrings& i):orbits(o),dict(i),active(o.cycles.size()){}
    void insert(int row){
        auto [c,x]=orbits.loc[row];auto& s=active[c];auto n=s.lower_bound(x);
        if(s.empty())arc(c,x,x,1);
        else {
            if(n==s.end())n=s.begin();
            assert(x!=*n);
            int prev=*(n==s.begin()?s.rbegin():std::make_reverse_iterator(n));
            arc(c,prev,*n,-1);arc(c,prev,x,1);arc(c,x,*n,1);
        }
        s.insert(x);
    }
    void erase(int row){
        auto [c,x]=orbits.loc[row];auto& s=active[c];auto curr=s.find(x);assert(curr!=s.end());
        if(s.size()==1)arc(c,x,x,-1);
        else {
            int prev=*(curr==s.begin()?s.rbegin():std::make_reverse_iterator(curr));
            auto n=std::next(curr);if(n==s.end())n=s.begin();
            arc(c,prev,x,-1);arc(c,x,*n,-1);arc(c,prev,*n,1);
        }
        s.erase(curr);
    }
    Price price() const{return dict.price();}
};

Price dense(const std::vector<int>& psi,const Bytes& byte,int a,int m){
    std::set<Bytes> words,proper;
    for(int x=a;x<a+m;x++){
        Bytes word;int y=x;
        do {word.push_back(byte[y]);y=psi[y];}while(y<a || y>=a+m);
        words.insert(std::move(word));
    }
    int d=0;
    for(const auto& word:words) {
        d+=word.size();
        Bytes key(word.rbegin(),word.rend()-1);
        proper.insert(std::move(key));
    }
    int q=proper.empty()?0:1;
    for(auto it=proper.begin();it!=proper.end();++it){
        int h=0;
        if(it!=proper.begin()){
            auto& p=*std::prev(it);
            while(h<int(it->size()) && h<int(p.size()) && (*it)[h]==p[h])++h;
        }
        q+=int(it->size())-h;
    }
    return {int(words.size()),d,q};
}

std::pair<std::vector<int>,Bytes> from_column(const Bytes& column){
    int n=column.size();std::vector<int> psi(n);
    std::array<int,256> base{};
    for(auto c:column)++base[c];
    int sum=0;for(int& f:base){int old=f;f=sum;sum+=old;}
    for(int i=0;i<n;++i)psi[base[column[i]]++]=i;
    Bytes byte(n);for(int i=0;i<n;++i)byte[i]=column[psi[i]];
    return {std::move(psi),std::move(byte)};
}

void small(){
    int64_t cases=0,windows=0,multi=0;
    auto check=[&](const std::vector<int>& perm,const Bytes& bytes){
        Orbits o(perm,bytes);ExactSubstrings index(o.reversed);int n=perm.size();++cases;
        for(int m=1;m<=n;++m){
            Sweep s(o,index);
            for(int k=0;k<m;++k)s.insert(k);
            for(int a=0;;++a){
                assert(s.price()==dense(perm,bytes,a,m));++windows;multi+=o.cycles.size()>1;
                if(a+m==n)break;
                s.erase(a);s.insert(a+m);
            }
        }
    };
    for(int n=1;n<=5;n++){
        std::vector<int> perm(n);std::iota(perm.begin(),perm.end(),0);
        do {for(int mask=0;mask<(1<<n);++mask){
            Bytes b(n);for(int i=0;i<n;i++)b[i]=(mask>>i)&1;
            check(perm,b);
        }}while(std::next_permutation(perm.begin(),perm.end()));
    }
    for(int n=1;n<=7;n++)for(int mask=0;mask<(1<<n);++mask){
        Bytes col(n);for(int i=0;i<n;++i)col[i]=(mask>>i)&1;
        auto [f,b]=from_column(col);check(f,b);
    }
    std::cout<<"labelled_cases "<<cases<<" windows "<<windows<<" windows_on_multicycles "<<multi<<" exact\n";
}

using Clock=std::chrono::steady_clock;
double since(Clock::time_point start){return std::chrono::duration<double>(Clock::now()-start).count();}
int main(int argc,char** argv){
    if(argc==1){small();return 0;}
    if(argc!=6){std::cerr<<"sweep byte-column mass named-lo results.csv checks.csv\n";return 2;}
    auto t=Clock::now();
    std::ifstream input(argv[1],std::ios::binary);assert(input.good());
    Bytes column((std::istreambuf_iterator<char>(input)),{});
    int n=column.size(),m=std::stoi(argv[2]),named=std::stoi(argv[3]);assert(0<m && m<=n);
    auto [f,b]=from_column(column);Orbits o(f,b);
    double prep=since(t);
    t=Clock::now();ExactSubstrings index(o.reversed);double indexing=since(t);
    t=Clock::now();Sweep s(o,index);for(int i=0;i<m;++i)s.insert(i);double init=since(t);
    std::ofstream out(argv[4]),audit(argv[5]);assert(out.good() && audit.good());
    out<<"start,words,distinct_bytes,proper_suffixes\n";audit<<"start,words,distinct_bytes,proper_suffixes\n";
    std::set<int> tests={0,named,n-m};
    for(int j=1;j<17;j++)tests.insert(int(int64_t(n-m)*j/17));
    auto output=[](std::ostream& out,int a,Price p){out<<a<<','<<p.words<<','<<p.bytes<<','<<p.nodes<<'\n';};
    t=Clock::now();double verify=0;
    for(int a=0;;++a){
        Price p=s.price();output(out,a,p);
        if(tests.contains(a)){
            auto u=Clock::now();auto ref=dense(f,b,a,m);assert(p==ref);output(audit,a,ref);verify+=since(u);
        }
        if(a+m==n)break;
        s.erase(a);s.insert(a+m);
    }
    out.flush();audit.flush();assert(out.good() && audit.good());
    std::cerr<<"rows "<<n<<" cycles "<<o.cycles.size()<<" mass "<<m<<" windows "<<n-m+1
             <<" permutation_and_orbits_s "<<prep<<" suffix_index_s "<<indexing<<" initial_dictionary_s "<<init
             <<" slides_and_csv_s "<<since(t)-verify<<" independent_checks "<<tests.size()<<" checks_s "<<verify<<'\n';
}
