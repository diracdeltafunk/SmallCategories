//! Reconstructing disconnected categories from their connected components.
//!
//! Every finite category is uniquely a disjoint union of connected ones, so a
//! disconnected category never has to be searched for: it is a multiset of
//! strictly smaller connected categories, which are already on disk.  That is far
//! cheaper than searching, and it is an independent check, because the component
//! formula predicts how many there will be before any are built.

use rayon::prelude::*;
use crate::table::{Entry, Table};
use crate::Result;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct CatalogCount {
    pub morphisms: usize,
    pub objects: usize,
    pub tables: usize,
    pub connected: usize,
}

#[derive(Clone, Debug)]
pub struct ComponentCatalog {
    terminal: Table,
    nonterminal: Vec<Table>,
    pub counts: Vec<CatalogCount>,
}

/// The component cells that assembling `(target_morphisms, target_objects)` reads,
/// excluding the one-morphism category, which is always needed and handled apart.
///
/// A connected category with `b` objects needs at least `b - 1` nonidentity
/// morphisms, so a component contributing excess `e` has at most `e + 1` objects.
/// Every size returned has strictly fewer morphisms than the target, which is why
/// sorting a work list by morphism count is always a valid build order.
pub fn component_sizes(target_morphisms: usize, target_objects: usize) -> Vec<(usize, usize)> {
    let mut sizes = Vec::new();
    let Some(excess) = target_morphisms.checked_sub(target_objects) else {
        return sizes;
    };
    for component_excess in 1..=excess {
        for objects in 1..=target_objects.min(component_excess + 1) {
            let morphisms = objects + component_excess;
            if morphisms < target_morphisms {
                sizes.push((morphisms, objects));
            }
        }
    }
    sizes
}

pub fn load_component_catalog(
    database: &Path,
    target_morphisms: usize,
    target_objects: usize,
) -> Result<ComponentCatalog> {
    if target_objects == 0 || target_objects > target_morphisms {
        return Err("component assembly requires 1 <= objects <= morphisms".into());
    }

    let mut counts = Vec::new();
    let terminal_tables = read_database_cell(database, 1, 1)?;
    let terminal_table_count = terminal_tables.len();
    let terminal_connected: Vec<_> = terminal_tables
        .into_iter()
        .filter(|table| {
            let endpoints = table.validate().expect("already validated");
            table.is_connected(&endpoints)
        })
        .collect();
    if terminal_connected.len() != 1 {
        return Err(format!(
            "cats1-1.txt must contain exactly one connected category; found {}",
            terminal_connected.len()
        ));
    }
    counts.push(CatalogCount {
        morphisms: 1,
        objects: 1,
        tables: terminal_table_count,
        connected: 1,
    });

    let mut nonterminal = Vec::new();
    {
        for (morphisms, objects) in component_sizes(target_morphisms, target_objects) {
            let tables = read_database_cell(database, morphisms, objects)?;
            let table_count = tables.len();
            let mut connected = 0;
            for table in tables {
                let endpoints = table.validate().expect("already validated");
                if table.is_connected(&endpoints) {
                    connected += 1;
                    nonterminal.push(table);
                }
            }
            counts.push(CatalogCount {
                morphisms,
                objects,
                tables: table_count,
                connected,
            });
        }
    }
    nonterminal.sort_by(|left, right| {
        (left.excess(), left.objects(), left.cells()).cmp(&(
            right.excess(),
            right.objects(),
            right.cells(),
        ))
    });
    Ok(ComponentCatalog {
        terminal: terminal_connected.into_iter().next().expect("one terminal"),
        nonterminal,
        counts,
    })
}

fn read_database_cell(database: &Path, morphisms: usize, objects: usize) -> Result<Vec<Table>> {
    let path = crate::io::cell_path(database, morphisms, objects);
    let text = crate::io::read_gzipped(&path)?;
    let lines: Vec<&str> = text.lines().collect();

    // Parsing and validating a line says nothing about any other line, so the
    // expensive part — `validate` is cubic in the morphism count — runs in parallel.
    let tables = lines
        .par_iter()
        .enumerate()
        .map(|(index, raw)| {
            let where_ = |error: String| format!("{}:{}: {error}", path.display(), index + 1);
            if raw.is_empty() {
                return Err(where_("blank line".into()));
            }
            let table = Table::parse(raw, morphisms, objects).map_err(&where_)?;
            table.validate().map_err(&where_)?;
            Ok(table)
        })
        .collect::<Result<Vec<Table>>>()?;

    // Uniqueness is a property of the file as a whole, so it stays sequential —
    // but it can borrow the tables rather than copying them.
    let mut seen = HashSet::with_capacity(tables.len());
    for (index, table) in tables.iter().enumerate() {
        if !seen.insert(table.cells()) {
            return Err(format!(
                "{}:{}: duplicate multiplication table",
                path.display(),
                index + 1
            ));
        }
    }
    Ok(tables)
}

