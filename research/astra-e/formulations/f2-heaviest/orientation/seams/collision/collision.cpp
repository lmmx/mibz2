// Exact first-return construction by two live affine frontiers, plus
// one-direction controls.
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <ctime>
#include <fstream>
#include <iostream>
#include <numeric>
#include <optional>
#include <stdexcept>
#include <string>
#include <unordered_map>
#include <utility>
#include <vector>

using U = uint32_t;
using Count = uint64_t;
using Byte = uint8_t;
using Word = std::vector<Byte>;
using Section = std::vector<std::pair<U, U>>;
enum Mode { Back, Forward, Collision };
const char *names[] = {"backward", "forward", "collision"};

struct Stats {
  Count visits = 0, edge_steps = 0, advance_pieces = 0, indexed_seeks = 0,
        seek_cmps = 0, sort_keys = 0, sort_cmps = 0, join_steps = 0,
        owner_steps = 0, survivor_writes = 0, hit_spans = 0,
        peak_packet_slots = 0, phases = 0;
};
struct Run {
  Byte byte;
  U width;
};
struct Piece {
  U lo, hi, dest;
  Byte byte;
};

struct Map {
  std::vector<Piece> table;
  Map(const std::vector<Run> &runs, bool forward) {
    std::array<U, 256> next{}, slot{};
    for (auto r : runs) {
      next[r.byte] += r.width;
      ++slot[r.byte];
    }
    std::exclusive_scan(next.begin(), next.end(), next.begin(), 0u);
    std::exclusive_scan(slot.begin(), slot.end(), slot.begin(), 0u);
    U row = 0;
    table.resize(runs.size());
    for (U j = 0; j < runs.size(); ++j) {
      auto r = runs[j];
      U source = next[r.byte];
      if (forward)
        table[slot[r.byte]++] = {source, source + r.width, row, r.byte};
      else
        table[j] = {row, row + r.width, source, r.byte};
      next[r.byte] += r.width;
      row += r.width;
    }
  }
  U locate(U row, U cache, Stats &s) const {
    if (table[cache].lo <= row && row < table[cache].hi)
      return cache;
    if (cache + 1 < table.size() && table[cache + 1].lo <= row &&
        row < table[cache + 1].hi)
      return cache + 1;
    ++s.indexed_seeks;
    auto it = std::upper_bound(table.begin(), table.end(), row,
                               [&](U x, const Piece &p) {
                                 ++s.seek_cmps;
                                 return x < p.lo;
                               });
    return U(it - table.begin() - 1);
  }
};
struct Node {
  U prev;
  Byte b;
};
struct Trail {
  std::unordered_map<Count, U> ids;
  std::vector<Node> nodes{{0, 0}};
  U append(U old, Byte b) {
    Count key = (Count(old) << 8) | b;
    auto [it, inserted] = ids.try_emplace(key, U(nodes.size()));
    if (inserted)
      nodes.push_back({old, b});
    return it->second;
  }
  Word read(U id, bool reverse) const {
    Word out;
    while (id) {
      auto v = nodes[id];
      out.push_back(v.b);
      id = v.prev;
    }
    if (reverse)
      std::reverse(out.begin(), out.end());
    return out;
  }
};
struct Packet {
  U row, n, anchor, word;
  U end() const { return row + n; }
  void consume(U k) {
    row += k;
    anchor += k;
    n -= k;
  }
};
using Front = std::vector<Packet>;
void add(Front &v, Packet p) {
  if (!v.empty()) {
    auto &q = v.back();
    if (q.end() == p.row && q.anchor + q.n == p.anchor && q.word == p.word) {
      q.n += p.n;
      return;
    }
  }
  v.push_back(p);
}
struct Band {
  U source, dest, n, word;
};
struct Construction {
  Mode mode;
  Stats s;
  std::optional<Map> psi, lf;
  Trail forward, back;
  std::unordered_map<Count, U> paired_words;
  std::vector<Word> words;
  std::vector<Band> bands;
  U dictionary_bytes = 0, raw_bands = 0;
  explicit Construction(Mode m) : mode(m) {}

