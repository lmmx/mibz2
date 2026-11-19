#pragma once
#include <algorithm>
#include <array>
#include <cassert>
#include <chrono>
#include <cstdint>
#include <limits>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <vector>

namespace shift {
using Byte=unsigned char;
using N=uint64_t;
struct InputRun {int byte,width;};
struct Range {int lo,hi;};
struct Segment {int lo,hi,base;};
struct Piece {int end,delta,byte,bias;};
constexpr int outside=std::numeric_limits<int>::min();
struct Item {int id,width; bool operator==(const Item&) const=default;};
struct Stats {
    int h=0,mass=0,selected_components=0,lf_pieces=0,forward_pieces=0,nodes=0,words=0,root_runs=0;
    N image_probes=0,image_cuts=0,setup_boundary_tests=0,setup_boundary_advances=0;
    N fused_root_tests=0,fused_domain_pieces=0,fused_early_rows=0,fused_early_bands=0;
    N live_packets=0,live_rows=0,lf_seeks=0,page_increments=0,translated_pieces=0;
    N nonterminal_bands=0,terminal_bands=0,coalesced_packets=0,selectors=0,unary_moves=0;
    N copied_merge_entries=0,stream_rows=0,dictionary_bytes=0,mark_probes=0,output_words=0;
    N global_ranges=0,gapped_hulls=0,skipped_holes=0,skipped_rows=0;
};
struct Section {
    std::vector<Segment> list;
    int mass=0;
    void add(int lo,int hi){
        if(lo>=hi)return;
        if(!list.empty() && list.back().hi==lo)list.back().hi=hi;
        else {assert(list.empty() || list.back().hi<lo);list.push_back({lo,hi,mass});}
        mass+=hi-lo;
    }
    int at(int row) const {
        auto it=std::lower_bound(list.begin(),list.end(),row,[](auto v,int q){return v.hi<=q;});
        return it!=list.end() && it->lo<=row? it->base+row-it->lo:-1;
    }
};
struct Counts {
    std::array<int,256> f{},runs{};
    int n=0;
    explicit Counts(const std::vector<InputRun>& input){
        for(auto [c,w]:input){f[c]+=w;++runs[c];n+=w;}
        int mass=0,total=0;
        for(int c=0;c<256;++c){int a=f[c],b=runs[c];f[c]=mass;runs[c]=total;mass+=a;total+=b;}
    }
};
struct Plan {
    std::vector<Section> levels;
    std::string prefix;
    bool available=true;
    Plan(const std::vector<InputRun>& input,const Counts& counts,Range initial,int depth,Stats& st){
        Section root;root.add(initial.lo,initial.hi);levels.push_back(std::move(root));
        if(depth==0)return;
        std::vector<Piece> forward(input.size());
        auto f=counts.f,slot=counts.runs;int x=0;
        for(auto [c,width]:input){
            forward[slot[c]++]={f[c]+width,x-f[c],c,0};f[c]+=width;x+=width;
        }
        st.forward_pieces=forward.size();
        for(int h=0;h<depth;h++){
            Section next;int mono=-1;
            for(auto s:levels.back().list){
                int y=s.lo;
                auto it=std::lower_bound(forward.begin(),forward.end(),y,[](auto r,int q){return r.end<=q;});
                ++st.image_probes;
                while(y<s.hi){
                    int hi=std::min(s.hi,it->end);++st.image_cuts;
                    if(mono<0)mono=it->byte;
                    if(mono!=it->byte){available=false;return;}
                    next.add(y+it->delta,hi+it->delta);y=hi;++it;
                }
            }
            assert(next.mass==levels[0].mass);levels.push_back(std::move(next));prefix.push_back(char(mono));
        }
    }
};
class Index {
    std::vector<Piece> pieces;
    std::vector<int> page;
public:
    Index(const std::vector<InputRun>& input,const Counts& co,const Section& sec,Stats& st){
        auto f=co.f;
        std::array<int,256> section_cursor{};
        int block=0;
        for(int c=0;c<256;c++){
            while(block<int(sec.list.size()) && sec.list[block].hi<=f[c])++block;
            section_cursor[c]=block;
        }
        pieces.reserve(input.size()+2*sec.list.size());
        int top=0;
        for(auto [c,width]:input){
            int z=f[c],end=z+width,d=z-top;
            auto& k=section_cursor[c];
            while(z<end){
                ++st.setup_boundary_tests;
                while(k<int(sec.list.size()) && sec.list[k].hi<=z){++k;++st.setup_boundary_advances;}
                int stop=end,bias=outside;
                if(k<int(sec.list.size())){
                    auto part=sec.list[k];
                    if(z<part.lo)stop=std::min(end,part.lo);
                    else {stop=std::min(end,part.hi);bias=part.base-part.lo;}
                }
                pieces.push_back({stop-d,d,c,bias});z=stop;
            }
            f[c]=end;top+=width;
        }
        assert(pieces.size()<=input.size()+2*sec.list.size());
        st.lf_pieces=pieces.size();
        for(int x=0,j=0;x<co.n;x+=64){while(pieces[j].end<=x)++j;page.push_back(j);}
    }
    int locate(int row,N& increments) const{
        int j=page[row/64],old=j;while(pieces[j].end<=row)++j;increments+=j-old;return j;
    }
    const Piece& operator[](int j) const{return pieces[j];}
};

struct Word {int base=-1,width=0;std::string text;};
struct Hull { Range extent{0,0}; bool holes=false; };
template<bool UseHull> using Storage=std::conditional_t<UseHull,Hull,std::vector<Range>>;
template<bool UseHull> struct Node {
    int parent=-1,edge=-1,sel_begin=0,sel_end=0;
    Storage<UseHull> live;
};
struct Early {int lo,hi,id;};
template<bool UseHull> class ObjectImpl {
    std::vector<Node<UseHull>> nodes;
    std::vector<Range> forbidden;
    std::vector<Item> selector;
    std::vector<Early> ended;
    const Section& target;
    const Index& index;
    const Plan& plan;
    bool fused;
    int root_length;
    void append_item(std::vector<Item>& out,int id,int width){
        if(width<=0)return;
        if(!out.empty() && out.back().id==id)out.back().width+=width;
        else out.push_back({id,width});
    }
    void append_range(Storage<UseHull>& list,int lo,int hi){
        if constexpr(UseHull){
            if(list.extent.lo==list.extent.hi){list.extent={lo,hi};return;}
            assert(list.extent.hi<=lo);
            if(list.extent.hi==lo)++st.coalesced_packets;
            else list.holes=true;
            list.extent.hi=hi;
        }else{
            if(!list.empty() && list.back().hi==lo){list.back().hi=hi;++st.coalesced_packets;}
            else {assert(list.empty() || list.back().hi<lo);list.push_back({lo,hi});}
        }
    }
    template<class F> void live_ranges(const Storage<UseHull>& live,F fn){
        if constexpr(UseHull){
            auto [x,end]=live.extent;if(x==end)return;
            if(!live.holes){fn(x,end);return;}
            ++st.gapped_hulls;
            if(forbidden.empty()){
                std::vector<Range> all;
                for(size_t i=1;i<plan.levels.size();i++)for(auto s:plan.levels[i].list)all.push_back({s.lo,s.hi});
                std::sort(all.begin(),all.end(),[](Range a,Range b){return a.lo<b.lo;});
                for(auto p:all){
                    if(!forbidden.empty() && p.lo<=forbidden.back().hi)forbidden.back().hi=std::max(p.hi,forbidden.back().hi);
                    else forbidden.push_back(p);
                }
                st.global_ranges=forbidden.size();
            }
            auto gap=std::lower_bound(forbidden.begin(),forbidden.end(),x,[](Range r,int q){return r.hi<=q;});
            while(gap!=forbidden.end() && gap->lo<end){
                assert(x<gap->lo && gap->hi<end);fn(x,gap->lo);
                ++st.skipped_holes;st.skipped_rows+=gap->hi-gap->lo;x=gap->hi;++gap;
            }
            fn(x,end);
        }else for(auto p:live)fn(p.lo,p.hi);
    }
    int make_word(const std::string& text){int id=words.size();words.push_back({-1,0,text});st.dictionary_bytes+=text.size();return id;}
    void hit(int id,int base,int width){
        auto& w=words[id];if(w.base<0)w.base=base;assert(base==w.base+w.width);w.width+=width;
    }
    void root(){
        nodes.push_back({});
        if(!fused || root_length==0){
            for(auto p:target.list)append_range(nodes[0].live,p.lo,p.hi);
            return;
        }
        // Domain is the common sorted-section ordinal, not physical rank.
        std::vector<Range> alive={{0,target.mass}};
        for(int j=1;j<=root_length;j++){
            int h=root_length-j;
            const auto& L=plan.levels[h].list;
            std::vector<Range> next;int k=0,v=0,word=-1;
            for(auto [lo,hi]:alive){
                while(lo<hi){
                    while(L[k].base+L[k].hi-L[k].lo<=lo)++k;
                    auto seg=L[k];int row=seg.lo+lo-seg.base;
                    int stop=std::min(hi,seg.base+seg.hi-seg.lo);
                    while(v<int(target.list.size()) && target.list[v].hi<=row)++v;
                    ++st.fused_root_tests;
                    if(v<int(target.list.size())){
                        auto q=target.list[v];
                        if(row<q.lo)stop=std::min(stop,lo+q.lo-row);
                        else {
                            stop=std::min(stop,lo+q.hi-row);
                            if(word<0)word=make_word(plan.prefix.substr(h));
                            hit(word,q.base+row-q.lo,stop-lo);
                            ended.push_back({lo,stop,word});
                            ++st.fused_early_bands;st.fused_early_rows+=stop-lo;
                            lo=stop;continue;
                        }
                    }
                    if(!next.empty() && next.back().hi==lo)next.back().hi=stop;
                    else next.push_back({lo,stop});
                    lo=stop;
                }
            }
            st.fused_domain_pieces+=next.size();alive.swap(next);
        }
        auto a=plan.levels[0].list.front().lo;
        for(auto p:alive)append_range(nodes[0].live,a+p.lo,a+p.hi);
        std::sort(ended.begin(),ended.end(),[](auto p,auto q){return p.lo<q.lo;});
    }
    void discover(){
        std::array<int,256> alive_stamp,hit_stamp,child,endpoint;
        alive_stamp.fill(-1);hit_stamp.fill(-1);
        for(int i=0;i<int(nodes.size());i++){
            auto live=std::move(nodes[i].live);
            int begin=selector.size();
            live_ranges(live,[&](int x,int end){
                st.live_rows+=end-x;++st.live_packets;++st.lf_seeks;
                int j=index.locate(x,st.page_increments);
                while(x<end){
                    const auto& p=index[j++];++st.translated_pieces;
                    int stop=std::min(end,p.end),width=stop-x,y=x+p.delta,c=p.byte,id;
                    if(p.bias!=outside){
                        ++st.terminal_bands;
                        if(hit_stamp[c]!=i){
                            std::string text(1,char(c));
                            for(int q=i;q>0;q=nodes[q].parent)text.push_back(char(nodes[q].edge));
                            if(fused)text+=plan.prefix;
                            endpoint[c]=make_word(text);hit_stamp[c]=i;
                        }
                        hit(endpoint[c],y+p.bias,width);id=~endpoint[c];
                    }else{
                        ++st.nonterminal_bands;
                        if(alive_stamp[c]!=i){
                            child[c]=nodes.size();alive_stamp[c]=i;Node<UseHull> n;n.parent=i;n.edge=c;nodes.push_back(std::move(n));
                        }
                        id=child[c];append_range(nodes[id].live,y,y+width);
                    }
                    if(int(selector.size())>begin && selector.back().id==id)selector.back().width+=width;
                    else selector.push_back({id,width});
                    x=stop;
                }
            });
            nodes[i].sel_begin=begin;nodes[i].sel_end=selector.size();
        }
        st.selectors=selector.size();st.nodes=nodes.size();st.words=words.size();
    }
    void merge(){
        struct Stream{std::vector<Item> data;int pos=0;};
        std::vector<Stream> streams(nodes.size());
        for(int i=int(nodes.size())-1;i>=0;i--){
            auto& dest=streams[i].data;
            auto [begin,end]=std::pair{nodes[i].sel_begin,nodes[i].sel_end};
            if(end==begin+1 && selector[begin].id>=0){
                dest=std::move(streams[selector[begin].id].data);++st.unary_moves;continue;
            }
            for(int j=begin;j<end;j++){
                auto [id,width]=selector[j];st.stream_rows+=width;
                if(id<0){append_item(dest,~id,width);++st.copied_merge_entries;continue;}
                auto& from=streams[id];
                while(width){
                    assert(from.pos<int(from.data.size()));
                    auto& p=from.data[from.pos];int take=std::min(p.width,width);
                    append_item(dest,p.id,take);p.width-=take;width-=take;++st.copied_merge_entries;
                    if(!p.width)++from.pos;
                }
                if(from.pos==int(from.data.size())){std::vector<Item>().swap(from.data);from.pos=0;}
            }
        }
        auto& remaining=streams[0].data;
        if(ended.empty()){incoming=std::move(remaining);st.root_runs=incoming.size();return;}
        int src=0,cursor=0;
        auto gap=[&](int width){
            while(width){auto& p=remaining[src];int take=std::min(width,p.width);append_item(incoming,p.id,take);
                width-=take;p.width-=take;if(!p.width)++src;}
        };
        for(auto e:ended){gap(e.lo-cursor);append_item(incoming,e.id,e.hi-e.lo);cursor=e.hi;}
        gap(target.mass-cursor);assert(src==int(remaining.size()));st.root_runs=incoming.size();
    }
public:
    Stats& st;std::vector<Word> words;std::vector<Item> incoming;
    ObjectImpl(const Plan& p,const Index& ix,Stats& counts,bool fuse)
       :target(p.levels.back()),index(ix),plan(p),fused(fuse),root_length(p.prefix.size()),st(counts){
        assert(!UseHull || fuse || root_length==0);
        st.h=root_length;st.mass=target.mass;st.selected_components=target.list.size();
        root();discover();merge();
    }
    std::pair<int,int> transfer(int p){
        int x=p,skip=0;
        for(;;){int ordinal=target.at(x);if(ordinal>=0)return {ordinal,skip};
            N inc=0;const auto& r=index[index.locate(x,inc)];x+=r.delta;++st.mark_probes;++skip;
            if(x==p)return {-1,0};
        }
    }
    std::string expand(int p,int n){
        auto [s,skip]=transfer(p);
        if(s<0){
            std::string cycle;int x=p;
            do {N inc=0;const auto& r=index[index.locate(x,inc)];cycle.push_back(char(r.byte));x+=r.delta;}while(x!=p);
            std::reverse(cycle.begin(),cycle.end());std::string out;
            while(int(out.size())<n)out.append(cycle,0,std::min<int>(n-out.size(),cycle.size()));
            return out;
        }
        std::vector<int> cursor,by_source(target.mass),next(target.mass);
        for(const auto& w:words)cursor.push_back(w.base);
        int dest=0;
        for(auto r:incoming)for(int k=0;k<r.width;k++){
            int y=cursor[r.id]++;by_source[y]=r.id;next[y]=dest++;
        }
        assert(dest==target.mass);
        std::string out;out.reserve(n);
        while(int(out.size())<n){
            const auto& w=words[by_source[s]].text;assert(skip<int(w.size()));
            int k=std::min<int>(w.size()-skip,n-int(out.size()));out.append(w,skip,k);
            skip=0;s=next[s];++st.output_words;
        }
        return out;
    }
};
using Object=ObjectImpl<false>;
}
