import coord,json
from unittest.mock import patch
n=6683737091
out={}
for kind in ['CHECKPOINT','CORRECTION','REVIEW','ACK','DECISION','QUEUED']:
 body=f'FROM: coordinator\nTO: oracle\nTYPE: {kind}\n'
 out[kind]={'inbox_reason':coord.misrouted(222,body,n),'channel_reason':coord.misrouted(145,body,n)}
body='FROM: coordinator\nTO: oracle\nTYPE: CORRECTION\n'
inbox=[{'id':n,'body':body,'user':{'login':'lmmx','type':'User'}}]
for name,body in [('no_citation','FROM: oracle\nTYPE: REVIEW\npage link, right now, weak chain'),('cited','FROM: oracle\nTYPE: REVIEW\nInbox '+str(n))]:
 out[name]=coord.state('oracle',channel=[{'id':n+1,'body':body}],inbox=inbox,cur={'processed_comment_ids':[n]})
with patch.object(coord,'state',return_value={'incomplete':[],'unreviewed':[],'queued':[]}):
 out['free_correction_route']=coord.route('oracle','CORRECTION')
with patch.object(coord,'state',return_value={'incomplete':[n],'unreviewed':[n],'queued':[]}):
 out['busy_correction_route']=coord.route('oracle','CORRECTION')
print(json.dumps(out,indent=2))
