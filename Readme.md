# SmallCategories

The SmallCategories Project (name inspired by the [SmallGroups](https://docs.gap-system.org/pkg/smallgrp/doc/chap1.html) library in GAP) aims to build a useful database of (isomorphism classes of) small finite categories. Currently, the database is complete for categories with ≤7 morphisms.

This repository contains the code that generates the database, and the complete list of already-computed multiplication tables.

## Requirements

If you want to generate the database yourself, you will need:

* [Python 3](https://www.python.org/downloads)
* [make](https://www.gnu.org/software/make)
* [rust/cargo](https://rustup.rs) (or just `rustc`, with some manual fiddling)
* [Minion](https://github.com/minion/minion)

## Running

To generate the multiplication tables of categories with n morphisms and k objects, run

> ./countcats.sh n k

This will produce a file named `catsn-k.txt` (containing the multipication tables) in the `database` directory, and print out the number of isomorphism classes of such categories. You can also use

> ./countcats_split.sh n k

or

> ./countcats_hypersplit.sh n k

which accomplishes the same task, using less disk space (but potentially twice as much memory). To run `./countcats.sh n k` for all 0≤k≤n≤N, run

> ./gen_full_database.sh N

You can also use

> ./countmonoids.sh n

to accomplish the same task as `./countcats_split.sh n 1`, using less memory.

## Using

Each text file in the `database` directory contains a list (one entry per line) of multiplication tables of categories. For example, there are 7 categories with 1 object and 3 morphisms, so the file `database/cats3-1.txt` has 7 lines. Each line is a matrix (written as a list of lists) representing the multiplication table of a category. If `m` is a matrix representing a category with morphism set {0,...,n-1}, then `m[i][j]` is the result of the composition `i∘j`, or `n` if the composition is undefined.

Each file is sorted lexicographically, and with the following guarantee: each line in the file is the (lexicographically) minimal element of its isomorphism class.

If you use this database or the code in this repository, I'd appreciate it if you cite me (Ben Spitz) and add a link to this repository to your acknowledgements. You are not required to do so, but I would appreciate it.

## Statistics

**Note.** More than this is known -- see [OEIS](https://oeis.org/A125696) and [Cruttwell-Leblanc](https://www.reluctantm.com/gcruttw/publications/ams2014CruttwellCountingFiniteCats.pdf). This table shows what is actually contained in the database.

| Objects →<br>Morphisms ↓ | 0 | 1     | 2     | 3       | 4       | 5    | 6    | 7   | 8   | 9   | 10 | 11 | Total     |
|--------------------------|---|-------|-------|---------|---------|------|------|-----|-----|-----|----|----|-----------|
| 0                        | 1 | 0     | 0     | 0       | 0       | 0    | 0    | 0   | 0   | 0   | 0  | 0  | **1**     |
| 1                        | 0 | 1     | 0     | 0       | 0       | 0    | 0    | 0   | 0   | 0   | 0  | 0  | **1**     |
| 2                        | 0 | 2     | 1     | 0       | 0       | 0    | 0    | 0   | 0   | 0   | 0  | 0  | **3**     |
| 3                        | 0 | 7     | 3     | 1       | 0       | 0    | 0    | 0   | 0   | 0   | 0  | 0  | **11**    |
| 4                        | 0 | 35    | 16    | 3       | 1       | 0    | 0    | 0   | 0   | 0   | 0  | 0  | **55**    |
| 5                        | 0 | 228   | 77    | 20      | 3       | 1    | 0    | 0   | 0   | 0   | 0  | 0  | **329**   |
| 6                        | 0 | 2237  | 485   | 111     | 21      | 3    | 1    | 0   | 0   | 0   | 0  | 0  | **2858**  |
| 7                        | 0 | 31559 | 4013  | 716     | 127     | 21   | 3    | 1   | 0   | 0   | 0  | 0  | **36440** |
| 8                        | 0 |       | 47648 | 5623    | 862     | 131  | 21   | 3   | 1   | 0   | 0  | 0  |           |
| 9                        | 0 |       |       | _60322_ | 6739    | 926  | 132  | 21  | 3   | 1   | 0  | 0  |           |
| 10                       | 0 |       |       |         | _68815_ | 7349 | 945  | 132 | 21  | 3   | 1  | 0  |           |
| 11                       | 0 |       |       |         |         |      | 7598 | 949 | 132 | 21  | 3  | 1  |           |
| 12                       | 0 |       |       |         |         |      |      |     | 950 | 132 | 21 | 3  |           |

Please note the italicized entries: these disagree with those reported by Cruttwell-Leblanc. I believe the values here are correct.

## Acknowledgements

The work here is greatly inspired by [Cruttwell-Leblanc](https://www.reluctantm.com/gcruttw/publications/ams2014CruttwellCountingFiniteCats.pdf). They deserve all of the credit for all of the ideas in the code -- I simply wanted to make available a complete database of small categories.

## Donate

Some of the larger computations were run on cloud computing services, which cost some money. If this database has been useful to you and you'd like to support its development, I'd be grateful for donations.

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/B0B3DOCLE)
