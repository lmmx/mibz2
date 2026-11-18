// Reduce standard Lackey output online. Each record is (size, access, address).
#include <array>
#include <cassert>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>

int main(int argc, char** argv) {
    if (argc != 4) return 2;
    const auto mark = std::strtoull(argv[1], nullptr, 16);
    FILE* trace = std::fopen(argv[2], "wb");
    FILE* counts = std::fopen(argv[3], "w");
    if (!trace || !counts || !mark) return 2;
    std::array<unsigned long long, 64> inst{};
    bool active = false, ended = false;
    unsigned phase = 0;
    char line[256];
    while (std::fgets(line, sizeof line, stdin)) {
        const bool instruction = line[0] == 'I';
        if (instruction) { if (active) ++inst[phase]; continue; }
        if (line[0] != ' ' || std::strchr("LSM", line[1]) == nullptr) continue;
        char* end;
        const uint64_t address = std::strtoull(line + 2, &end, 16);
        assert(*end == ',');
        const unsigned size = std::strtoul(end + 1, nullptr, 10);
        if (address >= mark && address < mark + 64 && line[1] == 'S') {
            phase = address - mark;
            if (phase == 31) { assert(!active && !ended); active = true; }
            if (phase == 32 && active) { active = false; ended = true; }
            if (active) {
                const uint64_t event = (uint64_t(255) << 56) | phase;
                std::fwrite(&event, sizeof event, 1, trace);
            }
        } else if (active) {
            // VEX reports L = load, S = store, M = read then modify.
            assert(size > 0 && size < 255 && address < (1ULL << 52));
            unsigned op = line[1] == 'L' ? 0 : line[1] == 'S' ? 1 : 2;
            const uint64_t event = (uint64_t(size) << 56) | (uint64_t(op) << 52) | address;
            std::fwrite(&event, sizeof event, 1, trace);
        }
    }
    if (!ended || active) { std::fprintf(stderr, "incomplete trace\n"); return 1; }
    for (unsigned i = 0; i < inst.size(); ++i)
        if (inst[i]) std::fprintf(counts, "%u,%llu\n", i, inst[i]);
    return std::fclose(trace) != 0 || std::fclose(counts) != 0;
}
