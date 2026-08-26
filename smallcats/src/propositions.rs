//! The categorical properties recorded for every category in the database.
//!
//! Each one is a property of the category itself, so all eighteen are computed
//! from the multiplication table and nothing else — there is no external data to
//! join against, and adding a cell to the database computes that cell's
//! propositions and touches nothing else.
//!
//! The bit assignment is fixed: it is the one the website has always published,
//! and changing it would silently reinterpret stored data.  Five of the eighteen
//! are the duals of five others and are computed by running the same test on the
//! opposite category.

use crate::table::Table;
use crate::Result;
use rayon::prelude::*;

/// Proposition names in bit order.  Position in this array *is* the bit.
pub const NAMES: [&str; 18] = [
    "has_terminal_object",   // 0
    "is_initial",            // 1
    "has_finite_products",   // 2
    "has_binary_products",   // 3
    "has_binary_coproducts", // 4
    "is_cocomplete",         // 5
    "has_equalizers",        // 6
    "has_coequalizers",      // 7
    "is_monoid",             // 8
    "is_connected",          // 9
    "is_groupoid",           // 10
    "is_skeletal",           // 11
    "has_initial_object",    // 12
    "is_complete",           // 13
    "is_discrete",           // 14
    "is_preorder",           // 15
    "has_finite_coproducts", // 16
    "is_terminal",           // 17
];

/// Every proposition packed into the low eighteen bits, in `NAMES` order.
pub fn mask(table: &Table) -> Result<u32> {
    let category = View::new(table)?;
    let opposite = table.opposite();
    let dual = View::new(&opposite)?;

    let terminal = category.has_terminal_object();
    let binary_products = category.has_binary_products();
    let equalizers = category.has_equalizers();
    let finite_products = binary_products && terminal;

    let initial = dual.has_terminal_object();
    let binary_coproducts = dual.has_binary_products();
    let coequalizers = dual.has_equalizers();
    let finite_coproducts = binary_coproducts && initial;

    let values = [
        terminal,
        category.morphisms == 0,
        finite_products,
        binary_products,
        binary_coproducts,
        finite_coproducts && coequalizers,
        equalizers,
        coequalizers,
        category.objects == 1,
        category.is_connected(),
        category.is_groupoid(),
        category.is_skeletal(),
        initial,
        finite_products && equalizers,
        category.morphisms == category.objects,
        category.is_preorder(),
        finite_coproducts,
        category.morphisms == 1 && category.objects == 1,
    ];

    Ok(values
        .iter()
        .enumerate()
        .filter(|&(_, &set)| set)
        .fold(0u32, |mask, (bit, _)| mask | 1 << bit))
}

/// Every table's mask, in the same order.  Each is independent of the others.
pub fn masks(tables: &[Table]) -> Result<Vec<u32>> {
    tables.par_iter().map(mask).collect()
}

/// A table with its hom-sets worked out, so the tests below read like their
/// definitions instead of like index arithmetic.
struct View<'a> {
    table: &'a Table,
    morphisms: usize,
    objects: usize,
    domain: Vec<usize>,
    codomain: Vec<usize>,
    /// `hom[x * objects + y]` lists the morphisms from `x` to `y`.
    hom: Vec<Vec<usize>>,
}

impl<'a> View<'a> {
    fn new(table: &'a Table) -> Result<Self> {
        let endpoints = table.validate()?;
        let objects = table.objects();
        let mut hom = vec![Vec::new(); objects * objects];
        for arrow in 0..table.morphisms() {
            hom[endpoints.domain[arrow] * objects + endpoints.codomain[arrow]].push(arrow);
        }
        Ok(Self {
            table,
            morphisms: table.morphisms(),
            objects,
            domain: endpoints.domain,
            codomain: endpoints.codomain,
            hom,
        })
    }

    fn hom(&self, from: usize, to: usize) -> &[usize] {
        &self.hom[from * self.objects + to]
    }

    /// `left ∘ right`, when they compose.
    fn compose(&self, left: usize, right: usize) -> Option<usize> {
        self.table.composite(left, right)
    }

    /// An object every object has exactly one morphism into.
    fn has_terminal_object(&self) -> bool {
        (0..self.objects).any(|x| (0..self.objects).all(|y| self.hom(y, x).len() == 1))
    }

