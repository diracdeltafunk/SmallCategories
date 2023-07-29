#!/bin/bash

for k in $(seq 1 $1)
do
if test -f "database/cats$1-$k.txt"; then
echo "Pushing ($1,$k)"
python3 supabase_push_cats.py $1 $k database/cats$1-$k.txt
else
echo "Skipping ($1,$k)"
fi
done