#!/usr/bin/env bash
#
# Build cells of the category database.
#
#   ./generate-database.sh -f cells.txt         the standard set
#   ./generate-database.sh 9 3                  that cell, plus whatever it needs
#   ./generate-database.sh --through 7 -y       everything up to 7 morphisms
#   ./generate-database.sh -f cells.txt --check rebuild in memory and diff
#   echo "10 4" | ./generate-database.sh        read the cell list from stdin
#
# All this does is build the binary and hand over.  `smallcats-enumerate build`
# closes the requested cells under dependency, drops the ones already on disk,
# orders the rest, shows the plan, and asks before computing anything.  Pass -y
# to skip the prompt and --force to rebuild cells that already exist.
#
# With no -f and no cells on the command line, the list is read from standard
# input until end-of-file.

set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cargo build --release --quiet --manifest-path smallcats/Cargo.toml
exec smallcats/target/release/smallcats-enumerate build "$@"
