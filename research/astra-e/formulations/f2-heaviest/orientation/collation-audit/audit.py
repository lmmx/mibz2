"""Independent enumeration of source ranks, first returns, and periodic continuation keys."""
import argparse
from collections import defaultdict
from fractions import Fraction
from itertools import product, combinations
import json


def permutation(last):
    # Stable row ordering by the pair (byte, original occurrence rank).
    psi = sorted(range(len(last)), key=lambda i: (last[i], i))
    first = bytes(last[i] for i in psi)
    return psi, first


def orbit(psi, first, p):
    ranks = []
    i = p
    while True:
        ranks.append(i)
        i = psi[i]
        if i == p:
            return ranks, bytes(first[j] for j in ranks)


def cylinders(orbits, rows):
    # List exact nonempty prefix cylinders, including cylinders of tied codes.
    n = len(rows)
    result = {(0,n)}
    prefixes = [bytearray() for _ in rows]
    for depth in range(n*n+1):
        for prefix, (_,period) in zip(prefixes,orbits,strict=True):
            prefix.append(period[depth%len(period)])
        cuts = [0, *(i for i in range(1,n) if prefixes[i] != prefixes[i-1]), n]
        result.update(zip(cuts,cuts[1:]))
        if all(rows[lo]==rows[hi-1] for lo,hi in zip(cuts,cuts[1:])):
            return result
    raise AssertionError("periodic words did not separate")


def returns(psi, first, section):
    chosen = set(section)
    result = []
    for start in section:
        word = bytearray()
        i = start
        while True:
            word.append(first[i])
            i = psi[i]
            if i in chosen:
                result.append((bytes(word), i))
                break
    return result


def real_key(word, period):
    # Exact fraction for 0.(word)(period)(period)... in base 4. Input digits
    # are at most 2, so two expansions cannot coincide via a trailing base-1.
    value = lambda digits: sum(b * 4**i for i,b in enumerate(digits[::-1]))
    return Fraction(value(word) * (4**len(period)-1) + value(period),
                    4**len(word) * (4**len(period)-1))


