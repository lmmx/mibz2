"""Exhaust strict RLE1 cuts and bounded physical seam bridges against a token oracle."""
import argparse
from collections import Counter
from itertools import combinations, product
import json

RUN_ERROR = "RunUnfinished"
WIDTH = 32


def token_scan(raw):
    """Independent scalar token parser. Withhold the final quartet on error like src/rle.rs."""
    output = bytearray()
    i = 0
    while i < len(raw):
        c = raw[i]
        if i+3 < len(raw) and raw[i+1:i+4] == bytes([c])*3:
            if i+4 == len(raw):
                return bytes(output), RUN_ERROR
            output.extend(bytes([c])*(4+raw[i+4]))
            i += 5
        else:
            output.append(c)
            i += 1
    return bytes(output), None


def split_scan(pieces):
    output = bytearray()
    for piece in pieces:
        segment, error = token_scan(piece)
        output.extend(segment)
        if error:
            return bytes(output), error
    return bytes(output), None


def local_cut(fragment, i, true_start=False):
    """Within a fragment, witness an actual difference among at most four preceding bytes."""
    if i <= 0 or i >= len(fragment) or fragment[i-1] == fragment[i]:
        return False
    seen = fragment[max(0,i-4):i]
    return any(c != seen[-1] for c in seen) or (true_start and i<4)


def plan(fragments, window):
    """Reference arithmetic in full logical coordinates; the implementation probes bounded fragments."""
    raw=b"".join(fragments)
    begin=[]
    end=[]
    total=0
    for part in fragments:
        begin.append(total)
        total+=len(part)
        end.append(total)
    def safe(i):
        return i in (0,len(raw)) or local_cut(raw,i,True)
    prefix=begin.copy()
    suffix=end.copy()
    bridged=0
    for j,seam in enumerate(end[:-1]):
        if safe(seam):
            continue
        a=next((i for i in range(seam-1,max(begin[j],seam-window)-1,-1) if safe(i)),None)
        b=next((i for i in range(seam+1,min(end[j+1],seam+window)+1) if safe(i)),None)
        if a is None or b is None:
            return [raw],0,True
        suffix[j]=a
        prefix[j+1]=b
        assert b-a<=2*window
        bridged+=b-a
    # If both nearest cuts exist, crossing is impossible. This tests the theorem on
    # every admitted partition rather than accepting an overlap fallback.
    assert all(a<=b for a,b in zip(prefix,suffix))
    pieces=[]
    for j in range(len(fragments)):
        pieces.append(raw[prefix[j]:suffix[j]])
        if suffix[j]!=end[j]:
            pieces.append(raw[suffix[j]:prefix[j+1]])
    return pieces,bridged,False


def crc32_bzip2(data):
    crc = 0xffffffff
    for b in data:
        crc ^= b<<24
        for _ in range(8):
            crc = ((crc<<1)^(0x04c11db7 if crc & 0x80000000 else 0)) & 0xffffffff
    return crc^0xffffffff


def finish(scan, stored):
    original = b"existing\0output"
    output, error = scan
    if error:
        return original, error
    crc = crc32_bzip2(output)
    if crc != stored:
        return original, ("BlockCrc",stored,crc)
    return original+output,crc


def exhaustive(max_n):
    counts = Counter()
    # Empty and every reachable (byte,count=1..4) entry type, with an unseen symbol as well.
    prefixes = [b""] + [bytes([b])*k for b in (0,1,2,255) for k in range(1,5)]
    for n in range(max_n+1):
        for word in product((0,1,2),repeat=n):
            raw = bytes(word)
            counts["raw_words"] += 1
            native = token_scan(raw)
            for i in range(1,n):
                if local_cut(raw,i,True):
                    assert split_scan([raw[:i],raw[i:]]) == native
                    counts["known_start_certificates"] += 1
                if local_cut(raw,i):
                    for prefix in prefixes:
                        full = token_scan(prefix+raw)
                        assert split_scan([prefix+raw[:i],raw[i:]]) == full
                        counts["arbitrary_state_certificates"] += 1
                        if full[1]:
                            counts["arbitrary_with_truncated_stream"] += 1
            # Enumerate every one- and two-seam physical partition. Window 2 forces short
            # fallbacks and overlapping neighborhoods; WIDTH is checked separately.
            for k in range(1,min(2,n-1)+1):
                for seams in combinations(range(1,n),k):
                    pos=(0,*seams,n)
                    chunks=[raw[a:b] for a,b in zip(pos,pos[1:])]
                    pieces,copied,fallback = plan(chunks,2)
                    assert b"".join(pieces)==raw and split_scan(pieces)==native
                    counts["physical_partitions"] += 1
                    counts["joined_fallbacks" if fallback else "bounded_paths"] += 1
                    if copied:
                        counts["bridged_paths"] += 1
    return counts


