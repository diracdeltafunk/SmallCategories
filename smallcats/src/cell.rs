//! Building one `(morphisms, objects)` cell of the database.
//!
//! A cell is assembled in two halves.  Connected categories are searched for
//! directly, shape by shape.  Disconnected ones are *not* searched for: a
//! disconnected category is a multiset of strictly smaller connected ones, so it
//! is reconstructed from cells that are already on disk.  That is both far
//! cheaper and an independent check, since the number of disconnected categories
//! is predicted by the component formula before any of them are built.

use crate::search::{search_shape, CellStats};
use crate::shape::shapes;
use crate::{assemble, load_component_catalog, Result, Table};
use rayon::prelude::*;
use std::collections::BTreeSet;
use std::path::Path;
use std::time::Instant;

pub struct CellReport {
    pub morphisms: usize,
    pub objects: usize,
    pub connected: usize,
    pub disconnected: usize,
    pub shapes: usize,
    pub solutions: u64,
    pub nodes: u64,
    pub duplicates: usize,
    pub seconds: f64,
}

impl CellReport {
    pub fn total(&self) -> usize {
        self.connected + self.disconnected
    }
}

/// Enumerate the connected categories with these dimensions.
pub fn connected(morphisms: usize, objects: usize) -> Result<(Vec<Table>, CellStats)> {
    let shapes = shapes(morphisms, objects, true)?;

    // Shapes partition the categories of a cell, so their searches are wholly
    // independent and can run at once.  The results are sorted afterwards, so the
    // output does not depend on the order they finish in.
    let searched = shapes
        .par_iter()
        .map(|shape| {
            let mut tables = Vec::new();
            let mut stats = CellStats::default();
            search_shape(shape, &mut tables, &mut stats)?;
            Ok((tables, stats))
        })
        .collect::<Result<Vec<(Vec<Table>, CellStats)>>>()?;

    let mut stats = CellStats {
        shapes: shapes.len(),
        ..CellStats::default()
    };
    let mut tables = Vec::new();
    for (found, found_stats) in searched {
        tables.extend(found);
        stats.merge(found_stats);
    }
    Ok((tables, stats))
}

/// Build a whole cell: search the connected part, reconstruct the rest from
/// `database`, then sort into the repository's canonical order.
pub fn build(morphisms: usize, objects: usize, database: &Path) -> Result<(Vec<Table>, CellReport)> {
    let started = Instant::now();
    let (mut tables, stats) = connected(morphisms, objects)?;

    tables.sort_unstable();
    let before = tables.len();
    tables.dedup();
    let duplicates = before - tables.len();
    let connected_count = tables.len();

    // A disconnected category splits into at least two components, each with at
    // least one object, so cells with fewer than two objects have none.  That
    // covers the empty category and every monoid cell.
    let disconnected = if objects < 2 {
        0
    } else {
        let catalog = load_component_catalog(database, morphisms, objects)?;
        let assembled = assemble(&catalog, morphisms, objects)?;
        let count = assembled.tables.len();
        tables.extend(assembled.tables);
        count
    };

    tables.sort_unstable();
    let before = tables.len();
    tables.dedup();
    if tables.len() != before {
        return Err(format!(
            "cats{morphisms}-{objects}: the connected search and the component \
             assembly produced {} shared tables, which cannot happen",
            before - tables.len()
        ));
    }

    Ok((
        tables,
        CellReport {
            morphisms,
            objects,
            connected: connected_count,
            disconnected,
            shapes: stats.shapes,
            solutions: stats.solutions,
            nodes: stats.nodes,
            duplicates,
            seconds: started.elapsed().as_secs_f64(),
        },
    ))
}

/// The cells that building `(morphisms, objects)` reads from disk.
///
/// These are exactly the component sizes the assembler loads, plus the
/// one-morphism category it always needs.  Cells with fewer than two objects are
/// entirely connected, so they read nothing.  Every dependency has strictly fewer
/// morphisms than the cell that needs it, so the dependency graph is acyclic and
/// sorting by morphism count is always a valid build order.
pub fn dependencies(morphisms: usize, objects: usize) -> Vec<(usize, usize)> {
    if objects < 2 {
        return Vec::new();
    }
    let mut needed = vec![(1, 1)];
    needed.extend(crate::component_sizes(morphisms, objects));
    needed.sort_unstable();
    needed.dedup();
    needed
}

/// A resolved build order: what was asked for, what is already on disk, and what
/// has to be computed, in an order that satisfies every dependency.
pub struct Plan {
    pub requested: Vec<(usize, usize)>,
    pub present: Vec<(usize, usize)>,
    pub to_compute: Vec<(usize, usize)>,
}

/// Close the requested cells under dependency, then split them by what exists.
///
/// With `force`, every cell in the closure is rebuilt even if its file is already
/// there; otherwise only missing files are computed.
pub fn plan(requested: &[(usize, usize)], database: &Path, force: bool) -> Result<Plan> {
    let mut closure: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut queue: Vec<(usize, usize)> = requested.to_vec();
    while let Some((morphisms, objects)) = queue.pop() {
        if objects > morphisms {
            return Err(format!(
                "cats{morphisms}-{objects}: a category cannot have more objects than morphisms"
            ));
        }
        if !closure.insert((morphisms, objects)) {
            continue;
        }
        queue.extend(dependencies(morphisms, objects));
    }

    let mut present = Vec::new();
    let mut to_compute = Vec::new();
    for &(morphisms, objects) in &closure {
        let path = crate::io::cell_path(database, morphisms, objects);
        if path.exists() && !force {
            present.push((morphisms, objects));
        } else {
            to_compute.push((morphisms, objects));
        }
    }
    // Dependencies always have fewer morphisms, so this order is valid.
    to_compute.sort_unstable();

    Ok(Plan {
        requested: requested.to_vec(),
        present,
        to_compute,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependencies_have_strictly_fewer_morphisms() {
        for morphisms in 0..14 {
            for objects in 0..=morphisms {
                for (needed_morphisms, needed_objects) in dependencies(morphisms, objects) {
                    assert!(
                        needed_morphisms < morphisms,
                        "cats{morphisms}-{objects} depends on the larger cats{needed_morphisms}-{needed_objects}"
                    );
                    assert!(needed_objects <= needed_morphisms);
                }
            }
        }
    }

    #[test]
    fn a_plan_never_builds_a_cell_before_something_it_needs() {
        let empty = std::path::Path::new("/nonexistent");
        let plan = plan(&[(12, 8), (10, 4), (9, 3)], empty, false).unwrap();
        let position = |cell: (usize, usize)| plan.to_compute.iter().position(|&c| c == cell);
        for (index, &cell) in plan.to_compute.iter().enumerate() {
            for needed in dependencies(cell.0, cell.1) {
                let at = position(needed).expect("every dependency is in the plan");
                assert!(at < index, "{needed:?} must be built before {cell:?}");
            }
        }
    }
}
