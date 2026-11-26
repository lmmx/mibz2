// Include after InputRun { int byte,width; } and Interval { int lo,hi; }.
// The matched experiment supplies one nonempty marker cylinder or one selected
// return-type interval. These sections have Direct's uniform terminal membership.
#pragma once
#include <algorithm>
#include <array>
#include <cassert>
#include <chrono>
#include <cstdint>
#include <stdexcept>
#include <string>
#include <vector>

namespace direct_native {
using Clock = std::chrono::steady_clock;

struct Stats {
    double setup_ms = 0, discover_ms = 0, order_ms = 0, dict_ms = 0, emit_ms = 0;
    std::uint64_t lf_probes = 0, page_increments = 0, scatter_writes = 0;
    std::uint64_t mark_probes = 0, mark_page_increments = 0, output_lookups = 0;
    std::size_t bands = 0, spelling_bytes = 0, lf_capacity_bytes = 0;
    std::size_t page_capacity_bytes = 0, band_capacity_bytes = 0;
    std::size_t spelling_capacity_bytes = 0, directory_capacity_bytes = 0;
};

struct Run {
    int end, displacement;
    unsigned char byte;
};

struct Band {
    int source, destination, width, offset, length;
};

class Direct {
    int n;
    Interval section;
    std::vector<Run> lf;
    std::vector<int> pages;
    std::vector<int> band_at;

    static double elapsed(Clock::time_point begin) {
        return std::chrono::duration<double,std::milli>(Clock::now()-begin).count();
    }

    const Run& locate(int x, std::uint64_t& increments) const {
        int first = pages[x/64], i = first;
        while (lf[i].end <= x) ++i;
        increments += i-first;
        return lf[i];
    }

public:
    std::vector<Band> bands;
    std::string spellings;
    Stats stats;

    Direct(int size, const std::vector<InputRun>& runs, const std::vector<Interval>& sections)
        : n(size) {
        auto start = Clock::now();
        if (sections.size()!=1) throw std::invalid_argument("Direct requires a single nonempty section");
        section = sections.front();
        assert(0 <= section.lo && section.lo < section.hi && section.hi <= n);
        std::array<int,256> cursor{};
        for (const auto& r : runs) cursor[r.byte] += r.width;
        int sum = 0;
        for (int& count : cursor) {
            int width = count;
            count = sum;
            sum += width;
        }
        assert(sum==n);
        int pos = 0;
        lf.reserve(runs.size());
        for (const auto& r : runs) {
            lf.push_back({pos+r.width, cursor[r.byte]-pos, static_cast<unsigned char>(r.byte)});
            cursor[r.byte] += r.width;
            pos += r.width;
        }
        pages.reserve((n+63)/64);
        for (int row=0, i=0; row<n; row+=64) {
            while (lf[i].end <= row) ++i;
            pages.push_back(i);
        }
        band_at.resize(section.hi-section.lo);
        stats.lf_capacity_bytes = sizeof(Run)*lf.capacity();
        stats.page_capacity_bytes = sizeof(int)*pages.capacity();
        stats.directory_capacity_bytes = sizeof(int)*band_at.capacity();
        stats.setup_ms = elapsed(start);
    }

    void discover() {
        auto start = Clock::now();
        for (int dest=section.lo; dest<section.hi;) {
            int row = dest, width = section.hi-dest;
            int begin = static_cast<int>(spellings.size());
            do {
                const auto& run = locate(row, stats.page_increments);
                ++stats.lf_probes;
                width = std::min(width,run.end-row);
                spellings.push_back(static_cast<char>(run.byte));
                row += run.displacement;
                // Every live affine span has uniform membership on the two
                // registered section families. Python Direct tests these same cuts.
                assert(row >= section.lo || row+width <= section.lo);
            } while (row < section.lo || row >= section.hi);
            assert(row+width <= section.hi);
            int length = static_cast<int>(spellings.size())-begin;
            std::reverse(spellings.begin()+begin,spellings.end());
            bands.push_back({row,dest,width,begin,length});
            dest += width;
        }
        stats.discover_ms = elapsed(start);
        start = Clock::now();
        for (int i=0; i<static_cast<int>(bands.size()); ++i) {
            const auto& band = bands[i];
            std::fill_n(band_at.begin()+(band.source-section.lo),band.width,i);
            stats.scatter_writes += band.width;
        }
        stats.order_ms = elapsed(start);
        stats.bands = bands.size();
        stats.spelling_bytes = spellings.size();
        stats.band_capacity_bytes = sizeof(Band)*bands.capacity();
        stats.spelling_capacity_bytes = spellings.capacity();
    }

    std::string expand(int mark) {
        auto start = Clock::now();
        int row = mark, skip = 0;
        while (row < section.lo || row >= section.hi) {
            row += locate(row, stats.mark_page_increments).displacement;
            ++stats.mark_probes;
            ++skip;
            if (row==mark) {
                stats.emit_ms = elapsed(start);
                return {};
            }
        }
        std::string out;
        while (static_cast<int>(out.size())<n) {
            const auto& band = bands[band_at[row-section.lo]];
            assert(band.source<=row && row<band.source+band.width && skip<band.length);
            int count = std::min(band.length-skip,n-static_cast<int>(out.size()));
            out.append(spellings,band.offset+skip,count);
            row = band.destination+(row-band.source);
            skip = 0;
            ++stats.output_lookups;
        }
        stats.emit_ms = elapsed(start);
        return out;
    }
};

inline std::string decode(int n, int mark, const std::vector<InputRun>& runs,
                          const std::vector<Interval>& sections, Stats* stats=nullptr) {
    Direct direct(n,runs,sections);
    direct.discover();
    auto out = direct.expand(mark);
    if (stats) *stats = direct.stats;
    return out;
}
} // namespace direct_native
