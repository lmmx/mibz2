// Construct first-return words from the forward inverse of LF, using only L's runs.
#include <algorithm>
#include <array>
#include <cassert>
#include <chrono>
#include <cstdint>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>
#include <tuple>
#include <vector>

using Clock = std::chrono::steady_clock;
using u64 = uint64_t;
struct InputRun { int byte, width; };
struct Interval { int lo, hi; };
struct Select { int hi, displacement; uint8_t byte; };
struct Node { int parent; uint8_t byte; };
struct Packet { int source, current, width; };
struct Group { int node, begin, end; };
struct Band { int source, dest, width, word, color; };
struct Stats {
    u64 groups=0, packets=0, translations=0, classified=0, live_outputs=0, raw_hits=0;
    u64 live_merges=0, full_seeks=0, comparisons=0, retained=0, next=0, sort_comparisons=0;
    u64 peak_frontier=0, peak_frontier_capacity=0, depth=0, source_rows=0, covered=0;
    u64 word_bytes=0, words=0, mark_seeks=0, output_lookups=0, mark_prefix=0;
};

template<class T> u64 cap(const std::vector<T>& v) { return sizeof(T)*v.capacity(); }

class Forward {
    int n;
    std::vector<Interval> section;
    std::vector<Select> select;
    std::vector<Node> nodes{{-1,0}};
    std::vector<Band> bands;
    std::vector<int> spelling_offset, spelling_len;
    std::string dictionary;

    // Search the translated coordinate in the F-ordered stable-select runs.
    int locate(int x, int prev, bool account=true) {
        if (prev>=0 && x<select[prev].hi) { if(account) ++s.retained; return prev; }
        if (prev>=0 && prev+1<int(select.size()) && x<select[prev+1].hi) {
            if(account) ++s.next;
            return prev+1;
        }
        int lo=prev>=0 ? prev+2 : 0, hi=select.size();
        if(account) ++s.full_seeks;
        while (lo<hi) {
            int mid=lo+(hi-lo)/2;
            if(account) ++s.comparisons;
            if (x<select[mid].hi) hi=mid; else lo=mid+1;
        }
        assert(lo<int(select.size()));
        return lo;
    }

    int next_component(int x) const {
        if (section.size()==1) return x<section.front().hi ? 0 : 1;
        auto it=std::lower_bound(section.begin(),section.end(),x,
            [](const Interval& a,int row) { return a.hi<=row; });
        return it-section.begin();
    }

public:
    Stats s;
    double setup_ms=0, discover_ms=0, order_ms=0, dict_ms=0, emit_ms=0;
    Forward(int n_,const std::vector<InputRun>& runs, std::vector<Interval> components)
        : n(n_), section(std::move(components)) {
        auto t=Clock::now();
        std::array<int,256> counts{}, positions{}, slots{}, run_counts{};
        for(auto r:runs) { counts[r.byte]+=r.width; ++run_counts[r.byte]; }
        for(int c=1;c<256;++c) {
            positions[c]=positions[c-1]+counts[c-1];
            slots[c]=slots[c-1]+run_counts[c-1];
        }
        select.resize(runs.size());
        int row=0;
        for(auto r:runs) {
            int f=positions[r.byte];
            select[slots[r.byte]++]={f+r.width,row-f,static_cast<uint8_t>(r.byte)};
            positions[r.byte]+=r.width; row+=r.width;
        }
        assert(row==n);
        setup_ms=std::chrono::duration<double,std::milli>(Clock::now()-t).count();
    }

