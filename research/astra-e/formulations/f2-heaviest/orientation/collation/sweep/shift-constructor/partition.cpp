// Independent dense geometry verifier. Production induction lives in induce.hpp.
#define SHIFT_PANEL_LIBRARY
#include "main.cpp"

#include <unordered_map>

struct Interner {
    std::unordered_map<N,int> ids;
    int next(int parent,int byte){
        auto [it,fresh]=ids.try_emplace((N(parent)<<8)|unsigned(byte),int(ids.size())+1);
        (void)fresh;return it->second;
    }
};
struct Dense {
    std::vector<int> f,b;std::vector<Byte> out,prev;
    explicit Dense(const Oracle& o):f(o.psi),b(f.size()),out(o.label.begin(),o.label.end()),prev(f.size()){
        for(int x=0;x<int(f.size());x++){b[f[x]]=x;prev[f[x]]=out[x];}
    }
    std::vector<int> classes(const std::vector<int>& section,const std::string& common,Interner& table) const{
        int h=common.size();std::vector<int> id(f.size(),-1);
        std::vector<bool> selected(f.size());for(int x:section)selected[x]=true;
        for(int dest:section){
            int row=dest,depth=0,node=0;
            do {
                if(depth>=h)id[row]=node;
                else {assert(prev[row]==Byte(common[h-1-depth]));id[row]=-2;}
                if(depth>=h)node=table.next(node,prev[row]);
                ++depth;row=b[row];
            }while(!selected[row]);
        }
        return id;
    }
};
struct Observed {int H=0,packets=0,raw=0,clipped=0;};
Observed count(const Dense& d,const std::vector<int>& classes,const std::vector<bool>& terminal,int interned){
    std::vector<bool> seen(interned+1);Observed v;
    for(int i=0;i<int(classes.size());i++){
        int k=classes[i];if(k<0)continue;
        if(!seen[k]){++v.H;seen[k]=true;}
        bool new_packet=i==0 || classes[i-1]!=k;
        bool new_run=new_packet || d.prev[i-1]!=d.prev[i]; // new_packet is true at i=0
        v.packets+=new_packet;v.raw+=new_run;
        v.clipped+=new_run || terminal[d.b[i-1]]!=terminal[d.b[i]];
    }
    return v;
}
int components(const std::vector<bool>& mask){
    int n=0;for(int i=0;i<int(mask.size());i++)n+=mask[i] && (i==0 || !mask[i-1]);return n;
}
struct Totals{N cells=0,ranks=0,marks=0,nontrivial_early=0,partial=0,sharp_excess=0,submaximal=0;};
void verify(const std::vector<InputRun>& runs,Range initial,int mark,int ordinal,int depth,bool all_marks,Totals& totals){
    Oracle oracle(runs);Dense d(oracle);int n=d.f.size();Counts counts(runs);
    std::vector<int> original(initial.hi-initial.lo),section;
    std::iota(original.begin(),original.end(),initial.lo);section=original;
    std::vector<bool> forbid(n),base(n);for(int x:original)base[x]=true;
    Interner dict;auto old=d.classes(original,"",dict);
    auto zero=count(d,old,base,dict.ids.size());assert(zero.packets==zero.H);
    std::string prefix;int partial_bound=0,old_q=zero.H;
    for(int h=0;h<=depth;h++){
        if(h){
            int symbol=d.out[section[0]];
            if(std::any_of(section.begin(),section.end(),[&](int x){return d.out[x]!=symbol;}))break;
            prefix.push_back(char(symbol));
            for(int& x:section){x=d.f[x];forbid[x]=true;}
            assert(std::is_sorted(section.begin(),section.end()));
        }
        std::vector<bool> terminal(n);std::vector<int> coordinate(n,-1);
        for(int i=0;i<int(section.size());i++){terminal[section[i]]=true;coordinate[section[i]]=i;}
        auto shifted=d.classes(section,prefix,dict);
        auto view=count(d,shifted,terminal,dict.ids.size());
        auto native=execute(runs,initial,h,true,mark);
        Stats z;Plan plan(runs,counts,initial,h,z);Index ix(runs,counts,plan.levels.back(),z);Object obj(plan,ix,z,true);
        for(int p=all_marks?0:mark;p<(all_marks?n:mark+1);p++){
            int x=p,j=0;
            while(coordinate[x]<0){x=d.b[x];++j;if(x==p)break;}
            std::pair<int,int> truth=coordinate[x]<0?std::pair{-1,0}:std::pair{coordinate[x],j};
            assert(obj.transfer(p)==truth);++totals.marks;
        }
        std::vector<int> erased(dict.ids.size()+1),kept(dict.ids.size()+1);
        for(int x=0;x<n;x++)if(old[x]>=0){
            if(forbid[x]){assert(shifted[x]<0);erased[old[x]]++;}
            else {assert(shifted[x]==old[x]);kept[old[x]]++;}
        }else assert(shifted[x]<0);
        int partial=0;
        for(int k=0;k<int(erased.size());k++)partial+=erased[k]>0 && kept[k]>0;
        int f=components(forbid),s=components(terminal);
        assert(view.packets-view.H<=f);
        int bound=zero.raw-(zero.H-view.H)+f+2*s;
        assert(view.clipped<=bound);
        assert(view.packets==int(native.stats.live_packets));
        assert(view.H==int(native.stats.nodes) || (view.H==0 && native.stats.nodes==1));
        assert(view.clipped==int(native.stats.translated_pieces));
        // Distinct full dictionary is unchanged. A single raw shifted trie counts Q.
        int q=execute(runs,initial,h,false,mark).stats.nodes;
        if(h)partial_bound+=native.stats.words-1-(old_q-q);
        old_q=q;assert(partial<=partial_bound);
        ++totals.cells;totals.ranks+=n;totals.partial+=partial;
        totals.sharp_excess+=f>0 && view.packets-view.H==f;
        totals.nontrivial_early+=h>=2 && native.stats.fused_early_rows>0 && native.stats.fused_early_rows<unsigned(section.size());
        totals.submaximal+=partial_bound>0;
        if(ordinal>=0){
            auto [row,skip]=obj.transfer(mark);
            std::cout<<ordinal<<','<<h<<','<<s<<','<<f<<','<<zero.H<<','<<q<<','<<view.H
                     <<','<<view.packets-view.H<<','<<partial<<','<<partial_bound<<','<<zero.raw<<','<<view.raw
                     <<','<<view.clipped<<','<<bound<<','<<row<<','<<skip<<'\n';
        }
    }
}
int main(int argc,char** argv){
    Totals all;
    if(argc==2){
        std::ifstream in(argv[1]);int i,n,p,a,b,r;
        std::cout<<"ordinal,h,terminal_components,forbidden_components,old_nodes,raw_shifted_nodes,fused_surviving_nodes,extra_packets,partially_cut_old_nodes,sum_one_shift_deficits,old_raw_LF_pieces,shifted_raw_LF_pieces,shifted_colored_LF_pieces,predicted_upper,marked_row,marked_skip\n";
        while(in>>i>>n>>p>>a>>b>>r){
            std::vector<InputRun> runs(r);for(auto& v:runs)in>>v.byte>>v.width;
            verify(runs,{a,b},p,i,4,false,all);
        }
    }else{
        for(int n=1;n<=8;n++)for(int mask=0;mask<(1<<n);mask++){
            std::vector<InputRun> runs;
            for(int j=0;j<n;j++){int b=(mask>>j)&1;
                if(runs.empty() || runs.back().byte!=b)runs.push_back({b,1});else ++runs.back().width;}
            for(int a=0;a<n;a++)for(int b=a+1;b<=n;b++)verify(runs,{a,b},0,-1,4,true,all);
        }
        std::cout<<"partition_cells "<<all.cells<<" checked_ranks "<<all.ranks<<" transferred_marks "<<all.marks
                 <<" partial_old_nodes "<<all.partial<<" submaximal_cells "<<all.submaximal<<" nontrivial_early_fusions "<<all.nontrivial_early
                 <<" saturated_nonzero_forbidden_component_bounds "<<all.sharp_excess<<'\n';
    }
}
