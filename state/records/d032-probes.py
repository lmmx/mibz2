import coord,json
from unittest.mock import patch
results=[]
for role,r in coord.ROLES.items():
 for delta in [0,1]:
  for kind in ['ACK','REVIEW','DECISION','LOG','','CHECKPOINT','PROPOSAL','CORRECTION','QUEUED']:
   body=f'FROM: coordinator\nTO: {role}\nTYPE: {kind}\n'
   flagged=bool(coord.misrouted(r['inbox'],body,r['offset']+delta))
   expected=delta==1 and kind not in coord.ASSIGNMENTS
   assert flagged==expected,(role,delta,kind)
   results.append({'role':role,'offset_delta':delta,'kind':kind,'flagged':flagged})
n=coord.ROLES['oracle']['offset']
inbox=[{'id':n+i,'body':'TYPE: CORRECTION','user':{'type':'User'}} for i in [1,2]]
channel=[{'id':n+3,'body':f'FROM: oracle\nTYPE: REVIEW\nCovered: {n+1}, {n+2}'}]
s=coord.state('oracle',channel=channel,inbox=inbox,cur={'processed_comment_ids':[n+1,n+2]})
assert s=={'incomplete':[],'unreviewed':[],'queued':[]}
print(json.dumps({'boundary_checks':len(results),'all_passed':True,'batch_state':s,'cases':results},indent=2))
