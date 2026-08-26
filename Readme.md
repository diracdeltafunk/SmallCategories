# SmallCategories

The [SmallCategories Project](https://smallcats.info/) (name inspired by the [SmallGroups](https://docs.gap-system.org/pkg/smallgrp/doc/chap1.html) library in GAP) aims to build a useful database of (isomorphism classes of) small finite categories. Currently, the database is complete for categories with ≤8 morphisms.

This repository contains the code that generates the database, and the complete list of already-computed multiplication tables.

## Requirements

You only need [rust/cargo](https://rustup.rs). The generator is a single Rust
crate with no external solver and no intermediate files.

## Running

Ask for the cells you want, as `morphisms objects` pairs:

> ./generate-database.sh -f cells.txt
>
> ./generate-database.sh 9 3
>
> ./generate-database.sh --through 7

With no `-f` and no cells on the command line, the list is read from standard
input until end-of-file, so this works too:

> echo "10 4" | ./generate-database.sh -y

You do not have to work out what a cell depends on, or what order to build in.
The generator closes your request under dependency, drops the cells already on
disk, orders the rest, shows you the plan, and asks before computing anything:

```
$ ./generate-database.sh 9 3
Requested 1 cell(s); 18 needed in total once dependencies are included.

To compute, in dependency order (1 requested, 17 pulled in as dependencies):
    1. cats1-1   (dependency)
    2. cats2-1   (dependency)
  ...
   18. cats9-3

Proceed? [y/N]
```

Pass `-y` to skip the prompt, `--force` to rebuild cells that already exist, and
`--check` to rebuild everything in memory and diff it against what is on disk
without writing anything.

The dependencies are the smaller cells a category's disconnected half is
reconstructed from. Every one of them has strictly fewer morphisms, so ordering
by morphism count always works.

`cells.txt` is just a convenient list of the standard set; edit it freely.

## How it works

Categories are enumerated directly, one per isomorphism class, rather than by
listing every labelled multiplication table and discarding duplicates
afterwards. There are three stages.

1. **Shapes.** The nonidentity morphisms of a category form a directed
   multigraph on its objects. We enumerate those multigraphs up to isomorphism
   first. Every category has exactly one shape, and two categories sharing a
   fixed shape are isomorphic exactly when some automorphism of the shape
   carries one to the other. This replaces the relabelling group
   $S_k\times S_{n-k}$ by $\operatorname{Aut}(\text{shape})$, which is usually
   trivial.
2. **Composition tables.** For each shape, a backtracking search fills the
   table. Fixing the shape gives every cell an explicit small domain — the
   hom-set forced by the endpoints — and associativity is checked
   incrementally. Cells are filled in *level order*, so each prefix is a square
   subtable and a partial table can be discarded as soon as an automorphism
   fixing that square maps it to something smaller.
3. **Disconnected categories.** These are not searched for at all. A
   disconnected category is a multiset of strictly smaller connected ones, so
   it is reconstructed from cells already on disk. That is both much cheaper
   and an independent check, since the component formula predicts how many
   there will be before any are built.

No canonical form is computed anywhere in this pipeline. The search's symmetry
test is exact — at the final level it compares against every automorphism of the
shape — so it emits the least member of each $\operatorname{Aut}(\text{shape})$
orbit and never the same category twice. Verification is a separate concern:
`smallcats-enumerate verify` canonicalizes every stored table and checks that no
two collide, and `--against DIR` checks two databases agree class for class.

The previous pipeline — a Minion constraint model plus a deduplication pass over
the full relabelling group — is preserved in `legacy/`. It is no longer used.

## Propositions

Beside each `cats<n>-<k>.txt.gz` is a `props<n>-<k>.txt.gz` holding one integer per
line, in the same order: eighteen categorical properties of that category packed
into eighteen bits (`smallcats/src/propositions.rs` lists them in bit order).

Every one is a property of the category itself, so they are computed from the
multiplication table and nothing else. The two files are written by the same
call, so they cannot drift apart, and adding a cell computes that cell's
propositions and touches nothing else. `smallcats-enumerate verify` recomputes
them and checks they still agree with the tables.

## Using

Each file in the `database` directory is a gzipped text file containing a list (one entry per line) of multiplication tables of categories. Read one with `zcat`, or search with `zgrep`; they are compressed because the largest is 1.8 GB of text, and GitHub refuses a blob over 100 MB however well git compresses it internally. Gzip takes that file to 15 MB. For example, there are 7 categories with 1 object and 3 morphisms, so the file `database/cats3-1.txt.gz` has 7 lines. Each line is a matrix (written as a list of lists) representing the multiplication table of a category. If `m` is a matrix representing a category with morphism set {0,...,n-1}, then `m[i][j]` is the result of the composition `i∘j`, or `n` if the composition is undefined.

Each file is sorted, and holds exactly one table per isomorphism class: no two lines describe isomorphic categories, and none of that size is missing.

The particular representative stored is *not* part of the contract. It used to be the lexicographically least relabelling, which is expensive to compute and was only ever needed to remove duplicates; the search no longer produces duplicates, so the table stored is simply the one it finds. Treat a line as a category, not as a string — to compare two databases, compare isomorphism classes (`smallcats-enumerate verify --against`).

If you use this database or the code in this repository, I'd appreciate it if you cite me (Ben Spitz) and add a link to this repository to your acknowledgements. You are not required to do so, but I would appreciate it.

## Statistics

**Note.** More than this is known -- see [OEIS](https://oeis.org/A125696) and [Cruttwell-Leblanc](https://www.reluctantm.com/gcruttw/publications/ams2014CruttwellCountingFiniteCats.pdf). This table shows what is actually contained in the database.

| Objects →<br>Morphisms ↓ | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | Total |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  | **1** |
| 1 | 0 | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  |  | **1** |
| 2 | 0 | 2 | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  | **3** |
| 3 | 0 | 7 | 3 | 1 |  |  |  |  |  |  |  |  |  |  |  |  | **11** |
| 4 | 0 | 35 | 16 | 3 | 1 |  |  |  |  |  |  |  |  |  |  |  | **55** |
| 5 | 0 | 228 | 77 | 20 | 3 | 1 |  |  |  |  |  |  |  |  |  |  | **329** |
| 6 | 0 | 2,237 | 485 | 111 | 21 | 3 | 1 |  |  |  |  |  |  |  |  |  | **2,858** |
| 7 | 0 | 31,559 | 4,013 | 716 | 127 | 21 | 3 | 1 |  |  |  |  |  |  |  |  | **36,440** |
| 8 | 0 | 1,668,997 | 47,648 | 5,623 | 862 | 131 | 21 | 3 | 1 |  |  |  |  |  |  |  | **1,723,286** |
| 9 | 0 |  | 1,868,157 | 60,322 | 6,739 | 926 | 132 | 21 | 3 | 1 |  |  |  |  |  |  |  |
| 10 | 0 |  |  | 1,993,623 | 68,990 | 7,349 | 945 | 132 | 21 | 3 | 1 |  |  |  |  |  |  |
| 11 | 0 |  |  |  | 2,066,599 | 74,212 | 7,598 | 949 | 132 | 21 | 3 | 1 |  |  |  |  |  |
| 12 | 0 |  |  |  |  | 2,110,772 | 76,742 | 7,674 | 950 | 132 | 21 | 3 | 1 |  |  |  |  |
| 13 | 0 |  |  |  |  |  | 2,133,915 | 77,730 | 7,693 | 950 | 132 | 21 | 3 | 1 |  |  |  |
| 14 | 0 |  |  |  |  |  |  | 2,144,305 | 78,036 | 7,697 | 950 | 132 | 21 | 3 | 1 |  |  |
| 15 | 0 |  |  |  |  |  |  |  | 2,148,159 | 78,116 | 7,698 | 950 | 132 | 21 | 3 | 1 |  |

The italicized entries disagree with the values 60201 and 65922 reported by Cruttwell-Leblanc. A mathematical and implementation audit supports the values 60322 and 68990 shown here. Earlier versions of this database listed 68815 in the second cell; the audit identified 175 additional isomorphism classes. See [Enumeration & Verification](https://smallcats.info/enumeration) for the counting method, the discrepancy analysis, and the correction.

## Acknowledgements

The work here is greatly inspired by [Cruttwell-Leblanc](https://www.reluctantm.com/gcruttw/publications/ams2014CruttwellCountingFiniteCats.pdf). They deserve all of the credit for all of the ideas in the code -- I simply wanted to make available a complete database of small categories.