def fixtures():
    assert crc32_bzip2(b"123456789")==0xfc891918
    wrong = bytes([9,9,9,9,0])
    assert token_scan(wrong)==(bytes([9])*4,None)
    assert split_scan([wrong[:4],wrong[4:]])==(b"",RUN_ERROR)
    # The count equals the ordinary byte; an adjacent mismatch exists at the next *valid* cut.
    equal = bytes([4])*5 + bytes([8])
    assert token_scan(equal)==(bytes([4])*8+bytes([8]),None)
    assert split_scan([equal[:5],equal[5:]])==token_scan(equal) and not local_cut(equal,5)
    # Checking only the last two left bytes differ can cut between equal literals.
    bare_short = bytes([1,2,2,2,2,0])
    assert token_scan(bare_short) != split_scan([bare_short[:2],bare_short[2:]])
    # A supposed block start at a short fragment can manufacture a false certificate.
    assert local_cut(bytes([7,0]),1,True) and not local_cut(bytes([7,0]),1)
    x = bytes([7,7,7,7,0])
    assert token_scan(x) != split_scan([x[:4],x[4:]])
    # Long constant periodic lap has no strict internal reset. Clip by logical n.
    uniform=bytes(70)
    assert token_scan(uniform) == split_scan([uniform[:35],uniform[35:]])
    assert plan([uniform[:35],uniform[35:]],WIDTH)[2]
    shifted=bytes(75)
    assert token_scan(shifted)[1] is None
    assert token_scan(shifted[:39])[1]==RUN_ERROR and token_scan(shifted[:40])[1] is None
    big=(bytes([0,0,0,0,255])*12)+bytes([0,1,0,1])
    assert len(big)==64 and len(token_scan(big)[0])==3112 and token_scan(big)[1] is None
    trials = []
    pads = bytes(range(40))
    for run in (0,1,2,3,4,5,6,9,31,32,33,63,64,65,133):
        for b in (0,4,255):
            x = pads+bytes([b])*run+pads[::-1]
            for cut in (len(pads),len(pads)+run//2,len(pads)+run):
                trials.append([x[:cut],x[cut:]])
    for n in range(36):
        for period in (bytes([7]),bytes([0,0,0,0,255]),bytes([0,0,0,0,0,7])):
            raw=(period*(1+n//len(period)))[:n]
            trials.append([raw[:n//3],b"",raw[n//3:2*n//3],raw[2*n//3:]])
    # A genuine 64-byte bridge span across a seam, with no intervening strict cut.
    boundary_left=bytes([1,2])+bytes([2])*31
    boundary_right=bytes([2])*31+bytes([1,2])
    # The left cut is 32 bytes before the junction, the right cut 32 after it.
    trials.append([boundary_left,boundary_right])
    every_count=0
    for c in (0,1,4,255):
        side=bytes([c^128,c^1])
        for extra in range(256):
            raw=side+bytes([c])*4+bytes([extra])+side
            native=token_scan(raw)
            assert native==(side+bytes([c])*(4+extra)+side,None)
            for seam in range(1,len(raw)):
                pieces,_,_=plan([raw[:seam],raw[seam:]],WIDTH)
                assert split_scan(pieces)==native
                every_count+=1
    widths = Counter()
    for chunks in trials:
        raw=b"".join(chunks)
        native=token_scan(raw)
        pieces,copied,fallback=plan(chunks,WIDTH)
        assert split_scan(pieces)==native and b"".join(pieces)==raw
        correct_or_dummy = crc32_bzip2(native[0])
        for stored in (correct_or_dummy,correct_or_dummy^0x01):
            assert finish(split_scan(pieces),stored)==finish(native,stored)
        widths["joined_fallbacks" if fallback else "bounded_paths"]+=1
        if copied: widths["bridged_paths"]+=1
        widths["max_copied"] = max(widths["max_copied"],copied)
    assert widths["max_copied"]==64 and widths["joined_fallbacks"] and widths["bridged_paths"]
    return dict(width32_trials=len(trials),full_count_range_partitions=every_count,**widths)


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("--max-length",default=8,type=int)
    args=parser.parse_args()
    result=dict(max_length=args.max_length,alphabet=[0,1,2],foreign_state=255,width32=fixtures(),exhaustive=exhaustive(args.max_length))
    print(json.dumps(result,indent=2,sort_keys=True))
