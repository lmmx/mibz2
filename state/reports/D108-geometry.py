"""Independent interval model of PR444's staged-rule copies; no Rust execution."""
cases = sends = streams = 0
for stage in (128, 1024):
    for start in range(0, stage, 16):
        for count in range(3 * stage // 16 + 2):
            end = start + 16 * count
            intervals = []
            for rp in range(start, end, 16):
                assert rp % stage + 16 <= stage
                if rp % stage == stage - 16:
                    intervals.append((max(rp + 16 - stage, start), rp + 16))
            intervals.append((max(end - end % stage, start), end))
            cursor = start
            for lo, hi in intervals:
                assert lo == cursor and start <= lo <= hi <= end
                assert hi - lo <= stage - lo % stage
                cursor = hi
            assert cursor == end
            for base_mod in range(64):
                for lo, hi in intervals:
                    n = hi - lo
                    head = (-(base_mod + lo) % 64)
                    i = min(head, n)
                    while i + 64 <= n:
                        assert (base_mod + lo + i) % 64 == 0
                        assert lo % stage + i + 64 <= stage
                        for q in range(4):
                            assert (base_mod + lo + i + 16*q) % 16 == 0
                        streams += 1
                        i += 64
                    assert 0 <= n - i < 64 or n < head
                    assert lo % stage + n <= stage
                    sends += 1
                cases += 1
print(f'{cases} layouts/lengths/base alignments; {sends} send intervals; {streams} whole lines: PASS')
