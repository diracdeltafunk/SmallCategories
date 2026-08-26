//! Exhaustive, isomorph-free search for the composition tables of a fixed shape.
//!
//! With the shape fixed, every cell of the table has an explicit small domain (the
//! hom-set forced by the endpoints) and the only remaining symmetry is
//! `Aut(shape)`.  The search fills cells in *level order* — morphism `d`'s row and
//! column complete the `(d+1) × (d+1)` square — so that each prefix is a square
//! subtable.  That lets us discard a partial table as soon as some automorphism
//! fixing the prefix setwise maps it to something lexicographically smaller.
//!
//! The test is exact, not just a filter: at the last level the square is the whole
//! table, so every surviving automorphism is compared, and an automorphism is only
//! dropped along the way when it already compared strictly greater on a prefix and
//! so can never witness a smaller image.  What survives is therefore the least
//! member of its `Aut(shape)` orbit — one table per isomorphism class, with no
//! deduplication step and no canonical form to compute.

use crate::shape::Shape;
use crate::{Entry, Result, Table};

/// A cell of the partial table.  `None` means "composable, not filled in yet",
/// which is a fact about the search rather than about a category — which is why
/// it lives here in an `Option` and not as a third `Entry` variant.  Nothing of
/// the sort can leak into a finished `Table`.
type Cell = Option<Entry>;

#[derive(Clone, Copy, Debug, Default)]
pub struct CellStats {
    pub shapes: usize,
    pub solutions: u64,
    pub nodes: u64,
}

impl CellStats {
    pub fn merge(&mut self, other: Self) {
        self.shapes += other.shapes;
        self.solutions += other.solutions;
        self.nodes += other.nodes;
    }
}

/// Every category with this shape, one table per isomorphism class, each the least
/// member of its orbit under `Aut(shape)` in level order.
pub fn search_shape(shape: &Shape, output: &mut Vec<Table>, stats: &mut CellStats) -> Result<()> {
    let n = shape.morphisms;
    if n == 0 {
        output.push(Table::from_cells(0, 0, Vec::new())?);
        return Ok(());
    }

    let mut table: Vec<Cell> = vec![None; n * n];
    for left in 0..n {
        for right in 0..n {
            table[left * n + right] = if !shape.composable(left, right) {
                Some(Entry::Undef)
            } else if right < shape.objects {
                Some(Entry::Mor(left as u8)) // f ∘ id = f
            } else if left < shape.objects {
                Some(Entry::Mor(right as u8)) // id ∘ g = g
            } else {
                None
            };
        }
    }

    // Free cells in level order, each with the hom-set it may take values from.
    let mut cells: Vec<(usize, usize)> = Vec::new();
    for level in 0..n {
        for column in 0..=level {
            cells.push((level, column));
        }
        for row in 0..level {
            cells.push((row, level));
        }
    }
    let free: Vec<((usize, usize), Vec<Entry>)> = cells
        .into_iter()
        .filter(|&(left, right)| table[left * n + right].is_none()) 
        .map(|(left, right)| ((left, right), shape.candidates(left, right)))
        .collect();

    // The level at which each free cell completes its square, so the symmetry test
    // runs once per level rather than once per assignment.
    let boundaries: Vec<Option<usize>> = free
        .iter()
        .enumerate()
        .map(|(index, &((left, right), _))| {
            let level = left.max(right);
            let last = index + 1 == free.len() || {
                let ((next_left, next_right), _) = free[index + 1];
                next_left.max(next_right) > level
            };
            if last {
                Some(level)
            } else {
                None
            }
        })
        .collect();

    let plan = SearchPlan { free, boundaries };
    let mut search = Search {
        shape,
        n,
        table,
        plan: &plan,
        live: (0..shape.automorphisms.len()).collect(),
        nodes: 0,
        solutions: 0,
    };
    let mut found = Vec::new();
    search.descend(0, &mut found)?;
    stats.nodes += search.nodes;
    stats.solutions += search.solutions;

    output.extend(found);
    Ok(())
}

/// The fixed part of a search: which cells to fill, in which order, from which
/// hom-sets, and where each square closes.  None of it changes as the search
/// runs, so it lives behind a shared reference and never has to be cloned.
struct SearchPlan {
    free: Vec<((usize, usize), Vec<Entry>)>,
    boundaries: Vec<Option<usize>>,
}

