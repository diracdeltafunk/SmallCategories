#!/bin/bash

echo "Writing minion file..."
rm -f minion-files/category$1-$2.minion
mkdir minion-files
python3 generate-minion-file.py $1 $2 > minion-files/category$1-$2.minion
echo "Running minion..."
rm -f minion-out/out$1-$2.txt
mkdir minion-out
minion -findallsols -noprintsols -solsout minion-out/out$1-$2.txt minion-files/category$1-$2.minion > /dev/null 2>&1
echo "Churning through the results..."
rm -f database/cats$1-$2.txt
mkdir database
python3 process-minion-out.py $1 $2 minion-out/out$1-$2.txt database/cats$1-$2.txt