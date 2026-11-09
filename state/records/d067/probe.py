"""D067: finite source-backed witnesses, not Rust/Miri or a kernel probe."""
import json
from copy import deepcopy
from pathlib import Path

p = Path(__file__).with_name('source-witness.txt').read_text()
assert 'return Quota::Unlimited;' in p[p.index('fn cpu_quota()'):p.index('fn parse_cpu_max')]
assert 'if let Ok(max)' in p[p.index('fn cpu_quota()'):p.index('fn parse_cpu_max')]
assert 'publish(shared, place, bit, res, symbols.syms);\n    hub.retire_counted' in p
assert 'self.closed && self.jobs.is_empty() && self.live == 0' in p

# cpu_quota: failure reading the membership file returns Unlimited;
# failed reads of every candidate cpu.max are skipped, retaining initial Unlimited.
quota_results = []
for what, membership_ok, reads in [
    ('membership EACCES', False, []),
    ('all cpu.max reads EACCES', True, [None, None]),
    ('leaf max; parent EACCES hiding a smaller quota', True, ['max 100000', None]),
]:
    q = 'Unlimited'
    if membership_ok:
        for value in reads:
            if value is not None:
                assert value.startswith('max')
    quota_results.append({'case': what, 'quota': q, 'other_target_facts_true': True,
                          'service_target': q == 'Unlimited'})

# A legal interleaving with four successful blocks and closed scanner input.
s = dict(issued=4, taken=3, ready=[], end=4, live=1, finished=3)
counter_trace = [dict(step='last finisher before publish', **deepcopy(s))]
s['ready'].append(3)
counter_trace.append(dict(step='publish wakes reader; finisher descheduled', **deepcopy(s)))
s['ready'] = []
s['taken'] = 4
counter_trace.append(dict(step='reader consumes last output then sees EOF', **s))
assert s['end'] == s['taken'] and s['finished'] == 3
s['live'] -= 1
s['finished'] += 1
counter_trace.append(dict(step='finisher retires later', **s))

# One live token is lost by outer unwind before publication, all other work done.
# Current hub exit predicate has no cancellation escape; notify alone cannot fix it.
lost = dict(closed=True, jobs=0, prepared=0, walked=0, live=1, cancelled=True)
done = lost['closed'] and lost['jobs'] == 0 and lost['live'] == 0
assert not done
result = dict(scope=__doc__, quota_read_failures=quota_results,
              eof_before_retirement=counter_trace,
              lost_token_after_cancel=dict(state=lost, hub_done=done))
Path(__file__).with_name('results.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps(result, indent=2))