    void discover() {
        auto t=Clock::now();
        std::vector<Packet> live, children;
        std::vector<Group> groups, next_groups;
        for(auto v:section) {
            live.push_back({v.lo,v.lo,v.hi-v.lo});
            s.source_rows+=v.hi-v.lo;
        }
        if (!live.empty()) groups.push_back({0,0,int(live.size())});
        while(!groups.empty()) {
            ++s.depth;
            s.peak_frontier=std::max<u64>(s.peak_frontier,live.size());
            next_groups.clear(); children.clear();
            for (auto g:groups) {
                ++s.groups;
                int prev_run=-1,last_byte=-1, node=-1;
                int prev_end=-1;
                for (int pi=g.begin;pi<g.end;++pi) {
                    ++s.packets;
                    auto p=live[pi];
                    assert(prev_end<=p.current);
                    prev_end=p.current+p.width;
                    int run=locate(p.current,prev_run);
                    for (int x=p.current;x<prev_end;) {
                        const auto &r=select[run];
                        if(r.byte!=last_byte) {
                            assert(r.byte>last_byte);
                            last_byte=r.byte;
                            node=nodes.size();
                            nodes.push_back({g.node,r.byte});
                        }
                        ++s.translations;
                        int stop=std::min(prev_end,r.hi);
                        int src=p.source+x-p.current;
                        int y=x+r.displacement;
                        int end=stop+r.displacement;
                        // The image of one select piece is affine. Cut at every U edge.
                        int c=next_component(y);
                        while(y<end) {
                            bool hit=c<int(section.size()) && section[c].lo<=y;
                            int lim=c==int(section.size()) ? end :
                                  std::min(end,hit ? section[c].hi : section[c].lo);
                            assert(y<lim);
                            int width=lim-y;
                            ++s.classified;
                            if(hit) {
                                ++s.raw_hits;
                                bands.push_back({src,y,width,node,c});
                                if(lim==section[c].hi) ++c;
                            } else {
                                ++s.live_outputs;
                                if(next_groups.empty() || next_groups.back().node!=node)
                                    next_groups.push_back({node,int(children.size()),int(children.size())});
                                auto& ng=next_groups.back();
                                if(ng.end>ng.begin && children.back().source+children.back().width==src &&
                                   children.back().current+children.back().width==y) {
                                    children.back().width+=width;
                                    ++s.live_merges;
                                } else {
                                    children.push_back({src,y,width}); ++ng.end;
                                }
                            }
                            src+=width; y=lim;
                        }
                        x=stop; prev_run=run;
                        if(x<prev_end) ++run;
                    }
                }
            }
            s.peak_frontier_capacity=std::max(s.peak_frontier_capacity,
                                              cap(live)+cap(children)+cap(groups)+cap(next_groups));
            live.swap(children); groups.swap(next_groups);
            assert(s.depth<=u64(n));
        }
        discover_ms=std::chrono::duration<double,std::milli>(Clock::now()-t).count();
        t=Clock::now();
        std::sort(bands.begin(),bands.end(),[this](const Band& a,const Band& b) {
            ++s.sort_comparisons; return a.source<b.source;
        });
        size_t k=0;
        for(auto b:bands) {
            if(k && bands[k-1].source+bands[k-1].width==b.source &&
                    bands[k-1].dest+bands[k-1].width==b.dest &&
                    bands[k-1].word==b.word && bands[k-1].color==b.color)
                bands[k-1].width+=b.width;
            else bands[k++]=b;
        }
        bands.resize(k);
        order_ms=std::chrono::duration<double,std::milli>(Clock::now()-t).count();
        t=Clock::now();
        spelling_offset.assign(nodes.size(),-1); spelling_len.resize(nodes.size());
        for(auto b:bands) {
            if(spelling_offset[b.word]<0) {
                ++s.words; int start=dictionary.size();
                for(int j=b.word;j!=0;j=nodes[j].parent) dictionary.push_back(char(nodes[j].byte));
                std::reverse(dictionary.begin()+start,dictionary.end());
                spelling_offset[b.word]=start; spelling_len[b.word]=int(dictionary.size())-start;
            }
            s.covered+=u64(b.width)*spelling_len[b.word];
        }
        s.word_bytes=dictionary.size();
        assert(s.covered<=u64(n));
        dict_ms=std::chrono::duration<double,std::milli>(Clock::now()-t).count();
    }

