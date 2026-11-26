// Build a delimiter phrase BWT by disjoint FM intervals and stable inverse merges.
// The dense Psi and token oracle below are validators only.
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <iterator>
#include <string>
#include <vector>

using U = uint32_t;
using Byte = uint8_t;
using Mask = std::array<bool, 256>;
constexpr U NONE = UINT32_MAX;

struct Run {
    U start, end, lf, delimiter_rank;
    Byte c;
};

struct Column {
    std::vector<Run> runs;
    std::array<U, 257> start{};
    U n=0, m=0;
    Column(const std::vector<uint16_t>& caps, const Mask& delim) {
        for (auto rec : caps) {
            Byte c = rec >> 8;
            U len = rec & 255;
            assert(len && len <= 127);
            start[c+1] += len;
            if (runs.empty() || runs.back().c != c)
                runs.push_back({n,n,0,m,c});
            runs.back().end += len;
            n += len;
            if (delim[c]) m += len;
        }
        for (U c=0;c<256;++c) start[c+1] += start[c];
        auto next = start;
        for (auto& r : runs) {
            r.lf = next[r.c];
            next[r.c] += r.end-r.start;
        }
    }
    U find(U x) const {
        return std::upper_bound(runs.begin(),runs.end(),x,
            [](U x,const Run& r){return x<r.start;})-runs.begin()-1;
    }
    U rank_d(U x, const Mask& d) const {
        const auto& r=runs[find(x)];
        return r.delimiter_rank + (d[r.c] ? x-r.start : 0);
    }
};

struct Piece { U id, len; };
struct Node {
    U lo, hi, parent, depth, head, count, word_offset=0, term=0, first=0, cursor=0, consumed=0;
    Byte c;
    std::vector<Piece> output;
    Node(U a,U b,U p,U k,Byte x):lo(a),hi(b),parent(p),depth(k),head(0),count(0),c(x) {}
};
struct Counts {
    uint64_t width=0, intersections=0, merge_pieces=0, alias_nodes=0,
        output_runs=0, peak_live_runs=0, token_runs=0, terms=0, seeks=0, dictionary_bytes=0;
};

struct Factor {
    const Column& l;
    Mask d;
    std::vector<Node> nodes;
    std::vector<Piece> selectors;
    Counts costs;
    std::vector<Byte> dictionary;
    std::vector<Piece> next; // (next section rank, phrase node)
    Factor(const Column& source,Mask delim):l(source),d(delim) {
        nodes.emplace_back(0,l.n,NONE,0,0); // root is L filtered to delimiters
        std::array<U,256> existing{},epoch{};
        epoch.fill(NONE);
        for (U i=0;i<nodes.size();++i) {
            U left=nodes[i].lo, right=nodes[i].hi;
            U k=i ? l.find(left) : 0;
            costs.seeks += i!=0;
            costs.width += i ? right-left : 0;
            nodes[i].head = selectors.size();
            for (; k<l.runs.size() && l.runs[k].start<right; ++k) {
                const auto& r=l.runs[k];
                U lo=std::max(left,r.start), w=std::min(right,r.end)-lo;
                if (!i && !d[r.c]) continue;
                costs.intersections += i!=0;
                U child=NONE;
                if (i && d[r.c]) {
                    nodes[i].term += w;
                } else {
                    U joined = r.lf + lo-r.start;
                    if (epoch[r.c]!=i) {
                        epoch[r.c]=i;
                        existing[r.c]=nodes.size();
                        nodes.emplace_back(joined,joined,i,nodes[i].depth+1,r.c);
                    }
                    child=existing[r.c];
                    assert(nodes[child].hi==joined);
                    nodes[child].hi += w;
                }
                auto h = nodes[i].head;
                if (selectors.size()>h && selectors.back().id==child) selectors.back().len+=w;
                else selectors.push_back({child,w});
            }
            nodes[i].count=selectors.size()-nodes[i].head;
            if (nodes[i].term) {
                ++costs.terms;
                nodes[i].first=l.rank_d(left,d);
                costs.seeks++;
                nodes[i].word_offset=dictionary.size();
                for (U j=i;j;j=nodes[j].parent) dictionary.push_back(nodes[j].c);
            }
        }
        costs.dictionary_bytes=dictionary.size();
        assert(costs.width<=l.n);
        assert(costs.intersections<=l.runs.size()+nodes.size()-1);
        uint64_t live=0;
        for (size_t j=nodes.size(); j-->0;) {
            auto& node=nodes[j];
            auto give=[&](U id,U len) {
                ++costs.merge_pieces;
                if (!node.output.empty() && node.output.back().id==id) node.output.back().len+=len;
                else { node.output.push_back({id,len});++live; }
                costs.peak_live_runs=std::max(costs.peak_live_runs,live);
            };
            bool alias=node.count==1 && selectors[node.head].id!=NONE;
            if (alias) {
                auto& child=nodes[selectors[node.head].id];
                node.output.swap(child.output);
                ++costs.alias_nodes;
            } else {
                for (U k=node.head;k<node.head+node.count;++k) {
                    auto [id,width]=selectors[k];
                    if (id==NONE) {give(j,width);continue;}
                    auto& child=nodes[id];
                    while (width) {
                        auto [word,run]=child.output[child.cursor];
                        U take=std::min(width,run-child.consumed);
                        give(word,take);
                        width-=take;
                        child.consumed+=take;
                        if (child.consumed==run) {child.cursor++;child.consumed=0;}
                    }
                    if (child.cursor==child.output.size()) {
                        live-=child.output.size();
                        std::vector<Piece>().swap(child.output);
                    }
                }
            }
            costs.output_runs+=node.output.size();
        }
        costs.token_runs=nodes[0].output.size();
        next.resize(l.m);
        U row=0;
        for (auto [word,w] : nodes[0].output) {
            for (U i=0;i<w;++i) next[nodes[word].first++]={row++,word};
        }
        assert(row==l.m);
    }