#[derive(Clone, Debug)]
pub struct AssemblyResult {
    pub tables: Vec<Table>,
    pub component_multisets: usize,
}

pub fn assemble(
    catalog: &ComponentCatalog,
    target_morphisms: usize,
    target_objects: usize,
) -> Result<AssemblyResult> {
    let target_excess = target_morphisms
        .checked_sub(target_objects)
        .ok_or_else(|| "objects exceed morphisms".to_string())?;

    // Choosing the multisets is cheap; building and canonicalizing them is not.
    // Separating the two lets the expensive half run in parallel, since the
    // multisets are independent of one another.
    let mut multisets = Vec::new();
    let mut chosen = Vec::new();
    choose(
        catalog,
        0,
        target_excess,
        target_objects,
        &mut chosen,
        &mut multisets,
    );

    let built = multisets
        .par_iter()
        .map(|(components, terminals)| {
            let mut parts: Vec<&Table> = components
                .iter()
                .map(|&index| &catalog.nonterminal[index])
                .collect();
            parts.extend(std::iter::repeat_n(&catalog.terminal, *terminals));
            disjoint_union(&parts, target_morphisms, target_objects)
        })
        .collect::<Result<Vec<Table>>>()?;

    let mut tables = built;
    tables.sort_unstable();
    let before = tables.len();
    tables.dedup();
    if tables.len() != before {
        return Err(format!(
            "{} component multisets collapsed to {} canonical tables",
            before,
            tables.len()
        ));
    }
    Ok(AssemblyResult {
        component_multisets: multisets.len(),
        tables,
    })
}

/// Every multiset of connected components with the target's total size, as the
/// indices of the nonterminal components plus a count of one-morphism ones.
fn choose(
    catalog: &ComponentCatalog,
    start: usize,
    remaining_excess: usize,
    remaining_objects: usize,
    chosen: &mut Vec<usize>,
    output: &mut Vec<(Vec<usize>, usize)>,
) {
    if remaining_excess == 0 {
        output.push((chosen.clone(), remaining_objects));
        return;
    }
    for index in start..catalog.nonterminal.len() {
        let component = &catalog.nonterminal[index];
        if component.excess() > remaining_excess {
            break;
        }
        if component.objects() > remaining_objects {
            continue;
        }
        chosen.push(index);
        choose(
            catalog,
            index,
            remaining_excess - component.excess(),
            remaining_objects - component.objects(),
            chosen,
            output,
        );
        chosen.pop();
    }
}

pub(crate) fn disjoint_union(
    components: &[&Table],
    target_morphisms: usize,
    target_objects: usize,
) -> Result<Table> {
    let actual_morphisms: usize = components.iter().map(|table| table.morphisms()).sum();
    let actual_objects: usize = components.iter().map(|table| table.objects()).sum();
    if (actual_morphisms, actual_objects) != (target_morphisms, target_objects) {
        return Err(format!(
            "component coordinates sum to ({actual_morphisms},{actual_objects}), expected ({target_morphisms},{target_objects})"
        ));
    }

    let n = target_morphisms;
    let mut cells = vec![Entry::Undef; n * n];
    let mut object_offset = 0;
    let mut nonidentity_offset = 0;
    for component in components {
        let mut labels = Vec::with_capacity(component.morphisms());
        for local in 0..component.morphisms() {
            labels.push(if local < component.objects() {
                object_offset + local
            } else {
                target_objects + nonidentity_offset + local - component.objects()
            });
        }
        for left in 0..component.morphisms() {
            for right in 0..component.morphisms() {
                if let Some(result) = component.composite(left, right) {
                    cells[labels[left] * n + labels[right]] = Entry::Mor(labels[result] as u8);
                }
            }
        }
        object_offset += component.objects();
        nonidentity_offset += component.excess();
    }
    Table::from_cells(n, target_objects, cells)
}