  void hit(U x, U y, U width, U a, U b) {
    ++s.hit_spans;
    Count key = (Count(a) << 32) | b;
    auto [it, new_word] = paired_words.try_emplace(key, U(words.size()));
    if (new_word) {
      auto text = forward.read(a, true), tail = back.read(b, false);
      if (mode == Collision)
        assert(text.size() == tail.size() || text.size() == tail.size() + 1);
      text.insert(text.end(), tail.begin(), tail.end());
      assert(!text.empty());
      dictionary_bytes += text.size();
      words.push_back(std::move(text));
    }
    bands.push_back({x, y, width, it->second});
  }
  Front advance(const Front &input, const Map &map, Trail &trail, bool sort) {
    Front next;
    U cache = 0;
    for (auto q : input) {
      while (q.n) {
        cache = map.locate(q.row, cache, s);
        auto p = map.table[cache];
        U n = std::min(q.n, p.hi - q.row);
        ++s.advance_pieces;
        s.edge_steps += n;
        add(next,
            {p.dest + q.row - p.lo, n, q.anchor, trail.append(q.word, p.byte)});
        q.consume(n);
      }
    }
    if (sort) {
      s.sort_keys += next.size();
      std::sort(next.begin(), next.end(),
                [&](const Packet &a, const Packet &b) {
                  ++s.sort_cmps;
                  return a.row < b.row;
                });
      size_t used = 0;
      for (auto p : next) {
        if (used) {
          auto &prev = next[used - 1];
          assert(prev.end() <= p.row);
          if (prev.end() == p.row && prev.anchor + prev.n == p.anchor &&
              prev.word == p.word) {
            prev.n += p.n;
            continue;
          }
        }
        next[used++] = p;
      }
      next.resize(used);
    }
    return next;
  }
  void intersect(Front &f, Front &b) {
    Front f_rem, b_rem;
    size_t i = 0, j = 0;
    while (i < f.size() && j < b.size()) {
      ++s.join_steps;
      auto &p = f[i];
      auto &q = b[j];
      if (p.end() <= q.row) {
        add(f_rem, p);
        ++s.survivor_writes;
        ++i;
        continue;
      }
      if (q.end() <= p.row) {
        add(b_rem, q);
        ++s.survivor_writes;
        ++j;
        continue;
      }
      if (p.row < q.row) {
        U n = q.row - p.row;
        add(f_rem, {p.row, n, p.anchor, p.word});
        p.consume(n);
        ++s.survivor_writes;
      }
      if (q.row < p.row) {
        U n = p.row - q.row;
        add(b_rem, {q.row, n, q.anchor, q.word});
        q.consume(n);
        ++s.survivor_writes;
      }
      U width = std::min(p.n, q.n);
      hit(p.anchor, q.anchor, width, p.word, q.word);
      p.consume(width);
      q.consume(width);
      if (!p.n)
        ++i;
      if (!q.n)
        ++j;
    }
    while (i < f.size()) {
      add(f_rem, f[i++]);
      ++s.survivor_writes;
    }
    while (j < b.size()) {
      add(b_rem, b[j++]);
      ++s.survivor_writes;
    }
    s.peak_packet_slots = std::max<Count>(
        s.peak_packet_slots,
        f.capacity() + b.capacity() + f_rem.capacity() + b_rem.capacity());
    f = std::move(f_rem);
    b = std::move(b_rem);
  }
  Front retain_or_emit(const Front &v, const Section &section, bool fw) {
    Front rem;
    for (auto q : v) {
      auto it = std::upper_bound(section.begin(), section.end(), q.row,
                                 [&](U r, const auto &x) {
                                   ++s.owner_steps;
                                   return r < x.second;
                                 });
      while (q.n) {
        ++s.owner_steps;
        if (it == section.end() || q.row < it->first) {
          U n = it == section.end() ? q.n : std::min(q.n, it->first - q.row);
          add(rem, {q.row, n, q.anchor, q.word});
          ++s.survivor_writes;
          q.consume(n);
        } else {
          U n = std::min(q.n, it->second - q.row);
          if (fw)
            hit(q.anchor, q.row, n, q.word, 0);
          else
            hit(q.row, q.anchor, n, 0, q.word);
          q.consume(n);
          if (q.row == it->second)
            ++it;
        }
      }
    }
    s.peak_packet_slots =
        std::max<Count>(s.peak_packet_slots, v.capacity() + rem.capacity());
    return rem;
  }
  void run(const std::vector<Run> &runs, const Section &section) {
    if (mode != Back)
      psi.emplace(runs, true);
    if (mode != Forward)
      lf.emplace(runs, false);
    Front seed;
    for (auto [lo, hi] : section) {
      seed.push_back({lo, hi - lo, lo, 0});
      s.visits += hi - lo;
    }
    if (mode == Collision) {
      Front f = seed, b = std::move(seed);
      while (!f.empty()) {
        bool fw = s.phases++ % 2 == 0;
        auto &active = fw ? f : b;
        Front next =
            advance(active, fw ? *psi : *lf, fw ? forward : back, true);
        s.peak_packet_slots = std::max<Count>(
            s.peak_packet_slots, f.capacity() + b.capacity() + next.capacity());
        active = std::move(next);
        intersect(f, b);
        assert(f.empty() == b.empty());
      }
    } else {
      bool fw = mode == Forward;
      Front active = std::move(seed);
      while (!active.empty()) {
        ++s.phases;
        Front next =
            advance(active, fw ? *psi : *lf, fw ? forward : back, false);
        s.peak_packet_slots = std::max<Count>(
            s.peak_packet_slots, active.capacity() + next.capacity());
        Front{}.swap(active);
        active = retain_or_emit(next, section, fw);
      }
    }
    raw_bands = bands.size();
    s.sort_keys += bands.size();
    std::sort(bands.begin(), bands.end(), [&](const Band &a, const Band &b) {
      ++s.sort_cmps;
      return a.source < b.source;
    });
    size_t used = 0;
    for (auto x : bands) {
      if (used) {
        auto &y = bands[used - 1];
        assert(y.source + y.n <= x.source);
        if (y.source + y.n == x.source && y.dest + y.n == x.dest &&
            y.word == x.word) {
          y.n += x.n;
          continue;
        }
      }
      bands[used++] = x;
    }
    bands.resize(used);
  }
  const Band *find(U row) const {
    auto it = std::upper_bound(bands.begin(), bands.end(), row,
                               [](U x, const Band &b) { return x < b.source; });
    if (it == bands.begin())
      return nullptr;
    const auto &b = *--it;
    return row < b.source + b.n ? &b : nullptr;
  }
  Word output(U mark, U length, bool *reaches = nullptr) const {
    const bool fw = mode == Forward;
    const Map &map = fw ? *psi : *lf;
    const Band *b = find(mark);
    U row = mark, skipped = 0, cache = 0;
    Stats ignored;
    Word result;
    result.reserve(length);
    auto step = [&]() {
      cache = map.locate(row, cache, ignored);
      auto piece = map.table[cache];
      row = piece.dest + row - piece.lo;
      return piece.byte;
    };
    while (!b) {
      auto byte = step();
      if (fw)
        result.push_back(byte);
      ++skipped;
      if (row == mark) {
        // This orbit contains no section rank. Recover its exact period on the
        // retained map.
        if (!fw) {
          result.resize(skipped);
          for (U j = skipped; j > 0; --j)
            result[j - 1] = step();
        }
        for (U j = skipped; j < length; ++j)
          result.push_back(result[j - skipped]);
        result.resize(length);
        if (reaches)
          *reaches = false;
        return result;
      }
      b = find(row);
    }
    if (reaches)
      *reaches = true;
    if (fw)
      skipped = 0;
    while (result.size() < length) {
      const auto &w = words[b->word];
      assert(skipped < w.size());
      auto count = std::min(w.size() - skipped, length - result.size());
      result.insert(result.end(), w.begin() + skipped,
                    w.begin() + skipped + count);
      skipped = 0;
      row = b->dest + row - b->source;
      b = find(row);
      assert(b);
    }
    return result;
  }
};