    /// At most one morphism between any ordered pair of objects.
    fn is_preorder(&self) -> bool {
        self.hom.iter().all(|arrows| arrows.len() <= 1)
    }

    fn is_iso(&self, arrow: usize) -> bool {
        let (source, target) = (self.domain[arrow], self.codomain[arrow]);
        self.hom(target, source).iter().any(|&back| {
            self.compose(arrow, back) == Some(target) && self.compose(back, arrow) == Some(source)
        })
    }

    /// No isomorphisms between distinct objects.
    fn is_skeletal(&self) -> bool {
        (0..self.morphisms)
            .all(|arrow| self.domain[arrow] == self.codomain[arrow] || !self.is_iso(arrow))
    }

    fn is_groupoid(&self) -> bool {
        (0..self.morphisms).all(|arrow| self.is_iso(arrow))
    }

    /// Connected as an undirected graph on the objects.  The empty category has
    /// no objects and is not connected.
    fn is_connected(&self) -> bool {
        if self.objects == 0 {
            return false;
        }
        let mut seen = vec![false; self.objects];
        seen[0] = true;
        let mut stack = vec![0];
        while let Some(object) = stack.pop() {
            for arrow in 0..self.morphisms {
                let (source, target) = (self.domain[arrow], self.codomain[arrow]);
                for (near, far) in [(source, target), (target, source)] {
                    if near == object && !seen[far] {
                        seen[far] = true;
                        stack.push(far);
                    }
                }
            }
        }
        seen.into_iter().all(|reached| reached)
    }

    /// `apex` with these projections is a product of `left` and `right` exactly
    /// when, for every object `z`, pairing up with the projections is a bijection
    /// `hom(z, apex) -> hom(z, left) × hom(z, right)`.
    fn is_product(&self, left: usize, right: usize, first: usize, second: usize) -> bool {
        let apex = self.domain[first];
        if self.codomain[first] != left
            || self.codomain[second] != right
            || self.domain[second] != apex
        {
            return false;
        }
        (0..self.objects).all(|z| {
            let mut pairs: Vec<(Option<usize>, Option<usize>)> = self
                .hom(z, left)
                .iter()
                .flat_map(|&a| self.hom(z, right).iter().map(move |&b| (Some(a), Some(b))))
                .collect();
            for &mediating in self.hom(z, apex) {
                let image = (
                    self.compose(first, mediating),
                    self.compose(second, mediating),
                );
                match pairs.iter().position(|pair| *pair == image) {
                    // Two mediating morphisms hitting the same pair breaks
                    // uniqueness; none hitting it breaks existence.
                    Some(index) => pairs.swap_remove(index),
                    None => return false,
                };
            }
            pairs.is_empty()
        })
    }

    fn has_binary_products(&self) -> bool {
        (0..self.objects).all(|left| {
            (0..self.objects).all(|right| {
                (0..self.objects).any(|apex| {
                    self.hom(apex, left).iter().any(|&first| {
                        self.hom(apex, right)
                            .iter()
                            .any(|&second| self.is_product(left, right, first, second))
                    })
                })
            })
        })
    }

    /// `mediator` equalizes `f` and `g` exactly when `f ∘ mediator = g ∘ mediator`
    /// and every other morphism that equalizes them factors through it uniquely.
    fn is_equalizer(&self, f: usize, g: usize, mediator: usize) -> bool {
        if self.domain[f] != self.codomain[mediator]
            || self.domain[g] != self.codomain[mediator]
            || self.codomain[f] != self.codomain[g]
        {
            return false;
        }
        if self.compose(f, mediator) != self.compose(g, mediator) {
            return false;
        }
        let source = self.domain[f];
        let apex = self.domain[mediator];
        (0..self.objects).all(|z| {
            self.hom(z, source)
                .iter()
                .filter(|&&candidate| self.compose(f, candidate) == self.compose(g, candidate))
                .all(|&candidate| {
                    self.hom(z, apex)
                        .iter()
                        .filter(|&&p| self.compose(mediator, p) == Some(candidate))
                        .count()
                        == 1
                })
        })
    }

    fn has_equalizers(&self) -> bool {
        (0..self.morphisms).all(|f| {
            self.hom(self.domain[f], self.codomain[f])
                .iter()
                .all(|&g| (0..self.morphisms).any(|h| self.is_equalizer(f, g, h)))
        })
    }
}

