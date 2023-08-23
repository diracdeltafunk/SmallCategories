#!/bin/bash

mkdir -p minion-files
mkdir -p minion-out
mkdir -p database

for ((i = 0; i < $1; i++))
do
    echo "[$i non-identity idempotents]"
    echo "Writing minion file..."
    rm -f minion-files/monoid$1-$i.minion
    python3 generate-minion-monoids.py $1 $i > minion-files/monoid$1-$i.minion
    echo "Running minion..."
    rm -f minion-out/out$1-1-$i.txt
    minion -findallsols -noprintsols -solsout minion-out/out$1-1-$i.txt minion-files/monoid$1-$i.minion > /dev/null 2>&1
    echo "Churning through the results..."
    rm -f database/cats$1-1-$i.txt
    process-minion-out/target/release/process-minion-out minion-out/out$1-1-$i.txt database/cats$1-1-$i.txt $1 1 $i
    echo "Deleting output file to save disk space..."
    rm -f minion-out/out$1-1-$i.txt
done
echo "Merging and sorting results..."
rm -f database/cats$1-1.txt database/cats$1-1-unsorted.txt
cat database/cats$1-1-*.txt > database/cats$1-1-unsorted.txt
sort database/cats$1-1-unsorted.txt > database/cats$1-1.txt
echo "Deleting temporary files..."
rm -f database/cats$1-1-*.txt
echo "Counting..."
wc -l database/cats$1-1.txt