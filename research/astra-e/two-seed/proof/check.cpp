// Independent checked model of source-labelled first returns and the two-forward cursor.
// This is an exactness test. It makes no machine timing measurements.
#include <algorithm>
#include <array>
#include <cassert>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <numeric>

using Byte = std::uint8_t;
constexpr int max_rows = 8;
constexpr int max_bytes = 80;
using Map = std::array<int, max_rows>;
using Output = std::array<Byte, max_bytes>;
struct Word {
    std::array<Byte, max_rows> bytes{};
    int len = 0;
};
struct Graph {
    int m;
    Map next{};
    std::array<Word, max_rows> word{};
};

static unsigned long long drops, arrivals[3], exact_checks, direct_checks, proper_repeat;
static int max_combined;

Output ordinary(const Graph& g, int p, int n) {
    Output output{};
    int pos = 0;
    while (pos < n) {
        for (int i = 0; i < g.word[p].len && pos < n; ++i)
            output[pos++] = g.word[p].bytes[i];
        p = g.next[p];
    }
    return output;
}

Output emit(const Graph& g, int p, int s, int n, int schedule, bool first_return_bound) {
    Output out{}; // Exactly indices [0,n) may be used.
    int a = p, b = s, left = 0, right = n;
    bool a_live = true, b_live = p != s, dropped = false;
    bool found_a_to_s = false, found_b_to_p = false;
    auto discard_b = [&]() { right = n; b_live = false; };
    auto a_step = [&]() {
        const Word& word = g.word[a];
        if (left + word.len > right) {
            assert(!first_return_bound);
            if (b_live || right < n) { discard_b(); dropped = true; }
        }
        for (int i = 0; i < word.len && left < n; ++i) out[left++] = word.bytes[i];
        a = g.next[a];
        found_a_to_s = a == s && s != p;
        if (a == p || left == n || (found_a_to_s && !dropped)) a_live = false;
    };
    auto b_step = [&]() {
        const Word& word = g.word[b];
        if (left + word.len > right) {
            assert(!first_return_bound);
            discard_b();
            dropped = true;
            // A can already be waiting at s. Continue its marked walk from there.
            if (a != p && left < n) a_live = true;
            return;
        }
        for (int i = 0; i < word.len; ++i) out[--right] = word.bytes[i];
        b = g.next[b];
        if (b == p || b == s) b_live = false;
        if (b == p) found_b_to_p = true;
        if (b == s) discard_b();
    };
    int tick = 0;
    while (a_live || b_live) {
        // Alternating and B-first, best- and worst-skewed completion orders.
        bool take_a = !b_live || (a_live && (schedule == 2 ||
                         (schedule != 3 && (tick + schedule) % 2 == 0)));
        if (take_a) {
            a_step();
            // A closed its own cycle, or already filled the request.
            if (!a_live && (a == p || left == n)) discard_b();
        } else {
            b_step();
        }
        ++tick;
        assert(left <= right && tick <= 4 * g.m + 4);
        max_combined = std::max(max_combined, left + n - right);
    }
    if (left < n && !dropped && found_a_to_s) {
        assert(found_b_to_p && right < n);
        std::reverse(out.begin() + right, out.begin() + n);
        // left <= right. A forward copy is safe even when ranges overlap.
        for (int i = right; i < n; ++i) out[left++] = out[i];
    }
    if (dropped) ++drops;
    if (p == s) ++arrivals[0];
    else if (found_a_to_s && found_b_to_p && !dropped) ++arrivals[1];
    else ++arrivals[2];
    assert(left > 0 && left <= n);
    if (left < n && n % left != 0) ++proper_repeat;
    const int period = left;
    while (left < n) { out[left] = out[left - period]; ++left; }
    return out;
}

// Keep both fragments forward-oriented. The first exhausted reservation certifies
// that the other fragment can finish inside its reservation. Capacity is a byte
// bound, independent of how a scheduler interleaves the two cursors.
static unsigned long long forward_checks, first_overflow[3];
Output forward_split(const Graph& g, int p, int s, int n, int k, int schedule) {
    Output out{};
    int a = p, b = s, na = 0, nb = 0, base = k, acap = k, bcap = n - k;
    bool took_a = false, took_b = false;
    int overflow = 0, tick = 0;
    auto a_done = [&]() { return took_a && (a == p || a == s); };
    auto b_done = [&]() { return took_b && (b == p || b == s); };
    auto append = [&](int& row, int at, int& size) {
        const Word& w = g.word[row];
        assert(at >= 0 && at + size + w.len <= n);
        std::copy_n(w.bytes.begin(), w.len, out.begin() + at + size);
        row = g.next[row]; size += w.len;
    };
    if (p == s) {
        do { append(a, 0, na); } while (a != p);
    } else {
        while (!a_done() && !b_done()) {
            bool choose_a = overflow == 2 || (overflow == 0 &&
                (schedule == 2 || (schedule != 3 && (tick + schedule) % 2 == 0)));
            if (choose_a) {
                if (na + g.word[a].len > acap) { assert(overflow == 0); overflow = 1; }
                else { append(a, 0, na); took_a = true; }
            } else {
                if (nb + g.word[b].len > bcap) { assert(overflow == 0); overflow = 2; }
                else { append(b, base, nb); took_b = true; }
            }
            assert(++tick <= 2 * g.m + 2);
        }
        ++first_overflow[overflow];
        if (b_done() && b == s) {
            // The speculative fragment is another cycle. Reclaim its reservation.
            do { append(a, 0, na); took_a = true; } while (!a_done());
            assert(a == p);
        } else if (a_done()) {
            if (a == s) {
                // The exact start of B is now known: place its prefix and finish.
                assert(na <= k);
                std::memmove(out.data() + na, out.data() + k, nb);
                while (!b_done()) { append(b, na, nb); took_b = true; }
                assert(b == p);
                na += nb;
            }
        } else {
            assert(b_done() && b == p);
            // b is complete. If A unexpectedly crosses k later, right-align b.
            while (!a_done()) {
                if (na + g.word[a].len > base) {
                    assert(base == k && n - nb >= k);
                    std::memmove(out.data() + n - nb, out.data() + base, nb);
                    base = n - nb;
                }
                assert(na + g.word[a].len <= base);
                append(a, 0, na); took_a = true;
            }
            assert(a == s);
            if (na != base) std::memmove(out.data() + na, out.data() + base, nb);
            na += nb;
        }
    }
    assert(na > 0 && na <= n);
    const int period = na;
    while (na < n) { out[na] = out[na - period]; ++na; }
    return out;
}

