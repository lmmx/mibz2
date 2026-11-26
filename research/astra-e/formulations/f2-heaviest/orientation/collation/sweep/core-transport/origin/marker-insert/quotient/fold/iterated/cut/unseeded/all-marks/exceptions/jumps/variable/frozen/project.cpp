#define main variable_full_check_main
#include "../states.cpp"
#undef main
#include <cstring>
#include <string>
struct Node {U parent;uint8_t prefix;};
using Dictionary=std::vector<std::vector<Node>>;

Input input(const char* path){
    std::ifstream f(path,std::ios::binary);char magic[8];f.read(magic,8);
    assert(std::string(magic,8)=="CLKMASK1");U n=read32(f),mark=read32(f);
    std::vector<uint8_t> l(n);f.read(reinterpret_cast<char*>(l.data()),n);assert(f);
    return Input(std::move(l),mark);
}
Dictionary train(const Input& in,U threshold){
    Dictionary dict;
    std::vector<State> active{{0,in.n,0}};
    Counts ac;
    for(U depth=1;depth<=16;depth++){
        std::array<std::vector<std::pair<State,Node>>,256> buckets;
        U parent=0;
        histograms(in.runs,active,ac,[&](State,const auto& sum,const auto& seen,U count){
            for(U i=0;i<count;i++){
                auto a=seen[i];auto [n,start]=sum[a];
                if(n>=threshold)buckets[a].push_back({{start,start+n,0},{parent,a}});
            }
            ++parent;
        });
        dict.emplace_back();active.clear();
        for(auto& b:buckets)for(auto& pair:b){
            active.push_back(pair.first);dict.back().push_back(pair.second);
        }
        if(active.empty())break;
    }
    return dict;
}
struct Ranks{
    std::array<std::vector<Run>,256> groups;
    std::array<U,256> base{};
    uint64_t lookups=0,compares=0;
    explicit Ranks(const Input& in){
        std::array<U,256> freq{};
        for(auto r:in.runs){groups[r.a].push_back(r);freq[r.a]+=r.end-r.begin;}
        for(U a=1;a<256;a++)base[a]=base[a-1]+freq[a-1];
    }
    U bound(uint8_t a,U x){
        lookups++;
        auto& v=groups[a];
        auto i=std::lower_bound(v.begin(),v.end(),x,[&](const Run& r,U z){compares++;return r.begin<z;});
        if(i==v.begin())return base[a];
        auto prev=i-1;return prev->base+std::min(x,prev->end)-prev->begin;
    }
};
struct Truth {
    std::vector<uint8_t> circular;std::vector<U> positions;
    explicit Truth(const Input& in):positions(in.n,in.n){
        U x=in.primary;
        for(U i=0;i<in.n;i++){
            positions[x]=i;circular.push_back(in.f[x]);x=in.psi[x];
        }
        assert(x==in.primary&&*std::max_element(positions.begin(),positions.end())<in.n);
        for(U i=0;i<16;i++)circular.push_back(circular[i]);
    }
    void check(const std::string& word,const State& range)const{
        auto cmp=[&](U pos){return std::memcmp(circular.data()+pos,word.data(),word.size());};
        auto l=std::lower_bound(positions.begin(),positions.end(),0,[&](U v,int){return cmp(v)<0;});
        auto r=std::upper_bound(l,positions.end(),0,[&](int,U v){return cmp(v)>0;});
        assert(r-l==range.end-range.begin);
        if(r!=l)assert(U(l-positions.begin())==range.begin);
    }
};
void run(const Input& in,const Dictionary& dict,U threshold){
    Ranks rank(in);Truth truth(in);Counts cost,final_cost;
    std::vector<State> prev{{0,in.n,0}},owner=prev;
    std::vector<std::string> labels(1); // audit only; the projection takes parent,byte
    U active=0,total=0;
    for(U depth=1;depth<=dict.size();depth++){
        std::vector<State> next,occupied;
        std::vector<std::string> words;
        for(auto [id,a]:dict[depth-1]){
            auto par=prev[id];U lo=0,hi=0;
            if(par.begin!=par.end){lo=rank.bound(a,par.begin);hi=rank.bound(a,par.end);}
            State state{lo,hi,(Key(depth)<<32)|lo};
            words.push_back(char(a)+labels[id]);truth.check(words.back(),state);
            next.push_back(state);
            if(lo<hi)occupied.push_back(state);
        }
        active+=occupied.size();total+=next.size();owner=overlay(owner,occupied,cost);
        prev.swap(next);labels.swap(words);
    }
    std::vector<U> ids(in.n);
    for(U s=0;s<owner.size();s++)std::fill(ids.begin()+owner[s].begin,ids.begin()+owner[s].end,s);
    std::vector<uint8_t> predicted;
    histograms(in.runs,owner,final_cost,[&](State,const auto& sum,const auto& seen,U used){
        uint8_t m=seen[0];
        for(U j=0;j<used;j++){
            auto a=seen[j];auto v=sum[a];
            assert(ids[v.base]==ids[v.base+v.n-1]);
            if(v.n>sum[m].n||(v.n==sum[m].n&&a<m))m=a;
        }
        predicted.push_back(m);
    });
    U x=in.primary,ev=0,chunks=0,gap=0;
    for(U i=0;i<in.n;i++){
        if(predicted[ids[x]]==in.last[x])gap++;
        else{chunks+=gap/8;gap=0;ev++;}
        x=in.lf[x];
    }
    chunks+=gap/8;
    std::cout<<threshold<<','<<in.n<<','<<total<<','<<active<<','<<owner.size()<<','<<ev<<','<<chunks
             <<','<<in.runs.size()<<','<<rank.lookups<<','<<rank.compares<<','<<cost.overlay_steps
             <<','<<final_cost.fragments<<','<<final_cost.context_edges<<"\n";
}
int main(int argc,char** argv){
    assert(argc==3);auto training=input(argv[1]),target=input(argv[2]);
    for(U t:{32,128,512})run(target,train(training,t),t);
}
