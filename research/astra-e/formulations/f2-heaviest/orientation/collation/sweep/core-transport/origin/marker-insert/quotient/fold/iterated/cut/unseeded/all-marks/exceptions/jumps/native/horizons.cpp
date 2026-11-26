#include "model.hpp"
#include <filesystem>
#include <iostream>

int main(int argc, char** argv) {
    assert(argc == 6);
    std::ifstream in(argv[1], std::ios::binary | std::ios::ate);
    assert(in);
    std::vector<uint16_t> caps(in.tellg() / sizeof(uint16_t));
    in.seekg(0);
    in.read(reinterpret_cast<char*>(caps.data()), caps.size() * sizeof(uint16_t));
    assert(in);

    U n = std::stoul(argv[2]), mark = std::stoul(argv[3]);
    bool variable = std::stoi(argv[4]) != 0;
    Input raw = from_caps(caps.data(), caps.size(), n, mark);
    auto geometry = contexts(raw, variable);
    std::vector<U> lf(n);
    std::vector<Byte> good(n);
    ::Geometry bases(raw);
    std::array<U, 256> cursor;
    std::copy_n(bases.base.begin(), 256, cursor.begin());
    U x = 0, state = 0;
    for (auto run : raw.runs) {
        for (U j = 0; j < run.width; ++j) {
            while (x == geometry.ends[state]) ++state;
            lf[x] = cursor[run.symbol]++;
            good[x++] = run.symbol == geometry.modes[state];
        }
    }
    std::vector<U> streaks;
    U row = mark, length = 0, exceptions = 0;
    for (U j = 0; j < n; ++j) {
        if (good[row]) {
            ++length;
        } else {
            if (length) streaks.push_back(length);
            length = 0;
            ++exceptions;
        }
        row = lf[row];
    }
    if (length) streaks.push_back(length);

    // A shortcut stays in the modal path, so each exceptional byte is a separate unit.
    U original = exceptions, binary = exceptions, g4 = exceptions;
    U ideal8 = exceptions, ideal16 = exceptions, ideal32 = exceptions;
    U maxstreak = 0, jump8 = 0;
    for (U len : streaks) {
        jump8 += len / 8;
        original += len / 8 + len % 8;
        g4 += len / 8 + (len % 8) / 4 + len % 4;
        binary += len / 8 + std::popcount(len % 8);
        ideal8 += (len + 7) / 8;
        ideal16 += (len + 15) / 16;
        ideal32 += (len + 31) / 32;
        maxstreak = std::max(maxstreak, len);
    }
    std::cout << argv[5] << ',' << variable << ',' << n << ',' << exceptions << ','
              << streaks.size() << ',' << maxstreak << ',' << jump8 << ',' << original
              << ',' << g4 << ',' << binary << ',' << ideal8 << ',' << ideal16 << ','
              << ideal32 << ',' << exceptions + streaks.size() << '\n';
}
