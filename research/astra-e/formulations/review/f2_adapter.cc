// Expose the pinned production-independent F2 constructor to the necklace oracle.
// No code from Factor's dense validator is called.
#define main astra_e_f2_original_main
#include "../f2/factor.cc"
#undef main

int main() {
    std::string last;
    unsigned mask;
    while (std::cin >> last >> mask) {
        std::vector<uint16_t> caps;
        for (char c : last) caps.push_back(uint16_t((unsigned(c - 'A') << 8) | 1));
        Mask selected{};
        for (unsigned i = 0; i < 8; ++i) selected[i] = (mask >> i) & 1;
        Column source(caps, selected);
        Factor built(source, selected);
        std::cout << built.nodes.size()-1 << ' ' << built.costs.intersections << " |";
        for (auto [target, token] : built.next) {
            const auto& w = built.nodes[token];
            std::cout << ' ' << target << ':';
            for (unsigned i = 0; i < w.depth; ++i)
                std::cout << char('A' + built.dictionary[w.word_offset+i]);
        }
        std::cout << " |";
        std::vector<Byte> output;
        for (unsigned p = 0; p < last.size(); ++p) {
            unsigned offset = 0;
            if (!built.decode(p, output, offset)) {
                std::cout << " -";
            } else {
                std::cout << ' ' << offset << ':';
                for (auto b : output) std::cout << char('A'+b);
            }
        }
        std::cout << '\n';
    }
}
