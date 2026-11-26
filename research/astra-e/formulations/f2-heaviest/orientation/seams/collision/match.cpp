// Complete marked inverse timing, with the same starting run values and
// section.
#define ASTRA_COLLISION_LIBRARY
#define ASTRA_COLLISION_QUIET
#include "collision.cpp"
struct InputRun {
  int byte, width;
};
struct Interval {
  int lo, hi;
};
#include "../../forward/direct_native.hpp"

int high_water_kib() {
  std::ifstream status("/proc/self/status");
  std::string key;
  while (status >> key) {
    if (key == "VmHWM:") {
      int kib;
      status >> kib;
      return kib;
    }
    std::string rest;
    std::getline(status, rest);
  }
  throw std::runtime_error("VmHWM unavailable");
}

int main(int argc, char **argv) {
  if (argc != 5 && argc != 7)
    throw std::runtime_error("match INPUT LAPS ORDINAL EXPECTED [ARM SECTION]");
  std::ifstream file(argv[1]);
  U n, mark, nruns, nsec;
  file >> n >> mark >> nruns >> nsec;
  std::vector<Run> runs(nruns);
  std::vector<InputRun> input(nruns);
  for (U i = 0; i < nruns; ++i) {
    file >> input[i].byte >> input[i].width;
    runs[i] = {Byte(input[i].byte), U(input[i].width)};
  }
  std::ifstream gold(argv[4], std::ios::binary);
  std::string expected{std::istreambuf_iterator<char>(gold), {}};
  if (expected.size() != n)
    throw std::runtime_error("bad expected fixture");
  const char *methods[] = {"fixed_sort", "fixed_radix", "indexed_direct"};
  for (U section_no = 0; section_no < nsec; ++section_no) {
    std::string name;
    U count;
    file >> name >> count;
    Section section;
    std::vector<Interval> intervals(count);
    for (auto &interval : intervals) {
      file >> interval.lo >> interval.hi;
      section.emplace_back(interval.lo, interval.hi);
    }
    if (argc == 7 && name != argv[6])
      continue;
    int order[] = {0, 1, 2};
    for (int lap = 0; lap < std::stoi(argv[2]);
         ++lap, std::next_permutation(order, order + 3)) {
      for (int mode : order) {
        if (argc == 7 && methods[mode] != std::string(argv[5]))
          continue;
        const double start = cpu();
        Word bytes;
        std::string direct_bytes;
        if (mode == 2) {
          direct_bytes = direct_native::decode(n, mark, input, intervals);
        } else {
          Construction construction(Collision, mode == 1);
          construction.run(runs, section);
          bytes = construction.output(mark, n);
        }
        // Both representations have destroyed their constructor and retained
        // only the marked output. Validate all output bytes outside the timer.
        const double elapsed = cpu() - start;
        bool exact = mode == 2 ? direct_bytes == expected
                               : bytes.size() == n &&
                                     std::equal(bytes.begin(), bytes.end(),
                                                reinterpret_cast<const Byte *>(
                                                    expected.data()));
        if (!exact)
          throw std::runtime_error("marked output mismatch");
        std::cout << argv[3] << ',' << name << ',' << methods[mode] << ','
                  << lap << ',' << elapsed << ',' << high_water_kib() << '\n';
      }
    }
  }
}
