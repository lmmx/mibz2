// Cold, fully associative, write-allocate LRU, with final dirty drain.
#include <algorithm>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <limits>
#include <list>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <vector>

using Line = uint64_t;
constexpr uint32_t phase_marker = 0xffffffffU;
constexpr Line no_line = std::numeric_limits<Line>::max();

struct Entry {
    std::list<Line>::iterator pos;
    bool dirty;
};
struct Result { bool miss; Line dirty_eviction; };

class Cache {
    std::list<Line> order;
    std::unordered_map<Line, Entry> entries;
public:
    const size_t capacity;
    uint64_t reads = 0, writes = 0, read_fills = 0, store_allocations = 0, dirty_evictions = 0;
    explicit Cache(size_t n) : capacity(n) {
        if (!n) throw std::invalid_argument("zero capacity");
        entries.reserve(n+1);
    }
    Result touch(Line line, bool store) {
        if (store) ++writes; else ++reads;
        auto it = entries.find(line);
        if (it != entries.end()) {
            it->second.dirty |= store;
            order.splice(order.begin(), order, it->second.pos);
            return {false, no_line};
        }
        if (store) ++store_allocations; else ++read_fills;
        Line wb = no_line;
        if (order.size() == capacity) {
            auto old = entries.find(order.back());
            if (old->second.dirty) {
                wb = order.back();
                ++dirty_evictions;
            }
            entries.erase(old);
            order.pop_back();
        }
        order.push_front(line);
        entries.emplace(line, Entry{order.begin(), store});
        return {true, wb};
    }
    std::vector<Line> dirty_tail() const {
        std::vector<Line> result;
        for (auto at = order.rbegin(); at != order.rend(); ++at)
            if (entries.at(*at).dirty) result.push_back(*at);
        return result;
    }
};

std::vector<uint32_t> read_trace(const std::string &file) {
    std::ifstream stream(file, std::ios::binary | std::ios::ate);
    if (!stream) throw std::runtime_error("could not open " + file);
    std::streamoff bytes = stream.tellg();
    if (bytes <= 0 || bytes % 4) throw std::runtime_error("invalid trace size");
    std::vector<uint32_t> events(static_cast<size_t>(bytes)/4);
    stream.seekg(0);
    if (!stream.read(reinterpret_cast<char*>(events.data()), bytes))
        throw std::runtime_error("short trace");
    return events;
}

void single(const std::vector<uint32_t> &events, size_t capacity) {
    Cache cache(capacity);
    uint64_t lookup_reads = 0, lookup_misses = 0;
    bool walk = false;
    for (uint32_t event : events) {
        if (event == phase_marker) {
            if (walk) throw std::runtime_error("repeated marker");
            walk = true;
            continue;
        }
        bool store = event >> 31;
        uint32_t line = event & 0x7fffffffU;
        auto result = cache.touch(line, store);
        auto region = line >> 26;
        // Only interpreted for the two candidates; Cells has its own region mapping.
        if (walk && !store && (region == 2 || region == 3 || region == 5)) {
            ++lookup_reads;
            lookup_misses += result.miss;
        }
    }
    if (!walk) throw std::runtime_error("missing marker");
    std::cout << capacity << ',' << cache.reads << ',' << cache.writes << ','
              << cache.read_fills << ',' << cache.store_allocations << ','
              << cache.dirty_evictions + cache.dirty_tail().size() << ','
              << lookup_reads << ',' << lookup_misses << '\n';
}

void team(int argc, char **argv, bool warm) {
    // Equal line-reference opportunities per active worker. Each 256 KiB private
    // cache sends fills and complete dirty evictions to a shared noninvalidating LLC.
    const size_t workers = argc-3;
    Cache shared(std::stoul(argv[2]));
    std::vector<Cache> private_l2;
    std::vector<std::vector<uint32_t>> streams;
    std::vector<size_t> position(workers);
    private_l2.reserve(workers);
    for (size_t j=0; j<workers; ++j) {
        private_l2.emplace_back(4096);
        streams.push_back(read_trace(argv[j+3]));
    }
    const size_t waves = warm ? 3 : 1;
    for (size_t wave=0; wave<waves; ++wave) {
        std::fill(position.begin(), position.end(), 0);
        auto reads0 = shared.read_fills;
        auto writebacks0 = shared.dirty_evictions;
        bool progress;
        do {
            progress = false;
            for (size_t j=0; j<workers; ++j) {
                auto &p = position[j];
                const auto &stream = streams[(j+wave)%workers];
                if (p < stream.size() && stream[p] == phase_marker) ++p;
                if (p == stream.size()) continue;
                progress = true;
                auto event = stream[p++];
                Line line = (Line(j)<<32) | (event & 0x7fffffffU);
                auto r = private_l2[j].touch(line, event>>31);
                if (r.dirty_eviction != no_line) shared.touch(r.dirty_eviction, true);
                if (r.miss) shared.touch(line, false);
            }
        } while (progress);
        if (warm) std::cout << workers << ',' << shared.capacity << ',' << wave+1 << ','
                            << shared.read_fills-reads0 << ',' << shared.dirty_evictions-writebacks0 << '\n';
    }
    auto dirty_before_drain = shared.dirty_evictions;
    std::vector<std::vector<Line>> drains;
    size_t longest = 0;
    for (const auto &cache : private_l2) {
        drains.push_back(cache.dirty_tail());
        longest = std::max(longest, drains.back().size());
    }
    for (size_t i=0; i<longest; ++i)
        for (const auto &d : drains)
            if (i<d.size()) shared.touch(d[i], true);
    // An LLC miss on a full-line private writeback needs no read from memory.
    if (warm) {
        std::cout << workers << ',' << shared.capacity << ",drain,0,"
                  << shared.dirty_evictions-dirty_before_drain+shared.dirty_tail().size() << '\n';
        return;
    }
    std::cout << workers << ',' << shared.capacity << ',' << shared.reads << ','
              << shared.writes << ',' << shared.store_allocations << ',' << shared.read_fills
              << ',' << shared.dirty_evictions+shared.dirty_tail().size() << '\n';
}

int main(int argc, char **argv) {
    if (argc < 3) return 2;
    if (std::string(argv[1]) == "--team" || std::string(argv[1]) == "--warm-team") {
        if (argc < 4) return 2;
        team(argc, argv, std::string(argv[1]) == "--warm-team");
    } else {
        auto events = read_trace(argv[1]);
        for (int i=2; i<argc; ++i) single(events, std::stoul(argv[i]));
    }
}
