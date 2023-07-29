#!/bin/bash

for n in $(seq 1 $1)
do
./supabase_push_row.sh $n
done