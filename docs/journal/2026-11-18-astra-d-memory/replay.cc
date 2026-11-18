// Inclusive, serial reference hierarchy. Every payload transfer is a whole 64-byte line.
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <cstdio>
#include <fstream>
#include <iostream>
#include <map>
#include <sstream>
#include <string>
#include <vector>

using U = uint64_t;
enum Metric { loads, stores, l1_rmiss, l1_wmiss, l1_wb, l2_req, l2_miss, l2_wb,
              llc_req, llc_miss, dram_read, dram_rfo, dram_wb, terminal_wb, end_metrics };
const char* columns = "loads,stores,l1_rmiss,l1_wmiss,l1_wb,l2_req,l2_miss,l2_wb,llc_req,llc_miss,dram_read,dram_rfo,dram_wb,terminal_wb";
using Tag = unsigned;
struct Line { U id = 0, age = 0; Tag owner = 0; }; // age 0 is invalid, owner 0 is clean
struct Cache {
    unsigned ways;
    std::vector<Line> lines;
    Cache(unsigned kib, unsigned w): ways(w), lines(kib * 1024 / 64) {
        assert(lines.size() >= w && lines.size() % w == 0);
    }
    Line* set(U id) { return lines.data() + id % (lines.size() / ways) * ways; }
    Line* find(U id) {
        auto p = set(id);
        for (unsigned i = 0; i < ways; ++i) if (p[i].age && p[i].id == id) return p+i;
        return nullptr;
    }
    Line& victim(U id) {
        auto p = set(id);
        return *std::min_element(p, p+ways, [](Line a, Line b){return a.age < b.age;});
    }
};
class Model {
    std::array<Cache,3> cache;
    U clock = 0;
    bool terminal = false;
    // Upper levels carry the newest dirty owner. Merge it before removing a lower tag.
    void evict(int level, Line& old) {
        if (!old.age) return;
        if (level > 0)
            if (auto upper = cache[level-1].find(old.id)) evict(level-1, *upper);
        if (old.owner) {
            auto& c = counts[old.owner];
            if (level == 2) {
                ++c[dram_wb];
                if (terminal) ++c[terminal_wb];
            } else {
                ++c[level == 0 ? l1_wb : l2_wb];
                auto lower = cache[level+1].find(old.id);
                assert(lower); // full-line writeback into the inclusive parent
                lower->owner = old.owner;
                lower->age = ++clock;
            }
        }
        old = {};
    }
    Line& fetch(int level, U id, bool write, Tag tag) {
        auto& c = counts[tag];
        if (level == 1) ++c[l2_req];
        if (level == 2) ++c[llc_req];
        if (auto hit = cache[level].find(id)) { hit->age = ++clock; return *hit; }
        if (level == 0) ++c[write ? l1_wmiss : l1_rmiss];
        if (level == 1) ++c[l2_miss];
        if (level == 2) {
            ++c[llc_miss];
            ++c[write ? dram_rfo : dram_read];
        } else {
            fetch(level+1, id, write, tag);
        }
        Line& dest = cache[level].victim(id);
        evict(level, dest);
        dest = {id, ++clock, 0};
        return dest;
    }
public:
    std::array<std::array<U,end_metrics>,1024> counts{};
    Model(unsigned lk, unsigned lw, unsigned mk, unsigned mw, unsigned sk, unsigned sw):
        cache{Cache(lk,lw), Cache(mk,mw), Cache(sk,sw)} {}
    void touch(U id, bool write, Tag tag) {
        assert(tag && tag < counts.size());
        ++counts[tag][write ? stores : loads];
        Line& line = fetch(0, id, write, tag);
        if (write) line.owner = tag;
    }
    void private_handoff() { for (auto& line: cache[1].lines) evict(1,line); }
    void finish() { terminal = true; for (auto& line: cache[2].lines) evict(2,line); }
};