/// Propositions that swap when every arrow is reversed, as pairs of bits.
pub const DUALS: [(usize, usize); 5] = [
    (0, 12),  // has_terminal_object / has_initial_object
    (3, 4),   // has_binary_products / has_binary_coproducts
    (6, 7),   // has_equalizers / has_coequalizers
    (2, 16),  // has_finite_products / has_finite_coproducts
    (13, 5),  // is_complete / is_cocomplete
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn small_database() -> Vec<Table> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../database");
        let mut tables = Vec::new();
        for morphisms in 0..=6 {
            for objects in 0..=morphisms {
                let path = crate::io::cell_path(&root, morphisms, objects);
                let Ok(text) = crate::io::read_gzipped(&path) else { continue };
                for line in text.lines() {
                    tables.push(Table::parse(line, morphisms, objects).unwrap());
                }
            }
        }
        tables
    }

    fn bit(mask: u32, name: &str) -> bool {
        (mask >> NAMES.iter().position(|n| *n == name).unwrap()) & 1 == 1
    }

    /// Every proposition is a property of the category, so relabelling the
    /// morphisms cannot change any of them.
    #[test]
    fn propositions_are_isomorphism_invariants() {
        for table in small_database() {
            let n = table.morphisms();
            if n < 2 {
                continue;
            }
            // Swap the last two nonidentity morphisms, or two objects if there
            // are no two nonidentities to swap.
            let k = table.objects();
            let mut permutation: Vec<usize> = (0..n).collect();
            if n - k >= 2 {
                permutation.swap(n - 1, n - 2);
            } else if k >= 2 {
                permutation.swap(0, 1);
            } else {
                continue;
            }
            let mut inverse = vec![0; n];
            for (new, &old) in permutation.iter().enumerate() {
                inverse[old] = new;
            }
            let source = &table;
            let cells = (0..n)
                .flat_map(|left| {
                    let inverse = &inverse;
                    let permutation = &permutation;
                    (0..n).map(move |right| {
                        source.at(permutation[left], permutation[right]).relabel(inverse)
                    })
                })
                .collect();
            let relabelled = Table::from_cells(n, k, cells).unwrap();
            assert_eq!(mask(&table).unwrap(), mask(&relabelled).unwrap());
        }
    }

    /// Reversing every arrow turns limits into colimits and back, and leaves the
    /// self-dual properties alone.
    #[test]
    fn the_opposite_category_swaps_dual_propositions() {
        for table in small_database() {
            let here = mask(&table).unwrap();
            let there = mask(&table.opposite()).unwrap();
            let mut swapped = here;
            for (left, right) in DUALS {
                swapped &= !(1 << left | 1 << right);
                swapped |= ((here >> right) & 1) << left;
                swapped |= ((here >> left) & 1) << right;
            }
            assert_eq!(there, swapped, "on {}", table.format_line());
        }
    }

    /// Entailments that hold for every category, by definition of the terms.
    #[test]
    fn the_propositions_entail_each_other_correctly() {
        for table in small_database() {
            let m = mask(&table).unwrap();
            let line = table.format_line();
            assert_eq!(
                bit(m, "has_finite_products"),
                bit(m, "has_binary_products") && bit(m, "has_terminal_object"),
                "finite products is binary products with a terminal object, on {line}"
            );
            assert_eq!(
                bit(m, "is_complete"),
                bit(m, "has_finite_products") && bit(m, "has_equalizers"),
                "complete is finite products with equalizers, on {line}"
            );
            if bit(m, "is_discrete") {
                assert!(bit(m, "is_skeletal") && bit(m, "is_preorder") && bit(m, "is_groupoid"),
                    "a discrete category is skeletal, a preorder, and a groupoid, on {line}");
            }
            if bit(m, "is_terminal") {
                assert!(bit(m, "is_monoid") && bit(m, "is_complete") && bit(m, "is_cocomplete"),
                    "the terminal category is a complete, cocomplete monoid, on {line}");
            }
            if bit(m, "is_monoid") {
                assert!(bit(m, "is_connected"), "one object is connected, on {line}");
            }
            if bit(m, "is_initial") {
                assert!(bit(m, "is_discrete") && !bit(m, "is_connected"),
                    "the empty category is discrete and not connected, on {line}");
            }
        }
    }
}
