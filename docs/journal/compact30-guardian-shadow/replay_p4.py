#!/usr/bin/env python3
"""Verify the saved P4 receipt and exact pinned log; never contacts GitHub.

python replay_p4.py --out /scratch/p4
"""
import argparse,datetime,hashlib,json,pathlib,re

ap=argparse.ArgumentParser(description=__doc__)
ap.add_argument('--out',type=pathlib.Path,required=True)
a=ap.parse_args()
if not __debug__: ap.error('Assertions must be enabled')
root=pathlib.Path(__file__).resolve().parent
a.out.mkdir(parents=True,exist_ok=True)
inputs={n:(root/n).read_bytes() for n in ['p4-outgoing.json','p4-receipt.json','p4-channel-log.md']}
outgoing=json.loads(inputs['p4-outgoing.json'])
receipt=json.loads(inputs['p4-receipt.json'])
byid={c['id']:c for c in receipt['comments']}
observed=byid[6630660444]
ack=byid[6630672389]
assert outgoing['id']==observed['id']==6630660444
assert outgoing['body']==observed['body']
assert outgoing['advisory_commit']=='51657ab576a804531de0e71a40d4197c42f7c592'
assert 'guardian outgoing comment 6630660444' in ack['body']
assert outgoing['advisory_commit'] in ack['body']
assert 'TYPE: ACK' in ack['body'] and 'charter §12.2' in ack['body']
assert receipt['log']['commit'] in ack['body']
assert receipt['log']['path'] in ack['body']
assert 'No other action.' in ack['body']
assert 'issue_comment.created' in ack['body']
assert receipt['nudge']['text']=='Resume #149'
assert not receipt['nudge']['technical_content_relayed']
logbytes=inputs['p4-channel-log.md']
git_blob=hashlib.sha1(b'blob '+str(len(logbytes)).encode()+b'\0'+logbytes).hexdigest()
assert git_blob==receipt['log']['git_blob']
log=logbytes.decode()
heading='## '+receipt['log']['heading']+'\n'
assert log.count(heading)==1
section=log.split(heading,1)[1]
assert '**6630660444**' in section
quoted=section.split('Verbatim:\n\n',1)[1]
lines=quoted.splitlines()
assert all(l.startswith('>') for l in lines)
body='\n'.join(l[2:] if l.startswith('> ') else l[1:] for l in lines)
assert body==outgoing['body']
recorded_time=re.search(r'Comment \*\*6630660444\*\* on #149, (\S+)\.',section)[1]
parse=lambda s:datetime.datetime.fromisoformat(s.replace('Z','+00:00'))
assert parse(ack['created_at'])>parse(observed['created_at'])
timestamp_delta=(parse(recorded_time)-parse(observed['created_at'])).total_seconds()
manifest=[dict(path=n,sha256=hashlib.sha256(b).hexdigest(),bytes=len(b)) for n,b in sorted(inputs.items())]
mb=(json.dumps(manifest,indent=2,sort_keys=True)+'\n').encode()
(a.out/'p4-closure-manifest.json').write_bytes(mb)
result=dict(status='P4 CLOSED: correlated ACK and exact verbatim log verified on next invocation',
    outgoing_comment_id=outgoing['id'],outgoing_body_sha256=hashlib.sha256(outgoing['body'].encode()).hexdigest(),
    ack_comment_id=ack['id'],ack_body_sha256=hashlib.sha256(ack['body'].encode()).hexdigest(),
    log_commit=receipt['log']['commit'],log_path=receipt['log']['path'],log_git_blob=git_blob,
    log_sha256=hashlib.sha256(logbytes).hexdigest(),verbatim_body_matches=True,observed_outgoing_body_unchanged=True,
    nudge=receipt['nudge'],
    timestamp_finding=dict(id='D3',severity='LOW',github_created_at=observed['created_at'],logged_time=recorded_time,
        discrepancy_seconds=timestamp_delta,changes_delivery_outcome=False),
    wake_evidence='Coordinator reports issue_comment.created; matching ACK and pinned log evidence receipt. No event-queue telemetry supplied; no latency figure certified.',
    limitations=['Shared lmmx login, role headers and commit attribution do not authenticate the coordinator.',
        'No new authority, activation, research review, or unattended wake capability is established.',
        'Earlier P1/P2 evidence and decisions are not rerun or reassessed here.'],
    evidence_digest=hashlib.sha256(mb).hexdigest())
(a.out/'p4-closure-results.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
print(json.dumps(result,indent=2,sort_keys=True))
