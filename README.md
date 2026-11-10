# The Pathfinder's state

The Pathfinder's working state, kept apart from master. `state/cursor.json` lists the inbox
comments it has processed (`processed_comment_ids`), which `.github/coordination/coord.py`
reads to tell whether it's busy.
