#pragma once
#include "../shift-constructor/induce.hpp"
#include <unordered_map>
namespace core {
using shift::Byte;
using shift::Counts;
using shift::InputRun;
using shift::Range;
using shift::Item;
using shift::N;
using shift::Hull;
struct Band {int lo,hi,origin,prefix;};
struct Prefix {int parent,byte;};
struct Stats{
    int accepted=0,first_hit=0,mass=0,prefix_types=0,tag_bands=0,tag_components=0,full_types=0,core_types=0,root_runs=0,core_runs=0;
    N forward_key_comparisons=0,forward_seeks=0,forward_scans=0,formed_bands=0,formed_prefix_nodes=0,sort_comparisons=0,formation_slots=0;
    N lf_pieces=0,lookup=0,scans=0,page_increments=0,gapped=0,holes=0,states=0,selector_entries=0;
    N terminal_pieces=0,split_core_types=0,extra_word_lookups=0,root_zeros=0,root_zero_pieces=0;
    N full_bytes=0,core_bytes=0,prefix_bytes=0,source_scatter=0,merged_inputs=0,branch_aliases=0,mark_steps=0,output_words=0;
};
struct Plan{
    Range u;int h;bool ok=true;
    std::vector<Band> target;
    std::vector<Range> forbidden_input;
    std::vector<std::string> prefixes;
    Plan(const std::vector<InputRun>& raw,const Counts& counts,Range source,int depth,Stats& s):u(source),h(depth){
        std::vector<Band> current={{u.lo,u.hi,u.lo,0}};
        std::vector<Prefix> trie={{-1,-1}};s.mass=u.hi-u.lo;
        if(h>0){
            struct Forward{int hi,delta,c;};std::vector<Forward> fwd(raw.size());
            auto cursors=counts.f,slots=counts.runs;int x=0;
            for(auto [c,w]:raw){fwd[slots[c]++]={cursors[c]+w,x-cursors[c],c};cursors[c]+=w;x+=w;}
            for(int j=1;j<=h;j++){
                std::vector<Band> next;std::pair<int,int> prior{-1,-1};int id=-1;
                bool overlap=false;
                for(auto old:current){
                    int x=old.lo;auto it=std::lower_bound(fwd.begin(),fwd.end(),x,[&](auto r,int y){
#ifdef CORE_SETUP_COUNTS
                        ++s.forward_key_comparisons;
#endif
                        return r.hi<=y;});
                    ++s.forward_seeks;
                    while(x<old.hi){
                        ++s.forward_scans;int stop=std::min(old.hi,it->hi);
                        int from=x+it->delta,end=stop+it->delta,owner=old.origin+x-old.lo;
                        auto name=std::pair{old.prefix,it->c};
                        if(name!=prior){
                            assert(prior<name);id=trie.size();trie.push_back({name.first,name.second});prior=name;
                        }
                        if(!next.empty() && next.back().prefix==id && next.back().hi==from &&
                           next.back().origin+next.back().hi-next.back().lo==owner)next.back().hi=end;
                        else next.push_back({from,end,owner,id});
                        overlap|=from<u.hi && end>u.lo;x=stop;++it;
                    }
                }
                s.formed_bands+=next.size();
                if(overlap && !s.first_hit)s.first_hit=j;
                s.formed_prefix_nodes=trie.size()-1;
                if(overlap && j<h){ok=false;return;}
                for(auto v:next)forbidden_input.push_back({v.lo,v.hi});
                current.swap(next);
            }
        }
        // IDs at depth h form a consecutive layer in lexical/source order.
        int first=current.front().prefix;
        prefixes.resize(current.back().prefix-first+1);
        for(int i=first;i<=current.back().prefix;i++){
            auto& text=prefixes[i-first];
            for(int k=i;k>0;k=trie[k].parent)text.push_back(char(trie[k].byte));
            std::reverse(text.begin(),text.end());s.prefix_bytes+=text.size();
        }
        for(auto& v:current)v.prefix-=first;
        target=std::move(current);s.formation_slots=forbidden_input.size();
        std::sort(target.begin(),target.end(),[&](auto l,auto r){++s.sort_comparisons;return l.lo<r.lo;});
        int end=-1;for(auto t:target){assert(end<=t.lo);s.tag_components+=end!=t.lo;end=t.hi;}
        s.accepted=1;s.prefix_types=prefixes.size();s.tag_bands=target.size();
    }
};
struct LFRecord{int end,delta,byte,tag;};
class Index{
    std::vector<LFRecord> pieces;
    std::vector<int> page;
public:
    Index(const std::vector<InputRun>& raw,const Counts& co,const Plan& p,Stats& s){
        auto f=co.f;std::array<int,256> cursors{};
        for(int c=0,k=0;c<256;c++){while(k<int(p.target.size()) && p.target[k].hi<=f[c])++k;cursors[c]=k;}
        pieces.reserve(raw.size()+2*p.target.size());int top=0;
        for(auto [c,width]:raw){
            int z=f[c],end=z+width,d=z-top;auto& k=cursors[c];
            while(z<end){
                while(k<int(p.target.size()) && p.target[k].hi<=z)++k;
                int stop=end,tag=-1;
                if(k<int(p.target.size())){
                    auto t=p.target[k];
                    if(z<t.lo)stop=std::min(stop,t.lo);else{stop=std::min(stop,t.hi);tag=k;}
                }
                pieces.push_back({stop-d,d,c,tag});z=stop;
            }
            top+=width;f[c]=end;
        }
        s.lf_pieces=pieces.size();assert(pieces.size()<=raw.size()+2*p.target.size());
        for(int x=0,i=0;x<co.n;x+=64){while(pieces[i].end<=x)++i;page.push_back(i);}
    }
    int find(int row,N& increments) const{int i=page[row/64],old=i;while(pieces[i].end<=row)++i;increments+=i-old;return i;}
    const LFRecord& operator[](int i)const{return pieces[i];}
};
struct Node{int parent=-1,byte=-1,begin=0,end=0;Hull hull;};
struct Word{int prefix,core,base=-1,width=0;};
struct Initial{int lo,hi,id;};
class Object{
    const Plan& plan;const Index& lf;Stats& st;
    std::vector<Node> nodes;
    std::vector<Range> forbidden;
    std::vector<Item> selector;
    std::vector<Initial> zero;
    void add(int id,int lo,int hi){
        auto& h=nodes[id].hull;
        if(h.extent.lo==h.extent.hi){h.extent={lo,hi};return;}
        assert(h.extent.hi<=lo);h.holes|=h.extent.hi<lo;h.extent.hi=hi;
    }
    template<class F> void ranges(Hull h,F use){
        auto [x,end]=h.extent;if(x==end)return;
        if(!h.holes){use(x,end);return;}
        ++st.gapped;
        if(forbidden.empty()){
            auto parts=plan.forbidden_input;
            std::sort(parts.begin(),parts.end(),[&](auto a,auto b){++st.sort_comparisons;return a.lo<b.lo;});
            for(auto v:parts){
                if(!forbidden.empty() && v.lo<=forbidden.back().hi)forbidden.back().hi=std::max(forbidden.back().hi,v.hi);
                else forbidden.push_back(v);
            }
        }
        auto gap=std::lower_bound(forbidden.begin(),forbidden.end(),x,[](auto g,int y){return g.hi<=y;});
        while(gap!=forbidden.end() && gap->lo<end){
            assert(x<gap->lo && gap->hi<end);use(x,gap->lo);++st.holes;x=gap->hi;++gap;
        }
        use(x,end);
    }
    static void append(std::vector<Item>& s,int id,int w){
        if(w==0)return;
        if(!s.empty() && s.back().id==id)s.back().width+=w;else s.push_back({id,w});
    }
    int new_word(int p,int c){
        int id=words.size();words.push_back({p,c});st.full_bytes+=plan.h+cores[c].size();return id;
    }
    void hit(int id,int owner,int width){
        auto& w=words[id];int start=owner-plan.u.lo;
        if(w.base<0)w.base=start;
        assert(w.base+w.width==start);w.width+=width;
    }
    void begin(){
        nodes.emplace_back();auto [a,b]=plan.u;
        if(plan.h==0){add(0,a,b);return;}
        int x=a;
        std::unordered_map<int,int> empty_words;
        for(auto t:plan.target){
            if(t.lo>=b)break;
            int l=std::max(a,t.lo),r=std::min(b,t.hi);if(l>=r)continue;
            if(x<l)add(0,x,l);
            auto [it,fresh]=empty_words.try_emplace(t.prefix,-1);
            if(fresh){
                if(cores.empty())cores.emplace_back();
                it->second=new_word(t.prefix,0);
            }
            hit(it->second,t.origin+l-t.lo,r-l);
            zero.push_back({l-a,r-a,it->second});st.root_zeros+=r-l;++st.root_zero_pieces;x=r;
        }
        if(x<b)add(0,x,b);
        if(empty_words.size()>1)++st.split_core_types;
    }
    void build(){
        std::array<int,256> stamp,child,terminal,core,first_prefix,first_word;
        stamp.fill(-1);terminal.fill(-1);
        for(int i=0;i<int(nodes.size());i++){
            std::unordered_map<N,int> additional;
            int begin=selector.size();
            ranges(nodes[i].hull,[&](int x,int end){
                ++st.lookup;int j=lf.find(x,st.page_increments);
                while(x<end){
                    const auto& r=lf[j++];++st.scans;
                    int stop=std::min(end,r.end),w=stop-x,y=x+r.delta,c=r.byte,id;
                    if(r.tag>=0){
                        ++st.terminal_pieces;const auto t=plan.target[r.tag];int p=t.prefix,word;
                        if(terminal[c]!=i){
                            terminal[c]=i;core[c]=cores.size();std::string text(1,char(c));
                            for(int k=i;k>0;k=nodes[k].parent)text.push_back(char(nodes[k].byte));
                            st.core_bytes+=text.size();cores.push_back(std::move(text));
                            first_prefix[c]=p;word=first_word[c]=new_word(p,core[c]);
                        }else if(first_prefix[c]==p)word=first_word[c];
                        else{
                            ++st.extra_word_lookups;
                            auto [it,fresh]=additional.try_emplace((N(p)<<8)|unsigned(c),-1);
                            if(fresh)it->second=new_word(p,core[c]);
                            word=it->second;
                        }
                        hit(word,t.origin+y-t.lo,w);id=~word;
                    }else{
                        if(stamp[c]!=i){stamp[c]=i;child[c]=nodes.size();Node n;n.parent=i;n.byte=c;nodes.push_back(n);}
                        id=child[c];add(id,y,y+w);
                    }
                    if(int(selector.size())>begin && selector.back().id==id)selector.back().width+=w;
                    else selector.push_back({id,w});
                    x=stop;
                }
            });
            if(!additional.empty()){
                std::array<bool,256> mixed{};
                for(auto kv:additional)mixed[kv.first&255]=true;
                st.split_core_types+=std::count(mixed.begin(),mixed.end(),true);
            }
            nodes[i].begin=begin;nodes[i].end=selector.size();
        }
    }
    void merge(){
        struct Stream{std::vector<Item> items;int pos=0;};std::vector<Stream> output(nodes.size());
        for(int i=int(nodes.size())-1;i>=0;i--){
            auto& out=output[i].items;auto a=nodes[i].begin,b=nodes[i].end;
            if(b==a+1 && selector[a].id>=0){out=std::move(output[selector[a].id].items);++st.branch_aliases;continue;}
            for(int j=a;j<b;j++){
                auto [id,w]=selector[j];
                if(id<0){append(out,~id,w);++st.merged_inputs;continue;}
                auto& c=output[id];
                while(w){
                    auto& e=c.items[c.pos];int k=std::min(w,e.width);
                    append(out,e.id,k);e.width-=k;w-=k;++st.merged_inputs;
                    if(!e.width)++c.pos;
                }
                if(c.pos==int(c.items.size())){std::vector<Item>().swap(c.items);c.pos=0;}
            }
        }
        auto& rest=output[0].items;
        if(zero.empty())incoming=std::move(rest);
        else{
            int x=0,k=0;
            auto missing=[&](int w){while(w){auto& p=rest[k];int z=std::min(w,p.width);append(incoming,p.id,z);p.width-=z;w-=z;if(!p.width)++k;}};
            for(auto early:zero){missing(early.lo-x);append(incoming,early.id,early.hi-early.lo);x=early.hi;}
            missing(st.mass-x);assert(k==int(rest.size()));
        }
        int previous=-1;
        for(auto e:incoming){int c=words[e.id].core;st.core_runs+=c!=previous;previous=c;}
    }
public:
    std::vector<Word> words;std::vector<std::string> cores;std::vector<Item> incoming;
    Object(const Plan& p,const Index& index,Stats& counts):plan(p),lf(index),st(counts){
        begin();build();merge();
        st.states=nodes.size();st.full_types=words.size();st.core_types=cores.size();st.root_runs=incoming.size();st.selector_entries=selector.size();
    }
    std::string text(const Word& w) const{return plan.prefixes[w.prefix]+cores[w.core];}
    std::pair<int,int> mark(int p){
        int x=p,skip=0;while(x<plan.u.lo || x>=plan.u.hi){N z=0;x+=lf[lf.find(x,z)].delta;++skip;++st.mark_steps;if(x==p)return {-1,0};}
        return {x-plan.u.lo,skip};
    }
    std::string expand(int p,int n){
        auto [x,skip]=mark(p);std::string out;out.reserve(n);
        if(x<0){
            std::string cycle;x=p;
            do{N inc=0;auto r=lf[lf.find(x,inc)];cycle.push_back(char(r.byte));x+=r.delta;}while(x!=p);
            std::reverse(cycle.begin(),cycle.end());while(int(out.size())<n)out.append(cycle,0,n-int(out.size()));return out;
        }
        std::vector<int> tag(st.mass),next(st.mass),cursor;
        for(auto& w:words)cursor.push_back(w.base);
        int dest=0;
        for(auto e:incoming)for(int j=0;j<e.width;j++){int s=cursor[e.id]++;tag[s]=e.id;next[s]=dest++;}
        assert(dest==st.mass);st.source_scatter+=dest;
        while(int(out.size())<n){
            auto w=words[tag[x]];
            int l=plan.h;assert(skip<l+int(cores[w.core].size()));
            if(skip<l)out.append(plan.prefixes[w.prefix],skip,std::min(l-skip,n-int(out.size())));
            int k=std::max(0,skip-l);out.append(cores[w.core],k,std::min<int>(cores[w.core].size()-k,n-int(out.size())));
            skip=0;x=next[x];++st.output_words;
        }
        return out;
    }
};
}
