// LCP of finite substrings in an ordinary byte tape, with a distinct end marker.
#pragma once
#include <algorithm>
#include <bit>
#include <cassert>
#include <cstdint>
#include <numeric>
#include <vector>

class ExactSubstrings {
    static constexpr int block=32;
    std::vector<int> rank,lcp;
    std::vector<uint32_t> suffix_minima;
    std::vector<std::vector<int>> levels;

    int in_block(int lo,int hi) const {
        assert(lo<=hi && lo/block==hi/block);
        uint32_t candidates=suffix_minima[hi] & (~uint32_t{0}<<(lo%block));
        return lcp[hi/block*block+std::countr_zero(candidates)];
    }
    int minimum(int lo,int hi) const {
        int a=lo/block,b=hi/block;
        if(a==b)return in_block(lo,hi);
        int answer=std::min(in_block(lo,(a+1)*block-1),in_block(b*block,hi));
        if(b>a+1){
            int width=b-a-1,k=std::bit_width(unsigned(width))-1;
            answer=std::min({answer,levels[k][a+1],levels[k][b-(1<<k)]});
        }
        return answer;
    }
public:
    explicit ExactSubstrings(const std::vector<uint8_t>& tape) {
        int n=int(tape.size())+1;
        std::vector<int> sa(n),order(n),tmp(n),count(std::max(257,n));
        rank.resize(n);
        for(int i=0;i<n-1;i++)rank[i]=1+tape[i];
        for(int c:rank)++count[c];
        std::partial_sum(count.begin(),count.begin()+257,count.begin());
        for(int i=n-1;i>=0;--i)sa[--count[rank[i]]]=i;
        int classes=1;
        for(int i=1;i<n;i++) {
            if(rank[sa[i-1]]!=rank[sa[i]])++classes;
            tmp[sa[i]]=classes-1;
        }
        rank.swap(tmp);
        for(int half=1;classes<n;half*=2){
            std::fill_n(count.begin(),classes,0);
            for(int i=0;i<n;i++){
                order[i]=sa[i]-half;if(order[i]<0)order[i]+=n;
                ++count[rank[order[i]]];
            }
            std::partial_sum(count.begin(),count.begin()+classes,count.begin());
            for(int i=n-1;i>=0;--i)sa[--count[rank[order[i]]]]=order[i];
            classes=1;tmp[sa[0]]=0;
            for(int i=1;i<n;i++){
                int a=sa[i-1],b=sa[i],ca=a+half,cb=b+half;
                if(ca>=n)ca-=n;
                if(cb>=n)cb-=n;
                if(rank[a]!=rank[b] || rank[ca]!=rank[cb])++classes;
                tmp[b]=classes-1;
            }
            rank.swap(tmp);
        }
        lcp.assign(n,0);
        int common=0;
        for(int i=0;i<n-1;i++){
            int j=sa[rank[i]-1];
            while(i+common<n-1 && j+common<n-1 && tape[i+common]==tape[j+common])++common;
            lcp[rank[i]]=common;
            if(common)--common;
        }
        suffix_minima.resize(n);
        std::vector<int> base((n+block-1)/block,n);
        uint32_t mask=0;
        for(int i=0;i<n;i++){
            int offset=i%block;
            if(!offset)mask=0;
            while(mask){
                int last=31-std::countl_zero(mask);
                if(lcp[i-offset+last]<lcp[i])break;
                mask^=uint32_t{1}<<last;
            }
            mask|=uint32_t{1}<<offset;suffix_minima[i]=mask;
            base[i/block]=std::min(base[i/block],lcp[i]);
        }
        levels.push_back(std::move(base));
        for(int span=2;span<=int(levels[0].size());span*=2){
            const auto& prev=levels.back();
            std::vector<int> row(levels[0].size()-span+1);
            for(int i=0;i<int(row.size());i++)row[i]=std::min(prev[i],prev[i+span/2]);
            levels.push_back(std::move(row));
        }
    }
    int common(int a,int b,int cap) const {
        if(a==b)return cap;
        int x=rank[a],y=rank[b];if(x>y)std::swap(x,y);
        return std::min(cap,minimum(x+1,y));
    }
    bool ordered(int a,int b) const {return rank[a]<rank[b];}
};
