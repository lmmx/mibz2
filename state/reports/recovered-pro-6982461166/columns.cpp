#include <algorithm>
#include <cassert>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <vector>

using Bytes = std::vector<uint8_t>;
constexpr uint64_t block_sync = 0x314159265359ULL;
constexpr uint64_t mask = (1ULL << 48)-1;

struct Bits {
    const Bytes& bytes;
    size_t at;
    unsigned read(unsigned n) {
        if (at+n > bytes.size()*8) throw std::out_of_range("window ends inside block");
        unsigned value=0;
        while (n) {
            unsigned avail=8-(at%8);
            unsigned take=std::min(n,avail);
            unsigned b=(bytes[at/8]>>(avail-take))&((1<<take)-1);
            value=(value<<take)|b;
            at+=take;
            n-=take;
        }
        return value;
    }
};

struct Codes {
    std::vector<int> len, values;
    int minlen=21,maxlen=0;
    int limit[21]{}, base[21]{};
    Codes(Bits& b, unsigned count) {
        int k=b.read(5);
        len.resize(count);
        for (unsigned i=0;i<count;++i) {
            while (b.read(1)) k+=b.read(1)?-1:1;
            if (k<1 || k>20) throw std::runtime_error("invalid Huffman length");
            len[i]=k;
            minlen=std::min(minlen,k);
            maxlen=std::max(maxlen,k);
        }
        int code=0;
        for (int h=minlen;h<=maxlen;++h) {
            base[h]=code-int(values.size());
            for (unsigned i=0;i<count;++i)
                if (len[i]==h) {values.push_back(i);++code;}
            limit[h]=code-1;
            code <<= 1;
        }
    }
    unsigned decode(Bits& b) const {
        int code=b.read(minlen);
        for (int h=minlen;h<=maxlen;++h) {
            if (code<=limit[h]) return values.at(code-base[h]);
            if (h<maxlen) code=2*code+b.read(1);
        }
        throw std::runtime_error("invalid Huffman code");
    }
};

Bytes column(Bits& b, uint32_t& primary) {
    b.read(32);
    if (b.read(1)) throw std::runtime_error("randomised block");
    primary=b.read(24);
    unsigned groups=b.read(16);
    Bytes live;
    for (unsigned k=0;k<16;++k)
        if (groups&(1u<<(15-k))) {
            unsigned rows=b.read(16);
            for (unsigned j=0;j<16;++j)
                if (rows&(1u<<(15-j))) live.push_back(k*16+j);
        }
    unsigned alphabet=live.size()+2, ntables=b.read(3), nselectors=b.read(15);
    if (live.empty() || ntables<2 || ntables>6 || nselectors==0)
        throw std::runtime_error("invalid block alphabet or selector count");
    Bytes mtf, selectors;
    for (unsigned i=0;i<ntables;++i) mtf.push_back(i);
    for (unsigned i=0;i<nselectors;++i) {
        unsigned k=0;
        while (b.read(1)) ++k;
        auto tab=mtf.at(k);
        std::rotate(mtf.begin(),mtf.begin()+k,mtf.begin()+k+1);
        selectors.push_back(tab);
    }
    std::vector<Codes> tables;
    for (unsigned i=0;i<ntables;++i) tables.emplace_back(b,alphabet);
    Bytes out;
    out.reserve(900000);
    unsigned run=0, weight=1;
    for (unsigned i=0;;++i) {
        auto sym=tables[selectors.at(i/50)].decode(b);
        if (sym<2) {
            run+=(sym+1)*weight;
            weight*=2;
            if (run>900000) throw std::runtime_error("oversized run");
            continue;
        }
        if (run) {
            out.insert(out.end(),run,live.front());
            run=0;
            weight=1;
        }
        if (sym==alphabet-1) break;
        unsigned index=sym-1;
        out.push_back(live.at(index));
        std::rotate(live.begin(),live.begin()+index,live.begin()+index+1);
        if (out.size()>900000) throw std::runtime_error("oversized block");
    }
    if (out.size()>900000 || primary>=out.size()) throw std::runtime_error("invalid final column");
    return out;
}

int main(int argc,char** argv) {
    for (int i=1;i<argc;++i) {
        std::ifstream input(argv[i],std::ios::binary);
        Bytes data((std::istreambuf_iterator<char>(input)),{});
        size_t next=0;
        int blocks=0;
        while (next+48 <= data.size()*8) {
            uint64_t roll=0;
            Bits b{data,next};
            while (b.at < data.size()*8 && roll!=block_sync)
                roll=((roll<<1)|b.read(1))&mask;
            if (roll!=block_sync) break;
            try {
                uint32_t mark;
                Bytes out=column(b,mark);
                uint32_t n=out.size();
                next=b.at;
                ++blocks;
                std::cout.write(reinterpret_cast<const char*>(&n),4);
                std::cout.write(reinterpret_cast<const char*>(&mark),4);
                std::cout.write(reinterpret_cast<const char*>(out.data()),out.size());
            } catch (const std::out_of_range&) {
                if (b.at+24<data.size()*8) throw;
                break;
            }
        }
        std::cerr<<argv[i]<<": "<<blocks<<" blocks\n";
    }
}