struct Oracle {
  U n;
  Word l, f;
  std::vector<U> psi;
  explicit Oracle(const std::vector<Run> &runs) {
    for (auto r : runs)
      l.insert(l.end(), r.width, r.byte);
    n = l.size();
    psi.resize(n);
    std::iota(psi.begin(), psi.end(), 0u);
    std::stable_sort(psi.begin(), psi.end(),
                     [&](U a, U b) { return l[a] < l[b]; });
    for (auto x : psi)
      f.push_back(l[x]);
  }
  void check(const Construction &c, const Section &section) const {
    std::vector<bool> in(n);
    for (auto [x, y] : section)
      std::fill(in.begin() + x, in.begin() + y, true);
    U visits = 0;
    Count mass = 0;
    for (auto b : c.bands)
      for (U k = 0; k < b.n; k++) {
        U x = b.source + k;
        assert(in[x]);
        const auto &w = c.words[b.word];
        for (U i = 0; i < w.size(); i++) {
          assert(w[i] == f[x]);
          x = psi[x];
          assert(in[x] == (i + 1 == w.size()));
        }
        assert(x == b.dest + k);
        ++visits;
        mass += w.size();
      }
    assert(visits == std::count(in.begin(), in.end(), true) &&
           mass == c.s.edge_steps && mass <= n);
  }
  bool marked(const Construction &c, const Section &section, U p) const {
    bool actual_reaches;
    auto out = c.output(p, n, &actual_reaches);
    std::vector<bool> in(n);
    for (auto [x, y] : section)
      std::fill(in.begin() + x, in.begin() + y, true);
    U x = p;
    Word expected;
    bool reaches = false;
    for (U i = 0; i < n; i++) {
      reaches |= in[x];
      expected.push_back(f[x]);
      x = psi[x];
    }
    assert(actual_reaches == reaches && out == expected);
    return reaches;
  }
};

