# smallcats

The enumerator behind the database, plus the table utilities it shares with the
component assembler. No external solver, no intermediate files; [rayon][] is the
only dependency.

[rayon]: https://github.com/rayon-rs/rayon

Three things run in parallel, and each is independent by construction rather than
by locking: the searches of different shapes, the assembly of different component
multisets, and the parsing and validation of different lines of a cell file.
Output does not depend on completion order, because every cell is sorted before
it is written.

```sh
cargo test --manifest-path smallcats/Cargo.toml
cargo build --release --manifest-path smallcats/Cargo.toml
```

## Layout

| Module | What it does |
| --- | --- |
| `table.rs` | The table itself: entries, strict parsing, validation, connectivity |
| `canonical.rs` | The exact lexicographic least relabelling under $S_k\times S_{n-k}$, used only for verification |
| `shape.rs` | Domain/codomain skeletons up to isomorphism, with their automorphism groups |
| `search.rs` | Fills one skeleton's composition tables, emitting one per isomorphism class |
| `components.rs` | Rebuilds disconnected categories from smaller connected ones |
| `cell.rs` | A whole cell, and the order cells have to be built in |
| `io.rs` | Reading and writing cell files |

## Binaries

`smallcats-enumerate` builds cells. `build` is the normal entry point; `cell` and
`compare` are the single-cell primitives underneath it.

```sh
smallcats-enumerate build 9 3                       # resolves dependencies, prompts
smallcats-enumerate build -f cells.txt -y           # no prompt
smallcats-enumerate cell --morphisms 11 --objects 5 --connected-only
smallcats-enumerate compare --morphisms 9 --objects 3
```

`smallcats-components` is the standalone component assembler, kept because it
reconstructs a disconnected cell from a completely independent direction and so
makes a good cross-check. It never writes into `database/`: `assemble` requires
an explicit output path, rejects paths inside the input database directory, and
refuses to overwrite an existing file.

```sh
smallcats-components assemble --database database --morphisms 12 --objects 7 \
  --expect-count 7674 --output ../tmp/v2-candidates/cats12-7.txt
smallcats-components verify --input ../tmp/v2-candidates/cats12-7.txt \
  --morphisms 12 --objects 7 --expect-count 7674
```

## Why the shape layer

Every category determines a directed multigraph on its objects, one edge per
nonidentity morphism. Categories sharing a fixed shape are isomorphic exactly
when an automorphism of that shape carries one to the other, so stratifying by
shape replaces the relabelling group $S_k\times S_{n-k}$ by
$\operatorname{Aut}(\text{shape})$ — trivial for most shapes. It also gives every
table cell an explicit small domain, the hom-set forced by the endpoints.

That does nothing when $k\leq 2$, where there are one or two shapes and the
automorphism group is most of the original. There the work is carried by
isomorph rejection inside the search: cells are filled in level order, so each
prefix is a square subtable, and a partial table is discarded as soon as an
automorphism fixing that square maps it to something smaller. Correctness does
not depend on how good that pruning is — the pruning only ever discards a table
that some automorphism makes smaller, and never the least member of an orbit.
