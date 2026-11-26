#include "contexts.hpp"
#include "oracle.hpp"
#include <chrono>
#include <filesystem>
#include <iostream>
#include <time.h>

constexpr std::array<U,8> depths={1,2,3,4,6,8,12,16};
uint64_t cpu(){timespec t;clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t);return uint64_t(t.tv_sec)*1000000000+t.tv_nsec;}
U peak(){std::ifstream f("/proc/self/status");std::string line;while(std::getline(f,line))if(line.starts_with("VmHWM:"))return std::stoul(line.substr(6));return 0;}
void le32(std::ostream& f,U n){for(U j=0;j<4;++j)f.put(char(n>>(8*j)));}
void write_model(const std::filesystem::path& path,const Input& in,U k,const Model& m){
    std::ofstream f(path,std::ios::binary);f.write("CTXMOD01",8);
    const auto& e=m.exceptions;
    for(U w:{in.n,in.primary,k,m.states.size,U(e.begin.size()),m.exception_rows,m.primary_state})le32(f,w);
    U id=0;for(const auto& group:m.states.end)for(U end:group) {
        le32(f,end);le32(f,m.next[id]);f.put(char(m.modal[id++]));
    }
    for(U i=0;i<e.begin.size();++i){le32(f,e.begin[i]);le32(f,e.end[i]);le32(f,e.next[i]);f.put(char(e.symbol[i]));}
    assert(f.good());
}
void small(){
    uint64_t cases=0,multi=0,nondivide=0,periodic=0;
    for(U n=1,pow=3;n<=6;++n,pow*=3)for(U code=0;code<pow;++code){
        U bits=code;Input in;in.n=n;
        for(U j=0;j<n;++j){Byte a=Byte(bits%3);bits/=3;
            if(in.runs.empty()||in.runs.back().symbol!=a)in.runs.push_back({1,a});
            else ++in.runs.back().width;
        }
        Dense ref(in);
        for(U k:depths){
            ref.advance(k);
            for(in.primary=0;in.primary<n;++in.primary){
                Charge cost;auto model=construct<true>(in,k,cost);ref.check(in,model);
                U row=in.primary,lap=0;do{row=ref.psi[row];++lap;}while(row!=in.primary);
                assert(ref.marked(in,model,n)&&ref.marked(in,model,n+1));
                ++cases;multi+=lap<n;nondivide+=n%lap!=0;periodic+=lap<n&&n%lap==0;
            }
        }
    }
    // R=2 maximizes the incremental bound until depth N. The huge input stays run-compressed.
    U N=UINT32_MAX-1;
    Input many{{{1,98},{N,97}},N-1,N+1};
    Charge cost;auto large=construct<true>(many,1024,cost);
    assert(large.states.size==1025&&large.edges==1026&&large.exception_rows==1);
    for(auto l:cost.layers){assert(l.states==l.depth+1&&l.intersections==l.depth+2&&l.edges==l.depth+2);}
    Input pure{{{UINT32_MAX,231}},17,UINT32_MAX};Charge constant;
    auto one=construct<true>(pure,UINT32_MAX,constant);
    assert(constant.layers.size()==2&&one.states.size==1&&one.exception_rows==0&&one.modal[0]==231);
    std::cout<<"{\"marked_depth_cases\":"<<cases<<",\"multiple_cycle\":"<<multi<<",\"nondividing\":"<<nondivide
             <<",\"proper_dividing\":"<<periodic<<",\"huge_rows\":"<<many.n<<",\"linear_growth_depth\":1024,\"fixed_depth_requested\":"<<UINT32_MAX<<"}\n";
}
int main(int argc,char** argv){
    assert(argc>=2);std::string cmd=argv[1];
    if(cmd=="small"){small();return 0;}
    assert(argc>=4);
    if(cmd=="check"||cmd=="dump"){
        auto raw=read_input(argv[2]);std::filesystem::path folder=argv[3];
        std::filesystem::create_directories(folder);
        if(cmd=="dump"){
            U k=std::stoul(argv[4]);Charge cost;auto m=construct(raw,k,cost);
            write_model(folder/(std::to_string(k)+".ctx"),raw,k,m);return 0;
        }
        Dense dense(raw);
        std::ofstream layers(folder/"layers.csv");
        layers<<"k,depth,states,overlap,edges,pure,revisit,exception_slices,post_advances,old_endpoint_bytes,next_dynamic_bytes\n";
        std::cout<<"k,n,runs,states,edges,exception_rows,descriptors,wire_bytes,peak_dynamic_bytes,original_run_bytes\n";
        for(U k:depths) {
            dense.advance(k);Charge costs;auto model=construct<true>(raw,k,costs);
            dense.check(raw,model);
            auto path=folder/(std::to_string(k)+".ctx");write_model(path,raw,k,model);
            for(auto l:costs.layers) {
                layers<<k<<','<<l.depth<<','<<l.states<<','<<l.intersections<<','<<l.edges<<','<<l.pure<<','
                      <<l.modal_revisit<<','<<l.exception_slices<<','<<l.post_advances<<','<<l.initial_bytes<<','<<l.final_bytes<<'\n';
            }
            std::cout<<k<<','<<raw.n<<','<<raw.runs.size()<<','<<model.states.size<<','<<model.edges<<','<<model.exception_rows
                     <<','<<model.exceptions.begin.size()<<','<<std::filesystem::file_size(path)<<','<<costs.peak_dynamic
                     <<','<<raw.runs.capacity()*sizeof(Run)<<'\n';
        }return 0;
    }
    if(cmd=="rss"){
        auto raw=read_input(argv[2]);U k=std::stoul(argv[3]);Charge c;auto model=construct(raw,k,c);
        std::cout<<peak()<<','<<model.states.size<<','<<model.exception_rows<<'\n';return 0;
    }
    assert(cmd=="panel"&&argc==14);U repetitions=std::stoul(argv[2]);
    std::vector<Input> inputs;for(int i=4;i<argc;++i)inputs.push_back(read_input(argv[i]));
    std::ifstream table(argv[3]);std::array<std::array<std::array<U,4>,8>,10> valid{};
    for(auto& tape:valid)for(auto& k:tape)for(auto& cell:k)table>>cell;assert(table);
    std::cout<<"round,tape,k,thread_ns,wall_ns\n";
    for(U round=0;round<repetitions;++round)for(U t=0;t<inputs.size();++t) {
        U slot=round%2?U(inputs.size()-1-t):t;const auto& raw=inputs[slot];
        for(U j=0;j<depths.size();++j) {
            U i=(round+j)%depths.size();if(round%2)i=depths.size()-1-i;
            uint64_t before=cpu();auto wall=std::chrono::steady_clock::now();Charge cost;auto model=construct(raw,depths[i],cost);
            uint64_t duration=std::chrono::duration_cast<std::chrono::nanoseconds>(std::chrono::steady_clock::now()-wall).count();
            uint64_t elapsed=cpu()-before;
            auto fields=std::array<U,4>{model.states.size,model.edges,model.exception_rows,U(model.exceptions.begin.size())};
            assert(fields==valid[slot][i]);
            std::cout<<round<<','<<slot<<','<<depths[i]<<','<<elapsed<<','<<duration<<'\n';
        }
    }
}
