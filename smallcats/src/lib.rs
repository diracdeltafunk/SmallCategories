//! Enumeration and validation of small finite categories.
//!
//! A category with `n` morphisms and `k` objects is stored as an `n × n`
//! multiplication table.  The crate is layered:
//!
//! | Module | Concern |
//! |---|---|
//! | [`table`] | the table itself: entries, parsing, validation, connectivity |
//! | [`canonical`] | the least relabelling of a table, one per isomorphism class |
//! | [`shape`] | domain/codomain skeletons up to isomorphism, with their automorphisms |
//! | [`search`] | filling in one skeleton's tables without ever producing two isomorphic ones |
//! | [`components`] | rebuilding disconnected categories from smaller connected ones |
//! | [`cell`] | a whole database cell, and the order cells have to be built in |
//! | [`io`] | reading and writing cell files |
//!
//! Nothing here depends on an external solver.

pub mod canonical;
pub mod cell;
pub mod components;
pub mod io;
pub mod propositions;
pub mod search;
pub mod shape;
pub mod table;

pub type Result<T> = std::result::Result<T, String>;

pub use canonical::{canonicalize, CanonicalStats};
pub use components::{
    assemble, component_sizes, load_component_catalog, AssemblyResult, CatalogCount,
    ComponentCatalog,
};
pub use io::{
    canonical_classes, cell_path, propositions_path, read_cell, read_gzipped, read_masks,
    serialize_masks, serialize_tables, verify_file, write_gzipped, write_new, Verification,
};
pub use table::{Endpoints, Entry, Table};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::disjoint_union;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn table(text: &str, morphisms: usize, objects: usize) -> Table {
        Table::parse(text, morphisms, objects).unwrap()
    }

    fn brute_canonicalize(table: &Table) -> Table {
        fn permutations(values: &mut [usize], position: usize, output: &mut Vec<Vec<usize>>) {
            if position == values.len() {
                output.push(values.to_vec());
                return;
            }
            for index in position..values.len() {
                values.swap(position, index);
                permutations(values, position + 1, output);
                values.swap(position, index);
            }
        }
        let n = table.morphisms();
        let k = table.objects();
        let mut object_values: Vec<_> = (0..k).collect();
        let mut arrow_values: Vec<_> = (k..n).collect();
        let mut object_permutations = Vec::new();
        let mut arrow_permutations = Vec::new();
        permutations(&mut object_values, 0, &mut object_permutations);
        permutations(&mut arrow_values, 0, &mut arrow_permutations);
        let mut best: Option<Vec<Entry>> = None;
        for objects in &object_permutations {
            for arrows in &arrow_permutations {
                let permutation: Vec<_> = objects.iter().chain(arrows).copied().collect();
                let mut inverse = vec![0; n];
                for (new, &old) in permutation.iter().enumerate() {
                    inverse[old] = new;
                }
                let candidate: Vec<_> = (0..n)
                    .flat_map(|left| {
                        let inverse = &inverse;
                        let permutation = &permutation;
                        (0..n).map(move |right| {
                            table.at(permutation[left], permutation[right]).relabel(inverse)
                        })
                    })
                    .collect();
                if best.as_ref().is_none_or(|old| candidate < *old) {
                    best = Some(candidate);
                }
            }
        }
        Table::from_cells(n, k, best.unwrap()).unwrap()
    }

    fn relabel(table: &Table, permutation: &[usize]) -> Table {
        let n = table.morphisms();
        let mut inverse = vec![0; n];
        for (new, &old) in permutation.iter().enumerate() {
            inverse[old] = new;
        }
        let cells = (0..n)
            .flat_map(|left| {
                let inverse = &inverse;
                (0..n).map(move |right| {
                    table.at(permutation[left], permutation[right]).relabel(inverse)
                })
            })
            .collect();
        Table::from_cells(n, table.objects(), cells).unwrap()
    }

    #[test]
    fn parser_is_strict() {
        let parsed = table("[[0, 2], [2, 1]]", 2, 2);
        assert_eq!(parsed.format_line(), "[[0, 2], [2, 1]]");
        assert!(Table::parse("[[0, 2], [2, 1],]", 2, 2).is_err());
        assert!(Table::parse("[[0, 2], [2]]", 2, 2).is_err());
        assert!(Table::parse("[[0, 2], [2, 1]] junk", 2, 2).is_err());
    }

    #[test]
    fn validates_endpoints_and_connectivity() {
        let discrete = table("[[0, 2], [2, 1]]", 2, 2);
        let endpoints = discrete.validate().unwrap();
        assert!(!discrete.is_connected(&endpoints));

        let arrow = table("[[0, 3, 3], [3, 1, 2], [2, 3, 3]]", 3, 2);
        let endpoints = arrow.validate().unwrap();
        assert_eq!(endpoints.domain, vec![0, 1, 0]);
        assert_eq!(endpoints.codomain, vec![0, 1, 1]);
        assert!(arrow.is_connected(&endpoints));
    }

    #[test]
    fn reduced_canonical_search_matches_full_group() {
        let fixtures = [
            table("[[0, 2], [2, 1]]", 2, 2),
            table("[[0, 3, 3], [3, 1, 2], [2, 3, 3]]", 3, 2),
            table(
                "[[0, 4, 4, 3], [4, 1, 4, 4], [4, 4, 2, 4], [3, 4, 4, 0]]",
                4,
                3,
            ),
        ];
        for fixture in fixtures {
            fixture.validate().unwrap();
            let expected = brute_canonicalize(&fixture);
            let (actual, _) = canonicalize(&fixture).unwrap();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn canonical_result_is_invariant_under_relabelling() {
        let arrow = table("[[0, 3, 3], [3, 1, 2], [2, 3, 3]]", 3, 2);
        let relabelled = relabel(&arrow, &[1, 0, 2]);
        assert_ne!(arrow, relabelled);
        assert_eq!(
            canonicalize(&arrow).unwrap().0,
            canonicalize(&relabelled).unwrap().0
        );
    }

    #[test]
    fn disjoint_union_is_valid_and_canonicalizable() {
        let terminal = table("[[0]]", 1, 1);
        let arrow = table("[[0, 3, 3], [3, 1, 2], [2, 3, 3]]", 3, 2);
        let union = disjoint_union(&[&arrow, &terminal], 4, 3).unwrap();
        let endpoints = union.validate().unwrap();
        assert!(!union.is_connected(&endpoints));
        let (canonical, _) = canonicalize(&union).unwrap();
        assert_eq!(canonical, brute_canonicalize(&union));
    }

    fn database() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../database")
    }

    #[test]
    fn reduced_search_matches_brute_force_for_database_through_six_morphisms() {
        for morphisms in 1..=6 {
            for objects in 1..=morphisms {
                let path = io::cell_path(&database(), morphisms, objects);
                for raw in io::read_gzipped(&path).unwrap().lines() {
                    let input = Table::parse(raw, morphisms, objects).unwrap();
                    assert_eq!(canonicalize(&input).unwrap().0, brute_canonicalize(&input));
                }
            }
        }
    }

    #[test]
    fn reconstructs_an_all_disconnected_cell_up_to_isomorphism() {
        // (6,4) is entirely disconnected: a connected category on four objects
        // needs at least three nonidentity morphisms and there are only two.  So
        // component assembly alone must reproduce the whole cell — as a set of
        // isomorphism classes, which is the only thing a cell file means.
        let catalog = load_component_catalog(&database(), 6, 4).unwrap();
        let assembled = assemble(&catalog, 6, 4).unwrap().tables;

        let classes = |tables: &[Table]| -> std::collections::HashSet<Table> {
            tables
                .iter()
                .map(|table| canonicalize(table).unwrap().0)
                .collect()
        };
        let stored: Vec<Table> = io::read_gzipped(&io::cell_path(&database(), 6, 4))
            .unwrap()
            .lines()
            .map(|line| Table::parse(line, 6, 4).unwrap())
            .collect();

        for table in &assembled {
            let endpoints = table.validate().unwrap();
            assert!(!table.is_connected(&endpoints));
        }
        assert_eq!(classes(&assembled).len(), assembled.len(), "no repeats");
        assert_eq!(classes(&assembled), classes(&stored));
    }

    #[test]
    fn canonicalizing_a_union_of_two_copies_agrees_with_brute_force() {
        // Two copies of the same component is the awkward case: the union has a
        // symmetry exchanging them, on top of the symmetries inside each.  Building
        // it lays the components out in blocks and the least relabelling generally
        // interleaves them.  Everything here is a literal, so the test says nothing
        // about which representative the database happens to store.
        let arrow = table("[[0, 3, 3], [3, 1, 2], [2, 3, 3]]", 3, 2);
        assert!(arrow.is_connected(&arrow.validate().unwrap()));

        let union = disjoint_union(&[&arrow, &arrow], 6, 4).unwrap();
        let canonical = canonicalize(&union).unwrap().0;
        canonical.validate().unwrap();

        assert_eq!(canonical, brute_canonicalize(&union), "agrees with brute force");
        assert_eq!(
            canonicalize(&canonical).unwrap().0,
            canonical,
            "canonicalizing is idempotent"
        );
    }
}