    bool expand(int mark,std::string& out) {
        auto t=Clock::now();
        if(section.empty()) return false;
        int row=mark;
        while(true) {
            int c=next_component(row);
            if(c<int(section.size()) && section[c].lo<=row) break;
            ++s.mark_seeks;
            auto r=select[locate(row,-1,false)]; out.push_back(char(r.byte));
            row+=r.displacement;
            if(row==mark) {out.clear(); return false;}
        }
        s.mark_prefix=out.size();
        while(int(out.size())<n) {
            ++s.output_lookups;
            auto it=std::upper_bound(bands.begin(),bands.end(),row,
                  [](int x,const Band& b){return x<b.source;});
            assert(it!=bands.begin()); auto b=*(--it);
            assert(row<b.source+b.width);
            out.append(dictionary,spelling_offset[b.word],std::min(n-int(out.size()),spelling_len[b.word]));
            row=b.dest+row-b.source;
        }
        emit_ms=std::chrono::duration<double,std::milli>(Clock::now()-t).count();
        return true;
    }

    static void hex(const char* data, size_t n) {
        const char digits[]="0123456789abcdef";
        std::cout << '"'; for(size_t i=0;i<n;++i) {
            unsigned char c=data[i]; std::cout << digits[c>>4] << digits[c&15];
        }
        std::cout << '"';
    }

    void write(int mark,bool detailed) {
        std::string output; bool hit=expand(mark,output);
        std::cout<<"{";
#define FIELD(k,v) std::cout << "\"" << k << "\":" << (v) << ","
        FIELD("n",n); FIELD("groups",s.groups); FIELD("packet_inputs",s.packets);
        FIELD("select_translations",s.translations); FIELD("component_pieces",s.classified);
        FIELD("live_outputs",s.live_outputs); FIELD("raw_hits",s.raw_hits); FIELD("live_merges",s.live_merges);
        FIELD("select_binary_searches",s.full_seeks); FIELD("select_binary_comparisons",s.comparisons);
        FIELD("select_finger_retained",s.retained); FIELD("select_finger_next",s.next);
        FIELD("sort_comparisons",s.sort_comparisons); FIELD("max_depth",s.depth); FIELD("peak_frontier",s.peak_frontier);
        FIELD("peak_frontier_capacity_bytes",s.peak_frontier_capacity); FIELD("trie_nodes",nodes.size());
        FIELD("select_capacity_bytes",cap(select)); FIELD("trie_capacity_bytes",cap(nodes)); FIELD("band_capacity_bytes",cap(bands));
        FIELD("word_index_capacity_bytes",cap(spelling_offset)+cap(spelling_len));
        FIELD("dictionary_capacity_bytes",dictionary.capacity()); FIELD("section_capacity_bytes",cap(section));
        FIELD("word_count",s.words); FIELD("spelling_bytes",s.word_bytes); FIELD("bands_count",bands.size());
        FIELD("covered_rows",s.covered); FIELD("source_rows",s.source_rows); FIELD("mark_seeks",s.mark_seeks);
        FIELD("mark_prefix",s.mark_prefix); FIELD("output_lookups",s.output_lookups);
        FIELD("setup_ms",setup_ms); FIELD("discover_ms",discover_ms); FIELD("order_ms",order_ms);
        FIELD("dict_ms",dict_ms); FIELD("emit_ms",emit_ms);
#undef FIELD
        std::cout<<"\"expanded_hex\":";
        if(hit) hex(output.data(),output.size()); else std::cout<<"null";
        if(detailed) {
            std::cout<<",\"bands\":["; bool first=true;
            for(auto b:bands) {
                if(!first) std::cout<<",";
                first=false;
                std::cout<<"["<<b.source<<","<<b.dest<<","<<b.width<<",";
                hex(dictionary.data()+spelling_offset[b.word],spelling_len[b.word]);
                std::cout<<","<<b.color<<"]";
            }
            std::cout<<"]";
        }
        std::cout<<"}\n";
    }
};

int main(int argc,char** argv) {
    std::ios::sync_with_stdio(false); std::cin.tie(nullptr);
    bool detailed=argc==2 && std::string(argv[1])=="BANDS";
    int n,mark,r,c;
    while(std::cin>>n>>mark>>r>>c) {
        assert(n>0 && 0<=mark && mark<n);
        std::vector<InputRun> runs(r); for(auto &v:runs) std::cin>>v.byte>>v.width;
        std::vector<Interval> comp(c); for(auto &v:comp) std::cin>>v.lo>>v.hi;
        Forward f(n,runs,std::move(comp)); f.discover(); f.write(mark,detailed);
    }
}