struct Search<'a> {
    shape: &'a Shape,
    n: usize,
    table: Vec<Cell>,
    plan: &'a SearchPlan,
    live: Vec<usize>,
    nodes: u64,
    solutions: u64,
}

impl Search<'_> {
    fn descend(&mut self, index: usize, found: &mut Vec<Table>) -> Result<()> {
        self.nodes += 1;
        if index == self.plan.free.len() {
            self.solutions += 1;
            // Every cell is filled at a leaf; collecting through `Option` states
            // that invariant instead of trusting it.
            let cells = self
                .table
                .iter()
                .copied()
                .collect::<Option<Vec<Entry>>>()
                .ok_or("reached a complete table with an unfilled cell")?;
            found.push(Table::from_cells(self.n, self.shape.objects, cells)?);
            return Ok(());
        }

        // `plan` is a shared reference, so copying it out of `self` frees us to
        // call `&mut self` methods while reading the candidate list.
        let plan = self.plan;
        let ((left, right), values) = &plan.free[index];
        let (left, right) = (*left, *right);
        for &value in values {
            self.table[left * self.n + right] = Some(value);
            if self.associative(left, right) {
                match plan.boundaries[index] {
                    None => self.descend(index + 1, found)?,
                    Some(level) => {
                        let saved = self.live.clone();
                        if self.reduce(level) {
                            self.descend(index + 1, found)?;
                        }
                        self.live = saved;
                    }
                }
            }
            self.table[left * self.n + right] = None;
        }
        Ok(())
    }

    /// The composite, when this cell has been filled in *and* the two morphisms
    /// actually compose.  Both "not yet" and "never" answer `None`, which is what
    /// every associativity check wants.
    fn get(&self, left: usize, right: usize) -> Option<usize> {
        self.table[left * self.n + right].and_then(Entry::morphism)
    }

    /// Check every associativity triple that the newly filled cell takes part in.
    ///
    /// A triple `(a, b, c)` constrains the table through `(a∘b)∘c = a∘(b∘c)`, and
    /// only once all four of its cells are known.  Filling `(left, right)` can
    /// complete such a triple in exactly four ways: the new cell may be the inner
    /// composite on either side, or the outer composite on either side.
    fn associative(&self, left: usize, right: usize) -> bool {
        let n = self.n;
        let Some(product) = self.get(left, right) else {
            return true;
        };

        // (a, b) = (left, right):  (left∘right)∘c  vs  left∘(right∘c)
        for c in 0..n {
            if let (Some(inner), Some(outer)) = (self.get(right, c), self.get(product, c)) {
                if matches!(self.get(left, inner), Some(other) if other != outer) {
                    return false;
                }
            }
        }
        // (b, c) = (left, right):  (a∘left)∘right  vs  a∘(left∘right)
        for a in 0..n {
            if let (Some(inner), Some(outer)) = (self.get(a, left), self.get(a, product)) {
                if matches!(self.get(inner, right), Some(other) if other != outer) {
                    return false;
                }
            }
        }
        // (a∘b, c) = (left, right):  the new cell is the outer composite on the left
        for a in 0..n {
            for b in 0..n {
                if self.get(a, b) != Some(left) {
                    continue;
                }
                if let Some(inner) = self.get(b, right) {
                    if matches!(self.get(a, inner), Some(other) if other != product) {
                        return false;
                    }
                }
            }
        }
        // (a, b∘c) = (left, right):  the new cell is the outer composite on the right
        for b in 0..n {
            for c in 0..n {
                if self.get(b, c) != Some(right) {
                    continue;
                }
                if let Some(inner) = self.get(left, b) {
                    if matches!(self.get(inner, c), Some(other) if other != product) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Drop automorphisms that can no longer witness a smaller table, and report
    /// whether the current square is still the least one seen.
    fn reduce(&mut self, level: usize) -> bool {
        let n = self.n;
        let size = level + 1;
        let mut kept = Vec::with_capacity(self.live.len());
        for &index in &self.live {
            let relabelling = &self.shape.automorphisms[index];
            let permutation = &relabelling.map;
            // Only automorphisms that fix the square setwise can be compared yet.
            if (0..size).any(|morphism| permutation[morphism] >= size) {
                kept.push(index);
                continue;
            }
            // Compare in the same level order the search fills cells in.  Row-major
            // order would not do: the prefix has to be an initial segment of the
            // comparison order, or pruning could discard an orbit's least member.
            let mut ordering = std::cmp::Ordering::Equal;
            let mut compare = |row: usize, column: usize| {
                // Cells move by the permutation; the composite inside a cell is
                // itself a morphism, so it moves by the inverse.
                let image = self.table[permutation[row] * n + permutation[column]]
                    .map(|entry| entry.relabel(&relabelling.inverse));
                let current = self.table[row * n + column];
                if image != current {
                    ordering = image.cmp(&current);
                    return false;
                }
                true
            };
            'compare: for level in 0..size {
                for column in 0..=level {
                    if !compare(level, column) {
                        break 'compare;
                    }
                }
                for row in 0..level {
                    if !compare(row, level) {
                        break 'compare;
                    }
                }
            }
            match ordering {
                std::cmp::Ordering::Less => return false,
                std::cmp::Ordering::Equal => kept.push(index),
                std::cmp::Ordering::Greater => {}
            }
        }
        self.live = kept;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::connected;
    use crate::shape::shapes;

    /// Connected counts published in OEIS A125699, rows 1 through 8.
    const PUBLISHED: &[(usize, usize, usize)] = &[
        (1, 1, 1),
        (2, 1, 2),
        (3, 1, 7),
        (3, 2, 1),
        (4, 1, 35),
        (4, 2, 6),
        (5, 1, 228),
        (5, 2, 28),
        (5, 3, 2),
        (6, 1, 2237),
        (6, 2, 159),
        (6, 3, 11),
        (7, 2, 1075),
        (7, 3, 77),
        (7, 4, 3),
        (8, 2, 9389),
        (8, 3, 497),
        (8, 4, 24),
    ];

    #[test]
    fn connected_counts_match_a125699() {
        for &(morphisms, objects, expected) in PUBLISHED {
            let (tables, _) = connected(morphisms, objects).expect("search succeeds");
            let mut tables = tables;
            tables.sort_unstable();
            tables.dedup();
            assert_eq!(
                tables.len(),
                expected,
                "C({morphisms},{objects}) should be {expected}"
            );
        }
    }

    /// The disputed Cruttwell-Leblanc cells, recomputed from scratch.
    #[test]
    fn connected_counts_of_the_corrected_cells() {
        for &(morphisms, objects, expected) in &[(9usize, 3usize, 3568usize), (10, 4, 1477)] {
            let (mut tables, _) = connected(morphisms, objects).expect("search succeeds");
            tables.sort_unstable();
            tables.dedup();
            assert_eq!(tables.len(), expected);
        }
    }

    /// The search claims to emit one table per isomorphism class without ever
    /// deduplicating.  Check that directly, using the canonical form only as an
    /// independent judge: two tables are isomorphic exactly when they share one.
    #[test]
    fn every_result_is_valid_and_no_two_are_isomorphic() {
        for &(morphisms, objects) in &[(6usize, 2usize), (7, 3), (8, 4), (7, 1)] {
            let (tables, _) = connected(morphisms, objects).expect("search succeeds");
            let mut classes = std::collections::HashMap::new();
            for (index, table) in tables.iter().enumerate() {
                let endpoints = table.validate().expect("a valid category");
                assert!(table.is_connected(&endpoints));
                let (canonical, _) = crate::canonicalize(table).expect("canonicalizes");
                if let Some(earlier) = classes.insert(canonical, index) {
                    panic!("({morphisms},{objects}): results {earlier} and {index} are isomorphic");
                }
            }
            assert_eq!(classes.len(), tables.len());
        }
    }

    /// Distinct shapes must be genuinely distinct: no two may be related by a
    /// relabelling of the objects.
    #[test]
    fn shapes_are_pairwise_non_isomorphic() {
        for &(morphisms, objects) in &[(6usize, 3usize), (7, 3), (8, 4), (9, 4)] {
            let found = shapes(morphisms, objects, true).expect("shapes enumerate");
            let mut seen = std::collections::HashSet::new();
            for shape in &found {
                // Canonical key: the sorted edge multiset under every object
                // relabelling, minimised.
                let key = crate::shape::permutations(objects)
                    .into_iter()
                    .map(|map| {
                        let mut edges: Vec<(usize, usize)> = (objects..morphisms)
                            .map(|arrow| (map[shape.domain[arrow]], map[shape.codomain[arrow]]))
                            .collect();
                        edges.sort_unstable();
                        edges
                    })
                    .min()
                    .expect("at least one relabelling");
                assert!(seen.insert(key), "two shapes are isomorphic");
            }
        }
    }
}
