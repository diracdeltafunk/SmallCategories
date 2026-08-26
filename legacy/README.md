# Legacy generator

This is the original pipeline, kept for provenance. It is no longer used and the
database is no longer built with it.

It encoded the category axioms as a [Minion](https://github.com/minion/minion)
constraint model, asked Minion for every *labelled* solution, and then removed
duplicates by minimising each table over the relabelling group
$S_k\times S_{n-k}$.

That last step is what made it expensive. Every isomorphism class appears up to
$k!\,(n-k)!$ times among the labelled solutions — 4,320 copies in the $(9,3)$
cell and 17,280 in $(10,4)$ — so the solution files grew to many gigabytes and
had to be split. `countcats_split.sh` and `countcats_hypersplit.sh` exist only to
cut a search into strata small enough to fit.

Each stratum also needed its symmetry group written out by hand and passed to
`process-minion-out`. That is where the last bug lived: the hypersplit driver
passed a plain symmetric block where the stratum's symmetry was really the
pair-preserving $S_2\wr S_p$, and minimising against the wrong group silently
discarded 175 categories from $(10,4)$.

The replacement enumerates one representative per isomorphism class directly.
See the repository Readme.

## Requirements, if you want to run it anyway

* [Python 3](https://www.python.org/downloads)
* [make](https://www.gnu.org/software/make)
* [rust/cargo](https://rustup.rs)
* [Minion](https://github.com/minion/minion)

Paths inside these scripts are relative to this directory.
