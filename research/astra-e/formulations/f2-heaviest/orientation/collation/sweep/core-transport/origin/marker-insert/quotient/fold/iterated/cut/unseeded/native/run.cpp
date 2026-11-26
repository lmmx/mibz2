#define POWER_SOLVER_LIBRARY
#include "direct_cut.cpp"
#include <array>
#include <memory>
#include "transport.hpp"
#include "observed-interval.hpp"
#include "observed-compact.hpp"

struct Trace {U rows,runs,ids,refined,after,out_runs,out_ids,flip,moved,skip,mark,power;};
struct Record {
    std::vector<Trace> trace;
    U final_skip=0,root_length=0,terminal_count=0;
    uint64_t moves=0,queries=0,hops=0,powers=0;
};

template<Carry Mode> struct Step {
    Transport<Mode> carry;
    std::vector<Slice> parts;
    std::vector<int8_t> color;
    U mass=0,refined=0;
    bool flip=false;
    Step(const Root& in,Mark& mark):carry(in,mark),color(in.bytes.size(),-1) {
        if(carry.done)return;
        const auto& b=carry.base;U ksize=U(color.size());
        std::vector<U> offset(ksize+1);parts.reserve(in.r.size()+ksize);
        U j=0,pos=0;
        auto slice=[&](Run r) {
            while(r.n){
                U w=std::min(r.n,b[j+1]-pos);parts.push_back({r.k,j,w,0});
                r.n-=w;pos+=w;if(pos==b[j+1])offset[++j]=parts.size();
            }
        };
        U i=0;
        if constexpr (Mode==Carry::Ordinal) {
            for(;i<carry.mark_run;++i)slice(in.r[i]);
            // A single raw run meets consecutive source buckets.
            carry.mark_slice=U(parts.size())+carry.source-j;
        }
        for(;i<in.r.size();++i)slice(in.r[i]);
        if constexpr (Mode==Carry::Singleton)carry.mark_slice=offset[carry.source];

        // Greedy undirected class graph in increasing order, ties outside (37c3bfd).
        std::vector<int> pending(ksize);
        for(U k=0;k<ksize;++k){
            int vote=pending[k];
            for(U t=offset[k];t<offset[k+1];++t) {
                auto s=parts[t];int8_t col=color[s.source];
                if(col>=0)vote+=col?-int(s.width):int(s.width);
            }
            color[k]=vote>0;
            int sign=color[k]?-1:1;
            for(U t=offset[k];t<offset[k+1];++t) {
                auto s=parts[t];
                if(color[s.source]<0)pending[s.source]+=sign*int(s.width);
            }
        }
        // Orientation's lexicographic tuple is symmetric in the two source sides.
        std::array<std::array<U,3>,2> cost{};
        for(auto col:color)++cost[1-col][0];
        std::vector<U> prev(ksize,absent);U last=absent;
        for(auto s:parts) {
            int col=color[s.source];
            if(prev[s.source]!=s.target){
                ++cost[col][0];
                cost[col][2]+=col!=color[s.target];
                prev[s.source]=s.target;
            }
            cost[col][1]+=last==s.source;last=s.source;
            if(!col&&color[s.target])mass+=s.width;
        }
        flip=cost[1]<cost[0];refined=cost[flip][0];
        if(flip)for(auto& col:color)col^=1;
    }
};

