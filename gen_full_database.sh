#!/bin/bash

echo "Generating full database up to $1 morphisms"
for n in $(seq 1 $1)
do
for k in $(seq 1 $n)
do
echo "Generating categories with $n morphisms, $k objects"
./countcats.sh $n $k
done
done

echo "Done!"