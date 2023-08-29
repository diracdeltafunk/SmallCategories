#!/bin/bash

echo "Generating full database up to $1 morphisms"
./countcats_rust.sh 0 0
for n in $(seq 1 $1)
do
for k in $(seq 1 $n)
do
echo "Generating categories with $n morphisms, $k objects"
./countcats_rust.sh $n $k
done
done

echo "Done!"