template<Carry Mode,bool Split>
std::vector<uint8_t> marked(const Root& original,Record& stats,bool trace) {
    const U horizon=std::accumulate(original.r.begin(),original.r.end(),U(0),[](U n,Run r){return n+r.n;});
    Root in=Split?seed(original):original;
    Mark m{Mode==Carry::Ordinal?original.mark:in.mark};
    std::vector<Node> grammar;
    std::vector<U> ids(256);
    for(auto b:original.bytes){ids[b]=grammar.size();grammar.push_back({1,b,0,Node::Literal});}
    for(auto& b:in.bytes)b=ids[b];
    bool normalized=false;
    for(;;) {
        Step<Mode> step(in,m);
        const auto& b=step.carry.base;
        const auto c=step.carry.source;
        assert(m.skip<grammar[in.bytes[c]].n);
        if(step.carry.done){
            stats.final_skip=m.skip;stats.root_length=grammar[in.bytes[c]].n;stats.terminal_count=b[c+1]-b[c];
            stats.moves=m.moved;stats.queries=m.queries;stats.hops=m.hops;
            return grammar_output(grammar,in.bytes[c],horizon,m.skip);
        }
        if(4*step.mass<b.back()) {
            assert(!normalized);
            auto old_hops=m.hops;
            in.mark=Mode==Carry::Ordinal?absent:m.value;
            auto v=observed_interval(in,step.carry);
            if constexpr(Mode==Carry::Ordinal) {
                assert(step.carry.power_new!=absent);
                m.skip+=U(m.hops-old_hops)*grammar[in.bytes[c]].n;
                m.value=step.carry.power_new;
            } else m.value=v.mark;
            Root next;next.r=std::move(v.r);next.mark=m.value;
            std::map<std::pair<U,U>,U> powers;
            for(auto sym:v.words) {
                U node=sym.base;
                if(sym.n>1) {
                    auto [it,fresh]=powers.emplace(std::make_pair(node,sym.n),U(grammar.size()));
                    if(fresh)grammar.push_back({grammar[node].n*sym.n,node,sym.n,Node::Power});
                    node=it->second;
                }
                next.bytes.push_back(node);
            }
            in=std::move(next);normalized=true;++stats.powers;continue;
        }
        const U before=m.moved;
        auto next=observed_compact<false>(in,grammar,step.parts,step.color,step.carry);
        if(trace)stats.trace.push_back({b.back(),U(in.r.size()),U(in.bytes.size()),step.refined,
            b.back()-step.mass,U(next.r.size()),U(next.bytes.size()),step.flip,U(m.moved-before),m.skip,m.value,normalized});
        in=std::move(next); normalized=false;
    }
}
std::vector<uint8_t> execute(const Root& in,U arm,Record& stats,bool trace=false) {
    switch(arm) {
        case 0:return marked<Carry::Singleton,true>(in,stats,trace);
        case 1:return marked<Carry::Ordinal,true>(in,stats,trace);
        case 2:return marked<Carry::Ordinal,false>(in,stats,trace);
        case 3:return direct(in);
    }
    std::abort();
}
const std::array<std::string,4> names={"singleton","split-ordinal","unseeded","dense"};
U arm_id(const std::string& s) {
    auto p=std::find(names.begin(),names.end(),s);assert(p!=names.end());return U(p-names.begin());
}
void small() {
    uint64_t cases=0,shared=0,relocated=0,powers=0,queries=0,hops=0,nondividing=0;
    for(U n=1,pow=3;n<=8;++n,pow*=3)for(U code=0;code<pow;++code) {
        U v=code,mask=0;Root in;for(U i=0;i<n;++i){U z=v%3;v/=3;append(in.r,z,1);mask|=1u<<z;}
        if(mask!=1&&mask!=3&&mask!=7)continue;
        for(U k=0;k<3;++k)if(mask&(1u<<k))in.bytes.push_back(k+1);
        for(in.mark=0;in.mark<n;++in.mark) {
            auto want=direct(in);
            Record records[3];
            for(U arm=0;arm<3;++arm)assert(execute(in,arm,records[arm],true)==want);
            const auto& a=records[0];const auto& b=records[1];const auto& u=records[2];
            assert(a.trace.size()==b.trace.size()&&a.moves==b.moves&&a.final_skip==b.final_skip);
            for(U i=0;i<a.trace.size();++i){
                auto x=a.trace[i],y=b.trace[i];
                assert(x.rows==y.rows&&x.after==y.after&&x.refined==y.refined&&x.out_runs==y.out_runs&&
                       x.out_ids==y.out_ids&&x.flip==y.flip&&x.moved==y.moved&&x.skip==y.skip&&x.power==y.power);
            }
            ++cases;shared+=u.terminal_count>1;relocated+=u.moves;powers+=u.powers;
            queries+=u.queries;hops+=u.hops;nondividing+=n%u.root_length!=0;
        }
    }
    for(U n:{17u,16384u,899998u})for(U x:{0u,1u,n/2,n-1,n}) {
        Root in{{{1,1},{0,n}},{65,98},x};
        Record r;assert(execute(in,2,r)==direct(in));assert(r.queries<=2);
    }
    std::cout<<"{\"marked_cases\":"<<cases<<",\"arms\":3,\"through_rows\":8,\"shared_fixed\":"<<shared
             <<",\"ordinary_moves\":"<<relocated<<",\"power_passes\":"<<powers<<",\"power_queries\":"<<queries
             <<",\"power_hops\":"<<hops<<",\"nondividing_horizons\":"<<nondividing<<",\"additional_large_marks\":15}\n";
}
int main(int argc,char** argv){
    if(argc==2&&std::string(argv[1])=="small"){small();return 0;}
    if(argc<4)return 2;
    auto in=load(argv[1]);std::string task=argv[2];
    if(task=="check") {
        auto want=direct(in);
        for(U a=0;a<3;++a) {
            Record r;auto out=execute(in,a,r,true);assert(out==want);
            std::filesystem::path dir=argv[3];
            std::ofstream bytes(dir/(names[a]+".bin"),std::ios::binary);
            bytes.write(reinterpret_cast<char*>(out.data()),out.size());
            std::ofstream tr(dir/(names[a]+".csv"));
            tr<<"depth,rows,runs,ids,refined,after,out_runs,out_ids,flip,moved,skip,mark,power\n";
            for(U d=0;d<r.trace.size();++d){auto x=r.trace[d];tr<<d<<','<<x.rows<<','<<x.runs<<','<<x.ids<<','<<x.refined<<','<<x.after<<','<<x.out_runs<<','<<x.out_ids<<','<<x.flip<<','<<x.moved<<','<<x.skip<<','<<x.mark<<','<<x.power<<'\n';}
            std::cout<<names[a]<<','<<r.final_skip<<','<<r.root_length<<','<<r.terminal_count<<','<<r.moves<<','<<r.queries<<','<<r.hops<<','<<r.powers<<'\n';
        }
        return 0;
    }
    std::filesystem::path root=argv[1];
    // In panel mode argv[1] names the first input as well, and argv[4..] includes the full panel.
    std::vector<Root> inputs;std::vector<std::string> tags;std::vector<std::vector<uint8_t>> expected;
    if(task=="panel"){
        for(int i=4;i<argc;++i){inputs.push_back(load(argv[i]));tags.push_back(std::filesystem::path(argv[i]).parent_path().filename());expected.push_back(direct(inputs.back()));}
    }else{tags.push_back(root.parent_path().filename());}
    int rounds=std::stoi(argv[3]);
    std::cout<<"round,tag,arm,cpu_ns,wall_ns,peak_kib\n";
    for(int i=0;i<rounds;++i) {
        size_t n=task=="panel"?inputs.size():1;
        for(size_t h=0;h<n;++h) {
            size_t t=i%2?n-1-h:h;const Root& r=task=="panel"?inputs[t]:in;
            for(U slot=0;slot<(task=="panel"?4u:1u);++slot){
                U a=task=="panel"?(i/4%2?3-(i+slot)%4:(i+slot)%4):arm_id(task);
                Record stats;auto c=cpu();auto t0=std::chrono::steady_clock::now();
                auto out=execute(r,a,stats);
                auto elapsed=std::chrono::steady_clock::now()-t0;auto ns=cpu()-c;
                if(task=="panel")assert(out==expected[t]);
                asm volatile("" : : "g"(out.data()) : "memory");
                std::cout<<i<<','<<tags[t]<<','<<names[a]<<','<<ns<<','<<std::chrono::duration_cast<std::chrono::nanoseconds>(elapsed).count()<<','<<resident_peak()<<'\n';
            }
        }
    }
}
