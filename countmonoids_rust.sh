#!/bin/bash

echo "Writing minion file..."
rm -f minion-files/monoid$1.minion
mkdir -p minion-files
python3 generate-minion-monoids.py $1 > minion-files/monoid$1.minion
echo "Running minion..."
rm -f minion-out/out$1-1.txt
mkdir -p minion-out
minion -findallsols -noprintsols -solsout minion-out/out$1-1.txt minion-files/monoid$1.minion > /dev/null 2>&1
echo "Churning through the results..."
rm -f database/cats$1-1.txt
mkdir -p database
process-minion-out/target/release/process-minion-out $1 1 minion-out/out$1-1.txt database/cats$1-1.txt