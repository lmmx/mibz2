#define ASTRA_REFINEMENT_LIBRARY
#include "../refine/refine.cpp"
#include <unordered_map>

// A true length-h backward word gets one integer ID; LF^h is its next row.
struct Words {
    std::vector<U> id, next;
    U alphabet;
    void twice(){
        std::unordered_map<uint64_t,U> names;
        std::vector<U> labels(id.size()),steps(id.size());
        for(U x=0;x<id.size();++x){
            uint64_t key=(uint64_t(id[x])<<32)|id[next[x]];
            labels[x]=names.try_emplace(key,names.size()).first->second;
            steps[x]=next[next[x]];
        }
        id.swap(labels);next.swap(steps);alphabet=names.size();
    }
};
struct Vote{U label,width;};
std::vector<Vote> compress(const std::vector<U>& ids){
    std::vector<Vote> runs;
    for(U label:ids){
        if(runs.empty()||runs.back().label!=label)runs.push_back({label,1});
        else ++runs.back().width;
    }
    return runs;
}
U dual(const std::vector<Vote>& votes,U alphabet,U penalty){
    std::vector<U> score(alphabet);
    U best=0;
    for(auto v:votes){
        U floor=best>penalty?best-penalty:0;
        auto& s=score[v.label];
        s=std::max(floor,s)+v.width;
        best=std::max(best,s);
    }
    return best;
}
U support(const std::vector<U>& ids,const std::vector<U>& end,U alphabet){
    std::vector<U> counts(alphabet),touched;
    U x=0,hit=0;
    for(U y:end){
        U mode=0;
        for(;x<y;++x){
            auto v=ids[x];if(!counts[v])touched.push_back(v);
            mode=std::max(mode,++counts[v]);
        }
        hit+=mode;
        for(auto v:touched)counts[v]=0;
        touched.clear();
    }
    return hit;
}
struct Policy {
    std::string name;
    U states;
    std::vector<unsigned char> hit;
    void twice(const std::vector<U>& next){
        auto old=hit;
        for(U x=0;x<old.size();++x)hit[x]=old[x]&&old[next[x]];
    }
};
Policy classify(const Source& p,const std::vector<U>& ends,std::string name){
    Model audit(p,ends);Policy m{std::move(name),U(ends.size()),{}};
    for(U x=0;x<p.last.size();++x)
        m.hit.push_back(audit.mode[audit.state(x)]==Pair(p.last[x],audit.state(p.lf[x])));
    return m;
}
void small(){
    uint64_t instances=0,subsets=0,checks=0;
    for(U n=1;n<=6;n++)for(U code=0;code<(1u<<n);code++){
        std::vector<U> raw(n);
        for(U x=0;x<n;++x)raw[x]=(code>>x)&1;
        Source p(raw);Words w{p.last,p.lf,2};std::vector<Policy> policies;
        std::vector<std::vector<U>> ends;
        for(U mask=0;mask<(1u<<(n-1));++mask){
            std::vector<U> e;
            for(U x=1;x<n;++x)if(mask>>(x-1)&1)e.push_back(x);
            e.push_back(n);ends.push_back(e);policies.push_back(classify(p,e,""));
        }
        std::vector<U> errors;
        for(auto& m:policies)errors.push_back(n-std::accumulate(m.hit.begin(),m.hit.end(),0u));
        for(U h:{1,2,4,8}){
            auto votes=compress(w.id);
            for(U lambda:{0,1,2,4,8}){
                int best=-1;U D=dual(votes,w.alphabet,lambda);
                for(U j=0;j<policies.size();j++){
                    U actual=std::accumulate(policies[j].hit.begin(),policies[j].hit.end(),0u);
                    U allowed=support(w.id,ends[j],w.alphabet),charge=lambda*(ends[j].size()-1);
                    assert(actual<=allowed&&allowed<=D+charge);
                    assert(n-actual<=h*errors[j]);
                    best=std::max(best,int(allowed)-int(charge));subsets++;
                }
                assert(int(D)==best);instances++;
            }
            for(auto& m:policies)m.twice(w.next);
            for(U x=0;x<n;++x){
                U y=x;for(U t=0;t<h;++t)y=p.lf[y];assert(y==w.next[x]);checks++;
            }
            w.twice();
        }
    }
    std::cout<<"dual_optima="<<instances<<" actual_partition_bounds="<<subsets<<" direct_powers="<<checks<<"\n";
}
void panel(const Source& p,Policy prefix){
    U n=p.last.size();
    std::vector<Policy> models;models.push_back(std::move(prefix));
    for(U width:{16,32,64,128,256,512,1024}){
        std::vector<U> e;for(U x=0;x<n;x+=width)e.push_back(std::min(n,x+width));
        models.push_back(classify(p,e,"w"+std::to_string(width)));
    }
    std::vector<U> initial;
    for(U x=1;x<n;x++)if(p.f[x]!=p.f[x-1])initial.push_back(x);
    initial.push_back(n);
    for(U lambda:{1,4,16,64}){
        auto e=initial;
        for(U r=0;r<16;r++){
            Scans s(p,e);Cost c;auto best=optimize(s,lambda,c);
            if(best.ends==e)break;
            e.swap(best.ends);
        }
        models.push_back(classify(p,e,"a"+std::to_string(lambda)));
    }
    Words w{p.last,p.lf,256};
    for(U h:{1,2,4,8,16,32}){
        auto votes=compress(w.id);std::vector<std::pair<U,U>> opts;
        for(U lambda:std::vector<U>{0,1,2,4,8,16,32,64,128,256,512,1024,n}){
            U value=dual(votes,w.alphabet,lambda);opts.emplace_back(lambda,value);
            std::cout<<"dual,"<<h<<','<<votes.size()<<','<<lambda<<','<<value<<"\n";
        }
        for(auto& m:models){
            uint64_t upper=n;U winning=0;
            for(auto [cost,value]:opts){
                auto bound=uint64_t(value)+uint64_t(cost)*(m.states-1);
                if(bound<upper){upper=bound;winning=cost;}
            }
            U got=std::accumulate(m.hit.begin(),m.hit.end(),0u);assert(got<=upper);
            std::cout<<"bound,"<<h<<','<<m.name<<','<<m.states<<','<<got<<','<<upper<<','<<winning<<','<<((n-upper+h-1)/h)<<"\n";
            if(h<32)m.twice(w.next);
        }
        if(h<32)w.twice();
    }
}
int main(int argc,char** argv){
    if(argc==1){small();return 0;}
    assert(argc==3);std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);
    assert(std::string(magic,8)=="CLKMASK1");U n=word(f);(void)word(f);
    std::vector<U> l(n);for(auto& b:l)b=static_cast<unsigned char>(f.get());
    Policy m{"k8",U(std::stoul(argv[2])),{}};
    for(U x=0;x<n;x++){int b=f.get();assert(b==0||b==1);m.hit.push_back(!b);}
    Source p(std::move(l));panel(p,std::move(m));
}