double cpu() {
  timespec t{};
  clock_gettime(CLOCK_THREAD_CPUTIME_ID, &t);
  return t.tv_sec + t.tv_nsec * 1e-9;
}
int main(int argc, char **argv) {
  if (argc == 2 && std::string(argv[1]) == "small") {
    Count columns = 0, sections = 0, marks = 0, disjoint = 0, nond = 0;
    for (U alphabet : {2u, 3u})
      for (U n = 1; n <= (alphabet == 2 ? 8u : 6u); n++) {
        U count = 1;
        for (U i = 0; i < n; i++)
          count *= alphabet;
        for (U code = 0; code < count; code++) {
          U t = code;
          Word l(n);
          bool uses2 = false;
          for (auto &b : l) {
            b = t % alphabet;
            uses2 |= b == 2;
            t /= alphabet;
          }
          if (alphabet == 3 && !uses2)
            continue;
          std::vector<Run> runs;
          for (auto b : l) {
            if (runs.empty() || runs.back().byte != b)
              runs.push_back({b, 1});
            else
              ++runs.back().width;
          }
          ++columns;
          Oracle o(runs);
          for (U mask = 0; mask < (1u << n); mask++) {
            Section sec;
            for (U j = 0; j < n; j++)
              if (mask >> j & 1) {
                if (!sec.empty() && sec.back().second == j)
                  ++sec.back().second;
                else
                  sec.push_back({j, j + 1});
              }
            ++sections;
            Construction c(Collision);
            c.run(runs, sec);
            o.check(c, sec);
            for (U p = 0; p < n; p++) {
              ++marks;
              bool reaches = o.marked(c, sec, p);
              disjoint += !reaches;
              U period = 1;
              for (U x = o.psi[p]; x != p; x = o.psi[x])
                ++period;
              nond += reaches && n % period != 0;
            }
          }
        }
      }
    std::cout << "columns " << columns << " arbitrary_sections " << sections
              << " all_marks " << marks << " disjoint " << disjoint
              << " nondividing_reachable " << nond << "\n";
    return 0;
  }
  if (argc != 5)
    throw std::runtime_error(
        "collision TAPE_INPUT LAPS ORDINAL EXPECTED_BYTES");
  std::ifstream file(argv[1]);
  U n, p, nruns, nsec;
  file >> n >> p >> nruns >> nsec;
  if (!file)
    throw std::runtime_error("input");
  std::vector<Run> runs;
  for (U i = 0; i < nruns; i++) {
    U b, w;
    file >> b >> w;
    runs.push_back({Byte(b), w});
  }
  Oracle o(runs);
  assert(n == o.n);
  std::ifstream gold(argv[4], std::ios::binary);
  Word expected((std::istreambuf_iterator<char>(gold)), {});
  assert(expected.size() == n);
  for (U k = 0; k < nsec; k++) {
    std::string name;
    U count;
    file >> name >> count;
    Section sec;
    for (U j = 0; j < count; j++) {
      U a, b;
      file >> a >> b;
      sec.push_back({a, b});
    }
    int order[] = {0, 1, 2};
    int laps = std::stoi(argv[2]);
    for (int lap = 0; lap < laps;
         lap++, std::next_permutation(order, order + 3)) {
      for (int m : order) {
        double start = cpu();
        Construction c{Mode(m)};
        c.run(runs, sec);
        double built = cpu();
        auto bytes = c.output(p, n);
        double done = cpu();
        o.check(c, sec);
        assert(o.marked(c, sec, p));
        assert(bytes == expected);
        auto &s = c.s;
        std::cout << argv[3] << ',' << name << ',' << names[m] << ',' << lap
                  << ',' << runs.size() << ',' << s.visits << ',' << s.phases
                  << ',' << s.edge_steps << ',' << s.advance_pieces << ','
                  << s.indexed_seeks << ',' << s.seek_cmps << ',' << s.sort_keys
                  << ',' << s.sort_cmps << ',' << s.join_steps << ','
                  << s.owner_steps << ',' << s.survivor_writes << ','
                  << s.hit_spans << ',' << c.bands.size() << ','
                  << c.forward.nodes.size() - 1 << ','
                  << c.back.nodes.size() - 1 << ',' << c.words.size() << ','
                  << c.dictionary_bytes << ',' << s.peak_packet_slots << ','
                  << built - start << ',' << done - built << '\n';
      }
    }
  }
}
