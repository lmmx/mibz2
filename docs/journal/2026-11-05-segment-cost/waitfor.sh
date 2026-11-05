#!/bin/bash
# wait (at most $2 seconds) until file $1 exists and is non-empty, or matches pattern $3
end=$((SECONDS + $2))
while [ $SECONDS -lt $end ]; do
  if [ -n "$3" ]; then grep -q "$3" "$1" 2>/dev/null && exit 0; else [ -s "$1" ] && exit 0; fi
  sleep 10
done
exit 1