def finite_compare(a, b, ranks, period):
    common = min(len(a), len(b))
    if a[:common] != b[:common]:
        return (a[:common] > b[:common]) - (a[:common] < b[:common])
    if len(a) == len(b):
        return 0
    if len(a) > len(b):
        return -finite_compare(b, a, ranks, period)
    tail = b[len(a):]
    qprefix = (period * ((len(tail)+len(period)-1)//len(period)))[:len(tail)]
    if qprefix != tail:
        return (qprefix > tail) - (qprefix < tail)
    old, new = ranks[0], ranks[len(tail)%len(ranks)]
    return (new > old) - (new < old)


def witness(last, section, q, src, rhs, a, b, cmp, numeric):
    return dict(last=list(last),section=list(section),q=q,earlier_source=src,later_source=rhs,
                earlier_word=list(a),later_word=list(b),comparison=cmp,numeric=numeric)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary",type=int,default=9)
    parser.add_argument("--ternary",type=int,default=6)
    args = parser.parse_args()
    counts = defaultdict(int)
    failures = {}
    for n in range(1,max(args.binary,args.ternary)+1):
        alphabets = [k for k,limit in [(2,args.binary),(3,args.ternary)] if n<=limit]
        for alphabet in alphabets:
            for last in product(range(alphabet),repeat=n):
                if alphabet==3 and 2 in alphabets and 2 not in last:
                    continue
                last=bytes(last)
                psi, first = permutation(last)
                orbits = [orbit(psi,first,q) for q in range(n)]
                counts["columns"]+=1
                rows = [real_key(b"", values) for _,values in orbits]
                assert rows == sorted(rows)
                rank_cylinders = cylinders(orbits,rows)
                cycles = [min(ranks) for ranks,_ in orbits]
                counts["multicycle_columns"] += any(len(ranks)<n for ranks,_ in orbits)
                counts["tied_code_columns"] += len(set(rows))<n
                # Cache exact infinite keys across every use of the same periodic tail.
                keys = {}
                for lo in range(n):
                    for hi in range(lo+1,n+1):
                        sec=range(lo,hi)
                        links=returns(psi,first,sec)
                        words=[word for word,_ in links]
                        maxlen=max(map(len,words))
                        counts["intervals"]+=1
                        counts["noncylinder_intervals"] += (lo,hi) not in rank_cylinders
                        counts["intervals_spanning_cycles"] += len(set(cycles[lo:hi]))>1
                        counts["intervals_cutting_tied_rotations"] += ((lo>0 and rows[lo-1]==rows[lo]) or
                                                                      (hi<n and rows[hi-1]==rows[hi]))
                        for q in range(n):
                            ok=lo<=q<hi
                            if not ok and {"outside_equal","outside_reverse"}<=failures.keys():
                                continue
                            ranks,period=orbits[q]
                            values=[]
                            for word in words:
                                key=word,period
                                if key not in keys:
                                    keys[key]=real_key(word,period)
                                values.append(keys[key])
                            if ok:
                                counts["inside_reference_marks"]+=1
                                for j in range(1,len(words)):
                                    a,b=words[j-1:j+1]
                                    k=(values[j-1]>values[j])-(values[j-1]<values[j])
                                    assert finite_compare(a,b,ranks,period)==k
                                    assert k==(0 if a==b else -1), (last,lo,hi,q,j)
                                    counts["adjacent_checks"]+=1
                                    if a!=b and (a.startswith(b) or b.startswith(a)):
                                        counts["prefix_checks"]+=1
                                        counts["reference_in_other_cycle_from_both_sources"] += cycles[q] not in {cycles[lo+j-1],cycles[lo+j]}
                                        counts["unequal_prefix_returns_at_tied_source_codes"] += rows[lo+j-1]==rows[lo+j]
                                        short,long=sorted([a,b],key=len)
                                        v=long[len(short):]
                                        if (period*((len(v)+len(period)-1)//len(period)))[:len(v)]==v:
                                            counts["rank_tiebreak_checks"]+=1
                                if "finite_suffix_wrong" not in failures:
                                    prefix=(period*((maxlen+len(period)-1)//len(period)))[:maxlen]
                                    for j in range(1,len(words)):
                                        a,b=words[j-1:j+1]
                                        if a!=b and a+prefix >= b+prefix:
                                            failures["finite_suffix_wrong"]=witness(last,sec,q,lo+j-1,lo+j,a,b,
                                                "finite padded order differs from true source order",[list(prefix)])
                            else:
                                for j in range(1,len(words)):
                                    if words[j-1]==words[j] or values[j-1]<values[j]:continue
                                    tag="outside_equal" if values[j-1]==values[j] else "outside_reverse"
                                    if tag not in failures:
                                        failures[tag]=witness(last,sec,q,lo+j-1,lo+j,*words[j-1:j+1],tag,
                                                               [str(x) for x in values[j-1:j+1]])
                for size in range(2,n+1):
                    if {"hole_equal","hole_reverse","hole_split_bucket"}<=failures.keys():break
                    for sec in combinations(range(n),size):
                        if sec[-1]-sec[0]+1==size:continue
                        words=[w for w,_ in returns(psi,first,sec)]
                        for j in range(2,len(words)):
                            if words[j] in words[:j-1] and words[j]!=words[j-1] and "hole_split_bucket" not in failures:
                                failures["hole_split_bucket"]=dict(last=list(last),section=list(sec),
                                    source_ordered_words=[list(w) for w in words])
                        for q in sec:
                            period=orbits[q][1]
                            vals=[real_key(w,period) for w in words]
                            for j in range(1,len(words)):
                                if words[j-1]==words[j] or vals[j-1]<vals[j]:continue
                                tag="hole_equal" if vals[j-1]==vals[j] else "hole_reverse"
                                if tag not in failures:
                                    failures[tag]=witness(last,sec,q,sec[j-1],sec[j],*words[j-1:j+1],tag,
                                                          [str(x) for x in vals[j-1:j+1]])
    print(json.dumps(dict(limits=vars(args),counts=counts,counterexamples=failures),indent=2))


if __name__=="__main__":
    main()