void check(const Graph& g, int p, int s, int n, bool bound, const Output* raw = nullptr) {
    const Output ref = ordinary(g, p, n);
    if (raw) { assert(std::equal(ref.begin(), ref.begin() + n, raw->begin())); ++direct_checks; }
    for (int schedule = 0; schedule < 4; ++schedule) {
        const auto got = emit(g, p, s, n, schedule, bound);
        assert(std::equal(ref.begin(), ref.begin() + n, got.begin()));
        ++exact_checks;
        if (bound) {
            // Every split on raw first-return graphs; centered and varying split
            // (including both ends) on every independently weighted permutation.
            int limits[2] = {n / 2, (p + 3*s + n) % (n+1)};
            auto test = [&](int split) {
                auto f = forward_split(g,p,s,n,split,schedule);
                assert(std::equal(ref.begin(), ref.begin()+n, f.begin()));
                ++forward_checks;
            };
            if (raw) { for (int split=0; split <= n; ++split) test(split); }
            else { for (const int split : limits) test(split); }
        }
    }
}

// Every arbitrary permutation through six rows; for every row independently choose
// 1 or 3 emitted bytes, so multi-byte words have an observable orientation.
void arbitrary_weights() {
    for (int m = 1; m <= 6; ++m) {
        Graph g{m};
        std::iota(g.next.begin(), g.next.begin() + m, 0);
        do {
            for (int mask = 0; mask < (1 << m); ++mask) {
                int total = 0;
                for (int row = 0; row < m; ++row) {
                    Word& w = g.word[row];
                    w.len = (mask & (1 << row)) ? 3 : 1;
                    for (int i = 0; i < w.len; ++i)
                        w.bytes[i] = static_cast<Byte>(29 * row + i);
                    total += w.len;
                }
                for (int p = 0; p < m; ++p) for (int s = 0; s < m; ++s) {
                    check(g, p, s, total, true);
                    // Missing/wider abstract area beyond requested size: word splits,
                    // wraparound truncation and dropped speculative work.
                    int clipped = 1 + (mask + p + s) % total;
                    check(g, p, s, clipped, false);
                }
            }
        } while (std::next_permutation(g.next.begin(), g.next.begin() + m));
    }
}

// Start from a distinct original one-byte edge at every row. For each nonempty
// retained subset independently construct source-labelled induced forward returns.
void original_rows() {
    unsigned long long graphs = 0, omitted = 0;
    for (int n = 1; n <= 6; ++n) {
        Graph full{n}; std::iota(full.next.begin(), full.next.begin() + n, 0);
        for (int row = 0; row < n; ++row) {
            full.word[row].len = 1;
            full.word[row].bytes[0] = static_cast<Byte>(row * 43);
        }
        do {
            for (int mask = 1; mask < (1 << n); ++mask) {
                Graph induced{0}; Map original{}, rank{}, visits{};
                for (int i = 0; i < n; ++i) if ((1 << i) & mask) {
                    rank[i] = induced.m; original[induced.m++] = i;
                }
                int covered = 0;
                for (int i = 0; i < induced.m; ++i) {
                    int j = original[i]; Word& w = induced.word[i];
                    do {
                        assert(++visits[j] == 1);
                        w.bytes[w.len++] = full.word[j].bytes[0];
                        ++covered; j = full.next[j];
                    } while (!(mask & (1 << j)));
                    induced.next[i] = rank[j];
                }
                assert(covered <= n); ++graphs; omitted += covered < n;
                for (int p = 0; p < induced.m; ++p) {
                    const auto direct = ordinary(full, original[p], n);
                    for (int s = 0; s < induced.m; ++s)
                        check(induced, p, s, n, true, &direct);
                }
            }
        } while (std::next_permutation(full.next.begin(), full.next.begin() + n));
    }
    std::printf("original-permutation/retention graphs %llu, omitted whole components %llu, "
                "direct original/induced p,s comparisons %llu\n", graphs, omitted, direct_checks);
}

int main() {
    arbitrary_weights();
    std::printf("arbitrary residual/weight/output-limit comparisons across four schedules %llu\n",exact_checks);
    const auto prior_drops = drops;
    original_rows();
    assert(prior_drops == drops);
    std::printf("all byte-exact schedule comparisons %llu, over-budget speculative drops %llu\n",exact_checks,drops);
    std::printf("forward-reservation exact schedule/split comparisons %llu, "
                "no reservation exceeded %llu, A first %llu, B first %llu (distinct seeds)\n",
                forward_checks,first_overflow[0],first_overflow[1],first_overflow[2]);
    std::printf("same seed %llu, both cut arcs completed %llu, other/short/drop %llu, "
                "non-integral period fills %llu, max staged bytes %d\n",
                arrivals[0],arrivals[1],arrivals[2],proper_repeat,max_combined);
}
