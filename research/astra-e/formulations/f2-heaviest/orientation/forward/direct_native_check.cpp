// Emit the actual raw band descriptors and decoded bytes for the independent validator.
#include <cstdint>
#include <iostream>
#include <vector>
struct InputRun { int byte, width; };
struct Interval { int lo, hi; };
#include "direct_native.hpp"

int main() {
    std::ios::sync_with_stdio(false);
    int n, mark, r, c;
    if (!(std::cin >> n >> mark >> r >> c)) return 1;
    std::vector<InputRun> runs(r);
    std::vector<Interval> sections(c);
    for (auto& run : runs) std::cin >> run.byte >> run.width;
    for (auto& section : sections) std::cin >> section.lo >> section.hi;
    direct_native::Direct direct(n,runs,sections);
    direct.discover();
    auto output = direct.expand(mark);
    const auto& s = direct.stats;
    std::cout << s.bands << " " << s.lf_probes << " " << s.page_increments << " "
              << s.spelling_bytes << " " << s.scatter_writes << " " << s.mark_probes << " "
              << s.output_lookups << " " << output.size() << "\n";
    // Headers carry lengths. Words and output below are unescaped bytes.
    for (auto b : direct.bands) {
        std::cout << b.source << " " << b.destination << " " << b.width << " " << b.length << "\n";
        std::cout.write(direct.spellings.data()+b.offset,b.length);
    }
    std::cout.write(output.data(),output.size());
}
