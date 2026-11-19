#define main interval_reference_cli
#include "../intervals.cpp"
#undef main
#include <numeric>
struct Atom{U end,label;};
struct Scans{
    std::vector<Atom> atoms;
    std::vector<U> safe,states;
    U modal=0,labels=0;
    Scans(const Source& p,const std::vector<U>& parts):states(parts){
        const auto n=U(p.last.size());
        std::array<U,257> base{};
        for(auto r:p.runs)base[r.byte+1]+=r.end-r.begin;
        std::partial_sum(base.begin(),base.end(),base.begin());
        std::vector<U> right=parts;
        for(U i=1;i<base.size();i++)if(base[i])right.push_back(base[i]);
        std::sort(right.begin(),right.end());
        right.erase(std::unique(right.begin(),right.end()),right.end());
        labels=right.size();
        std::array<std::vector<U>,256> allowed;
        std::array<U,256> dst{},seen{},last_end{},touched{};
        U src=0,used=0;
        std::vector<U> counts(labels),touched_labels;
        for(U b=0;b<256;b++)dst[b]=std::upper_bound(right.begin(),right.end(),base[b])-right.begin();
        for(auto r:p.runs){
            U x=r.begin,a=r.byte;
            while(x<r.end){
                U y=r.base+x-r.begin;
                while(right[dst[a]]<=y)dst[a]++;
                U target=dst[a];
                U w=std::min({r.end-x,parts[src]-x,right[target]-y});
                assert(w);
                if(!counts[target])touched_labels.push_back(target);
                counts[target]+=w;
                if(seen[a]!=src+1){touched[used++]=a;seen[a]=src+1;}
                last_end[a]=y+w;
                x+=w;atoms.push_back({x,target});
                if(x==parts[src]){
                    U best=0;for(U z:touched_labels){best=std::max(best,counts[z]);counts[z]=0;}
                    modal+=best;touched_labels.clear();
                    for(U j=0;j<used;j++){auto c=touched[j];allowed[c].push_back(last_end[c]);}
                    used=0;src++;
                }
            }
        }
        for(auto& group:allowed)safe.insert(safe.end(),group.begin(),group.end());
        assert(std::is_sorted(safe.begin(),safe.end())&&safe.back()==n);
        assert(std::adjacent_find(safe.begin(),safe.end())==safe.end());
        assert(atoms.size()<=p.runs.size()+2*(parts.size()-1));
    }
};
struct Cost {uint64_t fragments=0,score_visits=0,history=0;};
struct Node {U cut,previous;};
struct Choice {std::vector<U> ends;U objective=0;};
Choice optimize(const Scans& scans,U penalty,Cost& cost){
    struct Score {U value=0,path=0,epoch=0;};
    std::vector<Score> scores(scans.labels);
    Choice result;
    size_t a=0,j=0;U x=0,epoch=0;
    for(U stop:scans.states){
        ++epoch;U floor=0,floor_path=0,best=0,best_path=0;
        std::vector<Node> paths{{0,0}};
        while(x<stop){
            while(scans.safe[j]<=x)++j;
            U y=std::min({scans.safe[j],scans.atoms[a].end,stop});
            U t=scans.atoms[a].label;auto& entry=scores[t];
            if(entry.epoch!=epoch){entry={floor,floor_path,epoch};cost.score_visits++;}
            if(entry.value<floor){entry.value=floor;entry.path=floor_path;}
            entry.value+=y-x;++cost.fragments;
            if(entry.value>best){best=entry.value;best_path=entry.path;}
            x=y;
            if(y==scans.atoms[a].end)++a;
            if(y==scans.safe[j]&&y<stop&&best>floor+penalty){
                floor=best-penalty;floor_path=paths.size();paths.push_back({y,best_path});cost.history++;
            }
        }
        auto index=result.ends.size();
        for(U i=best_path;i;i=paths[i].previous)result.ends.push_back(paths[i].cut);
        std::reverse(result.ends.begin()+index,result.ends.end());
        result.ends.push_back(stop);result.objective+=best;
    }
    return result;
}
U brutescore(const Source& p,const std::vector<U>& ends){
    std::vector<std::map<Pair,U>> counts(ends.size());
    auto owner=[&](U x){return std::upper_bound(ends.begin(),ends.end(),x)-ends.begin();};
    for(U x=0;x<p.last.size();x++)++counts[owner(x)][{p.last[x],U(owner(p.lf[x]))}];
    U total=0;
    for(auto& hist:counts){U best=0;for(auto [t,c]:hist)best=std::max(best,c);total+=best;}
    return total;
}
void check(){
    uint64_t cases=0,subsets=0;
    for(U n=1;n<=6;n++)for(U code=0;code<(1u<<n);code++){
        std::vector<U> raw(n);for(U x=0;x<n;x++)raw[x]=(code>>x)&1;
        Source p(raw);
        for(U oldmask=0;oldmask<(1u<<(n-1));oldmask++){
            std::vector<U> parts;
            for(U x=1;x<n;x++)if(oldmask>>(x-1)&1)parts.push_back(x);
            parts.push_back(n);
            Scans s(p,parts);
            assert(s.modal==brutescore(p,parts));
            std::vector<U> cand;
            std::set_difference(s.safe.begin(),s.safe.end(),parts.begin(),parts.end(),std::back_inserter(cand));
            for(U lambda:{1,2,4}){
                Cost c;auto optimal=optimize(s,lambda,c);
                int best=-100;
                for(U mask=0;mask<(1u<<cand.size());mask++){
                    auto e=parts;
                    for(U j=0;j<cand.size();j++)if((mask>>j)&1)e.push_back(cand[j]);
                    std::sort(e.begin(),e.end());int score=int(brutescore(p,e))-int(lambda)*int(e.size()-parts.size());
                    best=std::max(best,score);subsets++;
                }
                assert(int(optimal.objective)==best);
                assert(int(brutescore(p,optimal.ends))-int(lambda)*int(optimal.ends.size()-parts.size())==best);cases++;
            }
        }
    }
    std::cout<<"parent_partition_penalties="<<cases<<" beaten_or_equal_finite_subsets="<<subsets<<"\n";
}
void run(const Source& p,U mark,U lambda){
    U n=p.last.size();std::array<U,256> count{};
    for(auto r:p.runs)count[r.byte]+=r.end-r.begin;
    std::vector<U> parts;U end=0;for(U q:count)if(q){end+=q;parts.push_back(end);}
    U expected=0;
    for(U round=0;round<=16;round++){
        Scans s(p,parts);
        if(round)assert(s.modal==expected);
        // Independent dense model and the marked trajectory are for diagnostics.
        Model audit(p,parts);std::vector<U> state(n);
        U left=0;for(U i=0;i<parts.size();i++){std::fill(state.begin()+left,state.begin()+parts[i],i);left=parts[i];}
        U x=mark,hits=0,gap=0,jumps=0;
        for(U t=0;t<n;t++){
            Pair step={p.last[x],state[p.lf[x]]};
            if(audit.mode[state[x]]==step){gap++;hits++;}
            else{jumps+=gap/8;gap=0;}
            x=p.lf[x];
        }
        jumps+=gap/8;assert(x==mark&&hits==s.modal);
        Cost c;auto best=round<16?optimize(s,lambda,c):Choice{parts,s.modal};
        U added=best.ends.size()-parts.size();
        expected=best.objective+lambda*added;
        assert(expected>=s.modal);
        std::cout<<lambda<<','<<round<<','<<n<<','<<parts.size()<<','<<s.modal<<','<<8*jumps<<','
                 <<s.atoms.size()<<','<<s.safe.size()<<','<<c.fragments<<','<<c.score_visits<<','
                 <<c.history<<','<<added<<','<<expected<<"\n";
        if(!added)break;
        parts.swap(best.ends);
    }
}
#ifndef ASTRA_REFINEMENT_LIBRARY
int main(int argc,char** argv){
    if(argc==1){check();return 0;}
    std::ifstream f(argv[1],std::ios::binary);char magic[8];f.read(magic,8);assert(std::string(magic,8)=="CLKMASK1");
    U n=word(f),mark=word(f);std::vector<U> l(n);for(auto& a:l)a=static_cast<unsigned char>(f.get());
    Source p(std::move(l));
    for(U lambda:{1,4,16,64})run(p,mark,lambda);
}
#endif
