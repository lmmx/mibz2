#pragma once
#include <array>
#include <cstdlib>
#include <limits>
#include <numeric>
#include "sweep.hpp"
namespace ranges {
struct Run {U b,w;};
struct Input {U n=0,mark=0;std::vector<Run> runs;std::vector<U> start;std::vector<uint8_t> modal;};
struct Context {U d=none,off=0,modal=0,gap=none,stop_begin=0,stops=0;};
struct Route {U to=none,off=0,dt=0;};
struct Birth {U target,off,label,w;};
struct Stop {U x,label,w;};
struct Packet {U off,root;};
struct Active {U old,d,dt,off1,w1,off2,w2;};
struct Band {U source,target,age;};
static_assert(sizeof(Band)==12);
struct Solver {
    Rope rope;
    std::vector<Context> ctx;
    std::vector<Route> routes;
    std::vector<U> active_old,path,degree,queue,anchor,cycle_dt;
    std::vector<Active> active;
    std::vector<Birth> birth;
    std::vector<Stop> stop;
    std::vector<Piece> raw;
    std::vector<Band> bands;
    std::vector<std::vector<Packet>> incoming;
    std::vector<std::vector<Ann>> holes,entry;
    U E=0;
    void layout(const Input& in) {
        U K=in.modal.size();ctx.assign(K,{});birth.clear();stop.clear();E=0;
        std::array<U,257> base{};std::array<U,256> at{},target{};
        for(auto run:in.runs)base[run.b+1]+=run.w;
        std::partial_sum(base.begin(),base.end(),base.begin());assert(base[256]==in.n);
        for(U b=0;b<256;++b)if(base[b]<base[b+1]){
            at[b]=base[b];target[b]=std::upper_bound(in.start.begin(),in.start.end(),base[b])-in.start.begin()-1;
        }
        U x=0,c=0;
        for(auto [b,remaining]:in.runs)while(remaining) {
            auto &z=ctx[c];U w=std::min(remaining,in.start[c+1]-x),y=at[b];
            U &j=target[b];while(in.start[j+1]<=y)++j;
            assert(y+w<=in.start[j+1]);U off=y-in.start[j];
            auto exceptional=[&](U left,U shift,U length) {
                if(!length)return;
                if(!z.stops)z.stop_begin=stop.size();
                if(z.stops && stop.back().x+stop.back().w==left)stop.back().w+=length;
                else{stop.push_back({left,E,length});++z.stops;}
                if(!birth.empty()&&birth.back().target==j&&birth.back().off+birth.back().w==shift
                   &&birth.back().label+birth.back().w==E)birth.back().w+=length;
                else birth.push_back({j,shift,E,length});
                E+=length;
            };
            if(b==in.modal[c]) {
                if(z.d==none){z.d=j;z.off=off;}
                assert(z.d==j&&z.off+z.modal==off);
                if(x<=in.mark&&in.mark<x+w){U t=in.mark-x;z.gap=z.modal+t;exceptional(in.mark,off+t,1);}
                z.modal+=w;
            }else exceptional(x,off,w);
            x+=w;at[b]+=w;remaining-=w;if(x==in.start[c+1])++c;
        }
        assert(c==K&&E);
    }
    void shrink(const Input& in) {
        U K=ctx.size();active_old.clear();routes.assign(K,{});
        for(U j=0;j<K;++j){
            assert(ctx[j].d!=none);
            if(ctx[j].stops){routes[j]={U(active_old.size()),0,0};active_old.push_back(j);}
        }
        constexpr U visiting=none-1,dead=none-2;
        for(U start=0;start<K;++start){
            if(routes[start].to!=none)continue;U j=start;path.clear();
            while(routes[j].to==none){
                routes[j].to=visiting;path.push_back(j);j=ctx[j].d;
            }
            if(routes[j].to==visiting||routes[j].to==dead) {
                for(U q:path)routes[q].to=dead;continue;
            }
            auto v=routes[j];
            for(auto it=path.rbegin();it!=path.rend();++it){
                U q=*it;assert(ctx[q].modal==in.start[q+1]-in.start[q]);
                v.off+=ctx[q].off;++v.dt;routes[q]=v;
            }
        }
        active.clear();active.reserve(active_old.size());
        for(U old:active_old) {
            auto z=ctx[old];auto dest=routes[z.d];assert(dest.to<active_old.size());
            U off=dest.off+z.off,w1=z.modal,off2=0,w2=0;
            if(z.gap!=none){w1=z.gap;off2=off+w1+1;w2=z.modal-w1-1;}
            active.push_back({old,dest.to,1+dest.dt,off,w1,off2,w2});
        }
    }
    void schedule(const Input& in) {
        U K=active.size();degree.assign(K,0);cycle_dt.assign(K,0);anchor.assign(K,none);
        const U walking=none-1,done=none-2;
        for(U start=0;start<K;++start){
            if(anchor[start]!=none)continue;
            U j=start;path.clear();
            while(anchor[j]==none){anchor[j]=walking;path.push_back(j);j=active[j].d;}
            if(anchor[j]==walking){
                U q=j,chosen=j,dt=0,wmin=none;
                do {U old=active[q].old,w=in.start[old+1]-in.start[old];dt+=active[q].dt;
                    if(w<wmin||(w==wmin&&q<chosen)){chosen=q;wmin=w;}
                    q=active[q].d;
                }while(q!=j);
                cycle_dt[chosen]=dt;
            }
            for(U k:path)anchor[k]=done;
        }
        for(U j=0;j<K;++j){U t=active[j].d;if(!cycle_dt[j]&&!cycle_dt[t])++degree[t];}
    }
    U collect(const Input& in,U j) {
        auto& v=incoming[j];std::sort(v.begin(),v.end(),[](auto l,auto r){return l.off<r.off;});
        U t=0;
        for(auto p:v){assert(p.off==rope.size(t));t=rope.join(t,p.root);}
        assert(rope.size(t)==in.start[active[j].old+1]-in.start[active[j].old]);
        return t;
    }
    void process(const Input& in,U j,U seq) {
        auto a=active[j];auto c=ctx[a.old];U keep=0,prev=in.start[a.old];
        for(U i=c.stop_begin;i<c.stop_begin+c.stops;++i){
            auto z=stop[i];auto [head,rem]=rope.split(seq,z.x-prev);keep=rope.join(keep,head);
            auto [ends,tail]=rope.split(rem,z.w);seq=tail;U target=z.label;
            auto terminate=[&](const Node& leaf) {
                if(leaf.kind<0)raw.push_back({U(leaf.first),target,leaf.width,leaf.age});
                else holes[U(leaf.kind)].push_back({U(leaf.first),target,leaf.width,leaf.age});
                target+=leaf.width;
            };
            rope.visit(ends,terminate);prev=z.x+z.w;
        }
        keep=rope.add(rope.join(keep,seq),a.dt);
        if(a.w1){auto [v,remain]=rope.split(keep,a.w1);incoming[a.d].push_back({a.off1,v});keep=remain;}
        if(a.w2){assert(rope.size(keep)==a.w2);incoming[a.d].push_back({a.off2,keep});keep=0;}
        assert(!keep);
    }
    template<class T> static void clear_vectors(std::vector<std::vector<T>>& v,U count){
        v.resize(count);for(auto& t:v)t.clear();
    }
    const std::vector<Band>& run(const Input& in) {
        layout(in);shrink(in);schedule(in);
        U K=active.size();clear_vectors(incoming,K);clear_vectors(holes,K);clear_vectors(entry,K);
        rope.reset(birth.size()+stop.size()+K+1);raw.clear();raw.reserve(birth.size()+stop.size());
        for(auto v:birth){
            auto d=routes[v.target];assert(d.to<K);
            incoming[d.to].push_back({d.off+v.off,rope.node(-1,v.label,v.w,d.dt+1)});
        }
        for(U j=0;j<K;++j)if(cycle_dt[j]) {
            U old=active[j].old;process(in,j,rope.node(j,0,in.start[old+1]-in.start[old],0));
        }
        queue.clear();for(U j=0;j<K;++j)if(!cycle_dt[j]&&!degree[j])queue.push_back(j);
        for(U i=0;i<queue.size();++i){
            U j=queue[i];process(in,j,collect(in,j));U t=active[j].d;
            if(!cycle_dt[t]&&!--degree[t])queue.push_back(t);
        }
        for(U a=0;a<K;++a)if(cycle_dt[a]){
            U pos=0,root=collect(in,a);
            auto arrived=[&](const Node& leaf){
                if(leaf.kind<0)entry[a].push_back({pos,U(leaf.first),leaf.width,leaf.age});
                else{assert(U(leaf.kind)==a&&leaf.age==cycle_dt[a]);}
                pos+=leaf.width;
            };
            rope.visit(root,arrived);hole_sweep(cycle_dt[a],holes[a],entry[a],rope,raw);
        }
        std::sort(raw.begin(),raw.end(),[](Piece l,Piece r){return l.source<r.source;});
        bands.clear();bands.reserve(raw.size());U n=0;
        for(auto z:raw){
            assert(z.source==n&&z.age>0);n+=z.w;
            if(!bands.empty() && bands.back().age==z.age &&
               bands.back().target+z.source-bands.back().source==z.target)continue;
            bands.push_back({z.source,z.target,z.age});
        }
        assert(n==E);return bands;
    }
};
} // namespace ranges
