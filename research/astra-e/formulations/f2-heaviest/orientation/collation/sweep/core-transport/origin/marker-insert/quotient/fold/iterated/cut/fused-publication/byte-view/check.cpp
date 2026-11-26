#include "capped-base.inc"
#include "matrix.hpp"
struct Checked{U K,mass,ingress,decision,first_rows,first_runs,aliases,zr,zu,split;};
Checked check_one(const std::vector<uint16_t>& caps,U n,U p){
    auto raw=read_caps(caps.data(),caps.size(),n,p);
    auto seeded=seed(raw);
    std::vector<U> id(256);std::vector<Node> grammar;
    for(U a:raw.bytes){id[a]=grammar.size();grammar.push_back({1,a,0,Node::Literal});}
    for(auto& a:seeded.bytes)a=id[a];
    ByteMatrix v(caps.data(),caps.size(),n,p);std::vector<Run> seen;
    v.seeded_pieces([&](ByteMatrix::Ptr,ByteMatrix::Piece t,U){append(seen,t.id,t.len);});
    assert(v.center==seeded.mark&&seen==seeded.r&&v.base==bases(seeded)&&v.old_node==seeded.bytes);
    assert(equal_nodes(v.grammar,grammar));StreamFused<false> ref(seeded);
    auto route=v.choose();assert((route==ByteMatrix::Route::fixed)==ref.done);
    U out_runs=0,aliases=0,rows=0;
    if(!ref.done){
        assert(v.self==ref.self&&v.ingress==ref.ingress&&v.col==ref.color);
        assert((route==ByteMatrix::Route::normalize)==(4*ref.ingress<ref.m));
        if(route==ByteMatrix::Route::cut){
            auto a=v.publish(),b=ref.apply(seeded,grammar);assert(equal_root(a,b)&&equal_nodes(v.grammar,grammar));
            rows=n-v.ingress;out_runs=a.r.size();aliases=a.bytes.size();
        }
    }
    U cap_width=caps[v.zr]&255;U extra=U(v.zu!=0)+U(v.zu+1!=cap_width);
    return{v.K,v.self,v.ingress,U(route),rows,out_runs,aliases,v.zr,v.zu,extra};
}
int main(int argc,char** argv){
    assert(argc==2);
    if(std::string(argv[1])=="small"){
        uint64_t cases=0,options[3]={};
        for(U n=1;n<=7;++n){
            U end=1;for(U k=0;k<n;++k)end*=3;
            for(U h=0;h<end;++h){
                U v=h;std::vector<U> c(n);U opt=0;
                for(U& k:c){k=v%3;v/=3;}for(U i=1;i<n;++i)opt+=c[i]==c[i-1];
                for(U mask=0;mask<(1u<<opt);++mask){
                    U bit=0;std::vector<uint16_t> caps;
                    for(U i=0;i<n;++i){
                        bool extend=i&&c[i]==c[i-1]&&((mask>>(bit++))&1);
                        if(extend)++caps.back();else caps.push_back((uint16_t((17u+c[i]*47)<<8))|1);
                    }
                    for(U p=0;p<n;++p){auto q=check_one(caps,n,p);++cases;++options[q.decision];}
                }
            }
        }
        std::cout<<"marked_cap_partitions,fixed,mandatory_power,ordinary_cut\n"<<cases<<','<<options[0]<<','<<options[1]<<','<<options[2]<<'\n';
        return 0;
    }
    // Each argument row is the real block ordinal, mark and absolute cap filename.
    std::cout<<"block,rows,mark,K,matrix_bytes,caps,self,ingress,route,new_rows,new_runs,aliases,mark_cap,mark_offset,extra_cap_boundaries\n";
    std::ifstream list(argv[1]);assert(list);U block,p;std::string file;
    while(list>>block>>p>>file){
        std::ifstream f(file,std::ios::binary);assert(f);std::vector<uint16_t> caps;U n=0;int lo,hi;
        while((lo=f.get())!=EOF){hi=f.get();assert(hi!=EOF&&lo>0);caps.push_back(U(hi<<8)|U(lo));n+=U(lo);}
        auto q=check_one(caps,n,p);
        std::cout<<block<<','<<n<<','<<p<<','<<q.K<<','<<4*q.K*q.K<<','<<caps.size()<<','<<q.mass<<','<<q.ingress<<','<<q.decision<<','<<q.first_rows<<','<<q.first_runs<<','<<q.aliases<<','<<q.zr<<','<<q.zu<<','<<q.split<<'\n';
    }
}