    // Return false for a marked cycle with no delimiter. A decoder then uses
    // the original kernel; no unrelated cycle is substituted for the mark.
    bool decode(U p,std::vector<Byte>& out,U& rewind) const {
        U x=p;
        rewind=0;
        if (!l.m) return false;
        for (;;) {
            auto r=l.runs[l.find(x)];
            if (d[r.c]) break;
            x=r.lf+x-r.start;
            ++rewind;
            if (x==p) return false;
        }
        U token=l.rank_d(x,d), skip=rewind;
        out.clear();
        while (out.size()<l.n) {
            auto [after,w]=next[token];
            auto& word=nodes[w];
            assert(skip<word.depth);
            U length=std::min<size_t>(word.depth-skip,l.n-out.size());
            auto a=dictionary.begin()+word.word_offset+skip;
            out.insert(out.end(),a,a+length);
            skip=0;
            token=after;
        }
        return true;
    }
};

struct Oracle {
    std::vector<U> psi;
    std::vector<Byte> first;
    Oracle(const Column& l):psi(l.n),first(l.n) {
        // Sort occurrences stably from a dense column independently of the FM map.
        std::vector<Byte> last(l.n);
        for (auto r:l.runs) std::fill(last.begin()+r.start,last.begin()+r.end,r.c);
        for (U x=0;x<l.n;++x) psi[x]=x;
        std::stable_sort(psi.begin(),psi.end(),[&](U a,U b){return last[a]<last[b];});
        for (U x=0;x<l.n;++x) first[x]=last[psi[x]];
    }
    void check_table(const Factor& f) const {
        std::vector<U> section(f.l.n,NONE);
        U i=0;
        for (auto r:f.l.runs) if (f.d[r.c])
            for (U x=r.start;x<r.end;++x) section[x]=i++;
        for (U x=0;x<f.l.n;++x) if (section[x]!=NONE) {
            auto [after,w]=f.next[section[x]];
            auto& word=f.nodes[w];
            U pos=x, len=0;
            do {
                assert(len<word.depth && f.dictionary[word.word_offset+len]==first[pos]);
                pos=psi[pos];++len;
            } while (section[pos]==NONE);
            assert(len==word.depth && section[pos]==after);
        }
    }
    bool check_output(const Factor& f,U p,U& rewind) const {
        std::vector<Byte> out;
        bool took=f.decode(p,out,rewind);
        U x=p;
        bool has=false;
        for (U j=0;j<psi.size();++j) {
            if (took) assert(out[j]==first[x]);
            has|=f.d[first[x]];
            x=psi[x];
        }
        assert(took==has);
        return took;
    }
};

void small() {
    uint64_t inputs=0, marks=0, accepted=0, fallback=0;
    for (U alphabet:{2,3}) for (U n=1,cases=alphabet;n<=(alphabet==2?9:6);++n,cases*=alphabet) {
        for (U code=0;code<cases;++code) {
            std::vector<uint16_t> caps;
            U k=code;
            for (U j=0;j<n;++j) {caps.push_back((k%alphabet)*256+1);k/=alphabet;}
            for (U choice=0;choice<1u<<alphabet;++choice) {
                Mask d{};
                for (U c=0;c<alphabet;++c) d[c]=(choice>>c)&1;
                Column l(caps,d); Oracle ref(l); Factor f(l,d); ref.check_table(f);++inputs;
                for (U p=0;p<n;++p) {
                    U rewind;
                    (ref.check_output(f,p,rewind)?accepted:fallback)++;++marks;
                }
            }
        }
    }
    std::cout<<"columns_and_masks="<<inputs<<", marks="<<marks<<", own="<<accepted<<", fallback="<<fallback<<"\n";
}

int main(int argc,char** argv) {
    if (argc==2 && std::string(argv[1])=="small") {small();return 0;}
    if (argc!=5) return 2;
    std::ifstream input(argv[1],std::ios::binary);
    assert(input);
    std::vector<uint16_t> caps;
    for (int low;(low=input.get())!=EOF;) {int high=input.get();assert(high!=EOF);caps.push_back(low+(high<<8));}
    Mask d{};
    for (Byte c:std::string(argv[4])) d[c]=true;
    Column l(caps,d);Oracle ref(l);
    Factor f(l,d);ref.check_table(f);
    U rewind;
    assert(ref.check_output(f,std::stoul(argv[3]),rewind));
    auto z=f.costs;
    std::cout<<argv[2]<<','<<l.n<<','<<caps.size()<<','<<l.runs.size()<<',';
    for (U c=0;c<256;++c) if (d[c]) std::cout<<c<<'.';
    std::cout<<','<<l.m<<','<<f.nodes.size()-1<<','<<z.width<<','<<z.intersections<<','<<f.selectors.size()
      <<','<<z.seeks<<','<<z.terms<<','<<z.dictionary_bytes<<','<<z.alias_nodes<<','<<z.merge_pieces
      <<','<<z.output_runs<<','<<z.peak_live_runs<<','<<z.token_runs<<','<<rewind
      <<','<<sizeof(Node)<<','<<sizeof(Run)<<'\n';
}
