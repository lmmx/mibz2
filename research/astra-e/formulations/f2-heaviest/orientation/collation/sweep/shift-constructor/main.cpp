#include "induce.hpp"
#include <fstream>
#include <iostream>
#include <map>
#include <numeric>
#include <set>
#include <time.h>
using namespace shift;
using Clock=std::chrono::steady_clock;
double cpu(){
    timespec t{};clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t);return t.tv_sec+1e-9*t.tv_nsec;
}
struct Oracle {
    std::vector<int> psi;
    std::string label;
    explicit Oracle(const std::vector<InputRun>& runs){
        std::string l;for(auto [c,w]:runs)l.append(w,char(c));
        psi.resize(l.size());std::iota(psi.begin(),psi.end(),0);
        std::stable_sort(psi.begin(),psi.end(),[&](int a,int b){return Byte(l[a])<Byte(l[b]);});
        for(int x:psi)label.push_back(l[x]);
    }
    std::string marked(int x) const {
        std::string out;for(int j=0;j<int(psi.size());j++){out.push_back(label[x]);x=psi[x];}return out;
    }
    void object(const Section& target,Object& obj) const {
        std::map<std::string,std::pair<int,int>> lookup;
        std::vector<int> order(psi.size(),-1),incoming(target.mass,-1);
        for(auto p:target.list)for(int x=p.lo;x<p.hi;++x)order[x]=p.base+x-p.lo;
        for(auto p:target.list)for(int x=p.lo;x<p.hi;++x){
            int start=order[x],y=x;std::string word;
            do {word.push_back(label[y]);y=psi[y];}while(order[y]<0);
            auto [it,fresh]=lookup.try_emplace(word,std::pair{start,start});
            (void)fresh;assert(it->second.second==start);++it->second.second;
            auto& dest=incoming[order[y]];assert(dest<0);dest=it->second.first;
        }
        assert(lookup.size()==obj.words.size());
        for(const auto& w:obj.words){auto it=lookup.find(w.text);assert(it!=lookup.end());
            assert(it->second.first==w.base && it->second.second==w.base+w.width);}
        int row=0,prior=-1;
        for(auto [id,width]:obj.incoming){
            assert(prior!=id && width>0);prior=id;int base=obj.words[id].base;
            for(int k=0;k<width;k++)assert(incoming[row++]==base);
        }
        assert(row==target.mass);
    }
};
struct Result {Stats stats;std::string bytes;double prepare,build,emit,total,wall;};
Result execute(const std::vector<InputRun>& raw,Range section,int shift,bool fuse,int mark,const Oracle* oracle=nullptr,
               bool all_marks=false){
    auto wall=Clock::now();double t=cpu();
    Result r{};auto& st=r.stats;Counts co(raw);Plan plan(raw,co,section,shift,st);
    if(!plan.available){r.stats.h=-1;return r;}
    Index ix(raw,co,plan.levels.back(),st);
    auto ready=cpu();Object obj(plan,ix,st,fuse);auto built=cpu();r.bytes=obj.expand(mark,co.n);auto done=cpu();
    r.prepare=ready-t;r.build=built-ready;r.emit=done-built;r.total=done-t;
    r.wall=std::chrono::duration<double>(Clock::now()-wall).count();
    if(oracle){
        oracle->object(plan.levels.back(),obj);assert(oracle->marked(mark)==r.bytes);
        if(all_marks)for(int p=0;p<co.n;p++)assert(obj.expand(p,co.n)==oracle->marked(p));
    }
    return r;
}
void small(){
    N sections=0,objects=0,marks=0,holes=0,early=0,interior_early=0,disjoint=0;
    std::string witness;
    for(auto [alphabet,maxn]:{std::pair{2,8},std::pair{3,5}}){
        for(int n=1;n<=maxn;n++){int count=1;for(int j=0;j<n;j++)count*=alphabet;
            for(int bits=0;bits<count;bits++){
                int seed=bits;std::vector<InputRun> runs;bool has_third=false;
                for(int j=0;j<n;j++){int c=seed%alphabet;seed/=alphabet;has_third|=c==2;
                    if(runs.empty() || runs.back().byte!=c)runs.push_back({c,1});else ++runs.back().width;}
                if(alphabet==3 && !has_third)continue;
                Oracle o(runs);Counts co(runs);
                for(int a=0;a<n;a++)for(int b=a+1;b<=n;b++){
                    ++sections;
                    for(int h=0;h<=4;h++){
                        Stats z;Plan plan(runs,co,{a,b},h,z);if(!plan.available)break;
                        for(int fusion=0;fusion<=(h>0);fusion++){
                            auto r=execute(runs,{a,b},h,fusion,0,&o,true);
                            ++objects;marks+=n;holes+=r.stats.selected_components>1;
                            if(fusion && r.stats.fused_early_rows){++early;
                                if(h>=2 && witness.empty()){
                                    witness="column=";for(auto run:runs)witness.append(run.width,char('0'+run.byte));
                                    witness+=" U=["+std::to_string(a)+","+std::to_string(b)+") h="+std::to_string(h);
                                }}
                            if(fusion && h>=2 && r.stats.fused_early_rows)++interior_early;
                        }
                        bool misses=false;for(int x=0;x<n;x++){
                            int row=x;while(plan.levels.back().at(row)<0){row=o.psi[row];if(row==x){misses=true;break;}}
                        }
                        disjoint+=misses;
                    }
                }
            }
        }
    }
    std::cout<<"sections "<<sections<<" constructor_objects "<<objects<<" marked_outputs "<<marks
             <<" holed_objects "<<holes<<" fused_with_early "<<early<<" fused_h_ge_2_with_early "<<interior_early
             <<" admissible_sections_with_disjoint_cycle "<<disjoint<<" witness "<<witness<<'\n';
}
void csv(const Result& r,int ordinal,bool fused,int rep,int pos){
    auto s=r.stats;
    std::cout<<ordinal<<','<<s.h<<','<<int(fused)<<','<<rep<<','<<pos<<','<<s.mass<<','<<s.selected_components
      <<','<<s.forward_pieces<<','<<s.image_probes<<','<<s.image_cuts<<','<<s.lf_pieces<<','<<s.setup_boundary_tests<<','<<s.setup_boundary_advances
      <<','<<s.fused_root_tests<<','<<s.fused_domain_pieces<<','<<s.fused_early_rows<<','<<s.fused_early_bands
      <<','<<s.nodes<<','<<s.live_packets<<','<<s.live_rows<<','<<s.lf_seeks<<','<<s.page_increments<<','<<s.translated_pieces
      <<','<<s.nonterminal_bands<<','<<s.terminal_bands<<','<<s.coalesced_packets<<','<<s.selectors<<','<<s.unary_moves
      <<','<<s.copied_merge_entries<<','<<s.stream_rows<<','<<s.words<<','<<s.dictionary_bytes<<','<<s.root_runs
      <<','<<s.mark_probes<<','<<s.output_words<<','<<r.prepare<<','<<r.build<<','<<r.emit<<','<<r.total<<','<<r.wall<<'\n';
}
int main(int argc,char** argv){
    if(argc==1){small();return 0;}
    assert(argc==4);std::string mode=argv[1];int trials=std::stoi(argv[3]);
    std::ifstream in(argv[2]);int id,n,mark,a,b,r;
    std::cout<<"ordinal,h,fused,round,position,mass,components,forward_pieces,image_probes,image_cuts,lf_pieces,setup_boundary_tests,setup_boundary_advances,fused_root_tests,fused_domain_pieces,fused_early_rows,fused_early_bands,live_nodes,live_packets,live_rows,lf_seeks,page_increments,translated_pieces,nonterminal_bands,terminal_bands,coalesced_packets,selectors,unary_moves,copied_merge_entries,stream_rows,words,dictionary_bytes,root_runs,mark_probes,output_words,setup_cpu_s,build_cpu_s,emit_cpu_s,cpu_s,wall_s\n";
    while(in>>id>>n>>mark>>a>>b>>r){
        std::vector<InputRun> runs(r);for(auto& x:runs)in>>x.byte>>x.width;
        Oracle o(runs);std::string gold=o.marked(mark);assert(int(gold.size())==n);
        std::vector<std::pair<int,bool>> arms={{0,false}};
        for(int h=1;h<=6;h++){
            Counts c(runs);Stats st;Plan p(runs,c,{a,b},h,st);if(!p.available)break;
            if(mode=="check")arms.emplace_back(h,false);
            arms.emplace_back(h,true);
        }
        int count=arms.size();
        for(int lap=mode=="check"?0:-1;lap<trials;lap++)for(int pos=0;pos<count;pos++){
            int arm=(pos+lap+1)%count;if(lap>=0 && (lap/count)%2)arm=count-1-arm;
            auto [h,fuse]=arms[arm];auto z=execute(runs,{a,b},h,fuse,mark,mode=="check"?&o:nullptr);
            assert(z.bytes==gold);if(lap>=0)csv(z,id,fuse,lap,pos);
        }
        std::cerr<<"verified "<<id<<'\n';
    }
}