struct Range { U lo, hi; std::string name; };
struct Ranges {
    std::map<U,std::vector<Range>> pages;
    std::vector<std::string> names{"other"};
    unsigned intern(const std::string& n) {
        auto it = std::find(names.begin(), names.end(), n);
        if (it != names.end()) return it - names.begin();
        names.push_back(n);
        assert(names.size() <= 32);
        return names.size()-1;
    }
    explicit Ranges(const char* path) {
        std::ifstream f(path);
        assert(f);
        for (std::string line; std::getline(f,line); ) {
            std::stringstream s(line);
            std::string code,name; U lo,len;
            s >> code;
            if (code != "range") continue;
            s >> name >> std::hex >> lo >> std::dec >> len;
            assert(len && lo && s);
            for (U p = lo/4096; p <= (lo+len-1)/4096; ++p)
                pages[p].push_back({lo,lo+len,name});
        }
    }
    // Split reused allocations by lifetime. A vector/table may end inside a line.
    std::string lookup(unsigned phase,U addr) const {
        auto it = pages.find(addr/4096);
        if (it == pages.end()) return "other";
        std::string name = "other";
        for (auto& r: it->second) {
            if (addr < r.lo || addr >= r.hi) continue;
            if (r.name == "mtf_rows" || r.name == "mtf_counts") {
                if (phase != 2) continue;
            }
            if (r.name == "bottom" || r.name == "entries" || r.name == "front_bytes")
                if (phase != 3 && phase != 4) continue;
            name = r.name;
        }
        if (name == "tops" && phase >= 5 && phase <= 7) return "positions";
        if (name == "rules" && phase == 7) return "commands";
        return name;
    }
};

static void selftest() {
    // One line per set effectively direct-mapped; all these addresses map to set 0.
    Model m(1,1,2,1,4,1);
    m.touch(0,true,1); m.touch(0,false,2); m.touch(16,false,2);
    m.touch(0,false,2); m.touch(64,true,2); m.finish();
    auto a = m.counts[1], b = m.counts[2];
    assert(a[dram_rfo]==1 && a[dram_wb]==1 && a[terminal_wb]==0);
    assert(b[dram_read]==1 && b[dram_rfo]==1 && b[dram_wb]==1 && b[terminal_wb]==1);
    assert(b[l1_rmiss]==2 && b[l2_miss]==2 && b[llc_miss]==2);
    // LLC conflict back-invalidates a dirty private line even if L2 has a free way.
    Model q(1,2,2,2,2,1);
    q.touch(0,false,1); q.touch(0,true,2); q.touch(32,false,1); q.touch(0,false,1); q.finish();
    assert(q.counts[2][l1_wb]==1 && q.counts[2][l2_wb]==1 && q.counts[2][dram_wb]==1);
    assert(q.counts[1][dram_read]==3);
}

int main(int argc,char** argv) {
    selftest();
    if (argc==1) return 0;
    if (argc!=4) return 2;
    Ranges ranges(argv[1]);
    FILE* f = std::fopen(argv[2],"rb");
    if (!f) return 2;
    Model model(32,8,256,4,std::stoul(argv[3]),16);
    unsigned phase = 0;
    struct Label { U line = UINT64_MAX; unsigned phase = 0, tag = 0; bool uniform = false; };
    std::vector<Label> memo(1<<16);
    U x;
    while (std::fread(&x,sizeof x,1,f)==1) {
        const unsigned size = x>>56, op = (x>>52)&15;
        if (size==255) {
            phase = x&63;
            if (phase == 2 || phase == 11) model.private_handoff();
            continue;
        }
        assert(phase > 0 && phase < 32 && size && op < 3);
        U addr = x & ((1ULL<<52)-1);
        for (int access=0; access <= (op==2); ++access) {
            const bool store = op==1 || access==1;
            for (U id=addr/64; id<=(addr+size-1)/64; ++id) {
                auto& saved = memo[id % memo.size()];
                if (saved.line != id || saved.phase != phase) {
                    const unsigned first = ranges.intern(ranges.lookup(phase,64*id));
                    // A nested tiny vector may occupy only the middle of a line, so
                    // check range boundaries too before memoising its classification.
                    auto it = ranges.pages.find(id/64);
                    bool uniform = true;
                    if (it != ranges.pages.end()) for (const auto& r: it->second) {
                        if ((r.lo>64*id && r.lo<64*id+64) ||
                            (r.hi>64*id && r.hi<64*id+64)) { uniform=false; break; }
                    }
                    saved = {id,phase,first,uniform};
                }
                unsigned area = saved.uniform ? saved.tag :
                    ranges.intern(ranges.lookup(phase,std::max(addr,64*id)));
                // The reverse writer reads rules and writes commands in the same slots.
                if (phase == 6 && store && ranges.names[area] == "rules") area = ranges.intern("commands");
                if (phase >= 12 && phase <= 15 && ranges.names[area] == "runs") area = ranges.intern("edits");
                model.touch(id,store,phase*32+area);
            }
        }
    }
    assert(std::feof(f) && phase==11);
    std::fclose(f);
    model.finish();
    std::cout << "phase,region," << columns << '\n';
    for (unsigned tag=1; tag<model.counts.size(); ++tag) {
        auto& c = model.counts[tag];
        if (!c[loads] && !c[stores]) continue;
        std::cout << tag/32 << ',' << ranges.names.at(tag%32);
        for (auto n:c) std::cout << ',' << n;
        std::cout << '\n';
    }
}
